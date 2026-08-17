# Guide des Modules Fonctionnels de ChiPanel

Ce document détaille le fonctionnement interne, les algorithmes et les interfaces de chaque sous-système de ChiPanel.

---

## 1. Acteur RCON Multiplexé & Résilience

### Problématique
Minecraft implémente le protocole RCON au-dessus de TCP, mais le serveur de jeu ferme brutalement les connexions en cas de surcharge ou lors de l'arrêt du conteneur. Ouvrir une socket par requête Web conduit à l'épuisement des descripteurs de fichiers (*file descriptors*) et à des crashs de thread.

### Solution ChiPanel
Le module `src/rcon/actor.rs` isole la communication RCON dans une boucle Tokio dédiée :
- **Multiplexage de commandes** : Les requêtes provenant de l'API REST, du hub WebSocket ou du collecteur de télémétrie sont empilées dans une queue FIFO `mpsc` non-bloquante.
- **Récupération automatique sur erreur** : Si la socket TCP est interrompue (ex: redémarrage du conteneur), l'acteur entre dans un état de reconnexion cyclique avec un intervalle de 3 secondes. Pendant ce temps, les requêtes entrantes reçoivent un retour immédiat `ServerOffline` sans bloquer le serveur HTTP Axum.

---

## 2. Console & Streaming de Logs avec Export mclo.gs

### Gestion du Flux de Logs
- **Buffer en mémoire tournant** : Le frontend conserve jusqu'à 5 000 lignes de logs dans un store réactif optimisé pour le rendu virtualisé du DOM.
- **Caviardage automatique des données sensibles** : Avant tout partage ou export, le module `src/routes/logs.rs` analyse le flux avec des expressions régulières pour masquer :
  - Les adresses IP publiques et IPv6.
  - Les jetons d'authentification et mots de passe RCON.
  - Les clés de déchiffrement et chemins de répertoires absolus de l'hôte.
- **Intégration API v1 `mclo.gs`** : Envoi direct du journal filtré vers `https://api.mclo.gs/1/log` avec récupération instantanée de l'URL de partage et du rapport d'analyse d'erreurs (*insights*).

---

## 3. Config Diff Viewer (`src/routes/diff.rs`)

### Comparaison Linéaire Différentielle
ChiPanel intègre la crate Rust `similar` pour comparer l'état en mémoire d'un fichier de configuration (`server.properties`, `paper-world-defaults.yml`, `purpur.yml`) avec sa version actuelle sur disque.
- **Calcul du Diff** : L'algorithme de Myers produit une liste d'opérations d'édition (`Insert`, `Delete`, `Equal`).
- **Prévisualisation interactive** : L'interface affiche les lignes modifiées en surbrillance (vert pour les ajouts, rouge pour les suppressions) et permet d'annuler ou de valider les changements avant écriture sur le disque SSD.

---

## 4. Diagnostic BDD & Purgeur SQL (`src/routes/database.rs`)

### Analyse de l'Empreinte Disque
Sur les serveurs Minecraft de longue durée, les plugins d'historique (CoreProtect) et de permissions (LuckPerms) accumulent des millions de lignes pouvant saturer le disque.
- **Inspection SQLite** : ChiPanel inspecte `plugins/CoreProtect/database.db` et calcule le poids réel des tables sans verrouiller la base de données.
- **Purge RCON sécurisée** : Les actions de maintenance sont déclenchées via les commandes natives du plugin (`/co purge t:30d`, `/lp prune 30`) pour préserver l'intégrité référentielle des données et éviter les corruptions de fichiers `.db-wal`.

---

## 5. Assistant Cross-Play Bedrock & Geyser (`src/routes/geyser.rs`)

### Interconnexion Java / Bedrock
Pour permettre aux joueurs sur smartphones, tablettes et consoles de rejoindre le serveur Java :
- **Contrôle d'intégrité des composants** : ChiPanel vérifie la présence du plugin `Geyser-Spigot.jar`, de l'extension `Floodgate.jar`, ainsi que la présence de la clé cryptographique partagée `key.pem`.
- **Validation réseau UDP 19132** : Vérifie que le port Bedrock standard est correctement mappé et accessible sur l'hôte.
- **Générateur de Configuration** : Assistant guidé permettant de générer automatiquement un fichier `config.yml` optimisé pour le homelab.

---

## 6. Spark Profiler & Flamegraphs (`src/routes/tools.rs`)

### Échantillonnage de Performances JVM
- **Déclencheur d'échantillonnage** : Démarre le sampler CPU Spark via RCON (`/spark sampler --viewer`).
- **Extraction automatique d'URL** : Le backend analyse le retour textuel de la commande via une regex (`https://spark\.lucko\.me/[a-zA-Z0-9]+`) et renvoie directement le lien cliquable vers le Flamegraph interactif.
- **Bilan de santé JVM** : Interroge les statistiques de la JVM (espace mémoire Old Gen, Young Gen, fréquence de Garbage Collection).

---

## 7. Moteur de Sauvegardes Avancées & Snapshots (`src/minecraft/server_backup.rs`)

### Fonctionnalités Clés
- **Périmètres sélectionnables (*Scopes*)** :
  - `full` : Intégralité du dossier serveur.
  - `world_only` : Uniquement les dossiers de mondes (`world`, `world_nether`, `world_the_end`).
  - `configs_only` : Fichiers `.yml`, `.json`, `.properties` et dossiers de configuration des plugins.
- **Règles d'exclusion intelligentes** : Exclusion automatique des fichiers temporaires volumineux (`*.log.gz`, `cache/*`, `dynmap/web/tiles/*`, `backups/*`).
- **Calcul d'intégrité SHA-256** : Chaque archive compressée est hachée à la volée pour garantir la détection de toute corruption.
- **Rétention automatique** : Suppression automatique des sauvegardes les plus anciennes au-delà du quota configuré (ex: conserver les 10 dernières sauvegardes).
- **Export distant S3 / MinIO** : Téléversement direct des archives vers un stockage objet hors-site.

---

## 8. Registre d'Audit Immuable (`src/audit/mod.rs`)

### Traçabilité Complète des Actions Administrateur
- **Stockage Append-Only** : Toutes les actions d'administration (création/restauration de sauvegarde, purge de base de données, démarrage/arrêt du serveur, modification de permissions) sont consignées dans `/app/data/audit_log.jsonl`.
- **Structure Événementielle** : Chaque entrée comprend un identifiant unique, un horodatage précis, le nom de l'utilisateur, la catégorie d'action, le statut de l'opération (`SUCCESS` / `FAILED`) et les détails JSON des paramètres modifiés.
- **Export CSV Conforme** : Possibilité de télécharger l'historique complet pour archivage ou audit de sécurité.

---

## 9. Supervision lazymc & Auto-Hibernation

### Économie d'Énergie et de Ressources
- **Proxy TCP Léger sur 25565** : Intercepte les pings de la liste de serveurs Minecraft sans réveiller le conteneur Java.
- **Réveil Transparent** : À la réception d'un paquet de handshake de connexion, `lazymc` démarre `minecraft.container` via le wrapper `minecraft-wake.sh` et synchronise le joueur dès que le serveur Java écoute sur le port 25566.
- **Mise en veille automatique** : Arrête le conteneur Java après 15 minutes sans joueur connecté pour libérer la RAM du serveur.
