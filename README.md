# ChiPanel

<p align="center">
  <img src="https://raw.githubusercontent.com/Chixiisreallynotme/chipanel/main/frontend/static/favicon.png" alt="ChiPanel Logo" width="96" height="96" />
</p>

<p align="center">
  <strong>Management console for Minecraft servers running on rootless Podman and systemd Quadlets.</strong>
</p>

<p align="center">
  <a href="https://github.com/Chixiisreallynotme/chipanel/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square" alt="Build Status" /></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/backend-Rust_1.85_%7C_Axum_0.7-DEA584?style=flat-square&logo=rust" alt="Rust 1.85" /></a>
  <a href="https://svelte.dev"><img src="https://img.shields.io/badge/frontend-Svelte_5_Runes_%7C_SvelteKit_2-FF3E00?style=flat-square&logo=svelte" alt="Svelte 5" /></a>
  <a href="https://podman.io"><img src="https://img.shields.io/badge/runtime-Podman_5.4_Rootless-892CA0?style=flat-square&logo=podman" alt="Podman 5.4" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 License" /></a>
</p>

---

ChiPanel is a web console built for homelabs and small dedicated servers running Minecraft on Linux.

Traditional server panels (Pterodactyl, AMP, Crafty) require Docker daemons, root privileges, background databases (MySQL/PostgreSQL), and consume 400 MB to 1 GB of RAM just running the management stack. On a low-power homelab host (e.g. Intel Core i3 or N100 with 8 GB RAM), that overhead directly reduces the memory available to the Minecraft JVM and other services.

ChiPanel solves this by pairing a compiled Rust backend (Axum + Tokio) with Linux user-mode infrastructure:
- **No Docker daemon, no root access** — Runs as a rootless user (UID 1000) and talks to Podman via `/run/user/$UID/podman/podman.sock` and systemd user units via D-Bus (`zbus`).
- **Zero external databases** — State is stored in local atomic JSON files with in-memory caching.
- **Low memory footprint** — Backend and static Svelte 5 frontend run in a single container consuming under 25 MB RAM idle.
- **Automatic hibernation** — Integrates with `lazymc` to sleep the Java process when no players are connected, dropping server RAM usage from 4 GB down to ~8 MB.

---

## How It Works

ChiPanel orchestrates the server across three lifecycle states:

```
       ┌────────────────────────────────────────────────────────┐
       │                        OFF                             │
       │     (Container stopped, lazymc.service stopped)        │
       │                 RAM Footprint: 0 MB                    │
       └───────────────────────────┬────────────────────────────┘
                                   │
                      Start via ChiPanel Dashboard
                                   │
                                   ▼
       ┌────────────────────────────────────────────────────────┐
       │                    HIBERNATION                         │
       │   (lazymc proxy on 25565, Java container stopped)      │
       │                RAM Footprint: ~8 MB                    │
       └───────────────────────────┬────────────────────────────┘
                                   │
                             Player Connects
                        (TCP Handshake Detected)
                                   │
                                   ▼
       ┌────────────────────────────────────────────────────────┐
       │                       ACTIVE                           │
       │     (Java container running on 25566, world loaded)    │
       │         Auto-sleep after 15 min idle inactivity        │
       │             RAM Footprint: ~2.5 to 8 GB                │
       └────────────────────────────────────────────────────────┘
```

1. **Hibernation**: The `lazymc` proxy listens on public port `25565`. The Minecraft container remains stopped.
2. **Wake-up**: When a player connects, `lazymc` holds the TCP connection open, executes `minecraft-wake.sh` to start the Podman container on loopback port `25566`, waits for the socket to accept traffic, and proxies the player session without disconnect.
3. **Sleep**: After 15 minutes of zero connected players, the container is stopped automatically, freeing 2.5 to 8 GB of RAM for other services.

---

## Architecture

