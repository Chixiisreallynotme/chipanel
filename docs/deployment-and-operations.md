# Guide de Déploiement & Exploitation de ChiPanel

Ce guide fournit la méthodologie complète pour compiler, conteneuriser, déployer et administrer ChiPanel en production sous Podman rootless et systemd quadlets.

---

## 1. Processus de Construction du Conteneur

ChiPanel utilise une construction multi-stage en 3 étapes définie dans [`Containerfile`](../Containerfile) :

1. **Stage 1 (`frontend-builder`)** : Utilise `node:20-alpine` pour compiler l'application Svelte 5 en fichiers statiques optimisés (`npm run build`).
2. **Stage 2 (`rust-builder`)** : Utilise `rust:bookworm` pour compiler le backend Axum avec les optimisations LTO (`cargo build --release --bin chipanel`) et supprime les symboles de débogage avec `strip`.
3. **Stage 3 (`runtime`)** : Image minimale basée sur `debian:bookworm-slim` avec uniquement `ca-certificates` et `libssl3`. L'application s'exécute en tant qu'utilisateur non-privilégié (`USER 1000:1000`).

### Commande de compilation locale :
```bash
podman build -t localhost/chipanel:latest -f Containerfile .
```

---

## 2. Déploiement Quadlet Systemd User

Sur l'hôte cible (**Host-007**), ChiPanel est déclaré en tant que Quadlet systemd utilisateur dans `~/.config/containers/systemd/chipanel.container` :

```ini
[Unit]
Description=ChiPanel - Panneau de Contrôle Minecraft Homelab
After=network-online.target local-fs.target

[Container]
Image=localhost/chipanel:latest
ContainerName=chipanel
Network=host

# Montage des répertoires de données
Volume=%h/chipanel-data:/app/data:Z
Volume=%h/minecraft:/minecraft:z

# Montage des sockets système pour le pilotage de l'hôte
Volume=/run/user/1000/podman/podman.sock:/run/user/1000/podman/podman.sock:ro
Volume=/run/user/1000/bus:/run/user/1000/bus:ro

# Variables d'environnement obligatoires
Environment=PORT=25500
Environment=HOST=0.0.0.0
Environment=RUST_LOG=info
Environment=ADMIN_USERNAME=admin
Environment=ADMIN_PASSWORD=VOTRE_MOT_DE_PASSE_SECURISE
Environment=JWT_SECRET=VOTRE_CLE_SECRETE_JWT_LONGUE
Environment=MINECRAFT_DATA_DIR=/minecraft
Environment=CONTAINER_NAME=minecraft
Environment=RCON_HOST=127.0.0.1
Environment=RCON_PORT=25575
Environment=RCON_PASSWORD=VOTRE_MOT_DE_PASSE_RCON
Environment=PODMAN_SOCKET_PATH=/run/user/1000/podman/podman.sock
Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus

[Service]
Restart=always
TimeoutStartSec=60
MemoryMax=150M

[Install]
WantedBy=default.target
```

---

## 3. Dictionnaire des Variables d'Environnement

| Variable | Valeur par défaut | Description |
| :--- | :--- | :--- |
| `PORT` | `25500` | Port d'écoute HTTP et WebSocket du panneau. |
| `HOST` | `0.0.0.0` | Adresse IP d'écoute sur le réseau. |
| `RUST_LOG` | `info` | Niveau de verbosité des logs (`error`, `warn`, `info`, `debug`, `trace`). |
| `ADMIN_USERNAME` | `admin` | Nom du compte administrateur racine (toujours autorisé). |
| `ADMIN_PASSWORD` | *(Obligatoire)* | Mot de passe initial du compte administrateur. |
| `JWT_SECRET` | *(Aléatoire si omis)* | Clé secrète de signature des jetons de session JWT. |
| `MINECRAFT_DATA_DIR` | `/minecraft` | Chemin absolu vers la racine des fichiers du serveur Minecraft. |
| `CONTAINER_NAME` | `minecraft` | Nom du conteneur Podman à superviser. |
| `RCON_HOST` | `127.0.0.1` | Adresse IP d'écoute du serveur RCON interne. |
| `RCON_PORT` | `25575` | Port RCON de Minecraft. |
| `RCON_PASSWORD` | *(Obligatoire)* | Mot de passe RCON défini dans `server.properties`. |
| `PODMAN_SOCKET_PATH` | `/run/user/1000/podman/podman.sock` | Chemin vers la socket Unix de l'API Libpod. |
| `DBUS_SESSION_BUS_ADDRESS` | `unix:path=/run/user/1000/bus` | Adresse de connexion au bus de session D-Bus. |

---

## 4. Procédure de Déploiement Pas-à-Pas (Homelab)

Puisqu'aucun registre d'images distant n'est utilisé, le déploiement s'effectue par transfert d'archive d'image :

```bash
# 1. Compiler l'image conteneur en local
podman build -t localhost/chipanel:latest -f Containerfile .

# 2. Exporter et transférer l'archive sur le serveur
podman save localhost/chipanel:latest -o /tmp/chipanel.tar
scp -i ~/.ssh/chiserv_host007 /tmp/chipanel.tar chiserv@192.168.1.109:/tmp/chipanel.tar
rm -f /tmp/chipanel.tar

# 3. Charger l'image et redémarrer le service systemd user
ssh -i ~/.ssh/chiserv_host007 chiserv@192.168.1.109 "podman load -i /tmp/chipanel.tar && rm -f /tmp/chipanel.tar && systemctl --user restart chipanel"
```

---

## 5. Runbook d'Exploitation & Dépannage

### Vérifier l'état du service :
```bash
systemctl --user status chipanel.service
```

### Consulter les journaux en direct :
```bash
journalctl --user -u chipanel.service -f --lines=100
```

### Tester la santé de l'API :
```bash
curl -s http://127.0.0.1:25500/api/health
```

### Recharger la configuration après modification du Quadlet :
> [!IMPORTANT]
> Après toute modification d'un fichier `.container`, il est impératif d'exécuter `systemctl --user daemon-reload` avant de relancer le service, sinon les nouveaux paramètres (volumes, limites mémoire) sont ignorés.

```bash
systemctl --user daemon-reload
systemctl --user restart chipanel.service
```
