# ChiPanel — Product Specification & Strategy

> **Statut du document :** Source de vérité produit pour **ChiPanel**. Ce document définit la vision stratégique, les utilisateurs cibles, les piliers de performance, l'architecture produit et la feuille de route.

---

## 🧭 Vision Produit & Positionnement

**ChiPanel** est la console de gestion de serveurs de jeux vidéo de nouvelle génération, ultra-légère, ultra-rapide, dark-mode first et conçue pour une efficience énergétique et matérielle maximale.

Contrairement aux panels existants (Pterodactyl, AMP, Crafty Controller, PufferPanel) qui constituent de lourdes suites logicielles consommant 400 Mo à 1 Go de mémoire vive pour faire tourner leurs propres dépendances (PHP-FPM, NodeJS, JVM, Python, daemons Docker, bases PostgreSQL/MySQL), ChiPanel redéfinit l'état de l'art :
- Un **binaire unique compilé en Rust (Axum + Tokio)** et une interface **SvelteKit / Svelte 5 (Runes)**.
- Une empreinte mémoire inférieure à **25 Mo de RAM**.
- Une politique stricte de **zéro gaspillage de ressources à vide (0% CPU idle)** via l'hibernation automatique.
- Une accessibilité absolue permettant à un **novice complet sans aucune compétence technique** de déployer et administrer un serveur de jeu en 1 clic, tout en offrant aux **power-users homelab** une granularité de contrôle bas-niveau sans compromis.

---

## ⚡ Différenciateur Clé N°1 : Performance & Optimisation Extrême

L'optimisation n'est pas une option dans ChiPanel, c'est son ADN fondateur :

### 1. Empreinte Mémoire Infime (< 25 Mo de RAM)
- **Backend Rust Axum & Tokio :** Binaire natif hautement optimisé, compilé en mode release avec LTO (*Link-Time Optimization*) et strip des symboles.
- **Zéro Base de Données Externe :** Aucun daemon PostgreSQL, MariaDB ou Redis n'est nécessaire. ChiPanel orchestre son état via des structures en mémoire vive sécurisées (`RwLock`, `Arc`) et une persistance atomique sur disque en fichiers JSON avec fsync garanti.
- **Distroless Runtime :** Image conteneur minimaliste (~50 Mo au total, frontend statique embarqué inclus).

### 2. Frontend Svelte 5 Instantané & Zéro Latence UI
- **Runes Svelte 5 (`$state`, `$derived`, `$props`, `$effect`) :** Réactivité granulaire au niveau du DOM sans surcharge de Virtual DOM.
- **Chargement initial < 100 ms & Navigation instantanée :** Bundle JavaScript minimaliste sans frameworks CSS lourds.
- **Temps de réponse API < 5 ms :** Handlers Axum asynchrones non-bloquants, communication WebSocket bidirectionnelle ultra-rapide (`/ws`).
- **Zéro CLS (*Cumulative Layout Shift*) :** Squelettes de chargement géométriquement identiques aux widgets finaux.

### 3. Zéro Gaspillage CPU à Vide (0% Idle) & Hibernation Intelligente (`lazymc`)
- **Hibernation automatique du serveur de jeu :** Après 15 minutes sans joueur connecté, le conteneur du serveur de jeu est endormi automatiquement.
- **Libération de 2,5 à 8 Go de RAM** sur la machine hôte et **0% d'utilisation CPU**.
- **Proxy d'éveil TCP natif (`lazymc`) :** Maintient le port public (ex. `:25565`) en écoute avec seulement ~8 Mo de RAM. Dès qu'un joueur initie une tentative de connexion, la poignée de main TCP est retenue, le conteneur est réveillé en quelques secondes et la session est transférée sans déconnexion.

---

## 👥 Utilisateurs Cibles & Expérience à Double Niveau

ChiPanel est pensé pour combler le fossé historique entre simplicité d'usage et maîtrise technique :

