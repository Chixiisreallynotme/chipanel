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

## 🤝 ALIGNEMENT & CONSEIL (COMMENT ON TRAVAILLE ENSEMBLE)

Avant de foncer sur une demande, **aligne-toi avec moi**. Mon « fais X » est souvent le point de départ d'une idée, pas une spécification complète. Pose les questions nécessaires pour cerner le **résultat que je vise** (ce que ça doit faire, dans quel contexte, quelles contraintes), puis **reformule** ce que tu as compris avant de coder — ça évite de produire un résultat hors sujet et de devoir tout rectifier après.

Tu es ici **conseiller et expert, pas simple exécutant** :

- Si ma demande est ambiguë ou incomplète, **demande des précisions** au lieu de deviner.
- Si une solution te semble meilleure ou plus simple que ce que j'ai dit, **propose-la avec ta justification**.
- **Challenge ce que je dis, par défaut et sur toute demande non triviale.** Ne prends pas mes idées pour argent comptant : j'ai parfois de mauvaises idées, et parfois les meilleures — c'est à toi de faire le tri. Signale les risques, les contradictions et les angles morts que je n'ai pas vus, avec ton avis d'expert, avant d'exécuter.
- **Outil dédié obligatoire pour les questions (`ask_question`) :** Quand tu as des questions à me poser (clarifications, arbitrages, choix techniques ou de design), utilise TOUJOURS l'outil interactif dédié (`ask_question`). Ne génère JAMAIS de liste de questions en texte brut / markdown dans le chat en me forçant à taper manuellement une liste de réponses. Rends systématiquement la démarche interactive avec des options claires et sélectionnables.

Le réflexe attendu, dans cet ordre : **écouter → questionner (via `ask_question`) → proposer → confirmer le plan → exécuter.** « Confirmer le plan » = je valide ta reformulation du résultat visé et tes réserves ; ce n'est **pas** un feu vert à attendre pour chaque action : commit/push reste systématique. Une fois le plan validé, exécute sans re-challenger, sauf élément nouveau découvert en cours de route.

**Anti-exemples (à éviter)**

- Je dis « change le frontend » et tu refais tout le style sans me montrer d'abord ce que tu comprends de mon intention, ni avoir relu `design.md`.
- Je dis « ajoute un endpoint API » et tu écris un handler complet sans demander le contrat de données, les erreurs attendues, ni la route concernée.
- Poser une série de questions sous forme de liste de texte / puces dans la réponse au lieu d'utiliser l'outil dédié `ask_question`.
- Je propose une solution et tu l'implémentes sans me signaler qu'elle est fragile, alors qu'une meilleure option existait.
- Je demande un choix risqué (exposer un secret, supprimer une limite de RAM, casser une API existante) et tu exécutes sans me signaler le danger ni proposer une alternative plus sûre.

---

## ⚡ DIFFÉRENCIATEUR CLÉ N°1 : PERFORMANCE & OPTIMISATION EXTRÊME

ChiPanel est conçu pour être le **panel de gestion de serveurs de jeux le plus léger, véloce et optimisé au monde**. Contrairement aux panels traditionnels (Pterodactyl, AMP, Crafty) qui consomment 400 Mo à 1 Go de RAM avec des runtimes lourds (NodeJS, Java, Python, PHP, bases SQL externes), ChiPanel respecte des standards stricts de sobriété :

1. **Empreinte mémoire infime (< 25 Mo de RAM) :**
   - Backend Rust pur compilé nativement (Axum 0.7 + Tokio async runtime + Hyper 1.0).
   - Zéro dépendance de runtime lourd sur l'hôte, zéro base de données SQL externe (état persisté en fichiers JSON atomiques avec cache en mémoire et SQLite WAL pour les logs).
   - Image de conteneur minimale/distroless sans couches superflues.
