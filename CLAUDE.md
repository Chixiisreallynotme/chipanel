# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

ChiPanel — homelab & Minecraft server management console. Rust/Axum backend + SvelteKit (Svelte 5) frontend, packaged into a single ~50MB distroless container and run via a rootless Podman Quadlet (`chipanel.container`) on the target host (user `chiserv`, UID 1000). There is no CI/registry — deployment is manual image build + transfer (see Commands).

## Commands

**Backend** (from `backend/`):
- `cargo run` — dev server, listens on `HOST:PORT` (defaults to `127.0.0.1:25500`); also `make dev-backend` from repo root
- `cargo build --release` — release binary; also `make build-backend`
- `cargo test` — run tests; `cargo test <name>` for a single test. Test modules currently live in `routes/players.rs`, `minecraft/inventory.rs`, `minecraft/effects.rs`, `minecraft/permissions.rs`
- If no local Rust toolchain is available, validate backend changes via `podman build -f Containerfile .` instead — it runs the same `cargo build --release` inside the `rust:slim` build stage and will surface compile errors

**Frontend** (from `frontend/`):
- `npm run dev` — Vite dev server on `:5173`; proxies `/api` and `/ws` to `http://localhost:25500` (see `vite.config.js`), so a backend must already be running there
- `npm run build` — static build to `frontend/build/` (`adapter-static`, SPA fallback to `index.html`)
- `npm run check` — `svelte-check`. This repo has a batch of pre-existing `any`-typed errors in the `players`/`worlds`/`plugins` routes; don't treat the total error count as a regression signal on unrelated changes — compare against a baseline run of the same command
- No test runner is configured for the frontend (no vitest/jest/playwright in `package.json`)

**Container & deployment** (from repo root):
- `make build-container` — `podman build -t chipanel:latest -f Containerfile .` (multi-stage: `node:22-alpine` frontend build → `rust:slim` backend build → `gcr.io/distroless/cc-debian12` runtime)
- `make deploy-quadlet` — copies `chipanel.container` to `~/.config/containers/systemd/` and runs `systemctl --user daemon-reload`
- Redeploying a running instance in practice: build locally → `podman save` → `scp` to the host → `podman load` → `systemctl --user restart chipanel` (a `podman load` alone does not restart the running container). **Rebuild with `podman build --no-cache`** when validating a fix meant to change behavior — Podman's layer cache has silently shipped a stale frontend build here even after source changes.
- Always build/tag as `localhost/chipanel:latest` — that's the exact reference `chipanel.container`'s `Image=` line expects.

## Local preview

To visually check changes before pushing to the host, run the built image locally instead of relying on `npm run dev` (which needs a live backend on `:25500` anyway):

```bash
podman build -t localhost/chipanel:latest -f Containerfile .
podman rm -f chipanel-local 2>/dev/null
podman run -d --name chipanel-local -p 127.0.0.1:25501:25500 \
  -e HOST=0.0.0.0 -e PORT=25500 \
  -e ADMIN_USERNAME=admin -e ADMIN_PASSWORD=preview \
  -e RUST_LOG=info \
  localhost/chipanel:latest
```

Then open **http://localhost:25501** (login `admin` / `preview`). Port `25501` on the host maps to `25500` inside the container. `HOST=0.0.0.0` is required inside the container or the port mapping silently can't reach it (the app's own default is `127.0.0.1`, only overridden to `0.0.0.0` in the real deployed quadlet). No Podman socket, RCON, or Minecraft data volumes are mounted, so container/player/telemetry data reads as disconnected/default — this is for reviewing UI, routing, and auth, not for exercising real server integration. Stop it with `podman rm -f chipanel-local`.

## Architecture

**One container serves both halves.** The Axum backend serves `/api/*` and the `/ws` WebSocket, and falls back to `ServeDir`/`ServeFile` of the SvelteKit static build for everything else (`main.rs`) — there's no separate web server or reverse proxy for the frontend in production. `vite.config.js`'s dev proxy is what lets `npm run dev` and `cargo run` work together locally.

