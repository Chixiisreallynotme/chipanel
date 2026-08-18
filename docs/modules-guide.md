# Functional Modules Guide for ChiPanel

This document provides an in-depth technical breakdown of the architecture, algorithms, and interfaces across all core subsystems in ChiPanel.

---

## 1. Multi-Container Runtime Engine (`src/container/`)

ChiPanel abstracts container management behind the `ContainerEngine` trait, making the panel completely portable across runtimes:
- **`PodmanEngine` (`podman.rs`)**:
  - Targets rootless Podman over `/run/user/<uid>/podman/podman.sock`.
  - Interfaces directly with `systemd --user` via D-Bus (`zbus`) for managing `.container` Quadlets.
  - Automatically handles cgroup v2 metrics collection for non-root containers.
- **`DockerEngine` (`docker.rs`)**:
  - Connects to standard Docker daemon sockets (`/var/run/docker.sock`, `/run/user/<uid>/docker.sock` or `DOCKER_HOST`).
  - Demultiplexes standard 8-byte frame headers (stdout/stderr channel indicators) for logs and streams.
- **`AutoDetector` (`detector.rs`)**:
  - Evaluates socket availability at startup and selects the optimal engine (`CONTAINER_ENGINE=auto|podman|docker`).
- **Container Profiles & Presets (`profiles.rs`)**:
  - Built-in profiles for `LazymcCustom`, `ItzgMinecraft` (`itzg/minecraft-server` standard variables: `EULA`, `TYPE`, `VERSION`, `MEMORY`, `JVM_OPTS`), and `GenericOci`.

---

## 2. Modular Game Drivers & Registry (`src/engine/`)

The core server management loop is decoupled from game-specific mechanics through the `GameDriver` trait and dynamic registry:
- **`MinecraftDriver` (`minecraft.rs`)**:
  - Manages Java & Bedrock editions.
  - Implements RCON actor protocol, NBT player data decoding, Modrinth/CurseForge addons, Chunky pregeneration, LuckPerms permissions, and lazymc hibernation.
- **`PalworldDriver` (`palworld.rs`) & `ValheimDriver` (`valheim.rs`)**:
  - Ready-to-use drivers managing dedicated server instances, lifecycle states, process telemetry, and player queries.
- **`GameEngineRegistry` (`registry.rs`)**:
  - Thread-safe registry (`Arc<RwLock<HashMap<String, Box<dyn GameDriver>>>>`) enabling multi-game hosting from a single ChiPanel instance.

---

## 3. Multiplexed RCON Actor (`src/rcon/`)

### Architecture & Connection Resilience
The RCON subsystem isolates all TCP communication with the Minecraft JVM within a single Tokio actor loop:
- **Actor Message Envelope**: Callers invoke `RconActorHandle::exec("command")`, which dispatches a message over `tokio::sync::mpsc::channel(128)` with a dedicated `oneshot` return channel.
- **Source Binary Framing**: Implements standard binary Source RCON protocol packets (`[Length: i32 LE] [Request ID: i32 LE] [Packet Type: i32 LE] [Payload: UTF-8] [0x00] [0x00]`).
- **Dummy Packet Multi-Packet Stream Terminator**: Minecraft servers split outputs across multiple TCP packets without an end-of-stream delimiter. ChiPanel immediately follows each command packet with an empty dummy packet (`SERVERDATA_RESPONSE_VALUE`). Responses matching the command ID are streamed and concatenated until the dummy response ID is received, preventing hangs and guaranteeing complete response delivery.
- **Automatic Recovery**: If the socket breaks (e.g. server restart or hibernation), the actor discards the socket and attempts a reconnect with backoff retry without blocking HTTP handlers.

---

## 4. Live Console & Log Redaction with mclo.gs (`src/routes/logs.rs`)

### Real-Time Log Pipeline
- **Memory Ring Buffer**: The frontend maintains a 5,000-line circular buffer optimized for virtualized DOM rendering.
- **Automated Regex Redaction**: Before logs are shared publicly or exported, `routes/logs.rs` strips sensitive data:
  - Public IPv4 and IPv6 addresses.
  - RCON authentication passwords and Discord webhook tokens.
  - Host filesystem paths containing server usernames.
