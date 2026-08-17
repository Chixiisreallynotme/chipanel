# Architecture & Mécanismes Internes de ChiPanel

ChiPanel est conçu comme une console d'administration ultra-légère, autonome et haute performance, optimisée pour fonctionner sur du matériel homelab contraint (ex: Intel Core i3 2C/4T @ 1.70 GHz, 8 Go de RAM) sans monopoliser les ressources CPU/RAM dédiées au serveur Minecraft.

---

## 1. Vue d'Ensemble de l'Architecture

ChiPanel repose sur une séparation claire entre un backend asynchrone compilé en Rust natif et une interface monopage (SPA) fluide développée avec Svelte 5.

```
+-------------------------------------------------------------------------+
|                              CLIENT (Navigateur)                        |
|   - Svelte 5 SPA (Runes: $state, $derived, $effect)                     |
|   - WebSocket Client (Logs streaming, télémétrie temps réel)            |
|   - CodeMirror 6 (Éditeur syntaxique & Diff Merge)                      |
+-------------------------------------------------------------------------+
                                     │  HTTP / WebSocket (Port 25500)
                                     ▼
+-------------------------------------------------------------------------+
|                         CHIPANEL BACKEND (Rust / Axum)                  |
|                                                                         |
|  ┌───────────────────────────────────────────────────────────────────┐  |
|  │                        HTTP Routing Layer (Axum 0.7)              │  |
|  │   - Auth Middleware (JWT / Argon2 / RBAC admin & viewer)          │  |
|  │   - REST Endpoints (/api/server, /api/backups, /api/audit...)     │  |
|  │   - WebSocket Hub (Multiplexeur d'état, broadcast 2s)             │  |
|  └──────────────────────────────────┬────────────────────────────────┘  |
|                                     │                                   |
|  ┌──────────────────────────────────┴────────────────────────────────┐  |
|  │                     Systèmes Métier & Acteurs Tokio               │  |
|  │                                                                   │  |
|  │  [Acteur RCON Multiplexé]        [Gestionnaire Sauvegardes]       │  |
|  │  - Single TCP stream persistant  - Compression asynchrone (zip)   │  |
|  │  - Reconnexion automatique       - Règles d'exclusion & Rétention │  |
|  │  - Queue mpsc(128) + oneshot     - Export distant S3 / MinIO      │  |
|  │                                                                   │  |
|  │  [Supervision Podman & D-Bus]     [Registre d'Audit Immuable]     │  |
|  │  - Socket /run/user/1000/podman   - Journal append-only .jsonl    │  |
|  │  - D-Bus session bus (zbus)       - Filtres & Exportation CSV     │  |
|  └──────────────────────────────────┬────────────────────────────────┘  |
+-------------------------------------┼───────────────────────────────────+
                                      │
              ┌───────────────────────┼───────────────────────┐
              ▼                       ▼                       ▼
   +─────────────────────+ +─────────────────────+ +─────────────────────+
   |   Podman Rootless   | |  lazymc (Proxy TCP) | |   Minecraft Server  |
   |   (UID 1000 / crun) | |   Port 25565 proxy  | |    Port 25566 TCP   |
   |  minecraft.container| |   Auto-hibernation  | |    Port 25575 RCON  |
   +─────────────────────+ +─────────────────────+ +─────────────────────+
```

---

## 2. Le Backend Rust (Axum 0.7 & Tokio)

Le backend est entièrement compilé en code machine avec optimisations LTO (`opt-level = "z"`, `panic = "abort"`, `codegen-units = 1`). L'exécutable résultant pèse moins de 15 Mo et consomme moins de 25 Mo de RAM en régime de croisière.

### A. Acteur Tokio RCON Persistant (`src/rcon/actor.rs`)
Dans les panels traditionnels, chaque requête de métrique ou commande console ouvre une nouvelle connexion TCP vers le port RCON de Minecraft, provoquant :
- Des tempêtes d'erreurs `Connection refused (os error 111)` quand le serveur est arrêté ou en hibernation.
- Une surcharge CPU inutile par allocation permanente de sockets.
- Un spam massif dans les logs du serveur.

**La Solution ChiPanel :**
ChiPanel implémente le patron de conception **Tokio Actor** :
1. Une seule tâche de fond Tokio gère l'unique socket TCP persistant vers le port RCON (`127.0.0.1:25575`).
2. Les requêtes HTTP et le hub WebSocket envoient leurs messages via un canal asynchrone `tokio::sync::mpsc::channel(128)` sous la forme `RconRequest::Execute { command, responder }`.
3. Chaque appelant reçoit un canal `oneshot` dédié pour récupérer la réponse sans risque de blocage.
4. En cas de coupure (ex: hibernation `lazymc`), l'acteur tente de se reconnecter en tâche de fond avec backoff exponentiel sans bloquer le serveur HTTP ni propager d'erreur 500 aux clients web.