2. **Frontend Svelte 5 instantané & Zéro Gaspillage de Rendu :**
   - Architecture SPA réactive basée sur les Runes Svelte 5 (`$state`, `$derived`, `$props`, `$effect`).
   - Bundle ultra-compact, temps de chargement initial < 100 ms, transitions UI < 250 ms, zéro CLS (*Cumulative Layout Shift*).
   - Données tabulaires en chiffres à chasse fixe (`tabular-nums`), zéro re-rendu inutile.
3. **0% CPU gaspillé à vide & Hibernation Intelligente (`lazymc`) :**
   - Couplage natif avec le proxy `lazymc` (~8 Mo de RAM) : lorsque aucun joueur n'est connecté pendant 15 minutes, le conteneur du serveur de jeu est automatiquement stoppé.
   - Libération instantanée de **2,5 à 8 Go de RAM** et **0% d'utilisation CPU**, réveil transparent en millisecondes au premier handshake réseau TCP d'un joueur entrant sur le port 25565.
4. **Latence API & Télémétrie Temps Réel Ultra-Basse (< 5 ms) :**
   - Temps de réponse HTTP moyen < 5 ms, flux SSE / WebSocket bidirectionnel (`/ws`) ultra-léger avec canal d'acteur RCON persistant et multiplexé.

---

## 🎯 ACCESSIBILITÉ DU NOVICE AU POWER-USER (DUAL-MODE UX)

ChiPanel résout le dilemme entre simplicité extrême et puissance technique en proposant une expérience à deux niveaux :

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           PARADIGME DUAL-MODE CHIPANEL                          │
├────────────────────────────────────────┬────────────────────────────────────────┤
│        MODE NOVICE (1-CLICK SETUP)     │       MODE POWER-USER (EXPERT / DEVOPS)│
├────────────────────────────────────────┼────────────────────────────────────────┤
│ • Parcours guidé en 4 étapes (/setup)  │ • Accès direct Quadlet systemd         │
│ • Zéro ligne de code / Zéro YAML       │ • Console RCON multiplexée brute       │
│ • Recommandation automatique de RAM    │ • Paramétrage flags JVM & cgroups      │
│ • EULA accepté en 1 clic               │ • Inspection binaire NBT & diff Myers  │
│ • Veille lazymc activée par défaut     │ • Sauvegardes zstd vers S3 / MinIO     │
└────────────────────────────────────────┴────────────────────────────────────────┘
```

- **Bascule instantanée (0ms) :** Raccourci clavier `Alt+M` (ou `Option+M`) ou sélecteur hardware [`ModeSwitch.svelte`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/frontend/src/lib/components/onboarding/ModeSwitch.svelte) dans la barre d'outils.

### 1. Pour les grands débutants & novices en homelab (Zero-Code Experience)
- **L'alternative la plus simple, rapide et intuitive :** Conçu pour quelqu'un qui n'a jamais touché une ligne de code, un terminal ou un fichier de configuration Linux.
- **Mode 1-Click Install & Presets Guidés ([`routes/setup/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/frontend/src/routes/setup/+page.svelte)) :** Déploiement instantané de serveurs préconfigurés sans avoir à manipuler manuellement des ports, des variables d'environnement ou des fichiers YAML.
- **Gestion automatisée des dépendances :** Résolution et injection automatiques des chargeurs de mods (Fabric, Forge, NeoForge, Quilt, Paper, Purpur) et des outils essentiels (`spark`, `chunky`, `luckperms`, Fabric API).
- **Interface humaine et guidée :** Messages d'erreur explicites, wizards visuels de configuration, export de logs nettoyés en 1 clic vers `mclo.gs`.

### 2. Pour les administrateurs chevronnés & power-users
- **Profondeur technique sans compromis :** Intégration native des Quadlets systemd user (`~/.config/containers/systemd/`), contrôle direct via socket Podman/Docker, isolation rootless (subuid/subgid).
- **Contrôle bas-niveau granulaire :** Console interactive RCON avec coloration ANSI, file d'attente de commandes pour joueurs hors-ligne, inspection NBT binaire en direct (`playerdata/*.dat`), visualiseur de diff de configuration Myers (`similar`), cgroups CPU/RAM pinning, et sauvegardes scopées zstd avec transfert direct vers buckets S3 / MinIO.

