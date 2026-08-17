# ChiPanel — Agent Instructions & Design System Contract

> **Source de vérité ChiPanel :** Ce fichier régit les règles d'intervention pour tout agent travaillant sur le projet **ChiPanel** (Backend Axum + Frontend SvelteKit / Svelte 5).

---

## 🎨 RÈGLE ABSOLUE & NON-NÉGOCIABLE : DIRECTION ARTISTIQUE & DESIGN (`design.md`)

Avant toute intervention, modification, ajout de composant, page ou style sur le **frontend de ChiPanel** (`frontend/`) :

1. **Lecture obligatoire de `design.md` :** L'agent a l'**obligation stricte et préalable** de lire et d'appliquer l'intégralité des spécifications contenues dans [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md).
2. **Application rigoureuse de la DA ChiPanel :**
   - **Esthétique « Double-Bezel » / Machined Hardware :** Surfaces ardoise sombre (`#0C0E14` / `#161922` / `#1F2330`), biseau intérieur 1px spéculaire, zéro noir OLED froid `#000000`.
   - **Directives Anti-AI-Slop strictes :** Zéro gradient textuel, zéro aura néon/violette sans justification sémantique, zéro badge superflu, arrondis plafonnés à 8-12px (`--radius-md`/`--radius-lg`), zéro emoji dans l'UI (icônes Lucide/Phosphor cohérentes uniquement).
   - **Règle de Rareté de la Couleur (Color Scarcity) & Accessibilité WCAG AA/AAA :** 90% monochrome tactile, 10% accents sémantiques calibrés (3 rôles par couleur : `--accent-*-bg`, `--accent-*-text`, `--accent-*-solid`).
   - **Design Engineering & Micro-Interactions (Emil Kowalski Standards) :** Retour haptique physique `scale(0.97)` sur `:active`, jamais d'entrée depuis `scale(0)`, popovers origin-aware, zéro animation sur raccourcis clavier fréquents, durées UI < 250ms, courbes personnalisées (`--ease-out: cubic-bezier(0.23, 1, 0.32, 1)`), animations GPU-safe (`transform` et `opacity` uniquement).
   - **Vérité en Télémétrie (Truth in Telemetry) & Bento Grid sans vide :** Données réelles tabulaires (`tabular-nums`), squelettes fidèles sans CLS, gestion digne de l'hibernation (`lazymc`).
   - **Identité & Logo :** Intégration du logo officiel *Hexagone Stencil Chi-Rack* avec voyant LED réactif aux statuts du serveur.

---

## 🔄 Règle de Versioning & Push Automatique

- **Commit + push systématique :** Après toute modification de code ou de documentation dans `chipanel/`, commit immédiatement puis push : `git add -A && git commit -m "..." && git push`.
- N'attends jamais de confirmation explicite pour commiter et pousser les changements validés.
- **Sécurité :** Ne commite aucun secret en clair. Le quadlet réel `chipanel.container` est gitignoré ; seul `chipanel.container.example` est versionné.

---

## 🛠️ Commandes de Référence & Build

- **Frontend :** `npm run dev` (proxy vers `:25500`), `npm run build`, `npm run check` (`svelte-check`).
- **Backend :** `cargo run`, `cargo build --release`, `cargo test`.
- **Conteneur :** `podman build -t localhost/chipanel:latest -f Containerfile .` (toujours utiliser `--no-cache` si le frontend a été modifié).
- **Déploiement Host-007 :** `podman save` -> `scp` -> `podman load` -> `systemctl --user restart chipanel`.

---

## 📚 Voir aussi

| Fichier | Rôle |
| :--- | :--- |
| [`design.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/design.md) | **Spécification complète du design system, tokens, anti-patterns et design engineering.** |
| [`PRODUCT.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/PRODUCT.md) | Vision produit, utilisateurs cibles et principes de conception. |
| [`CLAUDE.md`](file:///home/chixi/Documents/Projects%20/chiserv/chipanel/CLAUDE.md) | Détails approfondis de l'architecture Rust/Axum, RCON, Podman socket et gestion Minecraft. |
