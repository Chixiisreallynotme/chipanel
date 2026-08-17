# ChiPanel

<p align="center">
  <img src="https://raw.githubusercontent.com/Chixiisreallynotme/chipanel/main/frontend/static/favicon.png" alt="ChiPanel Logo" width="96" height="96" />
</p>

<p align="center">
  <strong>Lightweight management console for Minecraft servers and homelab services running on rootless Podman with systemd Quadlets.</strong>
</p>

<p align="center">
  <a href="https://github.com/Chixiisreallynotme/chipanel/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square" alt="Build Status" /></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/backend-Rust_1.85_%7C_Axum_0.7-DEA584?style=flat-square&logo=rust" alt="Rust 1.85" /></a>
  <a href="https://svelte.dev"><img src="https://img.shields.io/badge/frontend-Svelte_5_Runes_%7C_SvelteKit_2-FF3E00?style=flat-square&logo=svelte" alt="Svelte 5" /></a>
  <a href="https://podman.io"><img src="https://img.shields.io/badge/runtime-Podman_5.4_Rootless-892CA0?style=flat-square&logo=podman" alt="Podman 5.4" /></a>
  <a href="#benchmarks--resource-footprint"><img src="https://img.shields.io/badge/RAM_footprint-%3C_25_MB-success?style=flat-square" alt="RAM Footprint" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 License" /></a>
</p>

---

## Overview

ChiPanel is a self-contained management panel engineered for resource-constrained homelabs and dedicated servers. It controls Minecraft server containers, systemd user units, and runtime configurations without requiring root privileges, Docker daemons, or external database engines.

The backend is compiled with Link-Time Optimization (`opt-level = "z"`, `lto = true`, `panic = "abort"`) into a single binary that serves both the REST/WebSocket API and the pre-rendered SvelteKit SPA. The entire runtime occupies under 25 MB RAM in production.

---

## Architecture

```
                          Tailscale / Reverse Proxy / HTTPS
                                         │
                                         ▼ (Port 25500)
┌─────────────────────────────────────────────────────────────────────────────────┐
│ ChiPanel Container (localhost/chipanel:latest)                                  │
│                                                                                 │
│  ┌─────────────────────────────────┐   ┌─────────────────────────────────────┐  │
│  │ SvelteKit Static SPA            │   │ Axum 0.7 Async Backend              │  │
│  │ - Svelte 5 Runes                │   │ - Tokio multi-threaded runtime      │  │
│  │ - CodeMirror 6 / uPlot          │<─>│ - Persistent RCON Actor Channel     │  │
│  │ - Lucide vector icons           │   │ - WebSocket Hub (/ws)               │  │
│  │ - Dark-mode design system       │   │ - FastNBT parser & metrics engine   │  │
│  └─────────────────────────────────┘   └──────────────────┬──────────────────┘  │
└───────────────────────────────────────────────────────────┼─────────────────────┘
                                                            │
                                  ┌─────────────────────────┴─────────────────────────┐
                                  ▼                                                   ▼
                /run/user/1000/podman/podman.sock                  /run/user/1000/bus (D-Bus)
                                  │                                                   │
                                  ▼                                                   ▼
                ┌───────────────────────────────────┐               ┌───────────────────────────────────┐
                │ Podman Rootless Service           │               │ systemd --user (Quadlets & Units) │
                │ - Container stats & metrics       │               │ - minecraft.service               │
                │ - Libpod REST API                 │               │ - lazymc.service (TCP proxy)      │
                └───────────────────────────────────┘               └───────────────────────────────────┘
```

---

## Features

### Rootless Podman & systemd Integration
- Interacts with rootless Podman through the Libpod REST API over Unix Domain Sockets (`/run/user/$UID/podman/podman.sock`).
- Controls user units via D-Bus session bus (`zbus` talking to `org.freedesktop.systemd1.Manager`) with automatic fallback to Libpod API calls.
- Operates entirely under non-root UID 1000 without `sudo` escalation or host namespace pollution.

### Tokio Actor RCON Engine
- Maintains a single persistent TCP socket to the Minecraft RCON port (`127.0.0.1:25575`).
- Serializes commands via `tokio::sync::mpsc::channel(128)` with `oneshot` return channels and 8-second execution timeouts.
- Uses dummy packet framing (`SERVERDATA_RESPONSE_VALUE`) to handle multi-packet output concatenation without hangs.
- Recovers automatically with exponential backoff on server restart or hibernation.

