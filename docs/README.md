# ChiPanel Technical Documentation

Welcome to the technical documentation for **ChiPanel**, a management console for Minecraft servers running on rootless Podman and systemd Quadlets.

---

## Documentation Index

The documentation is organized into 5 guides:

1. [**Architecture & Internal Mechanisms** (`architecture.md`)](./architecture.md)
   - Rust async runtime (Axum 0.7 + Tokio) with under 25 MB RAM idle footprint.
   - Single-threaded Tokio actor pattern for RCON communication with automatic reconnection.
   - Reactive Svelte 5 frontend using Runes (`$state`, `$derived`, `$props`, `$effect`, `$bindable`, `untrack`).
   - System supervision via D-Bus session bus (`zbus`) and rootless Podman 5.4 Unix domain sockets.
   - Three-tier server lifecycle and `lazymc` auto-hibernation proxy integration.
   - Background engines: Mojang Version Watcher, Tools Synchronizer, Telemetry Sampler, and Deferred Command Queue.

2. [**REST API & WebSocket Reference** (`api-reference.md`)](./api-reference.md)
   - Authentication via Argon2id, JWT session tokens, and persistent SHA-256 API keys.
   - Complete reference for all 77 API endpoints across 18 route modules.
   - Bidirectional WebSocket protocol specification (`/ws`).

3. [**Functional Modules Guide** (`modules-guide.md`)](./modules-guide.md)
   - **Multiplexed RCON Actor**: Source protocol packet framing with dummy packet stream termination.
   - **Console & Log Redaction**: Memory ring buffer with automated regex redaction and one-click `mclo.gs` export.
   - **NBT Inspector & Player Inventory**: In-memory NBT parsing (`fastnbt`), 2D inventory visualizer, active potion effects, and LuckPerms groups.
   - **Addons & Modpack Catalog**: Modrinth v2 and CurseForge search, concurrent downloads, and profile manager.
   - **World Lifecycle & Chunky**: Dimension discovery, gamerules, worldborder, zip import/export, and real-time chunk pregeneration tracking.
   - **Visual Config Diff**: In-browser Myers difference algorithm (`similar` crate) before committing file edits.
   - **Database Maintenance**: Non-locking SQLite disk footprint inspection for CoreProtect and LuckPerms.
   - **Bedrock Cross-Play**: UDP 19132 port verification, encryption key validation, and automated Geyser config generator.
   - **Spark Profiling**: RCON sampler triggers and automated Flamegraph viewer URL extraction.
   - **Scoped Backups**: Selectable scopes (`full`, `world_only`, `configs_only`), SHA-256 hashing, retention quotas, and direct S3 streaming.
   - **Audit Trail**: Append-only event logging (`audit_log.jsonl`) with search, filtering, and RFC 4180 CSV export.

4. [**Deployment & Operations Runbook** (`deployment-and-operations.md`)](./deployment-and-operations.md)
   - Multi-stage container build process (`Containerfile`).
   - Production systemd user Quadlet configuration (`chipanel.container`).
   - Full dictionary of all 18 environment variables and defaults.
   - Homelab deployment workflow (local build, `podman save`, SCP transfer, `podman load`, systemd restart).
   - Operations runbook: health checks, journal logging, and `daemon-reload` requirements.

5. [**Security Architecture & Threat Model** (`security.md`)](./security.md)
   - Argon2id password hashing with async reactor offloading and rate limiting.
   - Role-Based Access Control (`admin`, `viewer`, `operator`).
   - Path canonicalization, prefix boundary enforcement, and Zip-Slip protection.
   - Discord alert SSRF filters (HTTPS enforcement, domain whitelisting, redirect blocking).
   - Credential and PII sanitization for log exports.
   - Rootless Podman container isolation and Linux user namespaces (`subuid` / `subgid`).
