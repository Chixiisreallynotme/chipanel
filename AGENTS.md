# ChiPanel — Contrat d'intervention agent

> Source de vérité pour tout agent ou développeur travaillant sur **ChiPanel** (backend Rust Axum + frontend SvelteKit/Svelte 5 + Containerfile).

---

## Lecture obligatoire avant toute intervention

Avant d'écrire ou modifier la moindre ligne dans `chipanel/` :

1. Ce fichier [`AGENTS.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/AGENTS.md).
2. [`PRODUCT.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/PRODUCT.md) — vision produit, cibles utilisateurs, positionnement.
3. [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) — obligatoire pour toute touche au frontend.
4. [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) et [`docs/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/) pour le backend, les routes API et les modèles de données.

---

## Comment on travaille ensemble

Mon « fais X » est souvent un point de départ, pas une spec complète. Avant de coder, cerne le **résultat que je vise** et reformule ce que tu as compris — ça évite de produire quelque chose à côté et de devoir tout reprendre.

Tu es conseiller et expert, pas exécutant :

- Si la demande est ambiguë, demande des précisions au lieu de deviner.
- Si une solution te semble meilleure, propose-la avec ta justification.
- **Challenge-moi par défaut sur toute demande non triviale.** Signale les risques, contradictions et angles morts avant d'exécuter.
- **Pour toute question, utilise l'outil `ask_question`.** Ne génère jamais une liste de questions en texte brut dans le chat : rends la démarche interactive avec des options sélectionnables.

Réflexe attendu dans cet ordre : **écouter → questionner (`ask_question`) → proposer → confirmer le plan → exécuter.** Une fois le plan validé, exécute sans re-challenger, sauf découverte en cours de route.

**À ne pas faire :**

- Refaire tout le style frontend sans avoir relu `design.md` ni demandé ce que j'attends.
- Écrire un handler API complet sans demander le contrat de données, les erreurs, la route.
- Poser des questions sous forme de liste de texte au lieu d'utiliser `ask_question`.
- Implémenter une solution fragile sans signaler qu'une meilleure existait.
- Exécuter un choix risqué (exposer un secret, casser une API existante, supprimer une limite RAM) sans alerter.

---

## Impératifs de performance

ChiPanel doit rester sobre face à Pterodactyl, AMP, Crafty (400 Mo–1 Go RAM, runtimes lourds). Les limites sont non-négociables :

- **< 25 Mo de RAM** pour le backend : Rust natif (Axum 0.7 + Tokio + Hyper 1.0), zéro base SQL externe (JSON atomiques + SQLite WAL pour les logs), image distroless minimale.
- **Frontend instantané** : Svelte 5 Runes (`$state`, `$derived`, `$effect`), bundle compact, chargement < 100 ms, transitions < 250 ms, zéro CLS, `tabular-nums` sur toutes les données numériques.
- **0% CPU à vide** : couplage avec `lazymc` (~8 Mo RAM) — après 15 min sans joueur, le conteneur de jeu s'arrête, libérant 2,5–8 Go RAM. Réveil au premier handshake TCP sur 25565.
- **Latence API < 5 ms** : canal RCON acteur multiplexé, SSE/WebSocket sur `/ws`.

---

## UX dual-mode : novice et power-user

```
┌────────────────────────────────────┬──────────────────────────────────────┐
│  Mode novice (1-click setup)       │  Mode power-user (expert / devops)   │
├────────────────────────────────────┼──────────────────────────────────────┤
│  Parcours guidé 4 étapes (/setup)  │  Accès direct Quadlet systemd        │
│  Zéro ligne de code, zéro YAML     │  Console RCON brute multiplexée      │
│  RAM recommandée automatiquement   │  Flags JVM & cgroups CPU/RAM         │
│  EULA accepté en 1 clic            │  Inspection NBT binaire & diff Myers │
│  lazymc activé par défaut          │  Sauvegardes zstd vers S3 / MinIO    │
└────────────────────────────────────┴──────────────────────────────────────┘
```

Bascule : `Alt+M` (ou `Option+M`) via [`ModeSwitch.svelte`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/frontend/src/lib/components/onboarding/ModeSwitch.svelte).

**Novice :** quelqu'un qui n'a jamais ouvert un terminal. Déploiement de serveurs préconfigurés via [`routes/setup/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/frontend/src/routes/setup/+page.svelte), résolution automatique des loaders de mods (Fabric, Forge, NeoForge, Paper, Purpur), messages d'erreur explicites, export logs vers `mclo.gs` en 1 clic.