- **mclo.gs API v1 Integration**: Directly uploads the sanitized log buffer to `https://api.mclo.gs/1/log` and returns the shareable URL along with automated syntax error insights.

---

## 5. Visual Config Diff Engine (`src/routes/diff.rs`)

### In-Browser Myers Difference Analysis
ChiPanel integrates the Rust `similar` crate to compare unsaved in-memory text with the existing configuration file on disk (`server.properties`, `paper-global.yml`, `purpur.yml`):
- **Diff Computation**: Computes Myers diff operations (`Insert`, `Delete`, `Equal`).
- **Side-by-Side Visualization**: Displays added lines in green and deleted lines in red with line numbers and change metrics before writing changes to disk.
- **Atomic File Writes**: Files are written to `.tmp_save` before being atomically renamed, preventing file corruption if interrupted.

---

## 6. Player NBT Inspector & Inventory Visualizer (`src/minecraft/`)

### Binary NBT Parsing via `fastnbt`
ChiPanel directly decompresses and decodes player `.dat` files (`world/playerdata/{uuid}.dat`):
- **Inventory Slicing (`inventory.rs`)**: Parses 36 main inventory slots (hotbar 0..=8, main grid 9..=35), 4 armor slots (100 boots, 101 leggings, 102 chestplate, 103 helmet), 1 offhand slot (-106 / 150), and 27 Ender Chest slots (0..=26).
- **Modern & Legacy Item Format Support**: Handles legacy NBT tags (`tag.Damage`, `tag.Enchantments`) and modern Minecraft 1.20.5+ item components (`components["minecraft:damage"]`, `components["minecraft:enchantments"]`, `components["minecraft:trim"]`).
- **Active Potion Effects (`effects.rs`)**: Decodes namespaced IDs, duration ticks, amplifier levels, and maps them to effect categories (`beneficial`, `harmful`, `neutral`).
- **Player Metrics (`player.rs`)**: Aggregates health, max health, food levels, XP progress, exact XYZ coordinates, dimension mapping, playtime from `world/stats/<uuid>.json`, and OP/Ban/Whitelist status.

---

## 7. Deferred Offline Command Queue & Fast-Path Log Watcher (`src/minecraft/command_queue.rs`)

### Automated Reconnection Execution
When an administrator executes an action on an offline player (e.g. giving items, changing gamemodes, modifying LuckPerms groups):
- **In-Memory Store**: Queues commands in `pending_commands.json` wrapped in an atomic write lock.
- **Fast-Path Instant Join Log Watcher (<50ms)**: Rather than waiting for the periodic 2-second telemetry tick, ChiPanel actively watches the live log stream. When a player join regex is detected, pending commands for that UUID are executed immediately via the RCON actor.
- **History Audit**: Executed commands are recorded in the history log (capped at 150 entries) and broadcast to connected clients via WebSocket event `pending_commands_executed`.

---

## 8. Addons & Modpack Catalog Manager (`src/routes/plugins.rs`, `src/routes/modpacks.rs`)

### Multi-Loader Addons Engine
- **Deep JAR Inspection**: Reads internal archive manifests (`plugin.yml`, `paper-plugin.yml`, `fabric.mod.json`, `mods.toml`, `neoforge.mods.toml`, `pack.mcmeta`) to discover addon metadata, versions, main classes, and loader types.
- **Modrinth v2 & CurseForge Catalog**: Searches plugins and mods with automatic compatibility filtering based on current server engine and game version.
- **Modpack Deployment**: Parses modpack `manifest.json`, extracts `overrides/`, and downloads mods concurrently using a `tokio::sync::Semaphore(4)` permit limiter.
- **Automated Update Checker**: Calculates SHA-512 hashes of installed JARs, queries Modrinth API for compatible upgrades, and performs atomic batch replacements.

---

## 9. Engine & Version Switcher (`src/routes/engine_catalog.rs`)

### Dynamic Multi-Engine Support
Supports 11 server engines: *Paper, Purpur, Folia, Fabric, Forge, NeoForge, Quilt, Mohist, Arclight, Spigot, Vanilla*.
- **Mojang Manifest Sync**: Synchronizes with Mojang release and snapshot manifests in the background.
- **Quadlet Rewriting**: Automatically updates `TYPE`, `VERSION`, and loader environment variables in systemd Quadlet files.
- **LuckPerms Migration (`luckperms.rs`)**: Automatically migrates LuckPerms data directories across engine switches (e.g. moving between `plugins/LuckPerms`, `mods/luckperms`, and `config/luckperms`) with pre-switch snapshots.