### 👶 Niveau 1 : Le Grand Débutant & Novice Homelab (Zéro Ligne de Code)
- **Le problème résolu :** Configurer un serveur Minecraft moddé ou un serveur de jeu dédié sur Linux implique habituellement la maîtrise de SSH, Docker, des ports réseau, des arguments JVM (`-Xms`, `-Xmx`), des fichiers YAML et des permissions Linux (`chown`, `chmod`).
- **L'expérience ChiPanel :**
  - **Déploiement 1-Click :** Sélection simple du type de serveur (Vanilla, Paper, Fabric, Modpack) et version Mojang via une interface guidée visuelle.
  - **Résolution automatique des dépendances :** Installation et configuration automatiques des bibliothèques requises (Fabric API, spark, chunky, luckperms).
  - **Zéro friction réseau :** Détection automatique des ports, détection des conflits et assistant de diagnostic intégré.
  - **Partage et export d'erreurs en 1 clic :** Export anonymisé et nettoyé de toute donnée sensible vers `mclo.gs` pour obtenir de l'aide sur Discord ou les forums.

### 🧙‍♂️ Niveau 2 : L'Administrateur Homelab & Power-User
- **Profondeur d'ingénierie :**
  - Support natif des **Quadlets systemd user** (`~/.config/containers/systemd/*.container`) sous **Podman rootless** (UID 1000, sans privilèges root).
  - Gestion directe du cycle de vie via D-Bus (`zbus`) et socket Unix.
  - Console interactive RCON multiplexée avec historique de commandes, auto-complétion et file d'attente pour joueurs déconnectés.
  - Visualiseur de **diff de configuration Myers** avant enregistrement des modifications.
  - Inspection NBT binaire en direct des fichiers `playerdata/*.dat` (inventaires 2D, Ender Chest, effets de potions, coordonnées).
  - Gestionnaire de sauvegardes scopées (`full`, `world_only`, `configs_only`) compressées en **zstd** avec envoi direct vers stockage objet S3 / MinIO.

---

## 🌐 Support Cross-Plateforme & Multi-Runtimes

ChiPanel s'adapte à l'environnement de l'utilisateur :

1. **Podman Rootless & Quadlets systemd (Standard Recommandé) :**
   - Sécurité maximale par isolation des espaces de noms utilisateurs Linux (`subuid` / `subgid`).
   - Intégration transparente avec systemd pour le démarrage automatique au boot, la gestion des cgroups et les limites de mémoire/CPU.
2. **Socket Docker Standard (Compatibilité Universelle) :**
   - Prise en charge native du socket Docker standard (`/var/run/docker.sock` ou Docker rootless) pour fonctionner sur n'importe quelle distribution Linux, NAS ou serveur existant.
3. **Modèles d'Images Conteneurs Configurables :**
   - Support out-of-the-box des images de l'écosystème communautaire (`itzg/minecraft-server`), d'images personnalisées Alpine légères, et de wrappers d'hibernation (`lazymc`).

---

## 🎮 Spécialisation Dédiée aux Serveurs de Jeux (Game Servers)

ChiPanel n'est **pas un tableau de bord homelab généraliste** (comme Homarr ou Dashy) ni un gestionnaire de conteneurs multi-usages (comme Portainer) : c'est une suite logicielle dédiée aux serveurs de jeux vidéo.

```
┌────────────────────────────────────────────────────────────────────────┐
│                        ChiPanel Engine Core                            │
│           (Axum HTTP + WebSocket Hub + JSON State + Backup)            │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
               ┌────────────────────┴────────────────────┐
               ▼                                         ▼
┌──────────────────────────────┐          ┌──────────────────────────────┐
│  Minecraft Driver (Premier)  │          │    Modular GameDriver Trait  │
│  - 11 Server Engines         │          │    - Palworld                │
│  - Mojang & Modrinth API     │          │    - Valheim                 │
│  - NBT & Inventory Inspector │          │    - Terraria                │
│  - Bedrock Geyser Cross-play │          │    - Enshrouded              │
│  - lazymc Hibernation Proxy  │          │    - Ark: Survival Ascended  │
└──────────────────────────────┘          └──────────────────────────────┘
```

1. **Minecraft en 1er citoyen de classe :**
   - Support complet de Java Edition et Bedrock Edition (cross-play Geyser/Floodgate automatisé).
   - Gestion fine des 11 moteurs de référence (Paper, Purpur, Folia, Fabric, Forge, NeoForge, Quilt, Spigot, Mohist, Arclight, Vanilla).
   - Intégration native des catalogues Modrinth v2 et CurseForge (recherche, téléchargement avec hash SHA-512, détection des mises à jour).
   - Synchronisation et maintenance automatisée des outils opérationnels (`spark`, `chunky`, `luckperms`).
   - Distribution automatique de packs de textures serveur (`/api/public/resourcepack/:filename`) avec calcul de hash SHA-1 et injection dans `server.properties`.
