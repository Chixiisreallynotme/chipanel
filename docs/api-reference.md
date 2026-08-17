# Référence Complète de l'API REST & WebSocket

Toutes les requêtes de l'API ChiPanel sont servies sous le préfixe `/api` sur le port configuré (`25500` par défaut). Les réponses sont au format standard `application/json` (sauf exports CSV / logs bruts).

---

## 1. Authentification & Sécurité

### Authentification JWT
ChiPanel utilise des tokens JWT (JSON Web Tokens) signés avec le secret applicatif.
- En-tête obligatoire pour les requêtes protégées : `Authorization: Bearer <TOKEN>` ou `x-api-key: <TOKEN>`.
- Les routes réservées aux administrateurs renvoient un code `403 Forbidden` si le token appartient à un utilisateur avec le rôle `viewer`.

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `POST` | `/api/auth/login` | Public | Authentifie un utilisateur avec nom d'utilisateur et mot de passe. |
| `GET` | `/api/auth/profile` | Authentifié | Renvoie le profil et les permissions du compte connecté. |
| `GET` | `/api/auth/users` | Admin | Liste l'ensemble des utilisateurs enregistrés dans la base locale. |
| `POST` | `/api/auth/users` | Admin | Crée un nouveau compte (`username`, `password`, `role`). |
| `DELETE` | `/api/auth/users/:username` | Admin | Supprime un compte utilisateur existant. |
| `GET` | `/api/auth/tokens` | Admin | Liste les jetons d'accès d'API persistants (`chipanel_sec_...`). |
| `POST` | `/api/auth/tokens` | Admin | Génère un nouveau jeton d'accès d'API persistant. |
| `DELETE` | `/api/auth/tokens/:id` | Admin | Révoque un jeton d'accès d'API persistant. |

---

## 2. Cycle de Vie du Serveur (`/api/server`)

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/server/status` | Authentifié | Renvoie l'état temps réel du serveur (état conteneur, état lazymc, joueurs en ligne, TPS, RAM). |
| `POST` | `/api/server/power` | Admin | Change l'état d'alimentation : `{ "action": "start" \| "stop" \| "restart" \| "kill" }`. |
| `GET` | `/api/server/properties` | Authentifié | Lit et parse le fichier `server.properties` sous forme structurée JSON. |
| `POST` | `/api/server/properties` | Admin | Enregistre et applique les modifications de `server.properties`. |
| `POST` | `/api/server/command` | Admin | Exécute une commande RCON arbitraire sur la console du serveur. |

---

## 3. Registre d'Audit & Conformité (`/api/audit`)

| Méthode | Endpoint | Rôle Requis | Paramètres Query / Body | Description |
| :--- | :--- | :--- | :--- | :--- |
| `GET` | `/api/audit` | Admin | `search`, `category`, `status`, `username`, `limit`, `offset` | Interroge le journal d'audit append-only avec filtres et pagination. |
| `POST` | `/api/audit/record` | Authentifié | Body: `{ "action": "...", "category": "...", "status": "...", "details": {...} }` | Enregistre un nouvel événement d'audit dans `audit_log.jsonl`. |
| `GET` | `/api/audit/export/csv` | Admin | Aucun | Télécharge l'intégralité du registre sous forme de fichier CSV standardisé. |

---

## 4. Sauvegardes Avancées & Snapshots (`/api/backups`)

| Méthode | Endpoint | Rôle Requis | Payload / Paramètres | Description |
| :--- | :--- | :--- | :--- | :--- |
| `GET` | `/api/backups` | Authentifié | Aucun | Liste toutes les archives de sauvegarde avec hachage SHA-256 et métadonnées. |
| `POST` | `/api/backups/create` | Admin | `{ "name": "...", "scope": "full" \| "world_only" \| "configs_only", "comment": "..." }` | Déclenche la création asynchrone d'une sauvegarde zstd/zip. |
| `POST` | `/api/backups/restore` | Admin | `{ "filename": "backup_2026-08-17.zip" }` | Restaure le serveur à partir de l'archive spécifiée. |
| `DELETE` | `/api/backups/:filename` | Admin | Nom du fichier dans l'URL | Supprime définitivement l'archive de sauvegarde du disque. |
| `GET` | `/api/backups/settings` | Admin | Aucun | Récupère les règles d'exclusion de dossiers et de rétention automatique. |
| `POST` | `/api/backups/settings` | Admin | `{ "retention_count": 10, "excluded_patterns": [...] }` | Enregistre la configuration de rétention et d'exclusion. |
| `POST` | `/api/backups/export/s3` | Admin | `{ "filename": "...", "endpoint": "...", "bucket": "...", "access_key": "...", "secret_key": "..." }` | Téléverse une archive locale vers un bucket compatible S3/MinIO. |

---

## 5. Diagnostic BDD & Purgeur (`/api/maintenance`)

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/maintenance/stats` | Admin | Calcule le volume exact occupé par CoreProtect (`database.db`), LuckPerms, les logs `.log.gz` et chaque dimension. |
| `POST` | `/api/maintenance/purge` | Admin | Lance une opération de purge guidée : `{ "target": "coreprotect", "days": 30 }` ou purge d'archives de logs. |

