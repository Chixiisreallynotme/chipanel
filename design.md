# ChiPanel Design System Specification — Directives & Standards

Ce document constitue la **spécification technique, visuelle et comportementale de référence** pour l'interface de **ChiPanel**. En tant que console d'administration homelab et Minecraft de production, l'interface privilégie la clarté opérationnelle, le **minimalisme monolithique**, le **chaos maîtrisé (Rebel Precision)**, la **haute densité utile**, la **navigation 100% clavier** et le respect rigoureux des contraintes de design engineering (standards Emil Kowalski et physique snappy inspirée du shell Hyprland).

---

## 1. Directives Anti-AI-Slop Strictes & Cadrage du « Chaos Maîtrisé »

Pour préserver l'intégrité, la puissance technique et l'authenticité de l'outil, tous les tics visuels et facilités génératives de l'IA ("AI Slop") sont formellement proscrits du codebase et des composants.

### 🚫 Éléments Formellement Proscrits & Alternatives Impeccables

| Anti-Pattern (Banni) | Pourquoi c'est banni (AI Tell) | Règle & Alternative Impeccable |
| :--- | :--- | :--- |
| **Gradients textuels** (`background-clip: text`) | Cliché cosmétique pur, dégrade la lisibilité et l'anti-aliasing sur fond sombre. | Utiliser exclusivement du texte uni solide. L'accentuation se fait par le poids de police (`font-weight: 600`), la typographie monospace ou l'échelle typographique. |
| **Glows / Auras violettes & néons** (`box-shadow: 0 0 25px rgba(139,92,246,0.5)`) | Esthétique "crypto / landing page IA" générique et bruyante qui fatigue les yeux en environnement sombre. | Bordures capillaires nettes et translucides (`1px solid var(--border-hairline)`). Halo lumineux réservé exclusivement aux micro-voyants LED d'état systémique (< 8px de rayon). |
| **Pseudo-dashboards & "Hero Metrics" creux** | Gros chiffres isolés avec variations inventées (`+12.4% vs last week`) sans réalité technique. | Toutes les métriques doivent avoir une signification système concrète (ex: *Delta CPU Podman*, *RAM cgroup allouée/utilisée*, *TPS RCON*, *Ping réseau*, *Tick time ms*). Pas de jauge circulaire purement décorative. |
| **Répétition systématique des "Eyebrows"** | Kicker all-caps sur-espacé (`OVERVIEW`, `ACTIONS`) au-dessus de chaque bloc de titre. | Règle de parcimonie : un eyebrow n'apparaît que pour indiquer un sous-système hiérarchique critique ou un contexte technique non évident. Les titres de cartes se suffisent à eux-mêmes. |
| **Inondation de Badges & Pills superflus** | Multiplication de capsules de tags pour décorer chaque ligne ou cellule de tableau. | Les badges sont réservés aux **états de cycle de vie dynamiques** (*Online*, *Hibernating*, *Stopped*, *Degraded*). Tout texte descriptif statique reste en texte simple `--text-secondary`. |
| **Arrondis disproportionnés** (`border-radius: 24px/32px+`) | Donne un aspect "jouet" ou prototype mobile inadapté à un panneau technique de production. | Cartes et conteneurs plafonnés à `--radius-sm` (4px), `--radius-md` (8px) ou `--radius-lg` (12px) max. Le format pilule (`rounded-full`) est strictement réservé aux tags d'état et avatars. |
| **Ghost-Cards (Border 1px + Shadow large diffuse)** | Motif typique combinant une bordure fine et un flou d'ombre lourd (≥16px). | Règle d'or : **soit** une bordure capillaire nette, **soit** une ombre de contact chirurgicale (`0 2px 8px rgba(0,0,0,0.4)`), jamais de superpositions diffuses artificielles. |
| **Side-Stripe Borders épaisses** (`border-left: 4px solid ...`) | Béquille décorative asymétrique datée pour cartes et alertes. | Utiliser un fond teinté uniforme à faible opacité (`var(--accent-*-bg)`) combiné à une icône d'état ou une bordure capillaire complète. |
| **Emojis dans l'UI** (ex: 🚀, 🟢, ⚡, ⚙️) | Rendu incohérent selon l'OS, manque de sérieux technique, pollution visuelle. | Utiliser exclusivement des icônes vectorielles SVG cohérentes (Phosphor ou Lucide en `stroke-width: 1.75px` ou `2px`). |
| **Illustrations vectorielles "doodles" ou sketchy SVG** | Filtres de distorsion et croquis pseudo-manuels qui font amateur. | Pas d'illustration de remplissage. Privilégier des vues schématiques épurées, des terminaux textuels ou des états vides informatifs. |
| **Copywriting IA creux** ("Seamless", "Elevate", "Supercharge", "Next-Gen") | Verbiage marketing incompatible avec un outil d'infrastructure. | Vocabulaire d'ingénierie direct, factuel et concis : état des conteneurs, diagnostics, latence, gestion des sockets et des threads. |

### ⚡ Définition du « Chaos Maîtrisé » (Rebel Precision)

Le « chaos maîtrisé » souhaité pour ChiPanel n'est **PAS** un désordre visuel brouillon ni un assemblage de néons cyberpunk kitsch. C'est une **émancipation délibérée des grilles rigides et ennuyeuses d'entreprise**, inspirée des interfaces de hacker, des window managers de pointe (Hyprland, Sway) et du hardware brut d'ingénierie :

1. **Asymétrie Fonctionnelle Forte :** Les modules ne sont pas contraints à des dimensions uniformes. Une tuile de log brute peut cohabiter avec une micro-jauge ultra-dense et un panneau d'inventaire 2D NBT.
2. **Micro-Stamps & Métadonnées Brutes :** Affichage décomplexé d'identifiants techniques utiles en typographie monospace compacte (`[0x7F::RCON]`, `PID:4012`, `zstd:9`, `sock:podman.sock`), renforçant la sensation d'un outil sans intermédiaire qui parle directement à la machine.
3. **Ruptures Visuelles Calculées :** Coins chanfreinés à 45° sur les badges d'urgence, micro-lignes matricielles discrètes (hairline grid), séparateurs verticaux à espacement dynamique et accents de couleur vifs mais extrêmement rares.
4. **Réactivité Immédiate sans Latence :** L'UI réagit au doigt et à l'œil, donnant l'impression physique d'un outil débridé, sans friction ni animations décoratives bloquantes.

---

## 2. Stratégie de Couleurs OKLCH & Contraste WCAG AA/AAA

La palette de ChiPanel repose sur le **principe de rareté de la couleur (Color Scarcity)** : 90 % de l'interface est monochrome et texturée en niveaux d'ardoise et noir obsidienne, tandis que les couleurs d'accent sont strictement réservées aux vecteurs d'information et aux changements d'état.

### Fond Sombre Monolithique (Dark Surfaces & Obsidian)
L'interface évite le noir absolu OLED (`#000000`) qui génère des contrastes violents et le "smearing", tout en bannissant les tons beiges/crème génériques. Elle utilise une rampe ardoise/bleu-nuit profonde :

- **Canvas Base (`--bg-base`)** : `#0B0D13` (`oklch(0.16 0.015 260)`) — Fond global d'application, ultra-sombre, reposant pour la rétine.
- **Surface Primaire (`--bg-surface`)** : `#12151E` (`oklch(0.20 0.018 260)`) — Cartes monolithiques, modules du bento, barres d'outils.
- **Surface Élevée (`--bg-elevated`)** : `#191D2A` (`oklch(0.24 0.022 260)`) — Modales, popovers, menus déroulants, tooltips, HUD flottant.
- **Surface Active / Focus (`--bg-active`)** : `#212638` (`oklch(0.28 0.024 260)`) — Éléments sélectionnés au clavier ou sous curseur actif.
- **Bordure Capillaire (`--border-hairline`)** : `rgba(255, 255, 255, 0.08)` — Ligne ultra-fine délimitant les surfaces monolithiques.
- **Bordure Structurelle (`--border`)** : `#232838` (`oklch(0.28 0.018 260)`) — Séparateurs principaux et contours de panneaux.
- **Bordure Subtile (`--border-subtle`)** : `rgba(255, 255, 255, 0.04)` — Délimitations internes légères dans les tableaux et jauges.