**Auth.** JWT bearer tokens (`auth::jwt`), verified per request by the `AuthUser` Axum extractor (`auth/middleware.rs`, implements `FromRequestParts`) — protected handlers just take `_auth: AuthUser` as a parameter and rejection happens before the handler body runs. Exception: `/ws` accepts the token as a `?token=` query param (browsers can't set custom headers during the WS handshake); everywhere else it must be an `Authorization: Bearer` header. `AppConfig` is injected via `Extension<Arc<AppConfig>>`, not app state — extract it the same way in new handlers.

**Two data paths to the real Minecraft server — route new telemetry through the existing hub rather than adding a third poller.**
- `podman::PodmanClient` talks to the rootless Podman socket over HTTP (`hyper` + `UnixStream`) for container lifecycle (`start_container`/`stop_container`/`restart_container`) and resource stats, against libpod's `/v4.0.0/libpod/...` API. Podman's non-streaming per-container `/stats` endpoint returns Docker cgroup-stats-compat JSON (`cpu_stats`/`memory_stats`), not the libpod-native flat shape — `get_container_metrics` takes two snapshots 500ms apart and computes the CPU delta itself (see `parse_stats_snapshot` for the shape-detection/fallback logic).
- `rcon::RconClient` talks directly to the Minecraft server's RCON port for everything gameplay-related (commands, `tps`, `list`, moderation).
- Both are polled every 2 seconds by `websocket::WsHub`'s background loop (`spawn_telemetry_loop`, spawned from `WsHub::new`), which owns the canonical live telemetry (container CPU/RAM + RCON tps/players) broadcast to WebSocket clients. `minecraft::metrics::start_telemetry_sampler` (a separate task feeding `/api/metrics/history` and the Discord alert thresholds) reads its own host-level `/proc/stat` + `/proc/meminfo` stats but pulls tps/player-count from `WsHub::get_latest_telemetry()` rather than opening a second RCON connection — the two loops previously reconnected to RCON independently every 2s each, which showed up as near-continuous RCON churn in the Minecraft server's own logs.

**Minecraft config file access is intentionally narrow.** `AppConfig.systemd_config_dir` is only ever joined with the literal filename `minecraft.container` (`routes/server.rs`, backing the engine/version switcher). The deployed quadlet bind-mounts that *single file* (`.../systemd/minecraft.container:/app/systemd-config/minecraft.container`), not the whole shared Podman Quadlet directory — ChiPanel has no visibility into any other service's config on the host. If a feature needs another host file, bind-mount that file specifically; don't widen this to a directory mount.

**The version catalog is live-fetched from Mojang, not hardcoded.** `routes/engine_catalog.rs::fetch_version_catalog` pulls the official version manifest (`piston-meta.mojang.com/mc/game/version_manifest_v2.json`) so every official release *and* snapshot is selectable in the engine switcher; it's cached in memory for 1h (5min on failure) and falls back to a built-in list of the 102 releases (no snapshots) when offline. Validation is charset-first (`is_version_charset_safe` — the quadlet-injection guard) then a case-insensitive lookup that resolves to Mojang's **canonical casing**: snapshot ids are case-sensitive (`24w14a`), which is why `update_engine_handler` must never uppercase `VERSION` the way it does `TYPE`.

**Version → protocol mapping lives in `minecraft/version_meta.rs`**, fetched from PrismarineJS `minecraft-data` (`protocolVersions.json`) with a 1h cache and a built-in fallback of the 102 release entries. `update_engine_handler` is the **single source of truth** for a version change: it resolves `LATEST`/`SNAPSHOT` to a concrete version, writes `minecraft.container` (TYPE/VERSION) **and** `lazymc.toml` (`public.version` + `public.protocol`) together — the proxy hint must never drift from the server version or wake-on-connect breaks. It also reads `world/level.dat` (via `minecraft/worlds.rs::read_world_data_version`) and returns a non-blocking `warning` on a data-version mismatch. Note the PrismarineJS JSON is camelCase (`dataVersion`) — a missing `#[serde(rename)]` silently zeroes the field and disables that warning.

**A background watcher detects newly released versions.** `minecraft/version_watch.rs::start_version_watch` (spawned from `main.rs`, like the telemetry sampler) polls the same Mojang manifest hourly (15min retry on failure), refreshing the shared catalog cache from the same fetch — the watcher and the HTTP layer never double-fetch. When `latest.release` / `latest.snapshot` *changes* vs. the persisted baseline (`DATA_DIR/version_watch.json`), it raises a `new_release` / `new_snapshot` alert that survives restarts and is served as `version_watch` on `GET /api/server/engine` (the engine page renders the "new version" banner from it). The very first successful check only records a silent baseline, so fresh installs don't alert on weeks-old versions.

**Frontend state is two Svelte 5 rune-based singleton stores**, not a framework store library: `lib/stores/auth.svelte.js` (JWT in `sessionStorage`, validated against `/api/auth/me` on boot) and `lib/stores/websocket.svelte.js` (the `/ws` connection, exponential-backoff reconnect, throttled telemetry, bounded 500-entry log buffer). `routes/+layout.svelte` is the only place that should own their connect/disconnect lifecycle and the login redirect (`$effect` blocks keyed on `auth.isAuthenticated`).

`frontend/src/routes/+page.svelte` is a thin re-export of `routes/dashboard/+page.svelte` (so `/` and `/dashboard` render the same component) — edit the dashboard route, not the root page.

## Environment variables (`backend/src/config.rs`)

| Variable | Default | Purpose |
|---|---|---|
| `HOST` / `PORT` | `127.0.0.1` / `25500` | bind address — the deployed quadlet overrides `HOST=0.0.0.0` |
| `JWT_SECRET` | random per boot | set explicitly in production or every restart invalidates sessions |
| `ADMIN_USERNAME` / `ADMIN_PASSWORD` (or `ADMIN_PASSWORD_HASH`) | `admin` / random-generated, logged once | login credentials |
| `ALLOWED_ORIGINS` | `http://127.0.0.1:25500,http://localhost:25500` | CORS; `*` disables the allow-list |
| `RCON_HOST` / `RCON_PORT` / `RCON_PASSWORD` | `127.0.0.1` / `25575` / empty | must match the Minecraft server's `server.properties` |
| `PODMAN_CONTAINER_NAME` (or `MINECRAFT_CONTAINER_NAME`) | `minecraft-server` | the container `PodmanClient` targets |
| `PODMAN_SOCKET` | auto-detected (`/run/user/<uid>/podman/podman.sock`, etc.) | rootless Podman API socket |
| `DATA_DIR` / `MINECRAFT_DATA_DIR` / `SYSTEMD_CONFIG_DIR` | chiserv-specific host paths | ChiPanel's own data, the Minecraft world/plugin data, and the single-file quadlet mount described above |
| `TOOLS_SYNC_INTERVAL_SECS` | `86400` (24h) | background re-sync interval for the spark/chunky auto-installer (see below) |

## Tools auto-sync (spark + chunky + LuckPerms)

`minecraft/tools.rs` keeps the three operational tools installed and up to date for
whatever engine/version the server runs, so profiling (`spark`), world
pre-generation (`chunky`) and permissions (`luckperms`) never break after an
engine switch:

- **Engine detection** (`detect_engine`): reads `TYPE`/`VERSION` from the
  `minecraft.container` quadlet, then the `engine_config.json` state file, then
  the itzg `.install-*.env` markers in the data dir — in that order.
- **Fetch strategy**: chunky, luckperms and spark-the-mod come from Modrinth
  (filtered by loader + game version). spark-the-*plugin* (Paper/Purpur/Spigot)
  is NOT on Modrinth — it is fetched from spark's official Jenkins CI
  (`ci.lucko.me/job/spark/lastSuccessfulBuild`). Fabric additionally gets
  `fabric-api` (a required dependency of chunky-fabric).
- **Triggers**: `POST /api/tools/sync` (admin, on demand), automatically after
  every engine change (`update_engine_handler`), and on a background loop
  (`start_tools_sync_loop`, interval `TOOLS_SYNC_INTERVAL_SECS`). `GET
  /api/tools/status` is the read-only snapshot.
- Idempotent: already-current files are skipped; stale versions of the same tool
  (`spark-*`, `Chunky-*`, `LuckPerms-*`, `fabric-api-*`) are removed before a
  fresh install.

## LuckPerms data migration across engine switches

LuckPerms stores its data in a *platform-specific* directory
(`mods/luckperms` on Fabric, `config/luckperms` on Forge/NeoForge,
`plugins/LuckPerms` on Paper/Purpur/Spigot/Folia), so switching engine would
silently orphan the permissions. `minecraft/luckperms.rs` handles this:

- On every engine change (`update_engine_handler`), the current data dir is
  **backed up** to `<DATA_DIR>/luckperms-backups/<timestamp>/` *before* the config
  is rewritten, then **restored** into the new engine's location *before* the
  server restarts (never clobbers a non-empty target).
- Manual control: `GET /api/tools/luckperms/backups` (list),
  `POST /api/tools/luckperms/backup`, `POST /api/tools/luckperms/restore`
  (body `{ "name": "<backup>" }`, latest when omitted).

## Notes from prior work in this repo

- The static prototype `bento_prototype.html` at the repo root, and the frontend route it inspired (`routes/bento/`), are a pre-production design exploration — not part of the shipped app. The real UI is `routes/dashboard` and its components, which already have WebSocket reconnect/error states, confirmation dialogs on destructive actions, and real alert thresholds the bento mockup never had. Nothing links to `routes/bento/`; it's safe to delete if it reappears.
- `RCON_PASSWORD` and `ADMIN_PASSWORD` are currently the same value in the deployed quadlet — a known credential-reuse issue across two different trust boundaries, not yet addressed.