---

## 10. World Lifecycle & Chunky Pregeneration (`src/minecraft/worlds.rs`, `src/routes/worlds.rs`)

### Dimension & World Management
- **NBT Discovery**: Scans world directories, decodes `level.dat` to extract seeds, generator types, spawn coordinates, and data versions.
- **Pending World Slots**: Allows configuring ungenerated worlds with custom parameters (`seed`, `level_type`, `gamemode`, `difficulty`) applied automatically on activation.
- **Chunky Integration**: Controls Chunky pregeneration via RCON (`/chunky start`, `/chunky pause`, `/chunky cancel`) and parses real-time progress (chunks rendered, completion %, CPS, ETA).
- **Import & Export**: Streams world ZIP archives with Zip-Slip protection and automated root folder unwrapping.

---

## 11. Database Maintenance & Storage Inspector (`src/routes/database.rs`)

### Disk Footprint Analysis
- **Non-Locking SQLite Inspection**: Reads SQLite header and table metadata from `plugins/CoreProtect/database.db` and `LuckPerms` without locking active databases.
- **Dimension Region Breakdown**: Calculates disk usage of Overworld (`region/`), Nether (`DIM-1/`), and End (`DIM1/`).
- **Safe RCON Purge**: Executes maintenance commands via native plugin hooks (`/co purge t:30d`, `/lp prune 30`) or dimension resets.

---

## 12. Cross-Play Bedrock & Geyser Assistant (`src/routes/geyser.rs`)

### Java / Bedrock Bridge Management
- **Integrity Validation**: Verifies installation of `Geyser-Spigot.jar`, `Floodgate.jar`, and presence of shared cryptographic key `key.pem`.
- **Network Port Check**: Validates UDP port 19132 exposure.
- **Configuration Generator**: Generates and applies optimized `plugins/Geyser-Spigot/config.yml` templates for homelab environments.

---

## 13. Spark JVM Profiling (`src/routes/tools.rs`)

### JVM Performance Profiling
- **Sampler Trigger**: Starts the Spark CPU profiler via RCON (`/spark sampler --viewer`) for configured durations (5 to 300 seconds).
- **URL Extraction**: Parses console output using regex (`https://spark\.lucko\.me/[a-zA-Z0-9]+`) and returns the Flamegraph viewer URL.
- **JVM Health Check**: Queries Old Gen, Young Gen, and Garbage Collection statistics via `/spark health --memory`.

---

## 14. Scoped Backup Engine (`src/minecraft/server_backup.rs`)

### High-Performance Archival
- **Selectable Scopes**:
  - `full`: Entire server directory.
  - `world_only`: World folders (`world`, `world_nether`, `world_the_end`).
  - `configs_only`: `.yml`, `.json`, `.properties`, and plugin configuration directories.
- **Smart Filters**: Automatically excludes transient logs (`*.log.gz`), caches (`cache/*`), backups (`backups/*`), and map render tiles (`dynmap/web/tiles/*`, `bluemap/web/maps/*`).
- **Integrity & Remote Export**: Computes SHA-256 archive checksums, enforces automatic retention limits, and streams backups directly to S3 / MinIO buckets.

---

## 15. Immutable Audit Trail (`src/audit/`)

### Append-Only Security Logging
- **Structured Storage**: Records administrative events (power state, config changes, RCON commands, backups, permissions) in `data/audit_log.jsonl`.
- **Query & Filters**: Supports searching by action category, username, timestamp range, and execution status (`SUCCESS` / `FAILED`).
- **CSV Export**: Generates compliant RFC 4180 CSV exports for security reviews.

---

## 16. Dual-Mode Preferences & Hardware Probe (`stores/preferences.svelte.js`)

### Svelte 5 Reactive Client State
- **Mode Switching**: Toggles between `novice` (1-Click setup) and `expert` (Power-User DevOps) modes via `$state` rune and `localStorage.chipanel_user_mode`.
- **Host Hardware Sonde**: Queries `/api/metrics/current` to detect total host RAM and active CPU topology for the smart RAM recommendation algorithm in the setup wizard.

