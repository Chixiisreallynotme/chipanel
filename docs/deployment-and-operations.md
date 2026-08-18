# Deployment & Operations Runbook for ChiPanel

This runbook provides complete instructions for building, containerizing, deploying, and managing ChiPanel across both rootless Podman (with systemd Quadlets) and standard Docker environments.

---

## 1. Container Build Process

ChiPanel uses a 3-stage build process defined in [`Containerfile`](../Containerfile):

1. **Stage 1 (`frontend-builder`)**: Uses `node:22-alpine` to compile the Svelte 5 Single Page Application into static assets via `npm run build`.
2. **Stage 2 (`rust-builder`)**: Uses `rust:bookworm` to compile the Axum backend with LTO optimizations (`cargo build --release --bin chipanel`) and strips debug symbols via `strip`.
3. **Stage 3 (`runtime`)**: Minimal runtime image based on `debian:bookworm-slim` containing only `ca-certificates` and `libssl3`. Runs as non-privileged user `USER 1000:1000`.

### Local Container Build Command
```bash
# Note: Always use --no-cache if frontend source files have changed
podman build --no-cache -t localhost/chipanel:latest -f Containerfile .
```

---

## 2. Deployment Options

### Option A: Systemd User Quadlet Deployment (Podman Rootless)

In production homelab environments, ChiPanel is deployed as a systemd user Quadlet in `~/.config/containers/systemd/chipanel.container`:

```ini
[Unit]
Description=ChiPanel - Minecraft & Game Server Management Console
After=network-online.target local-fs.target

[Container]
Image=localhost/chipanel:latest
ContainerName=chipanel
Network=host

# Persistent volume mounts
Volume=%h/chipanel-data:/app/data:Z
Volume=%h/minecraft:/app/minecraft-data:z
Volume=%h/.config/containers/systemd:/app/systemd-config:z
Volume=%h/lazymc:/app/lazymc-config:z

# System socket mounts for host supervision (read-only)
Volume=/run/user/1000/podman/podman.sock:/run/user/1000/podman/podman.sock:ro
Volume=/run/user/1000/bus:/run/user/1000/bus:ro

# Core Configuration
Environment=PORT=25500
Environment=HOST=0.0.0.0
Environment=RUST_LOG=info
Environment=ADMIN_USERNAME=admin
Environment=ADMIN_PASSWORD=YOUR_SECURE_ADMIN_PASSWORD
Environment=JWT_SECRET=YOUR_RANDOM_LONG_JWT_SECRET_KEY
Environment=MINECRAFT_DATA_DIR=/app/minecraft-data
Environment=CONTAINER_NAME=minecraft-server
Environment=CONTAINER_ENGINE=podman
Environment=RCON_HOST=127.0.0.1
Environment=RCON_PORT=25575
Environment=RCON_PASSWORD=YOUR_RCON_PASSWORD
Environment=PODMAN_SOCKET=/run/user/1000/podman/podman.sock
Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus
Environment=LAZYMC_CONFIG_FILE=/app/lazymc-config/lazymc.toml
Environment=SYSTEMD_CONFIG_DIR=/app/systemd-config

[Service]
Restart=always
TimeoutStartSec=60
MemoryMax=150M

[Install]
WantedBy=default.target
```

### Option B: Docker Compose / Standard Docker Deployment

For standard Docker hosts (Debian, Ubuntu, TrueNAS SCALE, Synology DSM):

```yaml
services:
  chipanel:
    image: localhost/chipanel:latest
    container_name: chipanel
    network_mode: host
    restart: unless-stopped
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ./data:/app/data
      - ./minecraft-data:/app/minecraft-data
    environment:
      - PORT=25500
      - HOST=0.0.0.0
      - CONTAINER_ENGINE=docker
      - CONTAINER_NAME=minecraft-server
      - CONTAINER_SOCKET=/var/run/docker.sock
      - ADMIN_USERNAME=admin
      - ADMIN_PASSWORD=YOUR_SECURE_ADMIN_PASSWORD
      - JWT_SECRET=YOUR_RANDOM_LONG_JWT_SECRET_KEY
      - RCON_HOST=127.0.0.1
      - RCON_PORT=25575
      - RCON_PASSWORD=YOUR_RCON_PASSWORD
```

---

## 3. Environment Variables Reference

