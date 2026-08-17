# ChiPanel Technical Documentation

Welcome to the official technical documentation for **ChiPanel**, the lightweight, high-performance web management console for Minecraft servers and homelab services running on rootless Podman and systemd Quadlets.

---

## 📚 Documentation Index

The documentation suite is structured into 5 specialized guides:

1. [**Architecture & Internal Mechanisms** (`architecture.md`)](./architecture.md)
   - High-performance Rust async runtime (Axum 0.7 + Tokio) with minimal memory footprint (< 25 MB RAM idle).
   - Single-threaded Tokio Actor pattern for RCON communication with automatic reconnection.
   - Reactive Svelte 5 frontend leveraging the modern *Runes* architecture (`$state`, `$derived`, `$props`, `$effect`, `$bindable`, `untrack`).
   - System supervision via D-Bus session bus (`zbus`) and rootless Podman 5.4 Unix Domain Sockets.
   - Three-tier server lifecycle and native `lazymc` auto-hibernation proxy integration with zero-RAM sleep mode.
   - Background engines: Mojang Version Watcher, Diagnostic Tools Synchronizer, Telemetry Sampler, and Deferred Command Queue.

2. [**Exhaustive REST API & WebSocket Reference** (`api-reference.md`)](./api-reference.md)
   - Authentication via Argon2id, JWT session tokens, and persistent SHA-256 API access keys.
   - Complete reference for all 77 API endpoints across 18 specialized modules (`/api/auth`, `/api/server`, `/api/plugins`, `/api/modpacks`, `/api/players`, `/api/permissions`, `/api/worlds`, `/api/files`, `/api/metrics`, `/api/profiles`, `/api/tools`, `/api/logs`, `/api/diff`, `/api/maintenance`, `/api/geyser`, `/api/backups`, `/api/audit`, `/api/health`).
   - Full WebSocket bidirectional streaming protocol specification (`/ws`).

3. [**Functional Modules Guide** (`modules-guide.md`)](./modules-guide.md)
   - **Multiplexed RCON Actor**: Zero-lag Source protocol execution with dummy packet multi-packet termination.
   - **Interactive Console & Log Stream**: High-capacity memory ring buffer with automated regex redaction and 1-click `mclo.gs` export.
   - **NBT Inspector & Player Inventory**: Binary NBT decoding (`fastnbt`), inventory & Ender Chest visualizer, active potion effects, and LuckPerms group assignments.
   - **Addons & Modpack Catalog**: Modrinth API v2 and CurseForge integration with concurrent downloads and profile management.
   - **World Lifecycle & Chunky Integration**: Dimension discovery, gamerules, worldborder controls, zip import/export, and real-time chunk pre-generation tracking.
   - **Visual Config Diff Viewer**: In-browser Myers difference algorithm (`similar` crate) before committing file writes.
   - **Database Maintenance & Purge Engine**: Non-locking SQLite disk footprint inspection and automated maintenance.
   - **Cross-Play Bedrock & Geyser Assistant**: Automatic UDP 19132 port verification, encryption key validation, and configuration generator.
   - **Spark JVM Profiling**: RCON sampler triggers and automated Flamegraph viewer URL extraction.
   - **Scoped Backup Engine**: Targeted scopes (`full`, `world_only`, `configs_only`), SHA-256 integrity hashing, automatic retention, and remote S3 / MinIO export.
   - **Immutable Audit Trail**: Append-only event logging (`audit_log.jsonl`) with search, filtering, and RFC 4180 CSV export.

4. [**Deployment & Operations Runbook** (`deployment-and-operations.md`)](./deployment-and-operations.md)
   - Multi-stage container build process (`Containerfile`).
   - Production systemd user Quadlet configuration (`chipanel.container`).
   - Complete dictionary of all environment variables and default values.
   - Step-by-step homelab deployment workflow (local build, `podman save`, SCP transfer, `podman load`, systemd restart).
   - Operational runbook, health checks, journal logging, and troubleshooting gotchas.

5. [**Security Architecture & Threat Model** (`security.md`)](./security.md)
   - Argon2id password hashing with async reactor offloading and rate limiting.
   - Role-Based Access Control (RBAC) model (`admin`, `viewer`, `operator`).
   - Strict path canonicalization, prefix verification, and Zip-Slip vulnerability prevention.
   - Discord alert SSRF filters (HTTPS enforcement, domain whitelisting, redirect blocking).
   - PII and credential sanitization for log exports.
   - Rootless Podman container isolation and Linux user namespaces (`subuid` / `subgid`).
