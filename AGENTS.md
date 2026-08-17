# ChiPanel — Agent Instructions & Architectural Governance Contract

> **Source de vérité ChiPanel :** Ce fichier régit l'intégralité des règles d'intervention pour tout agent ou développeur travaillant sur le projet **ChiPanel** (Backend Rust Axum + Frontend SvelteKit / Svelte 5 + Packaging Conteneur & Desktop).

---

## 🚨 OBLIGATION PRÉALABLE ABSOLUE POUR TOUT AGENT

Avant toute intervention, modification de code, de style, d'architecture ou de documentation sur **ChiPanel** (dans `chipanel/` ou depuis la racine du repo `chiserv/`) :

1. **Lecture obligatoire de ce fichier [`AGENTS.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/AGENTS.md)**.
2. **Lecture obligatoire de [`PRODUCT.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/PRODUCT.md)** pour intégrer la vision produit, les cibles utilisateurs et le positionnement concurrentiel.
3. **Lecture obligatoire de [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md)** pour toute intervention touchant au frontend (`frontend/`).
4. **Consultation de [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md)** et de la suite technique [`docs/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/) pour l'implémentation backend, les routes API et les modèles de données.

---

## ⚡ DIFFÉRENCIATEUR CLÉ N°1 : PERFORMANCE & OPTIMISATION EXTRÊME

ChiPanel est conçu pour être le **panel de gestion de serveurs de jeux le plus léger, véloce et optimisé au monde**. Contrairement aux panels traditionnels (Pterodactyl, AMP, Crafty) qui consomment 400 Mo à 1 Go de RAM avec des runtimes lourds (NodeJS, Java, Python, PHP, bases SQL externes), ChiPanel respecte des standards stricts de sobriété :

1. **Empreinte mémoire infime (< 25 Mo de RAM) :**
   - Backend Rust pur compilé nativement (Axum 0.7 + Tokio async runtime + hyper).
   - Zéro dépendance de runtime lourd sur l'hôte, zéro base de données SQL externe (état persisté en fichiers JSON atomiques avec cache en mémoire).
   - Distroless/minimal scratch container runtime.
2. **Frontend Svelte 5 instantané & Zéro Gaspillage de Rendu :**
   - Architecture SPA réactive basée sur les Runes Svelte 5 (`$state`, `$derived`, `$props`, `$effect`).
   - Bundle ultra-compact, temps de chargement initial < 100 ms, transitions UI < 250 ms, zéro CLS (*Cumulative Layout Shift*).
   - Données tabulaires en chiffres à chasse fixe (`tabular-nums`), zéro rendu inutile.
3. **0% CPU gaspillé à vide & Hibernation Intelligente (`lazymc`) :**
   - Couplage natif avec le proxy `lazymc` (~8 Mo de RAM) : lorsque aucun joueur n'est connecté pendant 15 minutes, le conteneur du serveur de jeu est automatiquement stoppé.
   - Libération instantanée de **2,5 à 8 Go de RAM** et **0% d'utilisation CPU**, réveil transparent en millisecondes au premier handshake réseau TCP d'un joueur entrant.
4. **Latence API & Télémétrie Temps Réel Ultra-Basse (< 5 ms) :**
   - Temps de réponse HTTP moyen < 5 ms, flux WebSocket bidirectionnel (`/ws`) ultra-léger avec canal d'acteur RCON persistant et multiplexé.

---

## 🎯 ACCESSIBILITÉ DU NOVICE AU POWER-USER

ChiPanel résout le dilemme entre simplicité extrême et puissance technique en proposant une expérience à deux niveaux :

### 1. Pour les grands débutants & novices en homelab (Zero-Code Experience)
- **L'alternative la plus simple, rapide et intuitive :** Conçu pour quelqu'un qui n'a jamais touché une ligne de code, un terminal ou un fichier de configuration Linux.
- **Mode 1-Click Install & Presets Guidés :** Déploiement instantané de serveurs préconfigurés sans avoir à manipuler manuellement des ports, des variables d'environnement ou des fichiers YAML.
- **Gestion automatisée des dépendances :** Résolution et injection automatiques des chargeurs de mods (Fabric, Forge, NeoForge, Quilt, Paper, Purpur) et des outils essentiels (`spark`, `chunky`, `luckperms`, Fabric API).
- **Interface humaine et guidée :** Messages d'erreur explicites, wizards visuels de configuration, export de logs nettoyés en 1 clic vers `mclo.gs`.

### 2. Pour les administrateurs chevronnés & power-users
- **Profondeur technique sans compromis :** Intégration native des Quadlets systemd user (`~/.config/containers/systemd/`), contrôle direct via socket Podman/Docker, isolation rootless (subuid/subgid).
- **Contrôle bas-niveau granulaire :** Console interactive RCON avec coloration ANSI, file d'attente de commandes pour joueurs hors-ligne, inspection NBT binaire en direct (`playerdata/*.dat`), visualiseur de diff de configuration Myers (`similar`), cgroups CPU/RAM pinning, et sauvegardes scopées zstd avec transfert direct vers buckets S3 / MinIO.

---

## 🌐 SUPPORT CROSS-PLATEFORME & MULTI-MOTEURS / RUNTIMES

ChiPanel n'est pas enfermé dans un écosystème unique et prend en charge nativement les environnements modernes :

1. **Podman Rootless & Quadlets systemd (Prioritaire & Natif) :**
   - Exécution 100% sans privilèges root (UID 1000) via socket `/run/user/<UID>/podman/podman.sock` et D-Bus (`zbus`).
   - Supervision native via les unités systemd user et Quadlets déclaratifs.
2. **Docker Socket Standard (Compatibilité Étendue) :**
   - Prise en charge du socket Docker standard (`/var/run/docker.sock` ou socket rootless) pour une compatibilité immédiate avec tout hôte Linux standard, VPS ou machine de développement.
3. **Modèles d'images conteneurs configurables :**
   - Prise en charge transparente des images de référence (`itzg/minecraft-server`), d'images personnalisées Alpine minimalistes et de wrappers d'hibernation (`lazymc`).

---

## 🎮 SPÉCIALISATION SERVEURS DE JEUX (GAME SERVERS)

ChiPanel est **exclusivement dédié aux serveurs de jeux vidéo** (et non un dashboard homelab générique ni un gestionnaire de conteneurs généraliste) :

1. **Minecraft en citoyen de 1ère classe :**
   - Prise en charge complète Java Edition & Bedrock Edition (cross-play via Geyser/Floodgate auto-configuré).
   - Support des 11 moteurs majeurs : Paper, Purpur, Folia, Fabric, Forge, NeoForge, Quilt, Spigot, Mohist, Arclight, Vanilla.
   - Intégration native des API Modrinth v2 & CurseForge avec calcul SHA-512 d'intégrité, auto-hébergement de packs de textures serveur (`/api/public/resourcepack/:filename`) avec SHA-1 automatique.
   - Migration automatique inter-moteurs des structures de données de permissions (LuckPerms).
2. **Architecture extensible multi-jeux :**
   - Abstraction modulaire du cycle de vie serveur (`GameDriver` trait) permettant d'étendre ChiPanel à d'autres serveurs de jeux dédiés (Palworld, Valheim, Terraria, Enshrouded, Ark, etc.) tout en conservant la télémétrie, la gestion des sauvegardes et l'interface ultra-optimisée.

---

## 🖥️ VISION APPLICATION DESKTOP LOCALE (TAURI / LOCALHOST)

L'architecture de ChiPanel est rigoureusement découplée (API REST Axum + WebSocket stateless + SPA SvelteKit) afin de permettre son packaging futur en **application desktop autonome** :

- **Stack cible :** Tauri (Rust natif + WebView légère multiplateforme).
- **Cas d'usage :** Permettre à un joueur ou créateur de lancer, paramétrer et superviser un serveur de jeu tournant directement sur son PC local (Windows, macOS, Linux) en 1 clic (« Jouer entre amis sans prise de tête »), sans nécessiter de serveur dédié distant, d'accès SSH ou de configuration réseau complexe.
- **Mode hybride :** L'application desktop pourra soit piloter le serveur local (localhost), soit se connecter à distance à une instance ChiPanel sur un serveur homelab via API key / Tailscale.

---

## 🎨 RÈGLE ABSOLUE & NON-NÉGOCIABLE : DIRECTION ARTISTIQUE & DESIGN (`design.md`)

Avant toute intervention, modification, ajout de composant, page ou style sur le **frontend de ChiPanel** (`frontend/`) :

1. **Lecture obligatoire de [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) :** L'agent a l'**obligation stricte et préalable** de lire et d'appliquer l'intégralité des spécifications contenues dans ce document.
2. **Application rigoureuse de la DA ChiPanel :**
   - **Esthétique « Double-Bezel » / Machined Hardware :** Surfaces ardoise sombre (`#0C0E14` / `#161922` / `#1F2330`), biseau intérieur 1px spéculaire, zéro noir OLED froid `#000000`.
   - **Directives Anti-AI-Slop strictes :** Zéro gradient textuel, zéro aura néon/violette sans justification sémantique, zéro badge superflu, arrondis plafonnés à 8-12px (`--radius-md`/`--radius-lg`), zéro emoji dans l'UI (icônes Lucide/Phosphor cohérentes uniquement).
   - **Règle de Rareté de la Couleur (Color Scarcity) & Accessibilité WCAG AA/AAA :** 90% monochrome tactile, 10% accents sémantiques calibrés (3 rôles par couleur : `--accent-*-bg`, `--accent-*-text`, `--accent-*-solid`).
   - **Design Engineering & Micro-Interactions (Emil Kowalski Standards) :** Retour haptique physique `scale(0.97)` sur `:active`, jamais d'entrée depuis `scale(0)`, popovers origin-aware, zéro animation sur raccourcis clavier fréquents, durées UI < 250ms, courbes personnalisées (`--ease-out: cubic-bezier(0.23, 1, 0.32, 1)`), animations GPU-safe (`transform` et `opacity` uniquement).
   - **Vérité en Télémétrie (Truth in Telemetry) & Bento Grid sans vide :** Données réelles tabulaires (`tabular-nums`), squelettes fidèles sans CLS, gestion digne de l'hibernation (`lazymc`).
   - **Identité & Logo :** Intégration du logo officiel *Hexagone Stencil Chi-Rack* avec voyant LED réactif aux statuts du serveur.

---

## 🔄 RÈGLE DE VERSIONING & PUSH AUTOMATIQUE

- **Commit + push systématique :** Après toute modification de code ou de documentation dans `chipanel/`, commit immédiatement puis push : `git add -A && git commit -m "..." && git push`.
- N'attends jamais de confirmation explicite pour commiter et pousser les changements validés.
- **Sécurité :** Ne commite aucun secret en clair. Le quadlet réel `chipanel.container` est gitignoré ; seul `chipanel.container.example` est versionné.

---

## 🛠️ COMMANDES DE RÉFÉRENCE & BUILD

- **Frontend :** `npm run dev` (proxy vers `:25500`), `npm run build`, `npm run check` (`svelte-check`).
- **Backend :** `cargo run`, `cargo build --release`, `cargo test`.
- **Conteneur :** `podman build -t localhost/chipanel:latest -f Containerfile .` (toujours utiliser `--no-cache` si le frontend a été modifié).
- **Déploiement Host-007 :** `podman save` -> `scp` -> `podman load` -> `systemctl --user restart chipanel`.

---

## 📚 VOIR AUSSI

| Fichier | Rôle |
| :--- | :--- |
| [`PRODUCT.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/PRODUCT.md) | **Vision produit, positionnement, utilisateurs cibles, piliers de performance et roadmap desktop.** |
| [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) | **Spécification complète du design system, tokens CSS, anti-patterns et design engineering.** |
| [`docs/onboarding-spec.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/onboarding-spec.md) | **Spécification détaillée du parcours d'onboarding débutant (Zero-Code 1-Click) et architecture Desktop.** |
| [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) | Détails opérationnels de l'architecture Rust/Axum, RCON, Podman/Docker socket et gestion Minecraft. |
| [`docs/architecture.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/architecture.md) | Architecture interne approfondie, runtime Tokio, acteur RCON et cycle de vie lazymc. |
| [`docs/api-reference.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/api-reference.md) | Référence exhaustive des 77 routes API REST et des événements WebSocket. |
| [`docs/modules-guide.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/modules-guide.md) | Guide exhaustif des modules (NBT, plugins, diff Myers, backups S3, audit log). |
| [`docs/deployment-and-operations.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/deployment-and-operations.md) | Guide de déploiement conteneurisé, variables d'environnement et runbook d'exploitation. |
| [`docs/security.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/security.md) | Modèle de sécurité, RBAC, hash Argon2id, mitigation SSRF et isolation rootless. |

