# ChiPanel

<p align="center">
  <img src="https://raw.githubusercontent.com/Chixiisreallynotme/chipanel/main/frontend/static/favicon.png" alt="ChiPanel Logo" width="96" height="96" />
</p>

<p align="center">
  <strong>A lightweight, dark-mode management console for Minecraft servers and homelab services running on rootless Podman with systemd Quadlets.</strong>
</p>

<p align="center">
  <a href="https://github.com/Chixiisreallynotme/chipanel/actions"><img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square" alt="Build Status" /></a>
  <a href="https://rust-lang.org"><img src="https://img.shields.io/badge/backend-Rust_1.85_%7C_Axum_0.7-DEA584?style=flat-square&logo=rust" alt="Rust 1.85" /></a>
  <a href="https://svelte.dev"><img src="https://img.shields.io/badge/frontend-Svelte_5_Runes_%7C_SvelteKit_2-FF3E00?style=flat-square&logo=svelte" alt="Svelte 5" /></a>
  <a href="https://podman.io"><img src="https://img.shields.io/badge/runtime-Podman_5.4_Rootless-892CA0?style=flat-square&logo=podman" alt="Podman 5.4" /></a>
  <a href="#benchmarks--footprint"><img src="https://img.shields.io/badge/RAM_footprint-%3C_25_MB-success?style=flat-square" alt="RAM Footprint" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 License" /></a>
</p>

---

## ⚡ Highlights

