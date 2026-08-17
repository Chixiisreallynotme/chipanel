# Architecture & Internal Mechanisms of ChiPanel

ChiPanel is an ultra-lightweight, self-contained, high-performance management console engineered specifically for resource-constrained homelab hardware (e.g., Intel Core i3 2C/4T @ 1.70 GHz, 8 GB RAM) without competing for the CPU and RAM allocations required by the Minecraft server JVM.

---

## 1. System Architecture Overview

ChiPanel decouples into a native, compiled asynchronous Rust backend and a client-side Single Page Application (SPA) built with Svelte 5.

```
+-------------------------------------------------------------------------+
|                        WEB BROWSER (Client SPA)                         |
|   - Svelte 5 Runes: $state, $derived, $effect, $props, $bindable        |
|   - Native WebSocket Client (Log streaming, live telemetry, queue)      |
|   - CodeMirror 6 (Syntax highlighting, Myers visual diff editor)        |
|   - uPlot (Ultra-fast Canvas time-series synchronized graphs)           |
+-------------------------------------------------------------------------+
                                     │  HTTP / WebSocket (Port 25500)
                                     ▼
+-------------------------------------------------------------------------+
|                     CHIPANEL BACKEND (Rust / Axum 0.7)                  |
|                                                                         |
|  ┌───────────────────────────────────────────────────────────────────┐  |
|  │                        HTTP Routing Layer (Axum 0.7)              │  |
|  │   - Auth Middleware (Argon2id / HS256 JWT / Persistent API tokens)│  |
|  │   - REST Endpoints (/api/server, /api/plugins, /api/players...)   │  |
|  │   - WebSocket Hub (Connection multiplexer, 2s telemetry broadcast)│  |
|  └──────────────────────────────────┬────────────────────────────────┘  |
|                                     │                                   |
|  ┌──────────────────────────────────┴────────────────────────────────┐  |
|  │                 Core Subsystems & Tokio Actors                    │  |
|  │                                                                   │  |
|  │  [Multiplexed RCON Actor]          [Backup Engine]                │  |
|  │  - Single persistent TCP socket    - Scoped ZIP creation (zstd)   │  |
|  │  - Source protocol framing         - SHA-256 integrity hashing    │  |
|  │  - Tokio mpsc(128) + oneshot       - Direct S3 / MinIO streaming  │  |
|  │  - Auto-reconnect backoff          - Automated retention policy   │  |
|  │                                                                   │  |
|  │  [Systemd & Podman Integration]    [Immutable Audit Log]          │  |
|  │  - UDS: /run/user/1000/podman.sock - Append-only audit_log.jsonl  │  |
|  │  - D-Bus: /run/user/1000/bus (zbus)- Filtered queries & CSV export│  |
|  │                                                                   │  |
|  │  [Background Async Engines]                                       │  |
|  │  - Mojang Version Watcher (1h)     - Telemetry Sampler (2s tick)  │  |
|  │  - Diagnostic Tools Sync (24h)     - Deferred Command Queue Loop  │  |
|  └──────────────────────────────────┬────────────────────────────────┘  |
+-------------------------------------┼───────────────────────────────────+
                                      │
              ┌───────────────────────┼───────────────────────┐
              ▼                       ▼                       ▼
   +─────────────────────+ +─────────────────────+ +─────────────────────+
   |   Podman Rootless   | |  lazymc (TCP Proxy) | |   Minecraft JVM     |
   |   (UID 1000 / crun) | |   Port 25565 proxy  | |    Port 25566 TCP   |
   |  minecraft.container| |   Zero-RAM Sleep    | |    Port 25575 RCON  |
   +─────────────────────+ +─────────────────────+ +─────────────────────+
```

---

## 2. The Rust Backend (Axum 0.7 & Tokio)

The backend compiles to a single static binary with Link-Time Optimization (`opt-level = "z"`, `lto = true`, `panic = "abort"`, `codegen-units = 1`). The resulting executable binary occupies under 15 MB on disk and maintains a baseline memory footprint of under 25 MB RAM.

### A. Persistent RCON Actor Pattern (`src/rcon/actor.rs`, `src/rcon/client.rs`)
Traditional server panels open a new TCP socket for every RCON command or telemetry query. This causes:
- Cascading `Connection refused (os error 111)` errors when the server is stopped or sleeping.
- Linux file descriptor exhaustion under concurrent web dashboard users.
- Substantial CPU overhead and log spam on the Minecraft JVM.

**The ChiPanel Solution:**
ChiPanel implements the **Tokio Actor Pattern**:
1. A single async task (`RconActor`) maintains an exclusive, persistent TCP connection to `127.0.0.1:25575`.
2. Callers send commands over a bounded `tokio::sync::mpsc::channel(128)` envelope with a dedicated `oneshot::Sender` return channel.
3. Callers enforce an 8-second timeout (`tokio::time::timeout`).
4. If a connection drops (e.g. server shutdown or hibernation), the actor enters an automatic reconnect state with backoff retry without blocking HTTP handlers or leaking 500 errors to web clients.