**Power-user :** Quadlets systemd (`~/.config/containers/systemd/`), socket Podman/Docker, isolation rootless (subuid/subgid), RCON ANSI, file de commandes hors-ligne, NBT live (`playerdata/*.dat`), diff Myers (`similar`), backups zstd vers S3/MinIO.

---

## Architecture

Le codebase vit dans [`chipanel/backend/src/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/backend/src/).

**Abstraction conteneurs (`src/container/`)** — trait [`ContainerEngine`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/backend/src/container/engine.rs) avec `get_status`, `get_stats`, `start_container`, `stop_container`, `restart_container`, `get_logs` :
- `PodmanEngine` : Podman rootless (socket `/run/user/<uid>/podman/podman.sock` ou `PODMAN_SOCKET`), Quadlets via D-Bus (`zbus`).
- `DockerEngine` : socket Docker (`/var/run/docker.sock` ou `DOCKER_HOST`), démultiplexage headers stdout/stderr.
- `AutoDetector` : détection automatique ou forcée via `CONTAINER_ENGINE=podman|docker|auto`.
- Profils : `LazymcCustom`, `ItzgMinecraft` (`itzg/minecraft-server`, variables `EULA`/`TYPE`/`VERSION`/`MEMORY`), `GenericOci`.

**Game drivers (`src/engine/`)** — trait [`GameDriver`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/backend/src/engine/driver.rs) : cycle de vie, télémétrie, commandes, joueurs, sauvegardes :
- `MinecraftDriver` : Java & Bedrock (RCON, plugins, modpacks, mondes, hibernation lazymc).
- `PalworldDriver` / `ValheimDriver` : serveurs dédiés multijoueurs.
- `GameEngineRegistry` : registre dynamique thread-safe.

---

## Vision desktop (Tauri)

L'API REST Axum + WebSocket stateless + SPA SvelteKit permettent un packaging en application desktop via **Tauri v2** (Rust + WebView multiplateforme). Cas d'usage : lancer et superviser un serveur local (Windows, macOS, Linux) sans serveur dédié ni SSH. Mode hybride : pilotage local (Podman Desktop, Docker Desktop, Java local) ou connexion distante à une instance homelab via API key/Tailscale.

---

## Direction artistique (frontend uniquement)

Lire [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) en intégralité avant toute touche au frontend. Résumé des règles non-négociables :

- **Esthétique Double-Bezel / Machined Hardware :** surfaces ardoise sombre (`#0C0E14` / `#161922` / `#1F2330`), biseau intérieur 1px spéculaire (`inset 0 1px 0 rgba(255,255,255,0.09)`), zéro noir OLED pur `#000000`.
- **Anti-slop :** zéro gradient textuel, zéro aura néon sans justification sémantique, zéro badge superflu, arrondis plafonnés 8–12px (`--radius-md`/`--radius-lg`), zéro emoji dans l'UI (Lucide/Phosphor uniquement).
- **Color scarcity :** 90% monochrome, 10% accents sémantiques (3 rôles par couleur : `--accent-*-bg`, `--accent-*-text`, `--accent-*-solid`). WCAG AA/AAA obligatoire.
- **Micro-interactions :** `scale(0.97)` sur `:active`, jamais depuis `scale(0)`, popovers origin-aware, zéro animation sur raccourcis fréquents, durées < 250 ms, `--ease-out: cubic-bezier(0.23, 1, 0.32, 1)`, uniquement `transform` et `opacity` (GPU-safe).
- **Télémétrie :** données réelles `tabular-nums`, squelettes fidèles sans CLS, hibernation `lazymc` gérée proprement.
- **Logo :** Hexagone Stencil Chi-Rack avec voyant LED réactif aux statuts du serveur.

---

## Versioning

Commit + push après **toute** modification dans `chipanel/` : `git add -A && git commit -m "..." && git push`. Jamais de changement non commité en arrière.

**Ne pas demander de confirmation pour commiter — c'est systématique.** Ne commite jamais de secret en clair : `chipanel.container` (quadlet réel, mots de passe en clair) est gitignoré, seul `chipanel.container.example` est versionné.

---

## Mine de savoir ChiPanel

Chaque session doit laisser une trace. Dès qu'un piège est contourné ou un comportement inattendu découvert (build Rust/Svelte, Axum, SQLite WAL, socket Podman, animations), documente-le immédiatement dans [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) si opérationnel, ou dans le doc [`docs/`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/) concerné. Ne pas attendre qu'on te le demande. L'objectif est qu'un agent futur puisse reprendre sans réapprendre les mêmes leçons.

---

## Gotchas de déploiement

- **Frontend non pris en compte au build :** `podman build` cache les layers. Si le frontend a changé, utiliser obligatoirement `--no-cache` : `podman build --no-cache -t localhost/chipanel:latest -f Containerfile .`.
- **`podman load` ne redémarre pas le conteneur :** le service tourne encore sur l'ancienne image. Toujours enchaîner avec `systemctl --user restart chipanel`.
- **`daemon-reload` obligatoire après édition du quadlet :** `systemctl --user daemon-reload` PUIS `restart` — sinon la modif est inerte (piège réel sur `MemoryMax` et `Environment`).
- **`restart` systemd ≠ `rm` + `create` :** pour injecter des changements DNS/réseau, faire `podman rm chipanel` puis `start`, pas juste `restart`.
- **Tag fixe :** builder en `localhost/chipanel:latest` uniquement — c'est le tag référencé par le quadlet.
- **Secrets :** jamais dans un commit, dans les logs de build, ni dans une variable loggée. Voir [`docs/security.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/security.md).

---

## Commandes de référence

**Frontend** : `npm run dev` (proxy → `:25500`) · `npm run build` · `npm run check` (svelte-check)

**Backend** : `cargo run` · `cargo test` · `cargo build --release` · `cargo check`

**Conteneur** : `podman build -t localhost/chipanel:latest -f Containerfile .` (ajouter `--no-cache` si frontend modifié)

**Déploiement Host-007** : `podman save localhost/chipanel:latest | ssh ... "podman load" && ssh ... "systemctl --user restart chipanel"`

---

## Cartographie du codebase

```
chipanel/
├── backend/                             # Rust Axum (< 25 Mo RAM)
│   ├── Cargo.toml                       # Axum 0.7, Tokio, Hyper, Serde, Zbus, SQLite
│   └── src/
│       ├── main.rs                      # Point d'entrée, Tokio, bind :25500
│       ├── config.rs                    # Variables d'environnement & profils
│       ├── error.rs                     # AppError & conversion HTTP
│       ├── container/                   # Multi-runtime (Podman / Docker / Quadlets)
│       │   ├── engine.rs                # Trait ContainerEngine
│       │   ├── podman.rs                # Podman rootless (socket + D-Bus systemd)
│       │   ├── docker.rs                # Docker (socket + log demux)
│       │   ├── detector.rs              # Auto-détection du runtime
│       │   └── profiles.rs              # Profils d'images (itzg, lazymc, generic)
│       ├── engine/                      # Game drivers modulaires
│       │   ├── driver.rs                # Trait GameDriver (cycle de vie, télémétrie, RCON)
│       │   ├── minecraft.rs             # Minecraft (Paper, Fabric, Purpur, Bedrock)
│       │   ├── palworld.rs              # Palworld Dedicated Server
│       │   ├── valheim.rs               # Valheim Dedicated Server
│       │   └── registry.rs              # Registre dynamique thread-safe
│       ├── minecraft/                   # Sous-systèmes Minecraft
│       │   ├── player.rs                # NBT playerdata, inventaires, stats, skins
│       │   ├── worlds.rs                # Dimensions, sauvegardes, exports
│       │   ├── plugins.rs & modpacks.rs # Modrinth v2 & CurseForge API
│       │   ├── server_backup.rs         # Sauvegardes atomiques zstd & upload S3/MinIO
│       │   └── server_properties.rs     # Parser & diff Myers de configuration
│       ├── rcon/                        # Acteur RCON multiplexé persistant
│       ├── websocket/                   # SSE / WebSocket temps réel (/ws)
│       ├── routes/                      # Handlers HTTP Axum (77 endpoints REST)
│       ├── auth/ & audit/               # Argon2id, sessions & journaux d'audit
│       └── models/                      # DTOs & modèles Serde
├── frontend/                            # SvelteKit (Svelte 5 Runes SPA)
│   └── src/
│       ├── lib/
│       │   ├── components/
│       │   │   ├── onboarding/          # Wizard novice 1-click & ModeSwitch
│       │   │   │   ├── OnboardingWizard.svelte  # Orchestrateur 4 étapes
│       │   │   │   ├── GameSelectorStep.svelte  # Java / Bedrock / Geyser
│       │   │   │   ├── EngineSelectorStep.svelte# Purpur / Paper / itzg
│       │   │   │   ├── RamAllocationStep.svelte # Jauge segmentée matériel
│       │   │   │   ├── LaunchReviewStep.svelte  # EULA & veille lazymc
│       │   │   │   └── ModeSwitch.svelte        # Bascule novice/power-user (Alt+M)
│       │   │   └── ui/                  # Composants Double-Bezel
│       │   └── stores/
│       │       └── preferences.svelte.js# Store réactif novice/expert & sonde hôte
│       └── routes/
│           ├── +layout.svelte           # Shell global, header & navigation
│           ├── dashboard/               # Bento grid télémétrie live & contrôles
│           ├── setup/                   # Assistant 1-click débutant
│           ├── players/                 # Gestion joueurs & skins
│           └── logs/                    # Console xterm.js & inspection temps réel
├── docs/                                # Documentation technique
│   ├── onboarding-spec.md               # Spec UX/technique débutant & desktop
│   ├── architecture.md                  # Rust Axum interne & acteurs
│   ├── api-reference.md                 # 77 routes API REST & événements WebSocket
│   ├── modules-guide.md                 # NBT, diff Myers, backups S3
│   └── security.md                      # Sécurité & isolation rootless
├── design.md                            # Charte graphique & tokens CSS
├── PRODUCT.md                           # Vision produit, cibles, piliers
├── CLAUDE.md                            # Commandes & guide opérationnel
└── Containerfile                        # Image multi-stage de production
```

---

## Documentation de référence

| Fichier | Contenu |
| :--- | :--- |
| [`PRODUCT.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/PRODUCT.md) | Vision produit, positionnement, cibles (novice/expert), performance, roadmap desktop. |
| [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) | Design system complet, tokens CSS, anti-patterns, design engineering. |
| [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) | Architecture Rust/Axum, RCON, Podman socket, gestion Minecraft (opérationnel). |
| [`docs/onboarding-spec.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/onboarding-spec.md) | Spec onboarding 1-click, jauge RAM assistée, architecture desktop. |
| [`docs/architecture.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/architecture.md) | Architecture interne, runtime Tokio, acteur RCON, cycle de vie lazymc. |
| [`docs/api-reference.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/api-reference.md) | 77 routes API REST & événements WebSocket. |
| [`docs/modules-guide.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/modules-guide.md) | NBT, plugins, diff Myers, backups S3, audit log. |
| [`docs/deployment-and-operations.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/deployment-and-operations.md) | Déploiement conteneurisé, variables d'environnement, runbook. |
| [`docs/security.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/docs/security.md) | RBAC, Argon2id, mitigation SSRF, isolation rootless. |