| Variable | Default Value | Description |
| :--- | :--- | :--- |
| `HOST` | `127.0.0.1` | Network interface to bind the HTTP / WebSocket server (`0.0.0.0` for external access). |
| `PORT` | `25500` | HTTP and WebSocket listening port. |
| `RUST_LOG` | `info` | Logging verbosity filter (`error`, `warn`, `info`, `debug`, `trace`). |
| `ADMIN_USERNAME` | `admin` | Bootstrap administrator username. |
| `ADMIN_PASSWORD` | *(auto-generated)* | Bootstrap administrator password (hashed with Argon2id on startup). |
| `ADMIN_PASSWORD_HASH` | *(none)* | Pre-computed Argon2id password hash for bootstrap admin. |
| `JWT_SECRET` | *(auto-generated)* | 256-bit CSPRNG secret key for HMAC-SHA256 JWT session signing. |
| `CONTAINER_ENGINE` | `auto` | Container runtime to use: `podman`, `docker`, or `auto` (auto-detects socket). |
| `CONTAINER_SOCKET` | *(auto-detected)* | Path to container socket (`PODMAN_SOCKET` or `DOCKER_SOCKET` aliases accepted). |
| `GAME_DRIVER` | `minecraft` | Primary game driver to activate (`minecraft`, `palworld`, `valheim`). |
| `CONTAINER_NAME` | `minecraft-server` | Target container name for status and metrics (`PODMAN_CONTAINER_NAME` and `MINECRAFT_CONTAINER_NAME` accepted as aliases). |
| `DATA_DIR` | `/app/data` | Directory for persistent state files (`users.json`, `api_tokens.json`, `audit_log.jsonl`). |
| `MINECRAFT_DATA_DIR` | `/app/minecraft-data` | Path to Minecraft server root directory (worlds, plugins, configs). |
| `SYSTEMD_CONFIG_DIR` | `/app/systemd-config` | Directory containing systemd user Quadlet files (`*.container`). |
| `LAZYMC_CONFIG_FILE` | `/app/lazymc-config/lazymc.toml` | Path to lazymc hibernation configuration. |
| `DBUS_SESSION_BUS_ADDRESS` | `unix:path=/run/user/<UID>/bus` | D-Bus session bus address for systemd unit control. |
| `ALLOWED_ORIGINS` | `http://127.0.0.1:25500, http://localhost:25500` | Comma-separated list of allowed CORS origins (`*` to permit any origin). |
| `TOOLS_SYNC_INTERVAL_SECS` | `86400` | Interval in seconds for the automatic tools synchronization loop (Spark, Chunky, LuckPerms). |
| `CURSEFORGE_API_KEY` | *(empty)* | Optional API key for CurseForge modpack and addon catalog search. |

---

## 4. Local Development & UI Preview

To visually check frontend changes or UI routing locally without modifying a production server:

```bash
# Build the test image
podman build -t localhost/chipanel:latest -f Containerfile .

# Launch local preview on port 25501
podman rm -f chipanel-local 2>/dev/null
podman run -d --name chipanel-local -p 127.0.0.1:25501:25500 \
  -e HOST=0.0.0.0 -e PORT=25500 \
  -e ADMIN_USERNAME=admin -e ADMIN_PASSWORD=preview \
  -e RUST_LOG=info \
  localhost/chipanel:latest

# Open http://localhost:25501 in your browser (admin / preview)
# Cleanup when finished
podman rm -f chipanel-local
```

---

## 5. Operations & Diagnostic Runbook

### Check Service Status
```bash
systemctl --user status chipanel.service
```

### Stream Live Logs
```bash
journalctl --user -u chipanel.service -f --lines=100
```

### Health Check Endpoint
```bash
curl -s http://127.0.0.1:25500/api/health
```

### Quadlet Changes & Reload Gotcha
> [!IMPORTANT]
> Whenever you modify `chipanel.container` or any Quadlet file, you **must** execute `systemctl --user daemon-reload` before restarting the service. Without `daemon-reload`, changes to volume mounts, environment variables, or memory limits remain completely inactive.
>
> Additionally, `podman load` does not restart running containers. Always chain reload with restart.

```bash
systemctl --user daemon-reload
systemctl --user restart chipanel.service
```

### Inspect Container Resource Consumption
```bash
podman stats --no-stream chipanel
```