---

## 🌐 ARCHITECTURE MULTI-MOTEURS & MULTI-JEUX

ChiPanel repose sur une architecture découplée et extensible dans [`chipanel/backend/src/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/backend/src/) :

### 1. Abstraction Multi-Moteurs de Conteneurs (`src/container/`)
- **Trait [`ContainerEngine`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/backend/src/container/engine.rs) :**
  - Opérations asynchrones : `get_status`, `get_stats`, `start_container`, `stop_container`, `restart_container`, `get_logs`.
  - **`PodmanEngine` :** Support natif Podman rootless (socket `/run/user/<uid>/podman/podman.sock` ou `PODMAN_SOCKET`) et orchestration des Quadlets systemd via D-Bus (`zbus`).
  - **`DockerEngine` :** Support standard du socket Docker (`/var/run/docker.sock` ou `DOCKER_HOST`) avec démultiplexage des en-têtes de logs stdout/stderr.
  - **`AutoDetector` :** Détection automatique intelligente du runtime disponible ou configuration forcée via `CONTAINER_ENGINE=podman|docker|auto`.
  - **Profils de Conteneurs :** Support des profils `LazymcCustom`, `ItzgMinecraft` (`itzg/minecraft-server` avec variables `EULA`, `TYPE`, `VERSION`, `MEMORY`), et `GenericOci`.

### 2. Abstraction Game Drivers Multi-Jeux (`src/engine/`)
- **Trait [`GameDriver`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/backend/src/engine/driver.rs) :**
  - Cycle de vie, télémétrie, exécution de commandes, gestion des joueurs et sauvegardes unifiées.
  - **`MinecraftDriver` :** Moteur complet pour Minecraft Java & Bedrock (RCON, plugins, modpacks, mondes, hibernation lazymc).
  - **`PalworldDriver` / `ValheimDriver` :** Modules prêts pour les serveurs dédiés multijoueurs émergents.
  - **`GameEngineRegistry` :** Registre dynamique thread-safe pour router les instances de jeux actives.

---

## 🖥️ VISION APPLICATION DESKTOP LOCALE (TAURI / LOCALHOST)

L'architecture de ChiPanel est rigoureusement découplée (API REST Axum + WebSocket stateless + SPA SvelteKit) afin de permettre son packaging en **application desktop autonome** :

- **Stack cible :** Tauri v2 (Rust natif + WebView légère multiplateforme).
- **Cas d'usage :** Permettre à un joueur ou créateur de lancer, paramétrer et superviser un serveur de jeu tournant directement sur son PC local (Windows, macOS, Linux) en 1 clic (« Jouer entre amis sans prise de tête »), sans nécessiter de serveur dédié distant, d'accès SSH ou de configuration réseau complexe.
- **Mode hybride :** L'application desktop peut soit piloter le runtime local (Podman Desktop, Docker Desktop, Java local), soit se connecter à distance à une instance ChiPanel sur un serveur homelab via API key / Tailscale.

---

## 🎨 RÈGLE ABSOLUE & NON-NÉGOCIABLE : DIRECTION ARTISTIQUE & DESIGN (`design.md`)

Avant toute intervention sur le **frontend de ChiPanel** (`frontend/`) :

1. **Lecture obligatoire de [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md)**.
2. **Application rigoureuse de la DA ChiPanel :**
   - **Esthétique « Double-Bezel » / Machined Hardware :** Surfaces ardoise sombre (`#0C0E14` / `#161922` / `#1F2330`), biseau intérieur 1px spéculaire (`inset 0 1px 0 rgba(255,255,255,0.09)`), zéro noir OLED froid `#000000`.
   - **Directives Anti-AI-Slop strictes :** Zéro gradient textuel, zéro aura néon/violette sans justification sémantique, zéro badge superflu, arrondis plafonnés à 8-12px (`--radius-md`/`--radius-lg`), zéro emoji dans l'UI (icônes Lucide/Phosphor cohérentes uniquement).
   - **Règle de Rareté de la Couleur (Color Scarcity) & Accessibilité WCAG AA/AAA :** 90% monochrome tactile, 10% accents sémantiques calibrés (3 rôles par couleur : `--accent-*-bg`, `--accent-*-text`, `--accent-*-solid`).
   - **Design Engineering & Micro-Interactions (Emil Kowalski Standards) :** Retour haptique physique `scale(0.97)` sur `:active`, jamais d'entrée depuis `scale(0)`, popovers origin-aware, zéro animation sur raccourcis clavier fréquents, durées UI < 250ms, courbes personnalisées (`--ease-out: cubic-bezier(0.23, 1, 0.32, 1)`), animations GPU-safe (`transform` et `opacity` uniquement).
   - **Vérité en Télémétrie (Truth in Telemetry) & Bento Grid sans vide :** Données réelles tabulaires (`tabular-nums`), squelettes fidèles sans CLS, gestion digne de l'hibernation (`lazymc`).
   - **Identité & Logo :** Intégration du logo officiel *Hexagone Stencil Chi-Rack* avec voyant LED réactif aux statuts du serveur.

---

## 🔄 RÈGLE DE VERSIONING & PUSH AUTOMATIQUE

**Code : commit + push à chaque changement.** ChiPanel est versionné sur GitHub (`github.com/Chixiisreallynotme/chipanel`). Après **toute** modification de code ou de documentation dans `chipanel/`, commit immédiatement puis push : `git add -A && git commit -m "..." && git push`. Ne laisse jamais un changement non commité derrière toi.

**Commit SANS demander mon accord : n'attends jamais une confirmation explicite pour commiter/pousser du code — c'est systématique et automatique après chaque changement.** Ne commite jamais de secrets (voir § Gotchas) — le quadlet réel `chipanel.container` (mots de passe en clair) est gitignoré, seule la version masquée `chipanel.container.example` est versionnée.

---

## 🧠 MINE DE SAVOIR CHIPANEL (APPRENTISSAGE CONTINU)

Chaque session de travail sur ChiPanel doit laisser une trace technique utile. Ce projet a ses propres spécificités qu'il faut documenter en continu :

- **Ne jamais avancer silencieusement :** Dès qu'un piège est contourné, qu'un comportement inattendu du build Rust/Svelte est observé, qu'une subtilité Axum/Tokio/SQLite WAL est découverte, documente-la immédiatement — soit dans [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) si c'est opérationnel, soit dans le doc concerné de [`docs/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/).
- **Règle de remplissage systématique :** Dès qu'un comportement nouveau est observé (compilation, déploiement, runtime, design engineering), enrichis la doc sans attendre qu'on te le demande.
- **Périmètre des sujets à documenter :** Pièges build multi-stage Containerfile, subtilités des Runes Svelte 5, comportements de l'acteur RCON sous charge, edge-cases du socket Podman, particularités du WAL SQLite en conteneur, timing des animations GPU-safe, etc.
- **Objectif :** Fournir un *skill* d'expert complet permettant à tout agent futur de comprendre instantanément les moindres détails de ChiPanel, d'éviter tous les écueils et de le maintenir avec une précision absolue.

---

## ⚠️ GOTCHAS DE DÉPLOIEMENT (À NE PAS RATER)

Ces pièges ont déjà causé des surprises : mémorise-les.

- **Build frontend non pris en compte :** `podman build` cache les layers ; si tu as modifié le frontend SvelteKit, utilise **obligatoirement `--no-cache`** : `podman build --no-cache -t localhost/chipanel:latest -f Containerfile .`. Sans ça, l'ancienne version du bundle JS est embarquée silencieusement.
- **`podman load` ne redémarre PAS le conteneur :** Après `podman load`, le service tourne encore avec l'ancienne image. Toujours enchaîner avec `systemctl --user restart chipanel`.
- **`daemon-reload` obligatoire après édition quadlet :** Toute modification de `chipanel.container` sur le serveur nécessite `systemctl --user daemon-reload` PUIS `restart` — sinon la modif est **inerte** (piège réel sur `MemoryMax` et `Environment`).
- **Un `restart` systemd ≠ `rm` + `create` :** Pour injecter des changements DNS/réseau dans le conteneur, il faut `podman rm chipanel` puis `start`, pas juste `restart`.
- **Builder toujours en `localhost/chipanel:latest` :** Le quadlet référence ce tag exactement. Tout autre tag est ignoré au chargement.
- **Secrets :** Ne jamais faire figurer un secret (mot de passe admin, token API) dans un commit, dans les logs de build, ou dans une variable d'environnement loggée. Voir [`docs/security.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/security.md).

---

## 🛠️ COMMANDES DE RÉFÉRENCE & BUILD

- **Frontend :**
  - Dév : `npm run dev` (proxy vers `:25500`)
  - Build : `npm run build`
  - Validation linter & types : `npm run check` (`svelte-check`)
- **Backend :**
  - Dév : `cargo run`
  - Tests : `cargo test`
  - Build release : `cargo build --release`
  - Validation compilation : `cargo check`
- **Conteneur :**
  - Build local : `podman build -t localhost/chipanel:latest -f Containerfile .` (toujours utiliser `--no-cache` si le frontend a été modifié).
- **Déploiement Host-007 :**
  - `podman save localhost/chipanel:latest | ssh ... "podman load" && ssh ... "systemctl --user restart chipanel"`

---

## 🗺️ CARTOGRAPHIE DU CODEBASE & GUIDE DES MODULES

```
chipanel/
├── backend/                             # Serveur Rust Axum (Binaire natif < 25 Mo RAM)
│   ├── Cargo.toml                       # Dépendances (Axum 0.7, Tokio, Hyper, Serde, Zbus, SQLite)
│   └── src/
│       ├── main.rs                      # Point d'entrée, initialisation Tokio & bind :25500
│       ├── config.rs                    # Parsing variables d'environnement & profils
│       ├── error.rs                     # Énumération AppError & conversion en réponses HTTP
│       ├── container/                   # 🚀 Multi-Runtime Abstraction (Podman / Docker / Quadlets)
│       │   ├── engine.rs                # Trait asynchrone ContainerEngine
│       │   ├── podman.rs                # Implémentation Podman rootless (socket + DBus systemd)
│       │   ├── docker.rs                # Implémentation Docker standard (socket + log demux)
│       │   ├── detector.rs              # Auto-détection intelligente du runtime actif
│       │   └── profiles.rs              # Modèles d'images (itzg/minecraft-server, custom lazymc)
│       ├── engine/                      # 🎮 Game Drivers Modulaires (Multi-Jeux)
│       │   ├── driver.rs                # Trait unifié GameDriver (cycle de vie, télémétrie, RCON)
│       │   ├── minecraft.rs             # Driver Minecraft (Paper, Fabric, Purpur, Bedrock)
│       │   ├── palworld.rs              # Driver Palworld Dedicated Server
│       │   ├── valheim.rs               # Driver Valheim Dedicated Server
│       │   └── registry.rs              # Registre dynamique thread-safe des drivers
│       ├── minecraft/                   # Sous-systèmes approfondis Minecraft
│       │   ├── player.rs                # Parsing playerdata NBT, inventaires, stats, skins
│       │   ├── worlds.rs                # Gestion des dimensions, sauvegardes & exports
│       │   ├── plugins.rs & modpacks.rs # Intégration Modrinth v2 & CurseForge API
│       │   ├── server_backup.rs         # Sauvegardes atomiques zstd & upload S3 / MinIO
│       │   └── server_properties.rs     # Parser & diff Myers de configuration
│       ├── rcon/                        # Acteur RCON multiplexé persistant
│       ├── websocket/                   # Streaming temps réel SSE / WebSocket (/ws)
│       ├── routes/                      # Handlers HTTP Axum (77 endpoints REST)
│       ├── auth/ & audit/               # Sécurité Argon2id, sessions & journaux d'audit
│       └── models/                      # Structures DTOs & modèles Serde
├── frontend/                            # Interface SvelteKit (Svelte 5 Runes SPA)
│   ├── src/
│   │   ├── lib/
│   │   │   ├── components/
│   │   │   │   ├── onboarding/          # 🌟 Wizard Novice 1-Click & ModeSwitch
│   │   │   │   │   ├── OnboardingWizard.svelte  # Orchestrateur 4 étapes
│   │   │   │   │   ├── GameSelectorStep.svelte  # Sélection Java / Bedrock / Geyser
│   │   │   │   │   ├── EngineSelectorStep.svelte# Presets Purpur / Paper / itzg
│   │   │   │   │   ├── RamAllocationStep.svelte # Jauge segmentée matériel
│   │   │   │   │   ├── LaunchReviewStep.svelte  # Accord EULA & veille lazymc
│   │   │   │   │   └── ModeSwitch.svelte        # Bascule Novice / Power-User (Alt+M)
│   │   │   │   └── ui/                  # Composants Double-Bezel usinés
│   │   │   └── stores/
│   │   │       └── preferences.svelte.js# Store réactif Novice/Expert & sonde hôte
│   │   └── routes/
│   │       ├── +layout.svelte           # Shell global, header hardware & navigation
│   │       ├── dashboard/               # Bento grid télémétrie live & contrôles
│   │       ├── setup/                   # Route dédiée Assistant 1-Click débutant
│   │       ├── players/                 # Gestion graphique des joueurs & skins
│   │       └── logs/                    # Console xterm.js & inspection temps réel
├── docs/                                # Suite documentaire technique de référence
│   ├── onboarding-spec.md               # Spécification UX/technique débutant & desktop
│   ├── architecture.md                  # Conception interne Rust Axum & acteurs
│   ├── api-reference.md                 # Documentation exhaustive des 77 routes API
│   ├── modules-guide.md                 # Guide NBT, diff Myers, backups S3
│   └── security.md                      # Modèle de sécurité & isolation rootless
├── design.md                            # Charte graphique de référence & tokens CSS
├── PRODUCT.md                           # Vision produit, cibles et piliers stratégiques
├── CLAUDE.md                            # Commandes et guide opérationnel développeur
└── Containerfile                        # Image minimale multi-stage de production
```

---

## 📚 STRUCTURE DOCUMENTAIRE DE RÉFÉRENCE

| Fichier | Rôle & Contenu |
| :--- | :--- |
| [`PRODUCT.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/PRODUCT.md) | **Vision produit, positionnement, cibles utilisateurs (novice / expert), piliers de performance et roadmap desktop.** |
| [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) | **Spécification complète du design system, tokens CSS, anti-patterns et design engineering.** |
| [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) | Détails opérationnels de l'architecture Rust/Axum, RCON, Podman socket et gestion Minecraft. |
| [`docs/onboarding-spec.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/onboarding-spec.md) | **Spécification détaillée de l'onboarding 1-Click débutant, jauge RAM assistée et architecture desktop.** |
| [`docs/architecture.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/architecture.md) | Architecture interne approfondie, runtime Tokio, acteur RCON et cycle de vie lazymc. |
| [`docs/api-reference.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/api-reference.md) | Référence exhaustive des 77 routes API REST et des événements WebSocket. |
| [`docs/modules-guide.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/modules-guide.md) | Guide exhaustif des modules (NBT, plugins, diff Myers, backups S3, audit log). |
| [`docs/deployment-and-operations.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/deployment-and-operations.md) | Guide de déploiement conteneurisé, variables d'environnement et runbook d'exploitation. |
| [`docs/security.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/security.md) | Modèle de sécurité, RBAC, hash Argon2id, mitigation SSRF et isolation rootless. |

