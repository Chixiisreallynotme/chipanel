# Modèle de Sécurité & Bonnes Pratiques de ChiPanel

Ce document décrit en détail les mécanismes de protection, le modèle de menaces et les contrôles de sécurité appliqués au sein de ChiPanel.

---

## 1. Authentification & Chiffrement des Identifiants

### Hachage des Mots de Passe avec Argon2id
- Tous les mots de passe des comptes utilisateurs créés via ChiPanel sont hachés avec l'algorithme cryptographique **Argon2id** (standard recommandé par l'ANSSI et l'OWASP).
- Paramètres de hachage : 3 itérations temporelles, mémoire allouée de 64 Mo, parallélisme adapté au CPU, avec sel cryptographique aléatoire de 128 bits.
- La vérification des mots de passe s'effectue en temps constant pour neutraliser les attaques par analyse temporelle (*timing attacks*).

### Jetons de Session JWT & Clé Secrète
- Les sessions utilisateur sont authentifiées via des jetons **JSON Web Tokens (JWT)** signés avec l'algorithme HMAC-SHA256 (`HS256`).
- Le payload contient le nom du sujet (`sub`), la date d'émission (`iat`) et une date d'expiration stricte (`exp` fixée à 24 heures).
- Si la variable d'environnement `JWT_SECRET` n'est pas fournie, ChiPanel génère à chaque démarrage une clé secrète aléatoire de 256 bits via un générateur cryptographiquement sûr (`rand::rngs::OsRng`), invalidant immédiatement les anciennes sessions.

---

## 2. Contrôle d'Accès Basé sur les Rôles (RBAC)

ChiPanel applique le principe du moindre privilège à l'aide de deux rôles distincts :

```
                ┌──────────────────────────────────┐
                │          RÔLE "VIEWER"           │
                │  - Consultation console & logs   │
                │  - Lecture statut & télémétrie   │
                │  - Consultation des sauvegardes  │
                │  - Consultation plugins & mondes │
                └────────────────┬─────────────────┘
                                 │  Restrictions strictes
                                 ▼
                ┌──────────────────────────────────┐
                │           RÔLE "ADMIN"           │
                │  - Émission commandes RCON       │
                │  - Démarrage / Arrêt / Restart   │
                │  - Création / Restauration backup│
                │  - Purge de base de données      │
                │  - Gestion des utilisateurs & API│
                │  - Consultation Registre d'Audit │
                └──────────────────────────────────┘
```

- **Protection contre l'élévation de privilèges** : Le compte racine `ADMIN_USERNAME` conserve impérativement le rôle `admin` en mémoire afin d'éviter tout verrouillage accidentel du panneau.
- **Tokens d'API Persistants** : Les jetons d'API longue durée (`chipanel_sec_...`) sont réservés aux automatisations internes et disposent des droits administrateur.

---

## 3. Protection Contre le Path Traversal & Zip-Slip

### Sandboxing du Système de Fichiers (`src/routes/files.rs` & `src/minecraft/server_backup.rs`)
La manipulation de fichiers sur le serveur Minecraft présente un risque critique si un attaquant tente d'accéder à l'hôte via des chemins relatifs (`../../etc/shadow`).

**Mécanismes de protection appliqués :**
1. **Canonicalisation stricte** : Tous les chemins reçus sont résolus via `std::fs::canonicalize` pour éliminer les séquences `..`, les liens symboliques et les chemins relatifs.
2. **Vérification de préfixe racine** : Le chemin canonique doit obligatoirement commencer par le chemin canonique de `MINECRAFT_DATA_DIR` :
   ```rust
   if !canonical_target.starts_with(&canonical_root) {
       return Err(AppError::Forbidden("Accès refusé : chemin hors du répertoire autorisé."));
   }
   ```
3. **Protection contre l'attaque Zip-Slip** : Lors de l'extraction d'une archive de sauvegarde ou d'un plugin, chaque chemin de fichier interne est assaini pour empêcher toute écriture en dehors du dossier de destination.

---

## 4. Caviardage Automatique & Confidentialité des Données

### Export Sécurisé vers `mclo.gs`
Le partage public de logs de crash sur des plateformes externes expose couramment des adresses IP de joueurs et des identifiants système.
- ChiPanel applique une passe de désensibilisation par expressions régulières avant tout envoi :
  - IPs IPv4 publiques (ex: `185.220.101.5` -> `[REDACTED_IP]`) et IPv6.
  - Mots de passe RCON et tokens de webhooks Discord.
  - Chemins absolus contenant des noms d'utilisateurs Linux de l'hôte.

---

## 5. Isolation Système & Podman Rootless

- **Exécution sans privilèges root** : ChiPanel s'exécute dans un conteneur rootless appartenant à l'utilisateur non-privilégié `chiserv` (UID 1000).
- **Namespaces Utilisateur (subuid / subgid)** : Même en cas de compromission hypothétique du conteneur, l'attaquant reste confiné aux droits restreints de l'utilisateur non-root de l'hôte sans aucune possibilité d'accès aux fichiers `/etc/passwd`, `/etc/shadow` ou aux processus des autres utilisateurs.
- **Montages en Lecture Seule (`:ro`)** : Les sockets système sensibles (`/run/user/1000/podman/podman.sock` et `/run/user/1000/bus`) sont impérativement montées avec l'option `:ro` pour empêcher toute altération des descripteurs système.