### Typographie & Évitement de l'Éblouissement (Anti-Glare)
Le blanc 100 % pur (`#FFFFFF`) en pleine surface de lecture est banni pour prévenir la fatigue oculaire et les phénomènes de halo sur fond sombre.

- **Texte Primaire (`--text-primary`)** : `#F1F3F9` (`oklch(0.96 0.008 260)`) — Ratio de contraste **~14:1** sur `--bg-surface` (WCAG AAA).
- **Texte Secondaire (`--text-secondary`)** : `#9DA4B9` (`oklch(0.72 0.02 260)`) — Métadonnées, labels secondaires, ratio **~5.5:1** (WCAG AA).
- **Texte Muted (`--text-muted`)** : `#7A839E` (`oklch(0.60 0.025 260)`) — Placeholders, raccourcis clavier, ratio minimum **4.5:1** sur toutes les surfaces.
- **Texte Code / Monospace (`--text-code`)** : `#A5B4FC` (`oklch(0.78 0.08 275)`) — Identifiants bruts, ports, adresses IP, timestamps.

### Règle des Trois Rôles par Couleur d'Accent (The 3-Role Rule)
Sur un thème sombre, chaque accent sémantique est scindé en 3 variantes rigoureusement calibrées pour respecter WCAG AA/AAA (ratio ≥ 4.5:1) :

1. **`--accent-*-bg` (10–14% d'opacité)** : Fond de pill / cellule de statut.
2. **`--accent-*-text` (Lightness bump)** : Variante éclaircie pour assurer un ratio ≥ 4.5:1 sur fond sombre.
3. **`--accent-*-solid` (Darkened fill)** : Variante assombrie pour les boutons pleins avec texte blanc (`#FFFFFF`).

```css
/* --- Spécification des Accents Sémantiques ChiPanel --- */

/* 1. Vert Succès / En Ligne (Minecraft Server / Podman UP) */
--accent-green:        #0FA968;
--accent-green-bg:     rgba(15, 169, 104, 0.14);
--accent-green-border: rgba(15, 169, 104, 0.34);
--accent-green-text:   #34D399;                           /* Contraste 6.1:1 sur --bg-surface */
--accent-green-solid:  #0B8552;                           /* Bouton action primaire */

/* 2. Bleu Info / Télémesure & RCON */
--accent-blue:         #3B82F6;
--accent-blue-bg:      rgba(59, 130, 246, 0.12);
--accent-blue-border:  rgba(59, 130, 246, 0.30);
--accent-blue-text:    #60A5FA;                           /* Contraste 5.32:1 */
--accent-blue-solid:   #2563EB;

/* 3. Ambre Avertissement / Hibernation lazymc / Charge CPU Haute */
--accent-orange:        #D97706;
--accent-orange-bg:     rgba(217, 119, 6, 0.14);
--accent-orange-border: rgba(217, 119, 6, 0.34);
--accent-orange-text:   #FBBF24;                          /* Contraste 5.51:1 */
--accent-orange-solid:  #B45309;

/* 4. Rouge Danger / Crash / Action Destructive */
--danger:               #EF4444;
--danger-bg:            rgba(239, 68, 68, 0.12);
--danger-border:        rgba(239, 68, 68, 0.30);
--danger-text:          #F87171;                          /* Contraste 5.05:1 */
--danger-solid:         #DC2626;                          /* Bouton Stop/Kill */

/* 5. Violet / Lilas Électrique (Accentuation Technique & Raccourcis) */
--accent-purple:        #8B5CF6;
--accent-purple-bg:     rgba(139, 92, 246, 0.12);
--accent-purple-border: rgba(139, 92, 246, 0.28);
--accent-purple-text:   #A78BFA;                          /* Contraste 5.8:1 */
--accent-purple-solid:  #7C3AED;
```

---

## 3. Architecture Spatiale : Minimalisme Monolithique & Lignes Capillaires

### 3.1 Philosophie : « Milled Monolith & Translucent Hairlines »
L'interface adopte une approche **monolithique moderne épurée** (style Linear / Vercel) combinée à la robustesse d'un panneau de contrôle physique usiné :

* **Surfaces Plates & Lignes Capillaires :** Pas de biseaux 3D lourds ou de fausses simulations de plastique. Les cartes sont des monolithes plats d'ardoise sombre délimités par des bordures capillaires translucides de 1px (`1px solid var(--border-hairline)`).
* **Highlight Zénithal Subtil :** Les cartes majeures et le terminal HUD intègrent une micro-lumière spéculaire supérieure (`box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.07)`), donnant une sensation d'usinage de précision sans alourdir la vue.
* **Double-Bezel Optionnel (Modules Critiques) :** Pour les fenêtres maîtresses (Console xterm.js, Wizard de déploiement, Quake HUD), la structure concentrique double-bezel reste applicable :

```css
/* Monolithe Standard */
.monolith-card {
  background-color: var(--bg-surface);
  border: 1px solid var(--border-hairline);
  border-radius: var(--radius-md); /* 8px */
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05), 0 2px 8px rgba(0, 0, 0, 0.3);
  padding: 1rem;
}

/* Double-Bezel Concentrique (Console / Modales) */
.hardware-shell {
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 0.75rem; /* 12px */
  padding: 0.375rem;      /* 6px */
}

.hardware-core {
  background-color: var(--bg-surface);
  border-radius: calc(0.75rem - 0.375rem); /* 6px - Rayon Concentrique */
  border: 1px solid rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.07);
}
```

---

## 4. Cockpit Haute Densité (Power-User) & Bento Grid sans Vide

### 4.1 Organisation Cockpit Homelab
Le dashboard de ChiPanel est configuré pour une **haute densité utile**, permettant de superviser l'ensemble de l'infrastructure d'un seul coup d'œil :

* **Imbrication Dense (`grid-flow-dense`) :** Zéro pixel d'espace perdu. Les tuiles s'imbriquent avec des gouttières serrées (`gap: 0.5rem` / 8px en mode compact, `0.75rem` / 12px en mode standard).
* **Vue Multi-Serveurs Matricielle :** En-tête ou panneau latéral affichant l'état résumé de tous les conteneurs (Minecraft Paper, Fabric, Valheim, Palworld) sous forme de micro-cartes compactes (hauteur 36px–48px) avec dot d'état réactif, compte de joueurs, RAM instantanée et statut `lazymc`.
* **Micro-Jauges Linéaires Ultra-Plates (2px–4px) :** Remplacement des jauges circulaires encombrantes par des barres de progression horizontales capillaires intégrées directement sous les valeurs chiffrées, animées en `transform: scaleX(...)` GPU-safe.

```html
<!-- Micro-Jauge Linéaire GPU-Safe -->
<div class="h-1 w-full bg-white/[0.06] rounded-full overflow-hidden">
  <div 
    class="h-full bg-emerald-500 rounded-full origin-left transition-transform duration-160 ease-out"
    style="transform: scaleX(var(--usage-pct));"
  ></div>
</div>
```

### 4.2 Vérité en Télémétrie (« Truth in Telemetry »)
1. **Live Stream WebSocket / SSE :** Données réelles transmises toutes les 1000ms (ou sur événement RCON). Les valeurs numériques utilisent impérativement `font-family: var(--font-mono)` avec `font-variant-numeric: tabular-nums;` pour interdire tout tressautement d'alignement.
2. **Zéro Layout Shift (CLS = 0) :** Squelettes de chargement géométriquement identiques aux cartes réelles avec pulsation douce d'opacité (`0.2` → `0.5`).
3. **États d'Hibernation `lazymc` Dignifiés :** Si le serveur hiberne, la carte affiche un état ambre feutré avec le badge « Veille active (Proxy :25565 prêt) », le temps écoulé depuis la mise en sommeil et un bouton d'éveil immédiat.

---

## 5. Navigation 100% Clavier & Ergonomie TUI-Like

ChiPanel est entièrement opérable au clavier sans jamais nécessiter la souris, à la manière d'un terminal multiplexé (tmux) ou d'un gestionnaire de fenêtres tiling :

### 5.1 Raccourcis Globaux & Palette de Commande

| Raccourci | Action | Comportement / Latence |
| :--- | :--- | :--- |
| **`~`** ou **`Ctrl+\``** ou **`F12`** | **Ouvrir / Fermer le Quake Terminal HUD** | Descente instantanée (160ms) depuis le haut de l'écran |
| **`Ctrl+K`** ou **`Cmd+K`** | **Palette de commande globale** | Apparition instantanée **0ms**, recherche universelle |
| **`Alt+M`** | **Bascule Mode Novice ↔ Power-User** | Mutation instantanée de l'interface sans rechargement |
| **`j`** / **`k`** ou **`↓`** / **`↑`** | **Navigation entre serveurs / modules** | Déplacement du focus visuel avec highlight `--bg-active` |
| **`s`** | **Démarrer / Réveiller / Stopper le serveur** | Action immédiate ou confirmation rapide |
| **`l`** | **Focus sur la console de logs intégrée** | Scroll automatique en bas et focus sur l'input RCON |
| **`r`** | **Redémarrer le conteneur** | Déclenchement du redémarrage Podman |
| **`Esc`** | **Fermer HUD / Modale / Blur focus** | Escamotage immédiat |
| **`?`** | **Aide contextuelle des raccourcis clavier** | Affichage de l'aide sans quitter la vue active |

### 5.2 Anneaux de Focus Clavier Net & Non Intrusifs
Pour garantir l'accessibilité sans polluer la vue à la souris :
* Utilisation systématique de `:focus-visible` (jamais de contour sur simple clic souris).
* Anneau de focus capillaire double : `outline: 2px solid var(--accent-purple); outline-offset: 2px;`.

---

## 6. Terminal Flottant Quake HUD (`QuakeTerminalHUD`) & Console RCON

L'expérience des logs et du terminal RCON repose sur une **double approche complémentaire** :

```
┌────────────────────────────────────────────────────────────────────────┐
│  QUAKE TERMINAL HUD (Top Overlay, Drop-down, Activated via `~` or Ctrl+`)│
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ [Server: Minecraft-Paper] [Status: UP] [RCON: 25575] [mclo.gs ↗] │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │ [14:02:10 INFO]: Server initialized in 1.42s                    │  │
│  │ [14:02:12 INFO]: Player Chixii joined the game (192.168.1.50)    │  │
│  │ [14:02:15 WARN]: Memory heap reached 68% of cgroup limit         │  │
│  ├──────────────────────────────────────────────────────────────────┤  │
│  │ > rcon command input... (Tab completion / History ↑↓)            │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
```

### 6.1 Spécification du Quake Terminal HUD
1. **Montage Global :** Injecté dans `src/routes/+layout.svelte` au niveau racine de l'application. Persiste sa session WebSocket et son historique même lorsqu'il est fermé.
2. **Animation d'Entrée / Sortie Hyprland Snappy :**
   - Glissement depuis le haut de l'écran : `transform: translateY(0)` avec la courbe `--ease-hypr-smooth` (160ms).
   - `@starting-style { transform: translateY(-100%); opacity: 0.8; }`.
   - Fermeture ultra-rapide (120ms) sur `Esc` ou `~`.
3. **Fonctionnalités Intégrées :**
   - **Filtres de niveau en 1 touche :** `1` (ALL), `2` (INFO), `3` (WARN), `4` (ERROR), `5` (RCON).
   - **Auto-scroll intelligent :** Défilement automatique actif tant que le défileur est en bas ; se suspend dès que l'utilisateur scrolle manuellement vers le haut.
   - **Input RCON Sticky :** Historique persistant de commandes (touches `↑` / `↓`), auto-complétion contextuelle (commandes vanilla, plugins Bukkit/Paper, pseudos des joueurs connectés).
   - **Bouton d'export 1-clic `mclo.gs` :** Envoi sécurisé des logs anonymisés avec URL de partage copiée dans le presse-papiers.

### 6.2 Console Intégrée au Dashboard
En plus du HUD flottant, la console xterm.js reste accessible comme tuile maîtresse du Bento sur la page `/logs` et sur le dashboard power-user, avec les mêmes garanties de performance et de multiplexage.

---

## 7. Design Engineering, Micro-Interactions & Physique Hyprland / Emil Kowalski

### 7.1 Framework de Décision d'Animation (The Animation Decision Framework)
Avant d'écrire la moindre ligne d'animation, valider systématiquement sa fréquence :

| Fréquence | Décision | Justification |
| :--- | :--- | :--- |
| **100+ fois/jour** (Raccourcis clavier, `Ctrl+K`, navigation `j`/`k`) | **Zéro animation. 0ms.** | Raycast-style. L'utilisateur répète ces gestes en permanence, toute animation génère de la fatigue. |
| **Dizaines de fois/jour** (Boutons rapides, onglets, filtres) | **Ultra-snappy (100–160ms)** | Courbes Hyprland avec décélération vive. |
| **Occasionnel** (Modales de confirmation, Quake HUD, Toasts) | **Soigné & Délimité (160–220ms)** | Feedback physique clair et spatialement cohérent. |
| **Rare / 1-clic** (Wizard d'onboarding) | **Transitions fluides (200–250ms)** | Accompagnement visuel rassurant pour le novice. |

### 7.2 Tokens de Courbes d'Accélération (Hyprland Snappy Physics)

Inspirées des configurations d'animation les plus vives du shell Hyprland (dots ambxst) :

```css
:root {
  /* 1. Hyprland Snappy Snap : démarrage instantané avec micro-punch */
  --ease-hypr-snap:   cubic-bezier(0.05, 0.9, 0.1, 1.05);

  /* 2. Hyprland Smooth Deceleration : glissement de panneau & HUD */
  --ease-hypr-smooth: cubic-bezier(0.16, 1, 0.3, 1);

  /* 3. Interaction standard UI vive (Emil Kowalski) */
  --ease-out:         cubic-bezier(0.23, 1, 0.32, 1);

  /* 4. Mouvement traversant ou morphing de carte */
  --ease-in-out:      cubic-bezier(0.77, 0, 0.175, 1);
}
```

### 7.3 Physique du Clic : `scale(0.97)` sur `:active`
Tout élément interactif (bouton, tuile, switch) réagit physiquement à la pression :

```css
.btn, .clickable-tile {
  transition: transform 140ms var(--ease-out),
              background-color 140ms var(--ease-out),
              border-color 140ms var(--ease-out),
              opacity 140ms var(--ease-out);
}