```
                          Tailscale / Reverse Proxy / HTTPS / Localhost
                                         │
                                         ▼ (Port 25500)
┌─────────────────────────────────────────────────────────────────────────────────┐
│ ChiPanel Container (localhost/chipanel:latest)                                  │
│                                                                                 │
│  ┌─────────────────────────────────┐   ┌─────────────────────────────────────┐  │
│  │ SvelteKit Static SPA            │   │ Axum 0.7 Async Backend              │  │
│  │ - Svelte 5 Runes                │   │ - Tokio async runtime               │  │
│  │ - Dual-Mode UX (Novice/Expert)  │<─>│ - Persistent RCON Actor Channel     │  │
│  │ - CodeMirror 6 / uPlot          │   │ - WebSocket Hub (/ws)               │  │
│  │ - 1-Click Setup Wizard (/setup) │   │ - ContainerEngine & GameDriver APIs │  │
│  └─────────────────────────────────┘   └──────────────────┬──────────────────┘  │
└───────────────────────────────────────────────────────────┼─────────────────────┘
                                                            │
                                  ┌─────────────────────────┴─────────────────────────┐
                                  ▼                                                   ▼
                Container Socket (Podman / Docker)                  systemd --user / D-Bus (zbus)
                                  │                                                   │
                                  ▼                                                   ▼
                ┌───────────────────────────────────┐               ┌───────────────────────────────────┐
                │ Container Runtime Engine          │               │ System Supervisor                 │
                │ - PodmanEngine (rootless subuid)  │               │ - Quadlets (*.container)          │
                │ - DockerEngine (socket + demux)   │               │ - lazymc.service (TCP proxy)      │
                │ - AutoDetector & cgroups metrics  │               │ - Host resource monitor           │
                └───────────────────────────────────┘               └───────────────────────────────────┘
```

---

## Core Capabilities