### Server Lifecycle & lazymc Hibernation
- Coordinates three distinct operational states: `OFF`, `HIBERNATION`, and `ACTIVE`.
- In hibernation, `lazymc` listens on public port 25565 while the Minecraft container stays stopped, consuming ~8 MB RAM.
- Incoming player TCP handshakes trigger `minecraft-wake.sh` to start the backend container on internal port 25566, bridging the player session without disconnect.
- Automatically shuts down the container after 15 minutes of zero player activity to reclaim RAM for host workloads.

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

### Multi-Engine & Version Switcher
- Supports 11 server engines: **Paper, Purpur, Folia, Fabric, Forge, NeoForge, Quilt, Mohist, Arclight, Spigot, Vanilla**.
- Synchronizes with official Mojang release and snapshot manifests (`piston-meta.mojang.com`).
- Updates `TYPE`, `VERSION`, and loader variables inside systemd Quadlet files (`minecraft.container`) and `lazymc.toml`.
- Backs up and migrates LuckPerms data paths across loader switches (`plugins/LuckPerms`, `mods/luckperms`, `config/luckperms`) to prevent permission loss.

### Binary NBT Player & Inventory Inspector
- Decodes player `.dat` files via `fastnbt` without invoking JVM commands.
- Visualizes 2D inventories: 36 main inventory slots, 4 armor slots, offhand, and 27 Ender Chest slots.
- Parses legacy NBT tags alongside modern Minecraft 1.20.5+ item components (`minecraft:damage`, `minecraft:enchantments`, `minecraft:trim`).
- Inspects potion effect durations/amplifiers and tracks player coordinates, health, food levels, and dimension placement.
- Queues administrative actions for offline players and executes them automatically upon player reconnect.

### Addons & Modpack Catalog
- Inspects archive manifests (`plugin.yml`, `paper-plugin.yml`, `fabric.mod.json`, `mods.toml`, `neoforge.mods.toml`, `pack.mcmeta`).
- Searches Modrinth v2 and CurseForge with engine and game version compatibility filters.
- Computes SHA-512 hashes of installed JARs for automated update detection.
- Runs an automated background synchronization loop to keep `spark`, `chunky`, `luckperms`, and `fabric-api` up to date.

### World Lifecycle & Chunky Pregeneration
- Decodes `level.dat` to extract world seeds, generator types, data versions, and dimension boundaries.
- Dispatches and tracks Chunky pregeneration via RCON (`/chunky start`, `/chunky pause`, `/chunky cancel`) with real-time chunks rendered, completion %, CPS, and ETA metrics.
- Manages pending generation slots to configure ungenerated worlds before startup.
- Imports and exports world archives with Zip-Slip path validation.

### Configuration Management & Myers Diff Editor
- In-browser configuration editor powered by CodeMirror 6.
- Side-by-side visual Myers diff preview comparing unsaved buffer content against on-disk files (`server.properties`, YAML configs).
- Writes changes atomically via temporary files (`.tmp_save`) before replacing target files on disk.

### Scoped Backups & S3 Streaming
- Creates compressed archives in ZIP and zstd formats.
- Supports three backup scopes: `full`, `world_only`, and `configs_only`.
- Automatically excludes transient logs (`*.log.gz`), caches, backups, and map render tiles (`dynmap`, `bluemap`).
- Verifies archive integrity with SHA-256 checksums, enforces retention policies, and streams archives directly to S3 / MinIO targets.

### Security Architecture & Access Control
- **Authentication**: Password hashing with Argon2id (3 iterations, 64 MB memory cost) executed inside `tokio::task::spawn_blocking`.
- **Sessions**: HMAC-SHA256 JWT tokens with strict 24-hour expiration.
- **API Tokens**: High-entropy tokens (`chipanel_sec_<hex>`) generated via CSPRNG (`rand::rngs::OsRng`), stored exclusively as SHA-256 hashes.
- **Role-Based Access Control (RBAC)**: Enforces three privilege tiers: `admin`, `operator`, and `viewer`.
- **Filesystem Sandboxing**: Resolves paths with `std::fs::canonicalize` and enforces root directory prefix boundaries.
- **SSRF Mitigation**: Validates webhook URLs against Discord domain allowlists, enforces HTTPS, and disables HTTP redirects.
- **Log Sanitization**: Redacts IPv4, IPv6, RCON passwords, webhook tokens, and host filesystem paths before log export to `mclo.gs`.

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

## Benchmarks & Resource Footprint

| Metric | ChiPanel | Node.js / Java Server Panels |
| :--- | :--- | :--- |
| **Idle Memory (RAM)** | **< 25 MB** | 350 MB – 1.2 GB |
| **Container Image Size** | **< 50 MB** (Distroless runtime) | 400 MB – 1.5 GB |
| **External Database** | **None** (Zero DB dependencies) | MySQL / PostgreSQL / Redis required |
| **API Response Latency** | **< 1 ms** (Rust Axum core) | 25 – 150 ms |
| **RCON Connections** | **1 persistent actor socket** | New TCP socket per request |
| **Privileges Required** | **Rootless User (UID 1000)** | Root / Sudo / Docker daemon access |

