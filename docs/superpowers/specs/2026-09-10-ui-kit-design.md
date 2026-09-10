# ChiPanel UI Kit centralise — Design Spec

> Date : 2026-09-10 · Statut : approuvee · Reference visuelle : `frontend/src/routes/engine/+page.svelte` (Moteurs & Versions, figee telle quelle)

## 1. Contexte et probleme

- 15/20 routes ont un `<style>` > 50 lignes ; `engine` seule = 904 lignes, 85 hex hardcodees.
- Toast copie-colle dans 9 pages, chrome modal dans 5, KPI cards dans 5, `z-index` en dur dans 11 fichiers.
- Seul composant partage : `ConfirmDialog.svelte`. Les suites centrales `.btn`/`.badge`/`.card`/`.modal` de `app.css` sont ignorees par les pages.
- Les libs externes Svelte 5 ont ete ecartees : < 500 stars (`tsumikit` 0, `sv5ui` 17), ou incompatibles perf/deps (`shadcn-svelte` exige Tailwind v4 + 4 deps, +~100 Ko, restyle total). Kit maison, 0 dependance nouvelle.

## 2. Decisions figees

1. Approche A : kit `lib/components/ui/` (~12 composants) + migration complete des 20 routes, suppression du CSS duplique (~2500 lignes visees).
2. Palette Moteurs absorbee en tokens `app.css` : `#111827` -> `--bg-surface-2`, `#0b0f19` -> `--bg-inset`, `#4f46e5/#4338ca` -> `--accent-indigo(-hover)`, `#818cf8/#a5b4fc` -> `--accent-indigo-text/bg`. Zero hex en dur dans les pages apres migration (regle dssoca).
3. Z-index rabattu sur l'echelle `design.md` : `--z-dropdown:10`, `--z-sticky:20`, `--z-drawer:30`, `--z-modal-backdrop:40`, `--z-modal:50`, `--z-toast:60`, `--z-tooltip:70`. `app.css` (100/200/1000/1100) et `ConfirmDialog` (`--z-modal + 1`) migres en consequence.
4. Micro-interactions `design.md` inchangees : `scale(0.97)` au `:active`, `< 250 ms`, `transform`/`opacity` uniquement, `prefers-reduced-motion` respecte.

## 3. Kit : composants et API

| Composant | Props cles | Remplace |
|---|---|---|
| `PageHeader.svelte` | `title`, `subtitle`, `icon?`, `actions` (snippet) | headers des 9 routes auditees (h1 + subtitle + boutons) |
| `HeroBanner.svelte` | `badge`, `logo` (snippet), `title`, `meta` (snippet), `kpis` (snippet) | `section.banner-card` engine, hero players/worlds/console |
| `KpiCard.svelte` / `KpiGrid.svelte` | `label`, `value`, `status` (online/hibernating/stopped/na) | `.kpi-card` x5 pages, `.stat-card` players |
| `SearchBar.svelte` | `bind:value`, `placeholder`, `onClear` | `.search-wrap` engine + recherches deleguees |
| `CategoryTabs.svelte` | `options[{id,label,count?}]`, `bind:active` | tabs engine/addons/metrics, presets files |
| `SegmentedControl.svelte` | idem, variante compacte | subview addons, range metrics |
| `CustomSelect.svelte` | `bind:value`, `options`, `label?` | `.select-wrapper` engine (carte + modale) |
| `Callout.svelte` | `tone` (info/success/warning/danger), `title?` | `.config-alert`, `.callout-*`, `.alert-banner`, `.page-alert` |
| `ChecklistCard.svelte` | `title`, `detail`, `badge?`, `toggle?+bind:checked` | `.checklist-card` x4 modale engine |
| `FormModal.svelte` | `open`, `title`, `subtitle?`, `onClose`, `footer` (snippet) | modales a champs (files x3, worlds borderModal, addons updatesModal) |
| `ToastHost.svelte` + `stores/toast.js` | `toast.success/warning/error/info(title, msg)` | `addToast` copie-colle x9 pages (timeout 5 s, comme engine) |
| `EmptyState.svelte` / `LoadingState.svelte` | `icon`, `message`, `action?` (snippet) | `.empty-state`/`.loading-state` engine/worlds |

`ConfirmDialog.svelte` existant conserve ; `EngineLogo`, `VersionPickerButton/Modal` inchanges. Tous les composants : Svelte 5 runes, snippets (pas de slots legacy), `lucide-svelte` uniquement, `tabular-nums` sur valeurs numeriques.

## 4. Ordre de migration (pires dupliques d'abord)

1. Vague 1 — `players` (toast x18, modale quickAction, stat-cards), `worlds` (toast x18, borderModal, `.btn` local), `permissions` (toast x19, `.badge-purple`), `accounts` (toast x10, badge x7).
2. Vague 2 — `engine` (reecrite elle-meme sur le kit = exemple canonique), `addons`, `files`, `backups`, `database`, `audit`.
3. Vague 3 — `dashboard`, `console`, `metrics`, `network`, `setup`, `login`, `modpacks`, `plugins`, `profiles`.
4. Chaque vague : migrer -> `npm run check` (comparer au baseline, erreurs `any` pre-existantes ignorees) -> preview visuelle `podman build` + `chipanel-local:25501` -> commit+push.

## 5. Hors perimetre

- Pas de Tailwind, pas de nouvelle dependance, pas de light mode, pas de refonte `dashboard/` (composants dashboard/ conserves), pas de driver jeu hors Minecraft.

## 6. Risques

- Regression visuelle subtile (indigo `#4f46e5` vs `--accent-blue-solid #2563EB`) -> tokens dedies `--accent-indigo-*`, pas de fusion forcee.
- `--z-modal:50` peut entrer en conflit avec xterm.js/console overlays -> tester `console`/`files` en vague 3 avec overlay ouverts.
- `svelte-check` baseline bruyante -> comparer avant/apres par vague, ne jamais viser zero absolu.