2. **Architecture Modulaire Multi-Jeux (`GameDriver`) :**
   - Abstraction de l'interface de pilotage permettant de brancher de futurs pilotes de jeux dédiés (Palworld, Valheim, Terraria, Enshrouded, etc.) tout en réutilisant le moteur de télémétrie, la gestion des conteneurs, le système de diff et les sauvegardes.

---

## 🖥️ Feuille de Route & Vision Desktop Locale (Tauri / Localhost)

L'un des axes majeurs de l'évolution de ChiPanel est son packaging en **application desktop autonome** :

```
┌────────────────────────────────────────────────────────────────────────┐
│                     ChiPanel Desktop App (Tauri)                       │
│                                                                        │
│  ┌─────────────────────────────────┐   ┌────────────────────────────┐  │
│  │ Svelte 5 Native WebView UI      │   │ Embedded Rust Micro-Engine │  │
│  │ (Tauri Webview / IPC)           │<─>│ (Axum Local Controller)    │  │
│  └─────────────────────────────────┘   └─────────────┬──────────────┘  │
└──────────────────────────────────────────────────────┼─────────────────┘
                                                       │
                           ┌───────────────────────────┴───────────────────────────┐
                           ▼                                                       ▼
             Mode Localhost (PC Joueur)                              Mode Remote (Homelab / VPS)
      - Serveur de jeu local en 1 clic                         - Connexion distante sécurisée
      - Zéro configuration Linux requise                       - Pilotage via API Key / Tailscale
```

- **Objectif :** Permettre à n'importe quel joueur sur Windows, macOS ou Linux de créer et lancer un serveur de jeu directement sur sa machine pour jouer en LAN ou entre amis via Tailscale/ZeroTier en 1 clic (« Plug & Play »), sans avoir besoin de louer un serveur ou de maîtriser Linux.
- **Mode hybride :** L'application desktop pourra basculer d'une gestion locale (moteur embarqué) à la supervision d'une instance distante hébergée sur un serveur homelab.

---

## 🎨 Principes de Conception & Direction Artistique

ChiPanel adhère scrupuleusement au contrat de design défini dans [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) :

1. **Esthétique « Double-Bezel » / Machined Hardware :** Palette ardoise sombre tactile (`#0C0E14`, `#161922`, `#1F2330`), biseau intérieur 1px spéculaire, zéro noir OLED froid.
2. **Directives Anti-AI-Slop :** Zéro gradient textuel, zéro aura violette/néon artificielle, arrondis maîtrisés (8-12px), typographie technique nette (`Geist Mono`, `Inter`).
3. **Vérité en Télémétrie (*Truth in Telemetry*) :** Données réelles tabulaires (`tabular-nums`), zéro valeur fictive ou graphique statique trompeur.
4. **Color Scarcity & Accessibilité :** 90% d'interface monochrome reposante, 10% d'accents sémantiques calibrés, contraste conforme aux normes **WCAG AA/AAA (4.5:1 minimum)**.
5. **Micro-Interactions Tactiles (Emil Kowalski Standards) :** Retour physique `scale(0.97)` sur clic/appui, durées d'animation < 250 ms, courbes d'accélération fluides (`cubic-bezier(0.23, 1, 0.32, 1)`), transitions GPU-safe (`transform` et `opacity`).

---

## 📚 Références & Liens Utiles

| Document | Objet |
| :--- | :--- |
| [`AGENTS.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/AGENTS.md) | Contrat d'intervention et règles de gouvernance pour tout agent travaillant sur ChiPanel. |
| [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) | Spécification exhaustive du Design System (tokens CSS, palettes, anti-patterns, micro-interactions). |
| [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) | Guide technique pour le développement, tests, commandes et variables d'environnement. |
| [`docs/architecture.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/architecture.md) | Architecture logicielle détaillée, acteur RCON et supervision systemd/Podman. |
| [`docs/api-reference.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/api-reference.md) | Spécification complète des endpoints REST et protocoles WebSocket. |
| [`docs/modules-guide.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/modules-guide.md) | Guide d'implémentation des modules métier (NBT, Addons, Chunky, Backups, Audit). |
| [`docs/deployment-and-operations.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/deployment-and-operations.md) | Guide de déploiement conteneurisé et runbook d'exploitation. |
| [`docs/security.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/security.md) | Architecture de sécurité, modèle de menaces et isolation système. |

