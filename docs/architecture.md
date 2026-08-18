# Architecture & Internal Mechanisms of ChiPanel

ChiPanel is an ultra-lightweight, self-contained, high-performance management console engineered specifically for resource-constrained homelab hardware (e.g., Intel Core i3 2C/4T @ 1.70 GHz, 8 GB RAM) without competing for the CPU and RAM allocations required by the game server JVM.

---

## 1. System Architecture Overview

ChiPanel decouples into a native, compiled asynchronous Rust backend and a client-side Single Page Application (SPA) built with Svelte 5.

```
+-------------------------------------------------------------------------+
|                        WEB BROWSER (Client SPA)                         |
|   - Svelte 5 Runes: $state, $derived, $effect, $props, $bindable        |
|   - Dual-Mode UX (Novice 1-Click / Power-User DevOps, Alt+M)            |
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
|  │  [Multi-Container Engine Layer]    [Immutable Audit Log]          │  |
|  │  - Trait ContainerEngine           - Append-only audit_log.jsonl  │  |
|  │  - PodmanEngine (UDS + zbus D-Bus) - Filtered queries & CSV export│  |
|  │  - DockerEngine (Socket + demux)   - Auto-detector runtime        │  |
|  │                                                                   │  |
|  │  [Game Engine Drivers Registry]    [Background Async Engines]     │  |
|  │  - Trait GameDriver                - Mojang Version Watcher (1h)  │  |
|  │  - MinecraftDriver                 - Diagnostic Tools Sync (24h)  │  |
|  │  - Palworld / Valheim Drivers      - Fast-Path Log Watcher (<50ms)│  |
|  │  - Dynamic thread-safe Registry    - Telemetry Sampler (2s tick)  │  |
|  └──────────────────────────────────┬────────────────────────────────┘  |
+-------------------------------------┼────────────────────────────────---+
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

## 2. Multi-Container Engine Abstraction (`src/container/`)

ChiPanel is runtime-agnostic. Rather than coupling directly to Podman or Docker, it defines a unified asynchronous contract via the `ContainerEngine` trait:

```rust
#[async_trait]
pub trait ContainerEngine: Send + Sync {
    async fn get_status(&self, container_name: &str) -> Result<ContainerStatusResponse, AppError>;
    async fn get_stats(&self, container_name: &str) -> Result<Option<ContainerMetrics>, AppError>;
    async fn start_container(&self, container_name: &str) -> Result<bool, AppError>;
    async fn stop_container(&self, container_name: &str) -> Result<bool, AppError>;
    async fn restart_container(&self, container_name: &str) -> Result<bool, AppError>;
    async fn get_logs(&self, container_name: &str, tail: usize) -> Result<Vec<String>, AppError>;
    fn engine_type(&self) -> ContainerEngineType;
    fn is_available(&self) -> bool;
}
```

### Supported Runtime Implementations:
1. **`PodmanEngine` (`src/container/podman.rs`)**:
   - Primary engine for homelab hosts running Linux.
   - Communicates over the rootless Unix Domain Socket (`/run/user/<uid>/podman/podman.sock` or `PODMAN_SOCKET`) via low-level Hyper 1.0 HTTP.
   - Orchestrates systemd user Quadlets via D-Bus (`zbus`) to cleanly manage `minecraft.service` and `lazymc.service`.
2. **`DockerEngine` (`src/container/docker.rs`)**:
   - Universal engine for standard Docker hosts (`/var/run/docker.sock`, `/run/user/<uid>/docker.sock` or `DOCKER_HOST`).
   - Implements 8-byte multiplexed header decoding for Docker stdout/stderr streams.
3. **`AutoDetector` (`src/container/detector.rs`)**:
   - Probes available sockets at initialization (`CONTAINER_ENGINE=auto|podman|docker`) and seamlessly falls back to a mock/degraded mode if no daemon is accessible.

---

## 3. Modular Game Drivers Architecture (`src/engine/`)

ChiPanel separates the web management core from game-specific mechanics through the `GameDriver` trait:

```rust
#[async_trait]
pub trait GameDriver: Send + Sync {
    fn game_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    async fn get_status(&self) -> Result<GameServerStatus, AppError>;
    async fn send_command(&self, command: &str) -> Result<String, AppError>;
    async fn get_players(&self) -> Result<Vec<PlayerInfo>, AppError>;
    async fn kick_player(&self, player_id: &str, reason: Option<&str>) -> Result<(), AppError>;
    async fn ban_player(&self, player_id: &str, reason: Option<&str>) -> Result<(), AppError>;
    async fn unban_player(&self, player_id: &str) -> Result<(), AppError>;
    async fn get_telemetry(&self) -> Result<GameTelemetry, AppError>;
}
```

### Driver Registry & Multi-Game Dispatch:
- **`MinecraftDriver` (`src/engine/minecraft.rs`)**: Full-featured driver powering Java & Bedrock editions (RCON actor, 11 server loaders, NBT parsing, Modrinth API, Chunky, LuckPerms, lazymc).
- **`PalworldDriver` / `ValheimDriver` (`src/engine/palworld.rs`, `src/engine/valheim.rs`)**: Ready-to-use drivers managing dedicated servers for modern survival games.
- **`GameEngineRegistry` (`src/engine/registry.rs`)**: Thread-safe dynamic registry (`Arc<RwLock<HashMap<String, Box<dyn GameDriver>>>>`) allowing instances to be dispatched dynamically.

---

## 4. The Rust Backend (Axum 0.7 & Tokio)

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

### B. Fast-Path Instant Join Log Watcher (<50ms)
To execute queued moderation commands (`/api/players/action`) for offline players without waiting for the 2-second telemetry polling loop:
- ChiPanel inspects the live log stream emitted via WebSocket / container log reader.
- Detection of join regex patterns triggers immediate FIFO execution (<50ms response time) against the RCON actor.
- Execution results are broadcast to clients via WebSocket event `pending_commands_executed`.

---

## 5. Frontend Architecture (Svelte 5 Runes & Dual-Mode UX)

The user interface is an ultra-fast Single Page Application (SPA) built with SvelteKit 2 using `@sveltejs/adapter-static` and compiled with Vite 6.

### Svelte 5 Runes System
ChiPanel adopts modern Svelte 5 runes, eliminating legacy Svelte stores in favor of fine-grained, compile-time reactivity:
- **`$state()`**: Encapsulates component-local reactive state (modals, search queries, table filters, inputs, loading indicators).
- **`$derived()` & `$derived.by()`**: Computes memoized reactive calculations without unnecessary DOM re-renders (inventory slicing, Chunky percentage completion, audit statistics).
- **`$props()` & `$bindable()`**: Declares strictly typed component interfaces with bidirectional data bindings.
- **`$effect()`**: Manages lifecycle hooks, WebSocket event listeners, and `uPlot` chart resizing.
- **`untrack()`**: Isolates reactive dependencies inside polling loops and timer callbacks.

### Dual-Mode UX & Zero-Code Onboarding:
- **Novice Mode (1-Click Setup, `/setup`)**: 4-step visual wizard (`OnboardingWizard.svelte`), hardware-aware RAM slider, automatic loader recommendation (Purpur), 1-click EULA, and pre-activated lazymc hibernation.
- **Power-User Mode (DevOps)**: Unlocks direct Quadlet inspection, raw RCON console, Myers config diff viewer, cgroups memory limits, JVM flags, and S3 backup pipelines.
- **Zero-Latency Toggle**: `Alt+M` shortcut or hardware switch (`ModeSwitch.svelte`) with instantaneous reactivity managed by `stores/preferences.svelte.js`.

---

## 6. Server Lifecycle & lazymc Hibernation

ChiPanel manages the game server across three distinct operational states:

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

## 7. Desktop Application Vision (Tauri v2 / Localhost)

ChiPanel's clean decoupling (stateless REST API + WebSocket Hub + Static SPA) makes it fully ready for packaging into a standalone desktop application using **Tauri v2**:
- **Target Audience**: Players and creators wanting to host and manage games directly on local Windows, macOS, or Linux PCs without remote server setup.
- **Hybrid Operation**: Capable of either orchestrating local container runtimes (Podman Desktop, Docker Desktop) or remotely controlling a ChiPanel homelab instance via API keys over Tailscale.

---

## 8. Background Async Engines

ChiPanel runs four non-blocking background workers within Tokio:
1. **Mojang Version Watcher (`version_watch.rs`)**: Polls `launchermeta.mojang.com` every 1 hour, discovers newly released Minecraft versions or snapshots, updates the catalog cache, and notifies the UI.
2. **Diagnostic Tools Synchronizer (`tools.rs`)**: Runs every 24 hours to automatically provision and update engine-compatible builds of `spark`, `chunky`, `luckperms`, and `fabric-api`.
3. **Telemetry Sampler (`metrics.rs`)**: Ticks every 2 seconds, sampling host CPU (`/proc/stat`), RAM (`/proc/meminfo`), disk usage, and container metrics into a 3,600-sample in-memory ring buffer while evaluating Discord alert thresholds.
4. **Deferred Command Queue (`command_queue.rs`)**: Observes player joins over WebSocket/RCON and automatically executes pending moderation/give actions for previously offline players.
