# ChiPanel

A lightweight, dark-mode management console for Minecraft servers and homelab services running on rootless Podman with systemd Quadlets.

Built with a **Rust (Axum 0.7)** backend and a **SvelteKit (Svelte 5)** static SPA frontend. Single container deployment (<50 MB runtime), zero external database dependencies, and sub-millisecond API response times.

---

## Highlights

- **Native Rootless Podman & Quadlets** — Controls user-mode systemd services directly via UNIX sockets (`/run/user/$UID/podman/podman.sock` and `/run/user/$UID/systemd/private`) without requiring root privileges or a bloated daemon.
- **Actor-Based RCON Engine** — Tokio channel actor handles persistent RCON sessions, command serialization, rate limiting, and graceful reconnection.
- **Live Terminal & Sanitized Logs** — Real-time WebSocket terminal with ANSI parsing, command history, auto-scroll, and one-click export to [mclo.gs](https://mclo.gs) with automatic IP/token redaction.
- **Multi-Engine & Version Switcher** — Switch dynamically across 11 server engines (*Paper, Purpur, Folia, Fabric, Forge, NeoForge, Quilt, Mohist, Arclight, Spigot, Vanilla*) with automated Mojang manifest synchronization and Quadlet rewriting.
- **NBT Player & Inventory Inspector** — Real-time player monitoring, interactive inventory and ender chest viewer, potion effect applicator, and LuckPerms group management.
- **Addons & Modpack Catalog** — Search and install plugins and mods directly from Modrinth and CurseForge with atomic downloads and automatic Spark / Chunky tool synchronization.
- **World Lifecycle & Chunky Integration** — Hot-swap active worlds, manage pending generation slots, zip backup/restore, and trigger Chunky world pre-generation tasks.
- **Visual Config Diff** — In-browser editor for `server.properties` and YAML configs with side-by-side diff previews before committing changes.
- **Hardened Security** — Argon2id password hashing, CSPRNG SHA-256 persistent API tokens, role-based authorization (`admin` / `viewer`), strict path traversal guards, and RFC 1918 SSRF filters.

---

## Architecture

```
                          Tailscale / Reverse Proxy / HTTPS
                                         │
                                         ▼
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
```

---

## Quick Start

### Option A: Podman Quadlet (Recommended)

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

3. **Build and start the container**:
   ```bash
   make build-container
   make deploy-quadlet
   ```

4. **Verify service status**:
   ```bash
   systemctl --user status chipanel.service
   ```
   Open `http://localhost:3000` in your browser.

---

### Option B: Local Development

**Prerequisites**: Rust 1.85+, Node.js 22+, `make`

```bash
# Terminal 1 — Backend (http://localhost:3000)
make dev-backend

# Terminal 2 — Frontend with Vite dev server (http://localhost:5173)
make dev-frontend
```

---

## Configuration Reference

All settings are configured via environment variables or Quadlet `Environment=` directives:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `0.0.0.0` | Network interface to bind HTTP server |
| `PORT` | `3000` | HTTP listening port |
| `RUST_LOG` | `info` | Logging verbosity filter (`debug`, `info`, `warn`, `error`) |
| `ADMIN_USERNAME` | `admin` | Bootstrap administrator username |
| `ADMIN_PASSWORD` | *(generated)* | Bootstrap administrator password (hashed with Argon2 on startup) |
| `ADMIN_PASSWORD_HASH` | *(empty)* | Pre-computed Argon2 hash for the admin user |
| `JWT_SECRET` | *(auto-generated)* | 32-byte CSPRNG secret key for session JWT signing |
| `RCON_HOST` | `127.0.0.1` | Target Minecraft server RCON host |
| `RCON_PORT` | `25575` | Target Minecraft server RCON port |
| `RCON_PASSWORD` | *(empty)* | Target Minecraft server RCON password |
| `DATA_DIR` | `/app/data` | Persistent application state (tokens, profiles, users) |
| `MINECRAFT_DATA_DIR` | `/app/minecraft-data` | Minecraft server root directory (worlds, plugins, config) |
| `SYSTEMD_CONFIG_DIR` | `/app/systemd-config` | Directory containing systemd Quadlet files (`*.container`) |
| `LAZYMC_CONFIG_FILE` | `/app/lazymc-config/lazymc.toml` | Path to lazymc hibernation configuration |
| `PODMAN_SOCKET` | `/run/user/1000/podman/podman.sock` | Rootless Podman API socket path |
| `ALLOWED_ORIGINS` | `*` | Allowed CORS origins for API requests |
| `CURSEFORGE_API_KEY` | *(empty)* | Optional API key for CurseForge search and downloads |

---

## Makefile Targets

| Target | Description |
| :--- | :--- |
| `make dev-backend` | Runs backend in debug mode with hot-recompile on source change |
| `make dev-frontend` | Runs SvelteKit Vite dev server with backend API proxy |
| `make build-frontend` | Builds production-ready static assets in `frontend/build` |
| `make build-backend` | Compiles optimized release binary in `backend/target/release` |
| `make build-container` | Builds the multi-stage Podman container image (`localhost/chipanel:latest`) |
| `make deploy-quadlet` | Installs unit file to `~/.config/containers/systemd/` and reloads user daemon |
| `make test` | Runs backend unit test suite (60 tests) |
| `make clean` | Cleans build artifacts, node modules, and target directory |

---

## Tech Stack

- **Backend**: Rust 2021, Axum 0.7, Tokio, Hyper 1.x, FastNBT, Argon2, JSONWebToken, ZBus (D-Bus), Reqwest.
- **Frontend**: Svelte 5 (Runes), SvelteKit, Vite, Adapter-Static, CodeMirror 6, uPlot, Lucide Icons.
- **Containerization**: Multi-stage Containerfile (Node 22 + Rust 1.85 -> Distroless CC Debian 12 runtime).
- **Service Management**: Systemd user Quadlets + Rootless Podman.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
