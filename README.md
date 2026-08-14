# ChiPanel - Lightweight Homelab & Podman Management Console

ChiPanel is a high-performance, dark-mode-first homelab management console built with a Rust (Axum 0.7) backend and a SvelteKit (Svelte 5) frontend. It integrates natively with rootless Podman via systemd Quadlets and provides Tailscale Funnel / HTTPS endpoints.

---

## 🏗️ Architecture Overview

```
                        +-----------------------------------+
                        |    Tailscale Funnel / HTTPS       |
                        +-----------------+-----------------+
                                          |
                                          v
+---------------------------------------------------------------------------------+
| Podman Rootless Container (systemd Quadlet: chipanel.container)                 |
|                                                                                 |
|  +-----------------------------------+   +-----------------------------------+  |
|  | SvelteKit Static SPA (Build Stage) |   | Axum 0.7 Rust Backend             |  |
|  | - Svelte 5 + Vite                 |   | - Tokio + Hyper (Unix Socket)     |  |
|  | - uPlot / CodeMirror / Lucide     |<->| - WebSockets (/ws)                |  |
|  | - Served via Axum static fallback |   | - Health Check (/api/health)      |  |
|  +-----------------------------------+   +-----------------+-----------------+  |
+------------------------------------------------------------|--------------------+
                                                             |
                                                             v
                                        +--------------------+--------------------+
                                        | /run/user/1000/podman/podman.sock       |
                                        +-----------------------------------------+
```

### Core Components
1. **Rust Backend (`/backend`)**:
   - Built on Axum 0.7 and Tokio.
   - Communicates with rootless Podman via `/run/user/1000/podman/podman.sock` using `hyper-util`.
   - Supports JWT authentication (`jsonwebtoken`), password hashing (`argon2`), NBT parsing (`fastnbt`), and WebSockets (`axum::extract::ws`).
   - Serves API routes (`/api/*`) and falls back to static frontend assets (`../frontend/build`).

2. **SvelteKit Frontend (`/frontend`)**:
   - Built with Svelte 5 and `@sveltejs/adapter-static` (`fallback: 'index.html'`).
   - Styled with a dark-mode-first Linear/Vercel design system (`app.css`).
   - Includes `@codemirror` YAML editing, `uplot` metrics charts, and `lucide-svelte` iconography.

3. **DevOps & Containerization**:
   - **Multi-Stage `Containerfile`**: Node 22 builder + Rust 1.85 release compiler + Distroless `cc-debian12` runtime (< 50 MB total image size).
   - **Systemd Quadlet (`chipanel.container`)**: Declarative container management under user `chiserv`.

---

## 📁 Directory Structure

```
chipanel/
├── backend/
│   ├── Cargo.toml               # Rust 2024 edition, Axum 0.7, Tokio, Serde, Hyper dependencies
│   └── src/
│       └── main.rs              # Web server bootstrap, health route, static file fallback
├── frontend/
│   ├── package.json             # Svelte 5, SvelteKit, Adapter-Static, Vite, CodeMirror, uPlot
│   ├── svelte.config.js         # Static SPA configuration (fallback: 'index.html')
│   ├── vite.config.js           # SvelteKit plugin + dev proxy (/api -> 3000)
│   └── src/
│       ├── app.html             # HTML5 root template with Inter & JetBrains Mono fonts
│       ├── app.css              # ChiPanel Design System tokens & styles
│       └── routes/
│           ├── +layout.svelte   # App layout & styling container
│           └── +page.svelte     # Dashboard UI overview & metrics table
├── Containerfile                # Multi-stage image build (< 50MB runtime target)
├── chipanel.container           # Podman Quadlet systemd user service definition
├── Makefile                     # Build & development task runner
└── README.md                    # Project documentation
```

---

## 🚀 Quick Start Guide

### Prerequisites
- Rust 1.85+ (`cargo`)
- Node.js 22+ (`npm`)
- Podman (for Quadlet execution)

### 1. Running in Development Mode

Run backend and frontend in separate terminals:

```bash
# Terminal 1: Backend API server (http://localhost:3000)
make dev-backend

# Terminal 2: Frontend dev server with proxy (http://localhost:5173)
make dev-frontend
```

### 2. Local Release Build

```bash
# Build both frontend and backend
make build-frontend
make build-backend
```

### 3. Container & Quadlet Deployment

```bash
# Build Podman container image
make build-container

# Deploy systemd Quadlet configuration to ~/.config/containers/systemd/
make deploy-quadlet

# Check systemd user service status
systemctl --user status chipanel
```

---

## 🛠️ Make Commands

| Command | Description |
| :--- | :--- |
| `make dev-backend` | Starts the Rust Axum backend server in development mode |
| `make dev-frontend` | Starts the SvelteKit Vite dev server with proxy to backend |
| `make build-frontend` | Compiles the SvelteKit frontend to static files in `frontend/build` |
| `make build-backend` | Compiles the release Rust binary in `backend/target/release` |
| `make build-container` | Builds the optimized multi-stage Podman container image (`chipanel:latest`) |
| `make deploy-quadlet` | Copies `chipanel.container` to Quadlet directory and reloads systemd user daemon |
| `make clean` | Removes all build artifacts and node modules |

---

## ⚙️ Environment Variables

| Variable | Default | Description |
| :--- | :--- | :--- |
| `PORT` | `3000` | HTTP port for backend server |
| `RUST_LOG` | `info` | Log level filter for `tracing-subscriber` |
| `PODMAN_SOCKET` | `/run/user/1000/podman/podman.sock` | Path to rootless Podman API socket |
