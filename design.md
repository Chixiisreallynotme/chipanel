# ChiPanel Design System Specification — Directives & Standards

Ce document constitue la **spécification technique, visuelle et comportementale de référence** pour l'interface de **ChiPanel**. En tant que console d'administration homelab et Minecraft de production, l'interface privilégie la clarté opérationnelle, la densité utile, la durabilité visuelle et le respect rigoureux des contraintes d'accessibilité et de design engineering.

---

## 1. Directives Anti-AI-Slop Strictes (Negative Constraints)

Pour préserver l'intégrité, le professionnalisme et l'authenticité de l'outil, tous les tics visuels et facilités génératives de l'IA ("AI Slop") sont formellement proscrits du codebase et des composants.

### 🚫 Éléments Formellement Proscrits & Alternatives

| Anti-Pattern (Banni) | Pourquoi c'est banni (AI Tell) | Règle & Alternative Impeccable |
| :--- | :--- | :--- |
| **Gradients textuels** (`background-clip: text`) | Cliché cosmétique pur, dégrade la lisibilité et l'anti-aliasing sur fond sombre. | Utiliser exclusivement du texte uni solide. L'accentuation se fait par le poids de police (`font-weight: 600`) ou l'échelle typographique. |
| **Glows / Auras violettes & néons** (`box-shadow: 0 0 25px rgba(139,92,246,0.5)`) | Esthétique "crypto / landing page IA" générique et bruyante qui fatigue les yeux en environnement sombre. | Bordures nettes et fines (`1px solid var(--border)` ou `var(--border-subtle)`). Ombre ultra-diffuse, quasi-invisible (< 0.05 d'opacité) si nécessaire. |
| **Pseudo-dashboards & "Hero Metrics" creux** | Gros chiffres isolés avec variations inventées (`+12.4% vs last week`) sans réalité technique. | Toutes les métriques doivent avoir une signification système concrète (ex: *Delta CPU Podman*, *RAM cgroup allouée/utilisée*, *TPS RCON*, *Ping réseau*). Pas de jauge circulaire purement décorative. |
| **Répétition systématique des "Eyebrows"** | Kicker all-caps sur-espacé (`OVERVIEW`, `ACTIONS`) au-dessus de chaque bloc de titre. | Règle de parcimonie : un eyebrow n'apparaît que pour indiquer un sous-système hiérarchique critique ou un contexte technique non évident. Les titres de cartes se suffisent à eux-mêmes. |
| **Inondation de Badges & Pills superflus** | Multiplication de capsules de tags pour décorer chaque ligne ou cellule de tableau. | Les badges sont réservés aux **états de cycle de vie dynamiques** (*Online*, *Hibernating*, *Stopped*, *Degraded*). Tout texte descriptif statique reste en texte simple `--text-secondary`. |
| **Arrondis disproportionnés** (`border-radius: 24px/32px+`) | Donne un aspect "jouet" ou prototype mobile inadapté à un panneau technique de production. | Cartes et conteneurs plafonnés à `--radius-md` (8px) ou `--radius-lg` (12px) max. Le format pilule (`rounded-full`) est strictement réservé aux tags d'état et avatars. |
| **Ghost-Cards (Border 1px + Shadow large diffuse)** | Motif typique combinant une bordure fine et un flou d'ombre lourd (≥16px). | Règle d'or : **soit** une bordure nette, **soit** une ombre subtile, jamais les deux en superposition artificielle. |
| **Side-Stripe Borders épaisses** (`border-left: 4px solid ...`) | Béquille décorative asymétrique datée pour cartes et alertes. | Utiliser un fond teinté uniforme à faible opacité (`var(--accent-*-bg)`) combiné à une icône d'état ou une bordure subtile complète. |
| **Emojis dans l'UI** (ex: 🚀, 🟢, ⚡, ⚙️) | Rendu incohérent selon l'OS, manque de sérieux technique, pollution visuelle. | Utiliser exclusivement des icônes vectorielles SVG cohérentes (Phosphor ou Lucide en `stroke-width: 1.75px` ou `2px`). |
| **Illustrations vectorielles "doodles" ou sketchy SVG** | Filtres de distorsion et croquis pseudo-manuels qui font amateur. | Pas d'illustration de remplissage. Privilégier des vues schématiques épurées, des terminaux textuels ou des états vides informatifs. |
| **Copywriting IA creux** ("Seamless", "Elevate", "Supercharge", "Next-Gen") | Verbiage marketing incompatible avec un outil d'infrastructure. | Vocabulaire d'ingénierie direct, factuel et concis : état des conteneurs, diagnostics, latence, gestion des sockets et des threads. |

---

## 2. Stratégie de Couleurs OKLCH & Contraste WCAG AA/AAA

La palette de ChiPanel repose sur le **principe de rareté de la couleur (Color Scarcity)** : 90 % de l'interface est monochrome et texturée en niveaux de gris froids/tactiles, tandis que les couleurs d'accent sont strictement réservées aux vecteurs d'information et aux changements d'état.

### Fond Sombre Tactile (Dark Surfaces & Obsidian)
L'interface évite le noir absolu OLED (`#000000`) qui génère des contrastes violents et le "smearing", tout en bannissant les tons beiges/crème génériques. Elle utilise une rampe ardoise/bleu-nuit profonde :

- **Canvas Base (`--bg-base`)** : `#0F1117` (`oklch(0.18 0.015 260)`) — Fond global d'application, reposant pour la rétine.
- **Surface Primaire (`--bg-surface`)** : `#161922` (`oklch(0.22 0.018 260)`) — Cartes, modules du bento, barres d'outils.
- **Surface Élevée (`--bg-elevated`)** : `#1F2330` (`oklch(0.26 0.022 260)`) — Modales, popovers, menus déroulants, tooltips.
- **Bordure Structurelle (`--border`)** : `#242836` (`oklch(0.29 0.018 260)`) — Séparateurs et contours de cartes.
- **Bordure Subtile (`--border-subtle`)** : `rgba(255, 255, 255, 0.07)` — Délimitations internes légères.

### Typographie & Évitement de l'Éblouissement (Anti-Glare)
Le blanc 100 % pur (`#FFFFFF`) en pleine surface de lecture est banni pour prévenir la fatigue oculaire et les phénomènes de halo sur fond sombre.

- **Texte Primaire (`--text-primary`)** : `#F1F3F9` (`oklch(0.96 0.008 260)`) — Ratio de contraste **~14:1** sur `--bg-surface` (WCAG AAA).
- **Texte Secondaire (`--text-secondary`)** : `#9DA4B9` (`oklch(0.72 0.02 260)`) — Métadonnées, labels secondaires, ratio **~5.5:1** (WCAG AA).
- **Texte Muted (`--text-muted`)** : `#848CA6` (`oklch(0.64 0.025 260)`) — Placeholders, raccourcis clavier, ratio minimum **4.68:1** sur toutes les surfaces.

### Règle des Trois Rôles par Couleur d'Accent (The 3-Role Rule)
Sur un thème sombre, une même couleur hexadécimale ne peut pas servir simultanément de fond de bouton (avec texte blanc) et de texte d'alerte sur fond noir sans briser les règles WCAG. Chaque accent sémantique est donc scindé en 3 variantes calibrées :

1. **`--accent-*-bg` (10–14% d'opacité)** : Fond de pill / cellule de statut.
2. **`--accent-*-text` (Lightened / Lightness bump)** : Variante éclaircie pour assurer un ratio ≥ 4.5:1 lorsqu'elle est affichée en texte sur fond noir ou sur son propre fond teinté.
3. **`--accent-*-solid` (Darkened fill)** : Variante assombrie utilisée pour les boutons pleins avec texte blanc (`#FFFFFF`) pour garantir un ratio ≥ 4.5:1.

```css
/* --- Spécification des Accents Sémantiques ChiPanel --- */

/* 1. Vert Succès / En Ligne (Minecraft Server / Podman UP) */
--accent-green:        #0FA968;                           /* Fills graphiques, indicateurs dots */
--accent-green-bg:     rgba(15, 169, 104, 0.14);          /* Fond de statut */
--accent-green-border: rgba(15, 169, 104, 0.34);          /* Bordure de statut */
--accent-green-text:   #34D399;                           /* Texte de statut (Contraste 6.1:1 sur --bg-surface) */
--accent-green-solid:  #0B8552;                           /* Bouton action primaire */

/* 2. Bleu Info / Télémesure & RCON */
--accent-blue:         #3B82F6;
--accent-blue-bg:      rgba(59, 130, 246, 0.12);
--accent-blue-border:  rgba(59, 130, 246, 0.30);
--accent-blue-text:    #60A5FA;                           /* Texte (Contraste 5.32:1 sur --accent-blue-bg) */
--accent-blue-solid:   #2563EB;                           /* Bouton actif */

/* 3. Ambre Avertissement / Hibernation / Charge CPU Haute */
--accent-orange:        #D97706;
--accent-orange-bg:     rgba(217, 119, 6, 0.14);
--accent-orange-border: rgba(217, 119, 6, 0.34);
--accent-orange-text:   #FBBF24;                          /* Texte (Contraste 5.51:1) */
--accent-orange-solid:  #B45309;

/* 4. Rouge Danger / Crash / Action Destructive */
--danger:               #EF4444;
--danger-bg:            rgba(239, 68, 68, 0.12);
--danger-border:        rgba(239, 68, 68, 0.30);
--danger-text:          #F87171;                          /* Texte d'erreur (Contraste 5.05:1 sur fond teinté) */
--danger-solid:         #DC2626;                          /* Bouton Stop/Kill avec texte blanc (4.83:1) */
```

---

## 3. Architecture Spatiale & Matérialité « Double-Bezel » / Machined Hardware

### 3.1 Philosophie Matérielle : « Milled Console & Dark Obsidian »
L'interface est conçue comme une **pièce de matériel physique de haute précision** (console de monitoring usinée en aluminium anodisé sombre).

* **Refraction de Bordure 1px :** Aucune bordure grise standard. Toutes les séparations utilisent des lignes capillaires translucides (`1px solid rgba(255, 255, 255, 0.07)`).
* **Lumière Spéculaire Intérieure :** Chaque surface noble intègre un biseau spéculaire supérieur (`box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.09)`), simulant l'arête d'un panneau d'instrumentation physique recevant la lumière zénithale.

### 3.2 Anatomie du « Double-Bezel » (Doppelrand)
Toutes les cartes majeures, modales et consoles de logs adoptent une structure concentrique à double enveloppe :

```
┌────────────────────────────────────────────────────────────┐  <- Outer Shell (p-1.5, rounded-2xl, border 1px subtle)
│  ┌──────────────────────────────────────────────────────┐  │
│  │                                                      │  │  <- Inner Core (rounded-[calc(1rem-6px)], inset highlight)
│  │   Contenu Télémétrique / Action / Console            │  │
│  │                                                      │  │
│  └──────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────┘
```

#### Spécification CSS
```css
/* Outer Shell */
.hardware-shell {
  background-color: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 1rem; /* 16px */
  padding: 0.375rem;   /* 6px */
}

/* Inner Core */
.hardware-core {
  background-color: #161922;
  border-radius: calc(1rem - 0.375rem); /* 10px - Rayon Concentrique Parfait */
  border: 1px solid rgba(255, 255, 255, 0.04);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 4px 20px rgba(0, 0, 0, 0.25);
  padding: 1.25rem;
}
```

---

## 4. Bento Grid sans Vide & Télémétrie Vivante (Truth in Telemetry)

### 4.1 Bento Grid Homelab « Zero-Gap »
* **Imbrication dense obligatoire :** Utilisation systématique de `grid-auto-flow: dense` (`grid-flow-dense`) et d'un dimensionnement unitaire sur grille CSS.
* **Rythme asymétrique intentionnel :** 1 carte d'état primaire (4 ou 6 cols, 2 rows) entourée de tuiles télémétriques compactes (2 à 3 cols, 1 row) et d'un module console ou flux de joueurs (6 à 8 cols, 2 rows).
* **Collapse Mobile Prédictif :** En dessous de `768px` (`md:`), la grille se désengage vers un flux vertical `grid-cols-1 gap-4` sans décalage horizontal.

### 4.2 Vérité en Télémétrie (« Truth in Telemetry »)
1. **Live Stream (Connecté) :** Métriques réelles pulsées par SSE/WebSocket (TPS exact, RAM allouée vs utilisée, latence ping, état `lazymc`). Les valeurs numériques utilisent impérativement `font-family: var(--font-mono)` avec `tabular-nums` pour éviter tout tressautement visuel lors des rafraîchissements.
2. **Squelettes Fidèles (Chargement / Reconnexion) :** Aucun spinner circulaire opaque au milieu d'une page vide. Les placeholders squelettes reproduisent rigoureusement la géométrie exacte des cartes finales avec une pulsation douce (`opacity: 0.3` -> `0.6`). Layout Shift (CLS) = 0.
3. **États Hors-Ligne & Hibernation Dignifiés :** Si le serveur Minecraft hiberne via `lazymc`, l'interface ne crie pas à l'erreur (pas de rouge d'alerte agressif). Elle affiche une carte ambre chaleureuse « En veille - Prêt au réveil (Proxy 25565 actif) » avec le temps de sommeil et l'action d'éveil instantané.

---

## 5. Design Engineering, Micro-Interactions & Physique Haptique

### 5.1 Unseen details compound (L'effet cumulatif de l'invisible)
Un utilisateur ne remarque jamais consciemment une micro-interaction réussie : il constate simplement que l'interface "répond parfaitement". La qualité perçue découle de l'accumulation de dizaines de micro-corrections invisibles.

### 5.2 Règle d'or : Zéro animation sur les actions clavier fréquentes
- **Actions répétées 100+ fois/jour** (raccourcis clavier, toggle de palette de commande, navigation au clavier, raccourci d'ouverture rapide) : **Aucune animation. Jamais.** L'apparition doit être instantanée (0ms) à l'instar de Raycast.
- **Actions récurrentes (dizaines de fois/jour)** (onglets, boutons d'action rapide, filtres) : Transitions ultra-courtes (100–160ms).
- **Actions ponctuelles (quelques fois/jour)** (modales d'arrêt de serveur, drawers de configuration, toasts) : Transitions soignées (180–250ms).

### 5.3 Physique du clic : `scale(0.97)` sur `:active`
Tout élément interactif (bouton, carte d'action, toggle) doit réagir physiquement à l'enfoncement :

```css
.btn {
  transition: transform 160ms var(--ease-out),
              background-color 150ms var(--ease-out),
              border-color 150ms var(--ease-out),
              opacity 150ms var(--ease-out);
}

.btn:active:not(:disabled) {
  transform: scale(0.97);
}
```

### 5.4 Jamais d'apparition depuis `scale(0)`
Dans le monde physique, aucun objet n'apparaît à partir du néant absolu. 
- Les modales, menus et tooltips doivent démarrer leur entrée à **`scale(0.95)`** avec `opacity: 0`.

### 5.5 Popovers origin-aware vs Modales centrées
- **Popovers, dropdowns et menus contextuels** doivent obligatoirement s'animer depuis leur déclencheur : `transform-origin: top left` (ou variable dynamique du déclencheur).
- **Exception : Les modales**. Une modale n'est pas attachée à un bouton spécifique dans le flux mais au viewport global ; elle conserve donc impérativement `transform-origin: center`.

### 5.6 Tooltips : Warmup & Saut de délai au survol consécutif
- **Premier survol** : Délai d'attente de 300ms pour éviter de polluer la vue lors d'un simple passage de souris.
- **Survols successifs immédiats** : Dès qu'un tooltip est actif, le survol d'éléments voisins dans une barre d'outils doit ouvrir leur tooltip de manière **instantanée (0ms)** sans transition ni délai (`[data-instant]`).

### 5.7 Tokens de Courbes d'Accélération & Optimisation GPU
```css
:root {
  /* Interaction UI vive : décélération nette avec démarrage instantané */
  --ease-out: cubic-bezier(0.23, 1, 0.32, 1);

  /* Mouvement d'éléments traversant l'écran ou morphing de cartes */
  --ease-in-out: cubic-bezier(0.77, 0, 0.175, 1);

  /* Panneaux coulissants & Drawers latéraux (courbe physique iOS) */
  --ease-drawer: cubic-bezier(0.32, 0.72, 0, 1);
}
```
* **GPU-Safe :** Animer uniquement `transform` et `opacity`. Ne jamais animer `height`, `width`, `top`, `margin`, `padding`.
* **Accessibilité :** Respect strict de `prefers-reduced-motion` en neutralisant les déplacements (`transform`) tout en conservant les transitions d'opacité subtiles.

---

## 6. Identité Visuelle ChiPanel & Logo « Hexagone Stencil »

### 6.1 Concept Fusion A : Hexagone Stencil & Signature Matérielle
* **Forme Principale :** Silhouette hexagonale isométrique (angles 30°/60°) évoquant un écrou usiné d'un rack serveur et la géométrie de construction cubique de Minecraft.
* **Découpe Stencil :** Césure centrale asymétrique formant la lettre grecque **$\chi$ (Chi)** stylisée, inspirée du pochoir urbain technique (street art maîtrisé).
* **Finition :** Finition ardoise / titane et vert sauge / lavande feutrée avec un chanfrein gravé de 1px.

### 6.2 Voyant LED Réactif (Telemetry-Linked LED)
Au cœur du logo ou dans le header de navigation, une diode micro-luminescente diffuse l'état systémique global :

| État Serveur / Homelab | Teinte LED | Comportement Lumineux |
| :--- | :--- | :--- |
| **Serveur Actif / Joueurs en ligne** | Émeraude (`#10B981`) | Lueur fixe douce (`box-shadow: 0 0 8px rgba(16,185,129,0.3)`) |
| **Hibernation lazymc** | Ambre Chaud (`#F59E0B`) | Respiration lente (cycle 4s, pulsation sinusoïdale) |
| **Tâche de fond / Transcodage** | Cyan Électrique (`#06B6D4`) | Rotation orbitale discrète |
| **Arrêté / Maintenance** | Ardoise Neutre (`#64748B`) | Fixe mat sans halo lumineux |
| **Incident / Crash RCON** | Rouge Terre (`#EF4444`) | Double clignotement d'avertissement |

### 6.3 Composant Implémenté `ChiPanelLogo.svelte`
Le composant officiel `src/lib/components/common/ChiPanelLogo.svelte` encapsule l'identité vectorielle Stencil et la télémétrie LED réactive :
- **Props :** `status` ('active' | 'hibernating' | 'busy' | 'stopped' | 'crash', défaut 'active'), `size` (nombre, défaut 32), `showText` (booléen, défaut false).
- **Intégration :** Déployé dans `src/routes/+layout.svelte` et réactif aux flux SSE/WebSocket de `wsStore`.

---

## 7. Typographie, Densité & Mise en Page

### Pairings & Hiérarchie
- **Police UI / Titres / Navigation** : `Inter`, `Geist Sans`, ou `system-ui`.
- **Police Métriques / Logs / Ports / UUIDs** : `JetBrains Mono`, `Geist Mono`, ou monospace.
- **Tabular Nums** : `font-variant-numeric: tabular-nums;` systématique sur toutes les données chiffrées en direct.
- **Longueur de ligne (Measure)** : Plafonnée strictement entre `65ch` et `75ch` (`max-width: 70ch`).
- **Titres équilibrés** : `text-wrap: balance` sur tous les titres `h1`, `h2`, `h3`.
- **Letter-Spacing Display Floor** : Jamais de tracking inférieur à `-0.04em` (cible : `-0.02em` à `-0.03em`).

### Échelle de Z-Index Sémantique
Interdiction formelle des valeurs magiques (`z-index: 9999`). L'échelle est codifiée :
```css
--z-dropdown:       10;
--z-sticky-header:  20;
--z-drawer:         30;
--z-modal-backdrop: 40;
--z-modal:          50;
--z-toast:          60;
--z-tooltip:        70;
```

---

## 8. Tableau de Revue Before / After (Code Svelte / CSS ChiPanel)

| Before | After | Why |
| :--- | :--- | :--- |
| `transition: width 200ms ease;` sur barres de progression | `transform: scaleX(calc(var(--pct) / 100)); transform-origin: left; transition: transform 160ms var(--ease-out);` | Élimine les reflows/layouts CPU et garantit un rendu 60fps GPU-safe sans layout shifts. |
| `border-left: 3px/4px solid #color;` (side-tab) sur cartes/toasts | `border: 1px solid var(--border);` avec `border-color: var(--accent-*-border)` ou pastille sémantique | Élimine l'anti-pattern IA des side-tabs asymétriques au profit d'une structure matérielle homogène. |
| `transition: all 200ms ease;` | `transition: transform 160ms var(--ease-out), opacity 160ms var(--ease-out);` | Bannir `all` pour éviter d'animer accidentellement layout/paint et garantir 60fps constants. |
| `.btn:active { transform: translateY(1px); }` | `.btn:active { transform: scale(0.97); }` | `scale(0.97)` offre un retour haptique naturel englobant tout le bouton plutôt qu'un saut vertical 1D. |
| `@keyframes modalSlideUp { from { opacity: 0; transform: translateY(12px) scale(0.98); } }` | `modal { opacity: 1; transform: translateY(0) scale(1); transition: transform 200ms var(--ease-out), opacity 200ms var(--ease-out); } @starting-style { modal { opacity: 0; transform: translateY(8px) scale(0.95); } }` | Élimine les keyframes non interruptibles au profit d'une transition native interruptible partant de `scale(0.95)`. |
| `ease-out` natif CSS (`cubic-bezier(0, 0, 0.2, 1)`) | `var(--ease-out)` (`cubic-bezier(0.23, 1, 0.32, 1)`) | La courbe custom démarre avec un punch instantané et décélère avec élégance sans traîner. |
| `transform-origin: center` sur menu dropdown/popover | `transform-origin: top left` (ou variable d'ancrage trigger) | Les popovers doivent se déployer depuis leur bouton d'appel pour respecter la cohérence spatiale. |
| Animation d'ouverture 200ms sur la recherche globale (`Ctrl+K`) | Affichage instantané `0ms` (aucune animation de transition) | Les actions déclenchées au clavier 100+ fois par jour ne doivent subir aucune latence d'animation. |
| Délai fixe de 300ms sur tous les tooltips | Délai initial de 300ms + ouverture instantanée (0ms) sur les tooltips adjacents | Évite les faux positifs initiaux tout en fluidifiant le survol d'une rangée d'icônes d'action. |
| Animation `@keyframes btn-spin 1.2s linear infinite` | `@keyframes btn-spin 0.6s linear infinite` | Un spinner deux fois plus rapide améliore la perception psychologique de la vitesse réseau/serveur. |
| `transform: scale(0)` lors de l'apparition d'un badge de statut | `transform: scale(0.9)` combiné avec `opacity: 0` | Rien n'apparaît du néant absolu ; débuter à 0.9 crée une entrée naturelle et fluide. |

---

## 9. Architecture Onboarding Débutant & Navigation Dual-Mode

### 9.1 Paradigme "Novice 1-Click" vs "Power User Expert"
L'interface de ChiPanel s'adapte dynamiquement au profil de l'administrateur homelab :
- **Mode Novice (1-Click Setup) :**
  - Workflow linéaire en 4 étapes (`GameSelectorStep`, `EngineSelectorStep`, `RamAllocationStep`, `LaunchReviewStep`).
  - Abstraction totale des concepts de conteneurs, Quadlets, et variables d'environnement.
  - Recommandation intelligente d'allocation mémoire basée sur la RAM hôte détectée (`/proc/meminfo` ou sonde IPC Tauri).
  - Activation par défaut de la veille `lazymc` (0 Mo au repos).
- **Mode Power User (Expert) :**
  - Contrôle d'ingénierie complet : Quadlet `.container` éditable, terminal RCON direct, cgroups Linux et flags JVM avancés.
  - Bascule instantanée (0ms) via raccourci `Alt+M` ou switch tactile double-bezel.

### 9.2 Composants & Matérialité de l'Onboarding
1. **Conteneur Hardware Multi-Étapes :** Utilisation de l'enveloppe `.hardware-shell` (p-1.5, bordure 1px subtile) et du cœur `.hardware-core` (`#161922`, chanfrein zénithal 1px).
2. **Jauge d'Allocation Mémoire :** Visualisation segmentée (OS Reserve, Heap Minecraft, Headroom) avec indicateur de saturation au-delà de 75% de la RAM hôte (`transform: scaleX(...)` GPU-safe).
3. **Micro-Interactions Tactiles :** Boutons d'action et tuiles de sélection dotés de `active:scale-[0.97]` et courbes d'accélération `--ease-out`.