#### Multi-Packet Response Handling (Dummy Packet Technique)
The Minecraft Source RCON protocol splits large command outputs across multiple packets without an end-of-stream delimiter. ChiPanel guarantees complete message collection by sending an immediate empty dummy packet (`SERVERDATA_RESPONSE_VALUE`) following each command. Responses matching the request ID are concatenated until the response with `id == dummy_id` is received, cleanly terminating the stream.

### B. Rootless Podman & Systemd D-Bus Integration
ChiPanel runs inside an isolated rootless container (UID 1000) and communicates with the host through two read-only mounts:
1. **Rootless Podman Unix Socket (`/run/user/1000/podman/podman.sock`)**: Communicates with the Libpod REST API using low-level Hyper 1.0 HTTP client over Unix Domain Sockets (`tokio::net::UnixStream`) to query container status and compute CPU/RAM cgroups metrics.
2. **User D-Bus Session Bus (`/run/user/1000/bus`)**: Uses the `zbus` crate to directly instruct `systemd --user` manager (`org.freedesktop.systemd1.Manager`) to start, stop, or restart Quadlets (`minecraft.service`) and native units (`lazymc.service`). If D-Bus is temporarily unreachable, ChiPanel transparently falls back to direct Libpod API calls.

---

## 3. Frontend Architecture (Svelte 5 Runes)

The user interface is an ultra-fast Single Page Application (SPA) built with SvelteKit 2 using `@sveltejs/adapter-static` and compiled with Vite 6.

### Svelte 5 Runes System
ChiPanel adopts modern Svelte 5 runes, eliminating legacy Svelte stores (`writable()`, `readable()`) in favor of fine-grained, compile-time reactivity:
- **`$state()`**: Encapsulates component-local reactive state (modals, search queries, table filters, inputs, loading indicators).
- **`$derived()` & `$derived.by()`**: Computes memoized reactive calculations without unnecessary DOM re-renders (inventory slicing, Chunky percentage completion, audit statistics).
- **`$props()` & `$bindable()`**: Declares strictly typed component interfaces with bidirectional data bindings.
- **`$effect()`**: Manages lifecycle hooks, WebSocket event listeners, and `uPlot` chart resizing.
- **`untrack()`**: Isolates reactive dependencies inside polling loops and timer callbacks.

### Core Frontend Libraries
- **CodeMirror 6**: syntax highlighting for YAML, Properties, JSON, TOML, and integrated visual diff viewer.
- **uPlot**: high-performance Canvas-based charting engine rendering thousands of telemetry data points without UI frame drops.
- **Lucide Svelte**: lightweight SVG icon set.

---

## 4. Server Lifecycle & lazymc Hibernation

ChiPanel manages the Minecraft server across three distinct operational states:

```
             ┌──────────────────────────────────────────────┐
             │                 OFF MODE                     │
             │  (Container stopped, lazymc.service stopped) │
             │            RAM Consumption: 0 MB             │
             └───────────────────────┬──────────────────────┘
                                     │
                        Start via ChiPanel Dashboard
                                     │
                                     ▼
             ┌──────────────────────────────────────────────┐
             │              HIBERNATION MODE                │
             │  (lazymc TCP proxy active on port 25565,     │
             │   Minecraft Java container sleeping on 25566)│
             │           RAM Consumption: ~8 MB             │
             └───────────────────────┬──────────────────────┘
                                     │
                         Player Connects
                        (TCP Handshake Detected)
                                     │
                                     ▼
             ┌──────────────────────────────────────────────┐
             │                 ACTIVE MODE                  │
             │  (Java container running, world in memory)   │
             │      Auto-sleep after 15 min inactivity      │
             │         RAM Consumption: ~2.5 to 8 GB        │
             └──────────────────────────────────────────────┘
```

1. **Off Mode**: All processes are stopped.
2. **Hibernation Mode (Default)**: `lazymc.service` listens on port 25565. When a player attempts to connect, `lazymc` holds the TCP handshake open, triggers `minecraft-wake.sh` to start `minecraft.container` via Podman, waits for the internal port 25566 to accept traffic, and transparently bridges the session without disconnecting the player.
3. **Auto-Hibernation**: After 15 minutes of zero connected players, `lazymc` safely stops the Java container, immediately reclaiming 2.5 to 8 GB of RAM for other homelab workloads (Jellyfin, Ente Photos, Seafile).

---

## 5. Background Async Engines

ChiPanel runs four non-blocking background workers within Tokio:
1. **Mojang Version Watcher (`version_watch.rs`)**: Polls `launchermeta.mojang.com` every 1 hour, discovers newly released Minecraft versions or snapshots, updates the catalog cache, and notifies the UI.
2. **Diagnostic Tools Synchronizer (`tools.rs`)**: Runs every 24 hours to automatically provision and update engine-compatible builds of `spark`, `chunky`, `luckperms`, and `fabric-api`.
3. **Telemetry Sampler (`metrics.rs`)**: Ticks every 2 seconds, sampling host CPU (`/proc/stat`), RAM (`/proc/meminfo`), disk usage, and container metrics into a 3,600-sample in-memory ring buffer while evaluating Discord alert thresholds.
4. **Deferred Command Queue (`command_queue.rs`)**: Observes player joins over WebSocket/RCON and automatically executes pending moderation/give actions for previously offline players.
