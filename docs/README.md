# Documentation Technique de ChiPanel

Bienvenue dans la documentation officielle et exhaustive de **ChiPanel**, le panneau de contrôle moderne, léger et haute performance pour serveurs Minecraft et homelab conteneurisés.

---

## 📚 Sommaire de la Documentation

La documentation est organisée en 5 guides thématiques spécialisés :

1. [**Architecture & Mécanismes Internes** (`architecture.md`)](./architecture.md)
   - Runtime asynchrone Rust (Axum 0.7 + Tokio) et empreinte mémoire minimale (< 25 Mo).
   - Acteur Tokio RCON persistant et multiplexé avec reconnexion automatique.
   - Frontend Svelte 5 réactif exploitant l'architecture moderne des *Runes* (`$state`, `$derived`, `$props`, `$effect`).
   - Supervision système via D-Bus session bus (`zbus`) et conteneurs Podman 5.4 rootless.
   - Intégration avec le proxy natif d'auto-hibernation `lazymc`.

2. [**Référence Exhaustive de l'API REST & WebSocket** (`api-reference.md`)](./api-reference.md)
   - Authentification JWT, gestion de session et contrôle d'accès basé sur les rôles (RBAC).
   - Documentation complète de l'ensemble des endpoints (`/api/auth`, `/api/server`, `/api/metrics`, `/api/tools`, `/api/plugins`, `/api/worlds`, `/api/backups`, `/api/geyser`, `/api/maintenance`, `/api/diff`, `/api/logs`, `/api/audit`, `/api/permissions`).
   - Protocole de streaming WebSocket bidirectionnel (`/ws`).

3. [**Guide des Modules Fonctionnels** (`modules-guide.md`)](./modules-guide.md)
   - **Acteur RCON Multiplexé** : communication zéro lag avec la JVM.
   - **Console & Streaming de Logs** : buffer haute capacité et export sécurisé via l'API v1 `mclo.gs`.
   - **Config Diff Viewer** : comparateur différentiel avant enregistrement (`similar` crate).
   - **Diagnostic BDD & Purgeur SQL** : assainissement et compaction des bases SQLite (CoreProtect, LuckPerms).
   - **Assistant Cross-Play Bedrock & Geyser** : pont d'interconnexion pour joueurs mobiles et consoles (UDP 19132, Floodgate `key.pem`).
   - **Spark Profiler & Flamegraphs** : déclenchement et extraction d'URLs de profils de performance JVM.
   - **Moteur de Sauvegardes Avancées & Snapshots** : périmètres ciblés, règles d'exclusion de dossiers, politique de rétention et export distant S3 / MinIO.
   - **Registre d'Audit Immuable** : traçabilité des actions sensibles dans `audit_log.jsonl` et export CSV.
   - **Mondes, Chunky & NBT** : gestion des dimensions, pré-génération de chunks et lecture NBT.
   - **Catalogue Modrinth v2** : recherche d'extensions et résolution des dépendances.

4. [**Guide de Déploiement & Exploitation** (`deployment-and-operations.md`)](./deployment-and-operations.md)
   - Construction multi-stage du conteneur (`Containerfile`).
   - Déploiement par Quadlet systemd user (`chipanel.container`).
   - Dictionnaire exhaustif des variables d'environnement.
   - Runbook d'exploitation, commandes usuelles de diagnostic et procédures de mise à jour.

5. [**Modèle de Sécurité & Conformité** (`security.md`)](./security.md)
   - Chiffrement des mots de passe avec Argon2id et tokens JWT signés.
   - Séparation stricte des privilèges (`admin` vs `viewer`).
   - Protection contre le *Path Traversal* et assainissement des entrées NBT / Archives.
   - Isolation rootless sous namespaces utilisateur Linux (subuid / subgid).