---

## Quick Start

### Production Deployment (Podman Quadlet)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Chixiisreallynotme/chipanel.git
   cd chipanel
   ```

2. **Configure the Quadlet unit**:
   ```bash
   cp chipanel.container.example chipanel.container
   ```
   Set `ADMIN_PASSWORD`, `RCON_PASSWORD`, and volume paths in `chipanel.container`.

3. **Build and install**:
   ```bash
   make build-container
   make deploy-quadlet
   ```

4. **Verify unit status**:
   ```bash
   systemctl --user status chipanel.service
   ```
   Access the dashboard at `http://localhost:25500`.

---

### Local Container Preview

Run a standalone container for UI and route inspection without host socket bindings:

```bash
podman build -t localhost/chipanel:latest -f Containerfile .
podman run -d --name chipanel-preview -p 127.0.0.1:25501:25500 \
  -e HOST=0.0.0.0 -e PORT=25500 \
  -e ADMIN_USERNAME=admin -e ADMIN_PASSWORD=preview \
  -e RUST_LOG=info \
  localhost/chipanel:latest
```

Open `http://localhost:25501` (login: `admin` / `preview`).

---

### Local Development Setup

Prerequisites: Rust 1.85+, Node.js 22+, `make`.

```bash
# Terminal 1 — Axum backend on http://localhost:25500
make dev-backend

# Terminal 2 — SvelteKit Vite dev server on http://localhost:5173 (proxies /api to :25500)
make dev-frontend
```

---

## Configuration Reference

Settings are configured via environment variables or Quadlet `Environment=` entries:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `127.0.0.1` | Network interface to bind (`0.0.0.0` for all interfaces) |
| `PORT` | `25500` | HTTP and WebSocket listening port |
| `RUST_LOG` | `info` | Log verbosity filter (`error`, `warn`, `info`, `debug`, `trace`) |
| `ADMIN_USERNAME` | `admin` | Bootstrap administrator username |
| `ADMIN_PASSWORD` | *(auto-generated)* | Bootstrap administrator password (hashed with Argon2id on startup) |
| `ADMIN_PASSWORD_HASH` | *(none)* | Pre-computed Argon2id password hash for bootstrap administrator |
| `JWT_SECRET` | *(auto-generated)* | 256-bit CSPRNG secret key for HMAC-SHA256 JWT session signing |
| `DATA_DIR` | `/app/data` | Persistent state directory (`users.json`, `api_tokens.json`, `audit_log.jsonl`) |
| `MINECRAFT_DATA_DIR` | `/app/minecraft-data` | Minecraft server root directory (worlds, plugins, configs) |
| `SYSTEMD_CONFIG_DIR` | `/app/systemd-config` | Directory containing systemd user Quadlet files (`*.container`) |
| `LAZYMC_CONFIG_FILE` | `/app/lazymc-config/lazymc.toml` | Path to lazymc hibernation configuration |
| `PODMAN_CONTAINER_NAME` | `minecraft-server` | Target container name for Podman inspection and metrics |
| `PODMAN_SOCKET` | *(auto-detected)* | Path to rootless Podman Unix domain socket |
| `DBUS_SESSION_BUS_ADDRESS` | `unix:path=/run/user/<UID>/bus` | D-Bus session bus address for systemd unit control |
| `UID` | `1000` | User ID for user-level socket and bus resolution |
| `ALLOWED_ORIGINS` | `http://127.0.0.1:25500, http://localhost:25500` | Allowed CORS origins (`*` permits any origin) |
| `TOOLS_SYNC_INTERVAL_SECS` | `86400` | Re-sync interval in seconds for Spark, Chunky, LuckPerms, and Fabric API |
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

Detailed documentation is available in the [`docs/`](docs/) directory:

- [**Architecture & Internals**](docs/architecture.md) — Runtime model, Tokio actor pattern, Svelte 5 runes, and lazymc hibernation.
- [**REST API & WebSocket Reference**](docs/api-reference.md) — Specifications for all 77 HTTP endpoints and WebSocket events.
- [**Functional Modules Guide**](docs/modules-guide.md) — Subsystem breakdowns for NBT parsing, modpack deployment, and diff engines.
- [**Deployment & Operations Runbook**](docs/deployment-and-operations.md) — Multi-stage builds, systemd Quadlets, and maintenance operations.
- [**Security & Threat Model**](docs/security.md) — Argon2id, RBAC, path traversal prevention, and Discord SSRF protection.

---

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.