### Systems & Process Management
- **Multi-Runtime Container Abstraction** — Universal container driver support via `ContainerEngine` trait: rootless Podman with systemd Quadlets, standard Docker via socket, and automatic runtime detection.
- **Modular Game Server Drivers** — Extensible `GameDriver` architecture powering Minecraft (Java & Bedrock), Palworld, and Valheim dedicated servers through a dynamic thread-safe registry.
- **Persistent RCON Actor** — Single-threaded Tokio actor keeps one TCP connection open to `127.0.0.1:25575`. Commands are queued over `tokio::sync::mpsc::channel(128)` with 8-second execution timeouts and automatic reconnection. Uses empty dummy packets (`SERVERDATA_RESPONSE_VALUE`) to handle multi-packet output concatenation without socket hangs.
- **Systemd & Podman Control** — Dispatches lifecycle actions through D-Bus (`org.freedesktop.systemd1.Manager` via `zbus`) with automatic fallback to container engine APIs.
- **WebSocket Console & Fast-Path Watcher** — Real-time log streaming with ANSI color parsing, auto-scroll, command history, instant join log watcher (<50ms execution of queued moderation commands), and one-click sanitized export to [mclo.gs](https://mclo.gs) (automatic regex redaction of IPs, passwords, tokens, and host paths).

### Game & Content Management
- **Dual-Mode UX (Novice 1-Click vs Power-User)** — Instant toggle via `Alt+M` or hardware switch (`ModeSwitch.svelte`). Beginners get a 4-step zero-code onboarding wizard (`/setup`) with automatic engine recommendations, hardware-aware RAM slider, and 1-click EULA; power users get direct Quadlet editing, raw RCON, Myers diff analysis, and cgroups tuning.
- **11 Server Engines** — Switch between Paper, Purpur, Folia, Fabric, Forge, NeoForge, Quilt, Mohist, Arclight, Spigot, and Vanilla with automatic Mojang manifest polling (`piston-meta.mojang.com`) and Quadlet environment rewriting.
- **LuckPerms Migration** — Automatically snapshots and moves LuckPerms configuration paths across engine switches (`plugins/LuckPerms`, `mods/luckperms`, `config/luckperms`).
- **Binary NBT Inspector** — Decodes `playerdata/{uuid}.dat` files in-memory using `fastnbt`. Displays 2D inventory slots (hotbar, main, armor, offhand, Ender Chest), durability bars, enchantments, trims, active potion effects, and player coordinates.
- **Offline Player Command Queue** — Administrative actions on offline players (items, permissions, gamemodes) are queued in `pending_commands.json` and executed on their next login.
- **Addon & Modpack Catalog** — Search and install plugins and mods directly from Modrinth v2 and CurseForge. Computes SHA-512 hashes for update checks and runs a 24-hour sync loop for `spark`, `chunky`, and `luckperms`.
- **World & Chunky Control** — Reads `level.dat` metadata (seed, generator, worldborder), imports/exports ZIP archives with Zip-Slip protection, and controls Chunky chunk pregeneration with real-time ETA and CPS metrics.
- **Visual Config Diff** — In-browser editor with CodeMirror 6 and side-by-side Myers diff comparison before committing changes to `server.properties` or YAML configs.
- **Scoped Backups** — Generates ZIP/zstd archives with selectable scopes (`full`, `world_only`, `configs_only`), SHA-256 integrity verification, automatic retention quotas, and direct upload to S3 / MinIO buckets.

### Security
- **Authentication**: Password hashing with Argon2id (3 iterations, 64 MB memory) offloaded to `tokio::task::spawn_blocking`. Login rate limiting: 5 attempts/min per user, 10 hashes/10s globally.
- **Sessions & API Keys**: HMAC-SHA256 JWT sessions (24h lifespan) and CSPRNG persistent API tokens (`chipanel_sec_<hex>`) stored exclusively as SHA-256 hashes.
- **Role-Based Access**: Three privilege tiers (`admin`, `operator`, `viewer`) with immutable root administrator protection.
- **Filesystem Sandbox**: All file paths are canonicalized and verified against `MINECRAFT_DATA_DIR` and `DATA_DIR` roots before any read/write/delete operation.
- **SSRF Hardening**: Discord webhook notifications enforce HTTPS, strict domain whitelisting, and `redirect(Policy::none())`.

---

## Supported Server Engines

| Engine | Type | Config Directory | Addon Directory |
| :--- | :--- | :--- | :--- |
| **Paper** | Plugin | `config/`, `paper-global.yml` | `plugins/` |
| **Purpur** | Plugin | `purpur.yml`, `paper-global.yml` | `plugins/` |
| **Folia** | Plugin | `config/folia.yml`, `paper-global.yml` | `plugins/` |
| **Spigot** | Plugin | `spigot.yml`, `bukkit.yml` | `plugins/` |
| **Fabric** | Mod | `config/` | `mods/` |
| **Quilt** | Mod | `config/` | `mods/` |
| **Forge** | Mod | `config/`, `defaultconfigs/` | `mods/` |
| **NeoForge** | Mod | `config/`, `defaultconfigs/` | `mods/` |
| **Mohist** | Hybrid | `config/mohist.yml` | `plugins/`, `mods/` |
| **Arclight** | Hybrid | `arclight.conf` | `plugins/`, `mods/` |
| **Vanilla** | Official | `server.properties` | *N/A* |

---

## Quick Start

### Production (Podman Quadlet)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Chixiisreallynotme/chipanel.git
   cd chipanel
   ```

2. **Create your Quadlet unit file**:
   ```bash
   cp chipanel.container.example chipanel.container
   ```
   Edit `chipanel.container` to set your passwords and storage paths.

3. **Build and deploy**:
   ```bash
   make build-container
   make deploy-quadlet
   ```

4. **Check service status**:
   ```bash
   systemctl --user status chipanel.service
   ```
   Open `http://localhost:25500` in your browser.

---

### Local Container Preview

To inspect the web interface without mounting host system sockets:

```bash
podman build -t localhost/chipanel:latest -f Containerfile .
podman run -d --name chipanel-preview -p 127.0.0.1:25501:25500 \
  -e HOST=0.0.0.0 -e PORT=25500 \
  -e ADMIN_USERNAME=admin -e ADMIN_PASSWORD=preview \
  -e RUST_LOG=info \
  localhost/chipanel:latest
```

Open `http://localhost:25501` (credentials: `admin` / `preview`).

---

### Development Setup

Prerequisites: Rust 1.85+, Node.js 22+, `make`.

```bash
# Terminal 1 — Backend on http://localhost:25500
make dev-backend

# Terminal 2 — Vite dev server on http://localhost:5173 (proxies /api to :25500)
make dev-frontend
```

---

## Configuration Reference

All settings can be set via environment variables or Quadlet `Environment=` directives:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `127.0.0.1` | Network interface to bind (`0.0.0.0` for all interfaces) |
| `PORT` | `25500` | HTTP and WebSocket listening port |
| `RUST_LOG` | `info` | Logging filter (`error`, `warn`, `info`, `debug`, `trace`) |
| `ADMIN_USERNAME` | `admin` | Bootstrap administrator username |
| `ADMIN_PASSWORD` | *(auto-generated)* | Bootstrap administrator password (hashed with Argon2id on startup) |
| `ADMIN_PASSWORD_HASH` | *(none)* | Pre-computed Argon2id password hash for bootstrap admin |
| `JWT_SECRET` | *(auto-generated)* | 256-bit CSPRNG secret key for HMAC-SHA256 JWT session signing |
| `CONTAINER_ENGINE` | `auto` | Container runtime engine: `podman`, `docker`, or `auto` |
| `CONTAINER_SOCKET` | *(auto-detected)* | Path to runtime socket (`PODMAN_SOCKET` or `DOCKER_SOCKET` aliases accepted) |
| `GAME_DRIVER` | `minecraft` | Primary game driver to activate (`minecraft`, `palworld`, `valheim`) |
| `CONTAINER_NAME` | `minecraft-server` | Target container name for status and telemetry |
| `DATA_DIR` | `/app/data` | Path for persistent state files (`users.json`, `api_tokens.json`, `audit_log.jsonl`) |
| `MINECRAFT_DATA_DIR` | `/app/minecraft-data` | Path to Minecraft server root directory (worlds, plugins, configs) |
| `SYSTEMD_CONFIG_DIR` | `/app/systemd-config` | Directory containing systemd user Quadlet files (`*.container`) |
| `LAZYMC_CONFIG_FILE` | `/app/lazymc-config/lazymc.toml` | Path to lazymc hibernation configuration |
| `DBUS_SESSION_BUS_ADDRESS` | `unix:path=/run/user/<UID>/bus` | D-Bus session bus address for systemd unit control |
| `ALLOWED_ORIGINS` | `http://127.0.0.1:25500, http://localhost:25500` | Allowed CORS origins (`*` permits any origin) |
| `TOOLS_SYNC_INTERVAL_SECS` | `86400` | Interval in seconds for automatic Spark, Chunky, and LuckPerms sync |
| `CURSEFORGE_API_KEY` | *(empty)* | Optional API key for CurseForge modpack and addon search |

---

## Makefile Targets

| Target | Description |
| :--- | :--- |
| `make dev-backend` | Runs backend in debug mode with hot recompilation |
| `make dev-frontend` | Runs Vite development server with backend API proxy |
| `make build-frontend` | Compiles production static assets to `frontend/build` |
| `make build-backend` | Compiles optimized release binary to `backend/target/release` |
| `make build-container` | Builds multi-stage Podman container image (`localhost/chipanel:latest`) |
| `make deploy-quadlet` | Copies Quadlet to `~/.config/containers/systemd/` and reloads systemd user daemon |
| `make test` | Executes backend test suite |
| `make clean` | Removes build artifacts, dependencies, and target files |

---

## Documentation

- [**Architecture & Internals**](docs/architecture.md) — Multi-runtimes (Podman/Docker), modular Game Drivers, Tokio actor pattern, Svelte 5 runes, and lazymc hibernation.
- [**Novice Onboarding & Dual-Mode UX**](docs/onboarding-spec.md) — 4-step 1-Click setup wizard, hardware RAM recommendation, and desktop architecture.
- [**REST API & WebSocket Reference**](docs/api-reference.md) — Specifications for all 77 HTTP endpoints and WebSocket events.
- [**Functional Modules Guide**](docs/modules-guide.md) — Subsystem breakdowns for container engines, game drivers, NBT parsing, and diff engines.
- [**Deployment & Operations Runbook**](docs/deployment-and-operations.md) — Multi-stage builds, systemd Quadlets, Docker deployments, and local preview.
- [**Security & Threat Model**](docs/security.md) — Argon2id, RBAC, path traversal prevention, and container namespace isolation.
- [**Strategic Roadmap & Product Matrix**](docs/ROADMAP_STRATEGIQUE_CHIPANEL.md) — Full technical engineering roadmap, competitive matrix, and hardware auto-tuning specifications.

---

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.