---

## 6. Assistant Cross-Play Bedrock & Geyser (`/api/geyser`)

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/geyser/status` | Authentifié | Détecte la présence du plugin Geyser, de Floodgate, de la clé de chiffrement `key.pem` et l'ouverture du port UDP 19132. |
| `POST` | `/api/geyser/setup` | Admin | Génère ou met à jour la configuration optimale de `plugins/Geyser-Spigot/config.yml`. |

---

## 7. Outils & Diagnostic Spark Profiler (`/api/tools`)

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `POST` | `/api/tools/spark/sampler` | Admin | Déclenche ou arrête l'échantillonneur CPU Spark (`/spark sampler --viewer`) et extrait l'URL du profil web. |
| `GET` | `/api/tools/spark/health` | Authentifié | Récupère le bilan de santé JVM en direct via Spark (`/spark health --memory`). |

---

## 8. Plugins, Mods & Modrinth API v2 (`/api/plugins`)

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/plugins` | Authentifié | Liste les fichiers `.jar` installés dans le dossier `plugins/` ou `mods/`. |
| `GET` | `/api/plugins/search` | Authentifié | Recherche des plugins sur le catalogue Modrinth v2 (`query`, `loader`, `game_version`). |
| `POST` | `/api/plugins/install` | Admin | Télécharge et installe un plugin depuis Modrinth après validation du hachage SHA-512. |
| `POST` | `/api/plugins/updates/check` | Admin | Analyse les plugins installés et recherche les versions supérieures compatibles. |
| `POST` | `/api/plugins/updates/apply` | Admin | Applique la mise à jour groupée ou individuelle des extensions sélectionnées. |
| `DELETE` | `/api/plugins/:filename` | Admin | Supprime ou désactive une extension `.jar`. |

---

## 9. Gestionnaire de Mondes, Chunky & NBT (`/api/worlds`)

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/worlds` | Authentifié | Liste les mondes et dimensions disponibles (`world`, `world_nether`, `world_the_end`). |
| `GET` | `/api/worlds/:world_name/gamerules` | Authentifié | Lit les règles de jeu (`gamerules`) définies dans le `level.dat` du monde. |
| `POST` | `/api/worlds/:world_name/gamerules` | Admin | Met à jour une règle de jeu via commande RCON `/gamerule`. |
| `GET` | `/api/worlds/:world_name/datapacks` | Authentifié | Liste les datapacks installés et leur statut d'activation. |
| `POST` | `/api/worlds/chunky/start` | Admin | Démarre la pré-génération de chunks via Chunky (`/chunky start`). |
| `POST` | `/api/worlds/chunky/pause` | Admin | Met en pause la pré-génération Chunky. |

---

## 10. Console, Logs & Visualiseur Diff (`/api/logs` & `/api/diff`)

| Méthode | Endpoint | Rôle Requis | Description |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/logs` | Authentifié | Récupère les dernières lignes du fichier `latest.log`. |
| `POST` | `/api/logs/mclogs` | Authentifié | Caviarde automatiquement les données sensibles (IPs, tokens) et exporte le journal vers `mclo.gs`. |
| `POST` | `/api/diff` | Authentifié | Compare deux contenus textuels et renvoie le flux unifié de différences ligne par ligne. |

---

## 11. Protocole WebSocket Bidirectionnel (`/ws`)

Le WebSocket ChiPanel permet la synchronisation en temps réel de la console et des métriques du serveur.

### Connexion & Négociation
- URL : `ws://<HOST>:25500/ws?token=<JWT_TOKEN>`
- Si le token est invalide ou expiré, la connexion est immédiatement close avec le code `4401 Unauthorized`.

### Structure des Événements Reçus (Serveur -> Client)

```json
{
  "type": "log_line",
  "data": {
    "timestamp": "14:28:02",
    "level": "INFO",
    "message": "[Server thread/INFO]: Player Steve joined the game",
    "raw": "[14:28:02 INFO]: Player Steve joined the game"
  }
}
```

```json
{
  "type": "server_telemetry",
  "data": {
    "status": "RUNNING",
    "online_players": 3,
    "max_players": 20,
    "tps": 20.0,
    "mspt": 11.4,
    "cpu_percent": 12.8,
    "memory_bytes": 2684354560,
    "memory_max_bytes": 4294967296
  }
}
```

### Commandes Émises (Client -> Serveur)

```json
{
  "action": "send_command",
  "command": "say Bonjour tout le monde !"
}
```
*(Seuls les utilisateurs authentifiés avec le rôle `admin` sont autorisés à émettre des commandes console).*