### B. Communication Système & Podman Rootless
ChiPanel s'exécute lui-même dans un conteneur Podman rootless (UID 1000) et dialogue avec l'hôte via deux interfaces montées en lecture seule :
1. **Le Socket Podman Rootless (`/run/user/1000/podman/podman.sock`)** : Dialogue via le protocole REST de Libpod / Docker pour interroger l'état des conteneurs, surveiller les métriques CPU/RAM et streamer les logs.
2. **Le Bus D-Bus Utilisateur (`/run/user/1000/bus`)** : Dialogue avec `systemd --user` via la crate `zbus` pour superviser les quadlets (`systemctl --user start/stop/restart minecraft.service`) et contrôler l'unité native `lazymc.service`. La découverte du chemin de socket est résolue dynamiquement à l'exécution via `DBUS_SESSION_BUS_ADDRESS` et `libc::getuid()`.

---

## 3. Le Frontend Svelte 5 (Architecture Runes)

L'interface de ChiPanel est construite avec SvelteKit en mode SPA (`@sveltejs/adapter-static`). Elle utilise la syntaxe moderne de **Svelte 5 (Runes)**, éliminant les stores réactifs legacy (`writable()`) au profit d'un modèle d'état granulaire et ultra-rapide :

- **`$state`** : Stocke l'état réactif local et global (liste des joueurs, métriques, buffer de logs, état des sauvegardes).
- **`$derived`** : Calcule automatiquement les vues dérivées sans re-render superflu (filtrage des sauvegardes, KPIs de succès d'audit, conversion de formats).
- **`$props`** : Déclare les interfaces typées de composants réutilisables.
- **`$effect`** : Synchronise les timers, les connexions WebSocket et les paramètres d'URL.

### Composants Clés
- **LogViewer** : Affiche les logs avec virtualisation du scroll pour absorber plus de 50 000 lignes sans ralentissement du navigateur.
- **ConfigDiffModal** : Intègre CodeMirror Merge pour prévisualiser les différences avant enregistrement sur disque.
- **SparkProfilerCard** : Contrôle l'échantillonneur Spark et extrait directement les liens de Flamegraphs interactifs.
- **MetricsChart** : Graphiques de performance (TPS, MSPT, CPU, RAM) légers et réactifs.

---

## 4. Cycle de Vie du Serveur & Intégration lazymc

ChiPanel pilote le serveur Minecraft à travers 3 modes d'exploitation :

```
             ┌──────────────────────────────────────────────┐
             │                 MODE ÉTEINT                  │
             │  (Conteneur arrêté, lazymc.service stoppé)   │
             │           Consommation RAM : 0 Mo            │
             └───────────────────────┬──────────────────────┘
                                     │
                        Démarrer via ChiPanel UI
                                     │
                                     ▼
             ┌──────────────────────────────────────────────┐
             │              MODE HIBERNATION                │
             │  (Proxy lazymc actif sur le port 25565 TCP,  │
             │   conteneur Java en sommeil sur port 25566)  │
             │          Consommation RAM : ~8 Mo            │
             └───────────────────────┬──────────────────────┘
                                     │
                         Connexion d'un joueur
                        (Handshake TCP détecté)
                                     │
                                     ▼
             ┌──────────────────────────────────────────────┐
             │                 MODE ACTIF                   │
             │  (Conteneur Java éveillé, monde en mémoire)  │
             │      Auto-hibernation après 15 min d'inactivité│
             │        Consommation RAM : ~2.5 à 4 Go        │
             └──────────────────────────────────────────────┘
```

1. **Mode Éteint** : Aucun processus ne tourne.
2. **Mode Hibernation (Défaut)** : `lazymc.service` écoute sur le port 25565. Lorsque le premier joueur tente de se connecter, `lazymc` maintient la connexion TCP ouverte, lance le script de réveil `minecraft-wake.sh`, démarre `minecraft.container` sous Podman, attend que le port interne 25566 soit prêt, puis relaye la session de jeu de manière totalement transparente sans déconnecter le joueur.
3. **Auto-hibernation** : Dès que le serveur est vide depuis 15 minutes, `lazymc` ordonne l'arrêt du conteneur Java, libérant instantanément ~2.5 à 4 Go de RAM pour les autres services du homelab (Jellyfin, Ente, Seafile).