.btn:active:not(:disabled), .clickable-tile:active:not(:disabled) {
  transform: scale(0.97);
}
```

### 7.4 Jamais d'Apparition depuis `scale(0)`
Rien n'apparaît du néant absolu dans le monde physique :
* Les modales, popovers et menus démarrent leur entrée à **`scale(0.95)`** avec `opacity: 0`.

### 7.5 Popovers Origin-Aware vs Modales Centrées
* **Popovers, dropdowns et menus contextuels :** Déploiement obligatoire depuis le déclencheur (`transform-origin: var(--origin, top left)`).
* **Modales & Quake HUD :** Les modales conservent `transform-origin: center` et le Quake HUD conserve `transform-origin: top center`.

### 7.6 Micro-Crossfade avec `filter: blur(2px)`
Lorsqu'un composant change d'état interne (ex: bascule de sous-vue ou rechargement d'un bloc de métriques), un micro-flou de `2px` combiné à une variation d'opacité permet de fusionner les deux états sans à-coup visuel.

### 7.7 Tooltips avec Warmup & Saut de Délai
* **Premier survol :** Délai de 300ms pour éviter toute pollution visuelle au passage de la souris.
* **Survols consécutifs :** Dès qu'un tooltip est ouvert, le survol d'icônes adjacentes ouvre leur tooltip **instantanément (0ms)** sans délai ni animation (`[data-instant]`).

---

## 8. Identité Visuelle ChiPanel & Logo « Hexagone Stencil »

### 8.1 Concept Stencil $\chi$ (Chi) & Voyant LED Réactif
* **Forme Principale :** Silhouette hexagonale isométrique (angles 30°/60°) évoquant la mécanique d'un rack serveur et la géométrie de construction cubique de Minecraft.
* **Découpe Stencil :** Césure centrale asymétrique formant la lettre grecque **$\chi$ (Chi)** stylisée.
* **Voyant LED Télémétrique :** Diode micro-luminescente intégrée dans le logo et le header réagissant en direct aux événements système :

| État Serveur / Homelab | Teinte LED | Comportement Lumineux |
| :--- | :--- | :--- |
| **Serveur Actif / Joueurs en ligne** | Émeraude (`#10B981`) | Lueur fixe douce (`box-shadow: 0 0 8px rgba(16,185,129,0.35)`) |
| **Hibernation lazymc** | Ambre Chaud (`#F59E0B`) | Respiration lente (cycle 4s, pulsation sinusoïdale d'opacité) |
| **Tâche de fond / Sauvegarde zstd** | Cyan Électrique (`#06B6D4`) | Rotation orbitale discrète (cycle 0.8s) |
| **Arrêté / Maintenance** | Ardoise Neutre (`#64748B`) | Fixe mat sans halo lumineux |
| **Incident / Crash RCON** | Rouge Terre (`#EF4444`) | Double clignotement d'avertissement stroboscopique |

### 8.2 Composant Implémenté `ChiPanelLogo.svelte`
Le composant officiel `src/lib/components/common/ChiPanelLogo.svelte` encapsule l'identité vectorielle Stencil et la télémétrie LED réactive :
- **Props :** `status` ('active' | 'hibernating' | 'busy' | 'stopped' | 'crash', défaut 'active'), `size` (nombre, défaut 32), `showText` (booléen, défaut false).
- **Intégration :** Déployé dans `src/routes/+layout.svelte` et réactif aux flux SSE/WebSocket de `wsStore`.

---

## 9. Typographie, Échelle de Z-Index & Tokens Spatiaux

### Pairings Typographiques
- **Police UI & Titres** : `Geist Sans`, `Inter`, ou `system-ui`.
- **Police Métriques, Logs & RCON** : `Geist Mono`, `JetBrains Mono`, ou monospace.
- **Tabular Nums** : `font-variant-numeric: tabular-nums;` obligatoire sur toutes les métriques numériques.
- **Mesure de Lecture** : `max-width: 70ch` sur la documentation et les notes d'audit.
- **Titres Équilibrés** : `text-wrap: balance;` sur tous les titres `h1`, `h2`, `h3`.

### Échelle de Z-Index Sémantique
Interdiction formelle des valeurs magiques (`z-index: 9999`). L'échelle est strictement codifiée :
```css
--z-dropdown:          10;
--z-sticky-header:     20;
--z-drawer:            30;
--z-modal-backdrop:    40;
--z-modal:             50;
--z-toast:             60;
--z-tooltip:           70;
--z-quake-hud:         100; /* Terminal Quake au sommet de l'interface */
```

---

## 10. Tableau de Revue Before / After (Code Svelte / CSS ChiPanel)

| Before | After | Why |
| :--- | :--- | :--- |
| `transition: width 200ms ease;` sur barres de progression | `transform: scaleX(calc(var(--pct) / 100)); transform-origin: left; transition: transform 160ms var(--ease-hypr-smooth);` | Élimine les reflows CPU et garantit un rendu 60fps GPU-safe sans layout shifts. |
| `border-left: 4px solid #color;` (side-tab) sur cartes | `border: 1px solid var(--border-hairline);` avec fond teinté subtil ou pastille sémantique | Élimine l'anti-pattern IA des side-tabs au profit d'une structure monolithique nette. |
| `transition: all 200ms ease;` | `transition: transform 140ms var(--ease-out), opacity 140ms var(--ease-out);` | Bannir `all` pour éviter d'animer accidentellement layout/paint et garantir 60fps constants. |
| `.btn:active { transform: translateY(1px); }` | `.btn:active { transform: scale(0.97); }` | `scale(0.97)` offre un retour haptique naturel englobant tout le bouton. |
| `@keyframes modalSlideUp { from { opacity: 0; transform: translateY(12px) scale(0.98); } }` | `modal { opacity: 1; transform: translateY(0) scale(1); transition: transform 180ms var(--ease-hypr-snap), opacity 180ms var(--ease-out); } @starting-style { modal { opacity: 0; transform: translateY(8px) scale(0.95); } }` | Élimine les keyframes non interruptibles au profit d'une transition native interruptible. |
| `ease-out` natif CSS (`cubic-bezier(0, 0, 0.2, 1)`) | `var(--ease-hypr-smooth)` (`cubic-bezier(0.16, 1, 0.3, 1)`) | La courbe custom démarre avec un punch instantané et décélère avec élégance sans traîner. |
| `transform-origin: center` sur menu dropdown/popover | `transform-origin: top left` (ou variable d'ancrage trigger) | Les popovers doivent se déployer depuis leur bouton d'appel pour respecter la cohérence spatiale. |
| Animation d'ouverture 200ms sur la recherche globale (`Ctrl+K`) | Affichage instantané `0ms` (aucune animation de transition) | Les actions déclenchées au clavier 100+ fois par jour ne doivent subir aucune latence d'animation. |
| Délai fixe de 300ms sur tous les tooltips | Délai initial de 300ms + ouverture instantanée (0ms) sur les tooltips adjacents | Évite les faux positifs initiaux tout en fluidifiant le survol d'une rangée d'icônes. |
| Animation `@keyframes btn-spin 1.2s linear infinite` | `@keyframes btn-spin 0.6s linear infinite` | Un spinner deux fois plus rapide améliore la perception psychologique de la vitesse réseau. |
| `transform: scale(0)` lors de l'apparition d'un badge | `transform: scale(0.9)` combiné avec `opacity: 0` | Rien n'apparaît du néant absolu ; débuter à 0.9 crée une entrée naturelle et fluide. |
| Ouverture lente de terminal dans une modale classique | **Quake Terminal HUD** (`~` / `Ctrl+\``) descendant en 160ms | Permet d'injecter des commandes RCON d'urgence depuis n'importe où sans quitter sa tâche. |

---

## 11. Architecture Onboarding Débutant & Navigation Dual-Mode

### 11.1 Paradigme "Novice 1-Click" vs "Power User Expert"
L'interface de ChiPanel s'adapte dynamiquement au profil de l'administrateur homelab :
- **Mode Novice (1-Click Setup) :**
  - Workflow linéaire en 4 étapes (`GameSelectorStep`, `EngineSelectorStep`, `RamAllocationStep`, `LaunchReviewStep`).
  - Abstraction totale des concepts de conteneurs, Quadlets, et variables d'environnement.
  - Recommandation intelligente d'allocation mémoire basée sur la RAM hôte détectée (`/proc/meminfo` ou sonde IPC Tauri).
  - Activation par défaut de la veille `lazymc` (0 Mo au repos).
- **Mode Power User (Expert / Cockpit Haute Densité) :**
  - Contrôle d'ingénierie complet : Quadlet `.container` éditable, terminal RCON direct, cgroups Linux et flags JVM avancés.
  - Grille bento compacte multi-serveurs et Quake Terminal HUD actif.
  - Bascule instantanée (0ms) via raccourci `Alt+M` ou switch tactile double-bezel.

### 11.2 Composants & Matérialité de l'Onboarding
1. **Conteneur Hardware Multi-Étapes :** Utilisation de l'enveloppe `.hardware-shell` (p-1.5, bordure 1px subtile) et du cœur `.hardware-core` (`#12151E`, chanfrein zénithal 1px).
2. **Jauge d'Allocation Mémoire :** Visualisation segmentée (OS Reserve, Heap Minecraft, Headroom) avec indicateur de saturation au-delà de 75% de la RAM hôte (`transform: scaleX(...)` GPU-safe).
3. **Micro-Interactions Tactiles :** Boutons d'action et tuiles de sélection dotés de `active:scale-[0.97]` et courbes d'accélération `--ease-hypr-smooth`.

---

## 12. Session Design Council I — Standards & Implémentations Établis (2026-08-20)

Cette section consigne les décisions techniques et esthétiques validées lors de la première session du Design Council de ChiPanel, et constitue une mise à jour normative de la spécification.

### 12.1 Source de Vérité Unique des Tokens (Architecture)

Le fichier `src/lib/styles/tokens.css` est désormais **l'unique source de vérité** pour tous les tokens CSS. `src/app.css` l'importe via `@import` en ligne 1 et ne redéfinit aucune variable `:root`. Toute duplication est formellement interdite.

**Palette Milled Monolith Finalisée (implémentée & en production) :**
```css
--bg-base:     #0B0D13;  /* Fond global d'application — obsidien pur */
--bg-surface:  #12151E;  /* Cartes monolithiques, sidebar, header */
--bg-elevated: #191D2A;  /* Modales, popovers, Quake HUD */
--bg-active:   #212638;  /* Sélection clavier, hover actif */
--border:      #1F2433;  /* Séparateurs structurels */
--border-subtle: rgba(255, 255, 255, 0.07); /* Hairlines internes */
```

**Biseau Spéculaire Zénithal (Highlight de Surface) :**
```css
--bevel-top:      inset 0 1px 0 rgba(255, 255, 255, 0.07);
--bevel-top-core: inset 0 1px 0 rgba(255, 255, 255, 0.09); /* cœur Double-Bezel */
```

### 12.2 Classes Utilitaires Canoniques Ajoutées à `tokens.css`

| Classe | Usage |
| :--- | :--- |
| `.micro-stamp` | Identifiants techniques monospace en petite casse : `[0x7F::RCON]`, `PID:4012` |
| `.micro-stamp-active` | Variante illuminée (bleu) pour les stamps actifs |
| `.hardware-shell` | Enveloppe Double-Bezel extérieure (fond subtil, bordure 1px, padding 6px) |
| `.hardware-core` | Cœur Double-Bezel intérieur (bg-surface, bevel-top-core) |
| `.telemetry-led-green` / `-amber` / `-blue` / `-red` | Voyants LED d'état système (6×6px, halo radial `box-shadow`) |

### 12.3 Quake Terminal HUD — Spécification Implémentée

**Fichier :** `src/lib/components/console/QuakeTerminalHUD.svelte`
**Montage :** `src/routes/+layout.svelte` (niveau racine, z-index `var(--z-quake-hud)` = 1500)

**Raccourcis d'activation :** `~` (hors INPUT/TEXTAREA), `Ctrl+\``, `F12`, `Escape` (fermeture).

**Micro-stamps de statut :**
```
[0x7F::RCON LIVE]   [PORT:25565]   [BUF:300L]
```

**Filtres 1-touche numériques :** `1` ALL · `2` INFO · `3` WARN · `4` ERR · `5` RCON

**Animation d'entrée (160ms, `--ease-hypr-snap`) :**
```css
@starting-style { transform: translateY(-100%); opacity: 0; }
transform: translateY(0); opacity: 1;
transition: transform 160ms var(--ease-hypr-snap), opacity 120ms var(--ease-out);
```

**Fonctionnalités RCON :**
- Historique de commandes (`↑` / `↓`), autocomplétion `Tab`
- Export `mclo.gs` en 1 clic avec copie URL dans le presse-papiers
- Buffer DOM plafonné à 300 entrées pour garantir 60fps GPU

### 12.4 Corrections Responsives Mobile (375px) — Zero Overflow Garanti

**Cause racine diagnostiquée par Playwright :** `.login-card` avec `padding: var(--space-8)` excédait la largeur disponible à 375px (+35px hors-cadre).

**Correctifs appliqués sur `src/routes/login/+page.svelte` :**
```css
.login-container {
  box-sizing: border-box;
  width: 100%;
  overflow-x: hidden;
}
.login-shell {
  box-sizing: border-box;
  border-radius: var(--radius-card); /* unifié vers token, supprime le 1rem hardcodé */
}
.login-card {
  box-sizing: border-box;
  padding: var(--space-6);           /* réduit de space-8 */
}
@media (max-width: 480px) {
  .login-container { padding: var(--space-3); }
  .login-card      { padding: var(--space-4); }
}
```

**Résultat Playwright After — Overflow sur 68 pages × 4 viewports :**
| Viewport | Résultat |
| :--- | :--- |
| Desktop 1440px (17 pages) | ✅ Overflow: OK sur toutes |
| Laptop 1024px (17 pages) | ✅ Overflow: OK sur toutes |
| Tablet 768px (17 pages) | ✅ Overflow: OK sur toutes |
| Mobile 375px (17 pages) | ✅ Overflow: OK sur toutes |

### 12.5 Correctifs de Tokens Hardcodés

| Fichier | Avant | Après | Raison |
| :--- | :--- | :--- | :--- |
| `MetricsChart.svelte` | `#6366F1` (violet Tailwind) | `#3B82F6` / `rgba(59,130,246,0.12)` | Alignement palette bleue ChiPanel |
| `RamAllocationStep.svelte` | `#161922`, `#1F2330` | `var(--bg-surface)`, `var(--bg-elevated)` | Source de vérité tokens |
| `OnboardingWizard.svelte` | `#161922` | `var(--bg-surface)` | Idem |
| `login/+page.svelte` | `border-radius: 1rem` | `var(--radius-card)` | Alignement token systémique |

### 12.6 ModeSwitch Responsive (Header Mobile)

**Fichier :** `src/lib/components/onboarding/ModeSwitch.svelte`

En dessous de 640px, `ModeSwitch` passe en mode icon-only (icônes `Sparkles` et `Terminal` 32×32px) au lieu du texte complet (210px). Cette réduction divise l'emprise header de 210px à ~68px, éliminant le débordement horizontal.

```css
@media (max-width: 640px) {
  .mode-label { display: none; }
  .mode-btn   { width: 32px; padding: 0; justify-content: center; }
}
```

### 12.7 Physique Universelle du Header (`+layout.svelte`)

Haptics unifiés sur tous les éléments interactifs du header :
```css
.nav-item:active, .action-btn:active { transform: scale(0.97); }
```
Transition : `140ms var(--ease-out)` sur `transform`, `opacity`, `background-color`.

### 12.8 Résultats Build

```
npm run check  → svelte-check found 0 errors and 124 warnings in 34 files
npm run build  → Vite static build succeeded (exit 0)
```

---

## 13. Session Design Council II — Standardisation des Headers (2026-08-20)

Cette section consigne la standardisation des en-têtes de page autour d'un unique composant canonique, adopté par toutes les pages cockpit.

### 13.1 Composant Canonique `PageHeader.svelte`

**Fichier :** `src/lib/components/ui/PageHeader.svelte`

Composant unique remplaçant les **6 familles d'en-têtes divergentes** identifiées (`page-header`, `page-header-card card`, `hardware-shell` + `hardware-core`, `content-header`, `hero-header`, en-tête nu). API :

| Prop | Type | Usage |
| :--- | :--- | :--- |
| `title` | `string` | Titre de page (`h1`, 1.75rem / 700 / -0.02em) |
| `subtitle` | `string` | Sous-titre descriptif (`--text-secondary`, 14px) |
| `icon` | `Snippet` | Icône sémantique optionnelle (boîte 40×40px `--accent-blue-bg`) |
| `badge` | `Snippet` | Pill d'état optionnel, rendu à droite du titre |
| `children` | `Snippet` | Zone d'actions à droite (`header-actions`) |

**Structure imposée (référence `/engine`) :** titre + sous-titre + actions, sans carte enveloppante, typographie unifiée 1.75rem, espacements `--space-4`, tokens exclusifs (zéro couleur hardcodée).

### 13.2 Pages Standardisées

Les 15 pages cockpit (`dashboard`, `console`, `players`, `files`, `addons`, `worlds`, `backups`, `engine`, `diagnostics`, `metrics`, `database`, `network`, `audit`, `permissions`, `accounts`) utilisent désormais `PageHeader`. Le CSS d'en-tête dupliqué (`.page-header-card`, `.icon-badge`, `.title-with-badge`, `.hero-*`, `.content-header`, etc.) a été supprimé de chaque page.

**Correctifs collatéraux :**
- Classes mortes supprimées des headers : `badge-primary`, `badge-info` (aucune règle CSS n'existait).
- Couleurs hardcodées de l'en-tête `/engine` (`#f8fafc`, `#94a3b8`) → tokens (`--text-primary`, `--text-secondary`).

### 13.3 Correction du Header-Bar Global (Responsive)

Le header global (`src/routes/+layout.svelte`) débordait horizontalement sur laptop (1024px) et mobile (375px) — cause racine : le pill télémétrie `.quick-status-pill` (≈350px) restait visible jusqu'à 900px alors que la sidebar (250px) réduit la largeur utile du contenu.

- `.quick-status-pill` masqué à `max-width: 1200px` (au lieu de 900px).
- `.header-left { flex: 1 1 auto; min-width: 0 }` + `.status-badge`/`.status-label` avec `overflow: hidden; text-overflow: ellipsis`.
- Résultat Playwright : **0 overflow horizontal** sur 16 pages × 4 viewports (1440 / 1024 / 768 / 375).

### 13.4 Résultats Build

```
npm run check  → svelte-check found 0 errors and 124 warnings in 34 files
npm run build  → Vite static build succeeded (exit 0)
Playwright     → 0 horizontal overflow (16 pages × 4 viewports)
```

---

## 14. Session Design Council III — Onboarding, Auth & Routes Alias (2026-08-20)

### 14.1 Onboarding `/setup`

Le wizard novice reste une surface dédiée, mais reprend désormais les primitives d'identité et de responsive du produit :

- le logo texte `χ` est remplacé par `ChiPanelLogo.svelte` avec LED d'état ;
- les rayons et espacements du double-bezel utilisent les tokens (`--radius-card`, `--space-*`) ;
- sur mobile, le header devient une grille stable (identité/runtime à gauche, bascule de mode à droite) ;
- la bannière novice passe en colonne lisible au lieu de compresser le texte et le raccourci dans trois zones concurrentes ;
- `GameSelectorStep` utilise `--bg-active` pour l'état sélectionné.

### 14.2 Auth `/login`

L'écran d'accès conserve son double-bezel minimal et ajoute le contrôle attendu d'affichage du mot de passe (`Eye` / `EyeOff`). Le contrôle reste un vrai bouton clavier, avec `aria-label`, état visuel hover et retour tactile `scale(0.97)`.

### 14.3 Routes Alias

`/plugins`, `/modpacks` et `/profiles` conservent leur routage immédiat vers le Centre des Addons, mais partagent maintenant `src/lib/components/ui/RedirectState.svelte` : état de transition accessible, destination explicite, barre de progression discrète et lien de secours si la navigation est retardée.

### 14.4 Vérification

```
npm run check  → 0 erreur, warnings historiques du codebase conservés
npm run build  → Vite static build réussi
Playwright     → zéro overflow sur setup et toutes les pages cockpit
Routes alias   → plugins → /addons?tab=plugins, modpacks → /addons?tab=modpacks,
                 profiles → /addons?tab=profiles
```

## 15. Session Design Council IV — Kit UI Consolidé, Barrel d'Icones, Lazy-Load CodeMirror & A11y Modales (2026-08-20)

### 15.1 Kit de Primitives `ui/`

Répertoire `frontend/src/lib/components/ui/` — les primitives réutilisables du design system :

| Composant | Rôle | Contrat |
| :--- | :--- | :--- |
| `Spinner.svelte` | Loading inline (`--spin` global) | `size`, `label` (aria-label), `class` |
| `EmptyState.svelte` | État vide canonique | `title`, `description` (**string OU snippet**), `icon`/`action` (snippets), `card`, `compact`, `dashed` (booléens), `class` |
| `Modal.svelte` | Shell de modale accessible | `open`, `onclose`, `class` (classes du panneau), `backdropClass`, `role`/`ariaLabelledBy`, `initialFocus`, `closeOnBackdrop`, contenu via `{#snippet content()}` |
| `PageHeader.svelte` | Header de page canonique (Design Council II) | `title`, `subtitle`, `icon`/`badge` (snippets), actions via le contenu enfant |
| `RedirectState.svelte` | État de redirection | `title`, `description`, `href`, `actionLabel`, `children` |
| `ConfirmDialog.svelte` | Confirmation accessible de référence | `open`, `title`, `message`, `confirmLabel`/`cancelLabel`, `destructive`, `busy`, `onconfirm`, `oncancel` |
| `Callout.svelte` | Message de statut inline | `tone: info \| success \| warning \| danger`, `title`, `action` (snippet), contenu enfant |
| `KpiCard.svelte` / `KpiGrid.svelte` | Tuile et grille d'indicateurs | `label`, `value`, `icon`, `tone`, `mono`, `children` / grille via `children` |
| `SearchBar.svelte` | Champ de recherche canonique | `value` (bindable), `placeholder`, `label`, `compact`, `oninput`, `onclear` |
| `CategoryTabs.svelte` | Filtre segmenté | `options: {id,label,count}[]`, `active` (bindable), `variant` |
| `ChecklistCard.svelte` | Carte de configuration cochable | `title`, `detail`, `badge`, `checked` (bindable, `null` = lecture seule), `icon` |
| `HeroBanner.svelte` | Bandeau d'identité | `title`, `badge`, `titleExtra`, `visual`, `meta`, `kpis` |

Variants `EmptyState` calées sur des usages réels : `card` (bordure + bg-surface, ex-worlds), `compact` (titre base/medium, desc xs, icône opacity .5, ex-PlayerListCard), `dashed` (bg-base + bordure tirets `--border-focus`, ex-StatusEffectsPanel). Titre canonique `.empty-title` = `font-size-md` + `font-weight-semibold`.

### 15.2 Barrel d'Icones `icons.js`

`frontend/src/lib/icons.js` re-exporte les 133 icônes Lucide utilisées (seul fichier important `lucide-svelte`). Tous les composants importent depuis `$lib/icons.js` — permet un futur swap de librairie sans toucher 66 fichiers. Migration : `import { X } from 'lucide-svelte'` → `import { X } from '$lib/icons.js'`.

### 15.3 Lazy-Load CodeMirror (`/files`)

`routes/files/+page.svelte` charge `ConfigEditor` via `{#await import(...)}` — CodeMirror (296 Ko) n'est plus dans le chunk initial. Chunk JS non-lazy : **56 Ko** (était 103 Ko après barre-laterale, 320 Ko avant chantier). État `.editor-loading-state` avec `Spinner`.

### 15.4 Contrat A11y Modal Unifié

Toute modale passe par `Modal.svelte` (toutes les routes et tous les composants ; il ne reste aucun `modal-backdrop` brut hors de `Modal`/`ConfirmDialog`) :

- `role="dialog"` + `aria-modal="true"` + `aria-labelledby` + `tabindex="-1"` sur le panneau.
- Focus initial sur le premier élément focusable à l'ouverture, focus restauré sur le déclencheur à la fermeture.
- Fermeture ESC (`svelte:window`) et clic backdrop (`closeOnBackdrop`).
- Tab trap : le focus reste dans la modale (cycle premier/dernier focusable).
- Backdrops spécialisés externalisés : `.modal-backdrop--dim` (ex-VersionPicker) et `.modal-backdrop--deep` (ex-ConfigDiff) dans `app.css`.
- Gardes busy dans `onclose` (ex. `submitting ? null : onClose()`) pour bloquer la fermeture pendant un traitement.

### 15.5 Vérification

```
npm run check  → 0 erreur / 58 warnings (tous faux positifs "Unused CSS" :
                 classes passées à des composants icônes ou via props de modale,
                 requises à l'exécution — ne PAS supprimer)
npm run build  → 2,1 Mo total, chunk non-lazy 56 Ko, CodeMirror lazy 296 Ko
Playwright     → modales vérifiées (role/focus/ESC/backdrop/trap), zéro overflow
                 16 pages × 4 viewports
Micro-fixes    → 5 selecteurs segment-* morts supprimés (ModpackCatalogBrowser),
                 18 role="dialog" sans tabindex corrigés, role="button" sur slots
                 interactifs InventoryVisualizer + QuickCommands, handlers clavier
                 sur divs cliquables (backdrops/panneaux), line-clamp standard,
                 tabindex radiogroup ChunkyPanel, svelte-ignore state_referenced_locally
                 (initialiseurs de props intentionnels)
```

---

## 16. Session Design Council V — Source Unique Réelle, Shell Canonique & Garde-Fou (2026-11-22)

Cette session supprime la dette qui faisait dériver les pages malgré le kit existant.

### 16.1 Une seule source de tokens, pour de vrai

`app.css` redéfinissait `:root` (palette « engine », indigo/emerald, z-index) en violation de §12.1. C'est corrigé :

- `src/lib/styles/tokens.css` est la **seule** définition de tokens (121 tokens).
- `src/app.css` ne contient plus aucun `:root` ni aucun hex : uniquement la suite de composants globaux (`.btn`, `.card`, `.badge`, `.table`, `.modal`, formulaires).
- 37 alias legacy (`--engine-*`, `--accent-indigo*`, `--accent-emerald*`, `--bg-surface-2`, `--bg-inset`, `--bg-card`, `--bg-subtle`, `--border-hover`, `--space-*` manquants) ont été migrés puis supprimés. Les valeurs sémantiques canoniques sont désormais seules.
- Tokens manquants réintroduits : `--space-5/10/16`, `--radius-full`, `--radius-modal`, `--bg-code`, `--bg-terminal`, `--text-on-solid`, `--mc-*` (couleurs de jeu).

### 16.2 Shell de page canonique

Toutes les routes cockpit utilisent une seule colonne, définie dans `tokens.css` :

```css
.page-stack {
  display: flex; flex-direction: column; gap: var(--space-6);
  width: 100%; max-width: 1380px; margin-inline: auto;
  padding-bottom: var(--space-8); min-width: 0;
}
```

Références visuelles : **`/engine`** (header + toolbar + grille) et **`/dashboard`** (bento + télémétrie). Les 15 routes ont été migrées ; les anciens wrappers `.dashboard-page`, `.engine-page`, `.network-page-layout`, etc. sont supprimés. Seules les pages plein écran (`/login`, `/setup`, `/console`, `/files`) conservent des contraintes locales de hauteur.

### 16.3 Règles d'extension (non négociables)

1. Nouvelle page : `<div class="page-stack">` + `PageHeader` + primitives du kit. Jamais de wrapper local.
2. Alerte/erreur : `Callout` (action de retry via `{#snippet action()}`). Jamais de `.alert` local.
3. Indicateurs : `KpiGrid` + `KpiCard` (icône, ton sémantique, valeur mono).
4. Recherche : `SearchBar` (`compact` dans les barres d'outils, `onclear` pour relancer une requête).
5. Modale : `Modal` (focus trap, ESC, restore focus). Confirmation destructive : `ConfirmDialog`. Ne jamais utiliser `confirm()`/`alert()` natifs.
6. Toasts : `toast` store global via `toast.forward` (mappe `danger` → `error`). Pas d'état de toast local.
7. API : `apiGet` / `apiPost` / `apiDelete` / `apiDownload`. `apiFetch` uniquement pour un contrat HTTP spécifique.
8. Couleur : token existant ou nouveau token dans `tokens.css`. Les seules couleurs littérales admises dans un `<style>` sont annotées `/* token-ok: raison */` (ex. séries de graphes calculées en JS).

### 16.4 Garde-fou automatique

```bash
cd frontend && npm run check:tokens
```

`scripts/check-tokens.mjs` échoue si :

- un `var(--x)` n'est défini ni dans `tokens.css` ni localement dans le fichier ;
- un `<style>` (ou un CSS global) contient un hex non annoté `token-ok:` ;
- un `:root` est redéfini hors `tokens.css`.

`npm run check` exécute ce garde-fou avant `svelte-check`. Sortie attendue : `check:tokens — OK (121 tokens, 0 drift)`.

### 16.5 Migrations effectuées

- 31 fichiers : 113 hex remplacés par des tokens (zéro changement de structure).
- `/audit`, `/database`, `/network`, `/backups` : KPI locaux → `KpiGrid`/`KpiCard`, alertes → `Callout`.
- `/audit`, `/database`, `/engine`, `/players`, `/files`, `/backups`, `/diagnostics`, `/addons`, `/console`, `/modpacks` et leurs composants : modales brutes → `Modal` (a11y unifiée, gardes busy), y compris `QuickCommands`, `AlertSettings`, `ProfileManager`, `GroupEditor`, `WorldBackup`, `Modrinth`/`Modpack` (install/deploy), `InstalledPluginsList`.
- `/backups` : suppression via `apiDelete` ; `PlayerProfileModal` et `InstalledPluginsList` : `confirm()` natifs → `ConfirmDialog` (suppression groupée conservée).
- `/audit` : export CSV via `apiDownload` (plus de lecture `localStorage` ni de `fetch` manuel).
- `SearchBar` adoptée dans 11 vues (pages + composants), `toast.forward` remplace les adaptateurs locaux.
- `CustomSelect.svelte` supprimé : le `.select` global stylé est le seul contrat.

### 16.6 Vérification

```
npm run check:tokens → OK (121 tokens, 0 drift)
npm run check        → 0 erreur
npm run build        → exit 0 (adapter-static)
```

### 16.7 Dette restante assumée

- Les `rgba(...)` d'accent (halos, ombres, fonds translucides) restent littéraux : ils demandent des tokens `-bg-soft` par accent. À traiter lors d'une passe dédiée, sans changer les composants.
- Les couleurs de séries de `MetricsChart.svelte` vivent dans le `<script>` (uPlot) et ne sont donc pas couvertes par le garde-fou.
- Les icônes Lucide passées en props restent signalées « Unused CSS selector » par `svelte-check` : faux positifs connus, ne pas supprimer.

---

## 17. Session Design Council VI — Un seul code partout (2026-11-23)

Spec complète : [`docs/superpowers/specs/2026-11-23-ui-unification-design.md`](docs/superpowers/specs/2026-11-23-ui-unification-design.md).

### 17.1 Constat mesuré

L'audit croisé source + application live (Playwright, 20 routes, 1440/390) confirme que les cinq councils précédents ont bien unifié la structure : 15/15 routes cockpit en `.page-stack` + `PageHeader`, h1 28px/700, zéro overflow. La dérive restante était **la duplication de CSS scopé** : `.active` ×23, `.modal-header` ×14 (11 variantes), `.loading-state` ×11 (11 variantes), `.tabular-nums` ×12 (1 variante, déjà globale), `@keyframes spin` **×25** (3 variantes dont une incomplète sur `/engine`), `.empty-icon` ×15, `.data-table` ×13.

### 17.2 Décisions d'outillage (recherche + brainstorming)

| Outil évalué | Décision |
| :--- | :--- |
| Tailwind / UnoCSS | Écarté : réécriture, runtime, conflit avec les 121 tokens et « zéro framework CSS lourd ». |
| Stylelint `no-duplicate-selectors` | Écarté : ne détecte les doublons **que dans un seul fichier**, plugin custom + devDep nécessaires. |
| Docs Svelte 5 (`scoped-styles` / `global-styles`) | Retenues : `<style>` scopé = règles propres au composant, décisions partagées = CSS global. |
| `@layer` (css.dev) | Non retenu : changer la cascade globale à ce stade est risqué sans runner de test. |
| `check-tokens.mjs` existant | **Retenus et étendu** : zéro dépendance, déjà branché sur `npm run check`. |

### 17.3 Réductions appliquées

- **25 `@keyframes spin` → 1** : définition unique dans `app.css`. Les `animation: spin …` locales restent (durées contextuelles légitimes) et référencent le keyframe global. Vérifié en live : `getComputedStyle().animationName === "spin"`, une seule `CSSKeyframesRule` dans `document.styleSheets`.
- **11 copies locales de `.tabular-nums` supprimées** : la règle existait déjà dans `tokens.css`, identique.
- **3 wrappers morts supprimés** (`players-page`, `worlds-page`, `permissions-page` = `position: relative`) et **2 wrappers plein écran unifiés** (`console-page-layout`, `files-page-container` → `.page-stack--full` dans `tokens.css`).
- Primitives canoniques ajoutées une fois dans `app.css` : `@keyframes spin`, `.sr-only`, `.btn-md`.

### 17.4 Cohérence transversale

- **Langue** : `<html lang="fr">`, 4 h1/sous-titres anglais traduits, `meta description` ajoutée.
- **Titres** : 20 routes + `app.html` au format unique `Libellé | ChiPanel`, aucun suffixe `- ChiServ`, aucun tiret cadratin (R-02), 2 titres manquants ajoutés (`/files`, `/permissions`), titre ajouté à la route racine.
- **`/setup`** : 2 `<main>` imbriqués → 1, `<h1 class="sr-only">` ajouté, hiérarchie H1 → H2 rétablie, `ModeSwitch` du wizard supprimé (la bascule globale du header reste l'unique contrôle).
- **Mobile 390px** : cibles header ≥ 44×44 (hamburger, console, `ModeSwitch`, déconnexion), zéro overflow. `ModeSwitch` passe de 32px à 44px, y compris la variante `.compact`.
- **Assets** : aucun asset créé (R-23). `favicon.svg` 404 → `static/favicon.png` existant, dans `app.html` **et** `manifest.webmanifest` (`sizes: "32x32"`, `purpose: "any"`), `theme-color`/`background_color` alignés sur `--bg-base` (`#0B0D13`).

### 17.5 Garde-fou étendu

`frontend/scripts/check-tokens.mjs` échoue désormais si :

1. un `var(--x)` est inconnu, un `:root` est redéfini hors `tokens.css`, un hex non annoté `token-ok:` apparaît dans un `<style>` (contrôles historiques) ;
2. **un composant redéclare une classe canonique globale et n'est pas enregistré dans `scripts/ui-baseline.json`** (89 sélecteurs canoniques calculés depuis `app.css` + `tokens.css`, 51 variantes locales acceptées et verrouillées) ;
3. un composant déclare son propre `@keyframes spin` ;
4. une route n'a pas de `<title>`, n'utilise pas le suffixe ` | ChiPanel`, ou contient un tiret cadratin ;
5. `app.html` ne déclare pas `lang="fr"`.

Pour autoriser une **nouvelle** variante locale : ajouter le chemin (relatif à `src/`) à la clé du sélecteur dans `ui-baseline.json`, avec la raison dans la PR. Ne jamais régénérer la baseline en silence. Testé : une copie injectée fait échouer le garde-fou, son retrait le fait repasser.

### 17.6 Vérification

```
npm run check:tokens → OK (121 tokens, 89 canonical selectors, 0 drift)
npm run check        → 0 erreur / 75 warnings (identique à HEAD : faux positifs « Unused CSS selector »)
npm run build        → exit 0 (adapter-static)
Playwright :5173     → 16 routes par click-through SPA : titres/lang/h1/structure OK, 0 overflow
                       390px : 0 overflow, 0 cible < 44px
                       0 erreur / 0 warning console sur chargement frais
                       reduced-motion : animationDuration 0.01ms
```

### 17.7 Dette restante assumée (verrouillée par baseline)

Les variantes non identiques de `.modal-*`, `.loading-state`, `.data-table`, `.empty-icon` et des utilitaires de couleur restent locales ; elles sont enregistrées dans `ui-baseline.json` pour interdire toute **nouvelle** divergence. Une passe dédiée pourra les migrer vers le kit (`Modal`, `Spinner`, `EmptyState`) sans risquer cette session. Les 401 pré-auth sur `/login` et les 404/500 des endpoints sans données du preview restent hors périmètre frontend. Le message backend « Server engine configuration is not readable from ChiPanel » est désormais localisé via `$lib/utils/apiMessages.js` (Council VII).

---

## 18. Session Design Council VII — Le cadre de `/engine` partout (2026-11-24)

### 18.1 Demande et cadrage

« La page est moche, je veux les bordures et le travail du design de la page moteur répliqués à tout ChiPanel. » Cadrage délégué : normaliser `/engine` puis répliquer « un peu tout » via le kit, décider des défauts, reconstruire `:25500`.

### 18.2 Mesure Playwright avant (15 routes, 1440px)

| | `/engine` | Autres |
| :--- | :--- | :--- |
| Éléments bordés | 48 | 6 à 30 |
| Sous-panneaux imbriqués | 21 | 0 à 4 |
| Ombres `inset` (biseau) | 1 | 2 à 19 |
| `HeroBanner` | 1 (seule page) | 0 |

Le biseau 1px était déjà global (`app.css`), mais `/engine` était la seule page à **structurer** ses panneaux, avec `HeroBanner` + `KpiGrid`, une barre d'outils bordée et des tuiles à 16px. `:25500` était stale (`lang=en`, ancien titre, `favicon.svg`).

### 18.3 Brainstorm : 3 approches

1. Copier le CSS de `/engine` page par page → **rejeté** (dérive, contraire au Council VI).
2. Ajouter `HeroBanner` + eyebrow sur chaque route → **rejeté** (`design.md` §1 interdit la répétition systématique d'eyebrows ; R-38 interdit d'inventer du contenu).
3. **Canonicaliser la grammaire Double-Bezel dans `app.css` + le kit, puis normaliser `/engine`** → **retenu** : une seule source, les 16 routes héritent.

Dials **ENERGY 1 / RHYTHM 1 / MOTION 1** (cockpit calme, §7.1).

### 18.4 Normalisation de `/engine` (préalable)

`border-radius: 16px → var(--radius-card)` (plafond 12px), tous les `rgba(...)` indigo/ambre/rouge/vert remplacés par les tokens sémantiques, gradient décoratif de `.card-active` supprimé, `.config-alert` migré vers `Callout`, `.tag`/`.toolbar` locaux supprimés au profit des classes globales, `box-shadow` de la modale → `var(--modal-shadow)`. Aucun `rgba`/gradient/16px ne reste dans la page.

### 18.5 Grammaire canonique (`app.css`)

`.bezel-frame` (cadre extérieur), `.bezel-panel` (panneau interne), `.micro-label`, `.toolbar` (barre bordée), `.tag` + `.tag-blue/green/purple/orange/danger`. `.card` et `.card-header` passent en bordure capillaire `--border-subtle` + biseau spéculaire.

Kit mis à niveau une seule fois : `KpiGrid` (`framed=true` par défaut → `.bezel-frame`), `KpiCard` (bordure + biseau + hover + `.micro-label`), `Callout`, `CategoryTabs`, `SearchBar`, `HeroBanner`, `EmptyState` (nouveau prop `framed` → `.bezel-panel` + médaillon d'icône).

### 18.6 Défauts traités (décision déléguée)

Les 5 : densité (états vides cadrés sur `/players`, `/worlds`, `/audit`, `/backups`, `/engine`), cartes plates (biseau global), barres d'outils (`.toolbar` global, appliqué à `/engine`), badges (`.tag` canonique), et mélange FR/EN (traduction des libellés visibles de `/players`, `/worlds`, `/permissions`, `/files`, `/console`, `/metrics`, `/network` + composants `ChunkyPanel`, `PlayerList`, `LogViewer`, `CommandBar`, `ConfigEditor` ; `localizeServerError()` pour les messages backend).

### 18.7 Correction du garde-fou

`check-tokens.mjs` ne matche plus que les **sélecteurs simples réels** : extraction du contenu `<style>` seul, en-têtes entre accolades, commentaires retirés, token de classe délimité par espace/virgule/`>+~`. `.tag-meta` n'est plus compté comme copie de `.tag`, `.group-avatar.badge-danger` comme copie de `.badge-danger`, et les `{#if}` du template Svelte ne sont plus des sélecteurs. Baseline régénérée : **99 sélecteurs canoniques, 45 variantes locales acceptées**.

### 18.8 Vérification

```
npm run check:tokens → OK (121 tokens, 99 canonical selectors, 0 drift)
npm run check        → 0 erreur / 75 warnings (inchangé vs HEAD)
npm run build        → exit 0 (adapter-static)
Playwright :5173     → 15 routes en click-through SPA : lang=fr, 1 <h1>, titres conformes,
                       0 overflow, biseau inset sur 13/15 pages (1 → 41 sur /engine),
                       /engine rayon 12px, états vides cadrés
Playwright 390px     → 5 routes : 0 overflow, 0 cible < 44px, 1 <main>, 1 <h1>
Console              → 0 erreur JS ; 404/500 uniquement sur les endpoints sans données du preview
```

**Déploiement preview** : image reconstruite avec `docker build --no-cache -t localhost/chipanel:latest -f Containerfile .`, conteneur `chipanel-local` recréé (`DATA_DIR=/tmp/chipanel-data`). Vérifié sur **`:25500`** (l'URL réellement utilisée) : `lang=fr`, `Connexion | ChiPanel`, `favicon.png`, 15 routes en click-through SPA, `.card`/`.kpi-card`/`.banner-card` à 12px + 1px + biseau, `/engine` 41 biseaux / 50 éléments bordés, zéro overflow, mobile 390px sans cible < 44px. Sans `--no-cache`, l'ancien frontend restait servi (piège `CLAUDE.md`).

### 18.9 Dette restante

Les grands vides sous les états vides sont inhérents à un preview sans données (R-38 interdit de les remplir avec du contenu fictif). Quelques tooltips anglais subsistent dans des composants secondaires ; `localizeServerError()` est le point d'entrée pour tout nouveau message backend. Le message d'audit §17.7 est mis à jour en conséquence.