- **Native Rootless Podman & Quadlets** — Controls user-mode systemd services directly via Unix domain sockets (`/run/user/$UID/podman/podman.sock` and `/run/user/$UID/bus`) without requiring root privileges, Docker daemons, or sudo rights.
- **Actor-Based RCON Engine** — Single-threaded Tokio actor serializes commands over a persistent TCP socket with automatic reconnect, exponential backoff, and dummy packet multi-packet stream termination.
- **Live Terminal & Sanitized Logs** — Real-time WebSocket terminal with ANSI color rendering, auto-scroll, command history, and one-click export to [mclo.gs](https://mclo.gs) with automatic regex redaction of IPs, tokens, and credentials.
- **Multi-Engine & Version Switcher** — Switch dynamically across **11 server engines** (*Paper, Purpur, Folia, Fabric, Forge, NeoForge, Quilt, Mohist, Arclight, Spigot, Vanilla*) with automated Mojang manifest synchronization and Quadlet environment rewriting.
- **NBT Player & Inventory Inspector** — Real-time player monitoring, interactive 2D inventory and Ender Chest viewer, durability tracking, active potion effect modifier, and LuckPerms group management.
- **Addons & Modpack Catalog** — Search and install plugins and mods directly from Modrinth v2 and CurseForge with atomic downloads, SHA-512 update checks, and automatic Spark / Chunky tool synchronization.
- **World Lifecycle & Chunky Integration** — Hot-swap active worlds, manage pending generation slots, zip backup/restore, edit worldborder, and trigger Chunky world pre-generation with live progress tracking.
- **Visual Config Diff Editor** — In-browser editor for `server.properties` and YAML configs with side-by-side Myers diff previews before committing changes to disk.
- **Cross-Play Bedrock & Geyser** — Built-in assistant verifying Geyser, Floodgate, shared `key.pem` encryption, and UDP port 19132 exposure with automated configuration generation.
- **Scoped Backups & S3 Export** — Create compressed archives with selectable scopes (`full`, `world_only`, `configs_only`), SHA-256 integrity checksums, automatic retention quotas, and direct upload to S3 / MinIO buckets.
- **Database Maintenance & Purge** — Non-locking SQLite inspection for CoreProtect (`database.db`) and LuckPerms, dimension region size breakdowns, and scheduled cleanup.
- **Hardened Security & RBAC** — Argon2id password hashing, CSPRNG SHA-256 persistent API tokens, role-based authorization (`admin`, `viewer`, `operator`), strict path traversal guards, and Discord webhook SSRF filters.

---

## 🏗️ Architecture

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
│  │ - Linear-inspired dark system   │   │ - FastNBT parser & metrics engine   │  │
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

## 🚀 Quick Start

### Option A: Podman Quadlet (Recommended for Production)

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Chixiisreallynotme/chipanel.git
   cd chipanel
   ```

2. **Configure your Quadlet unit**:
   ```bash
   cp chipanel.container.example chipanel.container
   ```
   Edit `chipanel.container` to set your `ADMIN_PASSWORD`, `RCON_PASSWORD`, and volume mount paths.

3. **Build and deploy**:
   ```bash
   make build-container
   make deploy-quadlet
   ```

4. **Verify service status**:
   ```bash
   systemctl --user status chipanel.service
   ```
   Open `http://localhost:25500` in your browser.

---

### Option B: Local Development Setup

**Prerequisites**: Rust 1.85+, Node.js 22+, `make`

```bash
# Terminal 1 — Backend (http://localhost:25500)
make dev-backend

# Terminal 2 — Frontend with Vite dev server (http://localhost:5173)
make dev-frontend
```

---

## ⚙️ Configuration Reference

All settings can be configured via environment variables or Quadlet `Environment=` directives:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `127.0.0.1` | Network interface to bind the HTTP / WebSocket server (`0.0.0.0` for all interfaces) |
| `PORT` | `25500` | HTTP and WebSocket listening port |
| `RUST_LOG` | `info` | Logging verbosity filter (`error`, `warn`, `info`, `debug`, `trace`) |
| `ADMIN_USERNAME` | `admin` | Bootstrap administrator username |
| `ADMIN_PASSWORD` | *(auto-generated)* | Bootstrap administrator password (hashed with Argon2id on startup) |
| `ADMIN_PASSWORD_HASH` | *(none)* | Pre-computed Argon2id password hash for bootstrap administrator |
| `JWT_SECRET` | *(auto-generated)* | 256-bit CSPRNG secret key for HMAC-SHA256 JWT session signing |
| `DATA_DIR` | `/app/data` | Persistent application state (tokens, profiles, users, audit logs) |
| `MINECRAFT_DATA_DIR` | `/app/minecraft-data` | Minecraft server root directory (worlds, plugins, configs) |
| `SYSTEMD_CONFIG_DIR` | `/app/systemd-config` | Directory containing systemd user Quadlet files (`*.container`) |
| `LAZYMC_CONFIG_FILE` | `/app/lazymc-config/lazymc.toml` | Path to lazymc hibernation configuration |
| `PODMAN_CONTAINER_NAME` | `minecraft-server` | Target container name for Podman inspection and metrics |
| `PODMAN_SOCKET` | *(auto-detected)* | Path to rootless Podman Unix domain socket |
| `DBUS_SESSION_BUS_ADDRESS` | `unix:path=/run/user/<UID>/bus` | D-Bus session bus address for systemd unit control |
| `UID` | `1000` | User ID for user-level socket and bus resolution |
| `ALLOWED_ORIGINS` | `http://127.0.0.1:25500, http://localhost:25500` | Allowed CORS origins for API requests |
| `TOOLS_SYNC_INTERVAL_SECS` | `86400` | Interval in seconds for automatic Spark, Chunky, and LuckPerms sync loop |
| `CURSEFORGE_API_KEY` | *(empty)* | Optional API key for CurseForge modpack and addon catalog search |

---

## 🛠️ Makefile Targets

| Target | Description |
| :--- | :--- |
| `make dev-backend` | Runs backend in debug mode with hot-recompile on source change |
| `make dev-frontend` | Runs SvelteKit Vite dev server with backend API proxy |
| `make build-frontend` | Builds production-ready static assets in `frontend/build` |
| `make build-backend` | Compiles optimized release binary in `backend/target/release` |
| `make build-container` | Builds the multi-stage Podman container image (`localhost/chipanel:latest`) |
| `make deploy-quadlet` | Installs unit file to `~/.config/containers/systemd/` and reloads user daemon |
| `make test` | Runs backend test suite |
| `make clean` | Cleans build artifacts, node modules, and target directory |

---

## 📊 Benchmarks & Footprint

| Metric | ChiPanel | Traditional Java / Node.js Panels |
| :--- | :--- | :--- |
| **Idle Memory (RAM)** | **< 25 MB** | 350 MB – 1.2 GB |
| **Container Image Size** | **< 50 MB** (Distroless / Slim) | 400 MB – 1.5 GB |
| **External Database** | **None** (Zero DB dependencies) | MySQL / PostgreSQL / Redis required |
| **API Response Latency** | **< 1 ms** (Rust Axum core) | 25 – 150 ms |
| **RCON Connections** | **1 persistent actor socket** | Spawns new TCP socket per request |
| **Privileges Required** | **Rootless User (UID 1000)** | Root / Sudo / Docker daemon access |

---

## 📚 Documentation Portal

Comprehensive technical documentation is available in the [`docs/`](docs/) directory:

- [**Architecture & Internals**](docs/architecture.md) — Async runtime, Tokio actor pattern, Svelte 5 runes, and lazymc hibernation.
- [**REST API & WebSocket Reference**](docs/api-reference.md) — Complete specification of all 77 endpoints and WebSocket protocols.
- [**Functional Modules Guide**](docs/modules-guide.md) — Detailed breakdown of NBT parsing, modpack deployment, and diff engines.
- [**Deployment & Operations Runbook**](docs/deployment-and-operations.md) — Multi-stage builds, systemd quadlets, and homelab runbooks.
- [**Security & Threat Model**](docs/security.md) — Argon2id, RBAC, path traversal prevention, and Discord SSRF protection.

---

## 💻 Tech Stack

- **Backend**: Rust 2021, Axum 0.7, Tokio, Hyper 1.x, FastNBT, Argon2id, JSONWebToken, ZBus (D-Bus), Reqwest.
- **Frontend**: Svelte 5 (Runes), SvelteKit 2, Vite 6, Adapter-Static, CodeMirror 6, uPlot, Lucide Icons.
- **Containerization**: Multi-stage Containerfile (Node 22 + Rust 1.85 -> Debian Bookworm Slim runtime).
- **Service Orchestration**: Systemd User Quadlets + Rootless Podman 5.4.

---

## 📄 License

Apache License 2.0. See [LICENSE](LICENSE) for details.
