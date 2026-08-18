# Security Model & Threat Mitigation in ChiPanel

This document outlines the security architecture, cryptographic standards, threat mitigations, and isolation mechanisms implemented across ChiPanel.

---

## 1. Authentication & Cryptographic Standards

### Password Hashing with Argon2id
- **Algorithm**: User passwords are encrypted using **Argon2id** (OWASP and ANSSI recommended standard).
- **Parameters**: 3 time iterations, 64 MB memory cost, CPU-adaptive parallelism, with 128-bit CSPRNG cryptographic salts (`rand::rngs::OsRng`).
- **Reactor Offloading**: Password verification and hashing operations run inside `tokio::task::spawn_blocking` to ensure CPU-heavy cryptographic operations never stall Tokio's async event loop.
- **Brute-Force Rate Limiting**: The login endpoint `/api/auth/login` enforces per-user throttling (maximum 5 attempts per minute) and a global hashing rate cap (maximum 10 hash calculations per 10 seconds) to mitigate credential stuffing and denial of service.

### JWT Session Tokens & Secret Generation
- **HMAC-SHA256 (`HS256`)**: User sessions use signed JSON Web Tokens.
- **Payload Constraints**: Contains subject (`sub`), issued-at (`iat`), and strict expiration (`exp` set to 24 hours).
- **Zero-Config Secret**: If `JWT_SECRET` is not provided in the environment, ChiPanel generates a fresh 256-bit cryptographic key via `OsRng` at startup, immediately invalidating previous session tokens upon container restart.

### Persistent API Access Keys
- **High Entropy**: API tokens use the `chipanel_sec_<24_hex_chars>` format generated from OS entropy.
- **One-Way Hashing**: Plaintext tokens are only displayed once upon generation. The backend stores exclusively the SHA-256 hash in `data/api_tokens.json`. Prefixes (`chipanel_sec_<first_6_hex>...`) are preserved for identification in the administrative UI.

---

## 2. Role-Based Access Control (RBAC)

ChiPanel enforces the principle of least privilege across three roles:

```
                ┌──────────────────────────────────┐
                │          ROLE: VIEWER            │
                │  - Read live console & logs      │
                │  - View server telemetry & stats │
                │  - View world list & backups     │
                │  - Inspect plugins & modpacks    │
                └────────────────┬─────────────────┘
                                 │  Elevated Permissions
                                 ▼
                ┌──────────────────────────────────┐
                │         ROLE: OPERATOR           │
                │  - Kick / Ban / Pardon players   │
                │  - Execute player moderation     │
                │  - Manage pending command queue  │
                └────────────────┬─────────────────┘
                                 │  Full System Control
                                 ▼
                ┌──────────────────────────────────┐
                │          ROLE: ADMIN             │
                │  - Execute arbitrary RCON        │
                │  - Power actions & engine switch │
                │  - Filesystem read/write/delete  │
                │  - Create / Restore backups      │
                │  - Manage accounts & API tokens  │
                │  - Database maintenance & purges │
                └──────────────────────────────────┘
```

- **Root Admin Safeguard**: The bootstrap account `ADMIN_USERNAME` retains immutable `admin` privileges in memory to prevent accidental administrative lockout.

---

## 3. Filesystem Sandboxing & Path Traversal Guards

File access operations in `/api/files`, `/api/worlds`, and `/api/backups` interact with local files on the server host.

### Defense Mechanisms:
1. **Strict Canonicalization**: Paths are sanitized by stripping null bytes, newlines, carriage returns, and `..` segments, then resolved via `std::fs::canonicalize`.
2. **Root Prefix Boundary Enforcement**: The canonical target path must strictly begin with the canonical root of `MINECRAFT_DATA_DIR` or `DATA_DIR`:
   ```rust
   if !canonical_target.starts_with(&canonical_root) {
       return Err(AppError::Forbidden("Access denied: Target path outside allowed root directory."));
   }
   ```
3. **Zip-Slip Attack Prevention**: During backup restoration or world ZIP unarchiving, internal relative paths are inspected before writing. Any entry attempting to break out of the target directory is rejected immediately.

---

## 4. SSRF & External Integration Hardening

### Discord Alert Webhooks
The metrics alert engine sends notifications when TPS, CPU, or RAM breach thresholds:
- **Protocol Enforcement**: Only `https://` URLs are accepted.
- **Domain Whitelist**: Targets must resolve to approved Discord domains (`discord.com`, `discordapp.com`, `canary.discord.com`, `ptb.discord.com`).
- **Redirect Disabling**: The HTTP client enforces `redirect(Policy::none())` to prevent attackers from using open redirects to reach internal cloud metadata endpoints (`169.254.169.254`) or homelab services.

---

## 5. PII & Credential Redaction

Before exporting console logs to external diagnostic services like `mclo.gs`:
- **Automated Filtering**: Regular expressions scan and sanitize:
  - Public IPv4 addresses and IPv6 notations (`[REDACTED_IP]`).
  - RCON authentication passwords.
  - Discord webhook URLs and Bearer tokens.
  - Absolute host directories revealing local usernames.

---

## 6. Container Runtime Isolation & Least Privilege

### Podman Rootless & Namespace Isolation
- **Non-Root Execution**: ChiPanel runs as non-privileged user `USER 1000:1000` (`chiserv`) inside a rootless container.
- **Linux User Namespaces (`subuid` / `subgid`)**: In the unlikely event of a container breakout, the process is confined to an unprivileged subordinate UID on the host without root capabilities.
- **Read-Only System Sockets (`:ro`)**: System control sockets (`/run/user/1000/podman/podman.sock` and `/run/user/1000/bus`) are mounted in read-only mode to prevent descriptor corruption or unauthorized permission escalation.

### Docker Socket Security Guidelines
- **Read-Only Daemon Mounting**: When deployed against standard Docker, mount `/var/run/docker.sock:ro` to prevent container hijacking or arbitrary container injection.
- **Restricted Volume Scopes**: Bind-mount only necessary target data directories (`/app/data` and `/app/minecraft-data`), avoiding parent directory mounts.

