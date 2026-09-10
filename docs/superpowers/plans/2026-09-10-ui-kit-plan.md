# UI Kit + Vague 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Creer le kit UI central (`frontend/src/lib/components/ui/` + `stores/toast.svelte.js` + tokens `app.css`) et migrer les 4 pires pages dupliquees (players, worlds, permissions, accounts) dessus.

**Architecture:** Composants Svelte 5 runes additifs (aucune page cassee tant qu'elle n'est pas migree), store toast singleton style `preferences.svelte.js`, tokens Moteurs absorbes dans `app.css`, puis migration page par page avec suppression du CSS local.

**Tech Stack:** Svelte 5 runes, SvelteKit 2, `lucide-svelte`, CSS custom properties, `svelte-check`, preview podman `chipanel-local:25501`.

**Spec:** `chipanel/docs/superpowers/specs/2026-09-10-ui-kit-design.md`

## Global Constraints

- Zero dependance nouvelle (pas de Tailwind, pas de lib externe).
- Zero hex en dur dans les pages migrees : que des `var(--*)`.
- Echelle z-index `design.md` : `--z-dropdown:10; --z-sticky:20; --z-drawer:30; --z-modal-backdrop:40; --z-modal:50; --z-toast:60; --z-tooltip:70`.
- Micro-interactions : `scale(0.97)` sur `:active`, < 250 ms, `transform`/`opacity` uniquement, `prefers-reduced-motion` herite de `app.css`.
- `tabular-nums` sur toute valeur numerique affichee.
- Verification par tache : `npm run check` dans `frontend/` (comparer au baseline, erreurs `any` pre-existantes ignorees) + preview podman si visuel touche.
- Commit + push apres chaque tache dans le repo `chipanel/` (jamais de secret).
- Toute prop snippet (`children`, `actions`, `footer`, `visual`, `meta`, `kpis`) DOIT etre typee JSDoc via `@import`, sinon `svelte-check` echoue. Pattern exact en tete de chaque `<script>` concerne :
```js
/** @import { Snippet } from 'svelte'; */
/** @type {{ title: string, children?: Snippet, actions?: Snippet }} */
let { title, children, actions } = $props();
```

---

## File structure

```
frontend/src/
  app.css                                   # MODIFIE T1 : tokens Moteurs + z-index 7 tiers
  routes/+layout.svelte                     # MODIFIE T3 : monte <ToastHost/>
  lib/stores/toast.svelte.js                # CREE T2 : singleton ToastStore
  lib/components/ui/
    ToastHost.svelte                        # CREE T2
    Callout.svelte                          # CREE T4
    PageHeader.svelte / SearchBar.svelte    # CREES T5
    CategoryTabs.svelte / CustomSelect.svelte # CREES T6
    KpiCard.svelte / KpiGrid.svelte / HeroBanner.svelte # CREES T7
    EmptyState.svelte / LoadingState.svelte / ChecklistCard.svelte / FormModal.svelte # CREES T8
  routes/players/+page.svelte               # MIGRE T9
  routes/worlds/+page.svelte                # MIGRE T10
  routes/permissions/+page.svelte           # MIGRE T11
  routes/accounts/+page.svelte              # MIGRE T12
```

Vagues 2-3 (engine, addons, files, backups, database, audit, dashboard, console, metrics, network, setup, login, modpacks, plugins, profiles) = plan de suivi, pas ce plan.

---

### Task 1: Tokens Moteurs + z-index 7 tiers dans app.css

**Files:**
- Modify: `frontend/src/app.css` (`:root`)

**Interfaces:**
- Produces: `--bg-surface-2:#111827`, `--bg-inset:#0b0f19`, `--accent-indigo:#4f46e5`, `--accent-indigo-hover:#4338ca`, `--accent-indigo-text:#818cf8`, `--accent-indigo-bg:rgba(99,102,241,0.15)`, `--accent-indigo-border:rgba(99,102,241,0.3)`, `--accent-emerald-text:#34d399`, `--accent-emerald-bg:rgba(16,185,129,0.15)`, `--accent-emerald-border:rgba(16,185,129,0.3)` + z-index 7 tiers (remplace `--z-sticky:100; --z-dropdown:200; --z-modal:1000; --z-toast:1100`).

- [ ] **Step 1: Ajouter les tokens**

```css
--bg-surface-2: #111827;
--bg-inset: #0b0f19;
--accent-indigo: #4f46e5;
--accent-indigo-hover: #4338ca;
--accent-indigo-text: #818cf8;
--accent-indigo-bg: rgba(99, 102, 241, 0.15);
--accent-indigo-border: rgba(99, 102, 241, 0.3);
--accent-emerald-text: #34d399;
--accent-emerald-bg: rgba(16, 185, 129, 0.15);
--accent-emerald-border: rgba(16, 185, 129, 0.3);
```

- [ ] **Step 2: Remplacer le bloc z-index par**

```css
--z-dropdown: 10;
--z-sticky: 20;
--z-drawer: 30;
--z-modal-backdrop: 40;
--z-modal: 50;
--z-toast: 60;
--z-tooltip: 70;
```

- [ ] **Step 3: Verifier les usages existants**

Run: `grep -rn "z-modal\|z-toast\|z-sticky\|z-dropdown" frontend/src --include="*.svelte" --include="*.css"`
Expected: `+layout.svelte` utilise `var(--z-sticky)` et `var(--z-modal)` (calculs relatifs OK), `ConfirmDialog.svelte` utilise `calc(var(--z-modal) + 1)` (vaut 51, OK, ne pas toucher).

- [ ] **Step 4: Commit**

```bash
git add frontend/src/app.css
git commit -m "chipanel(ui): tokens Moteurs + z-index echelle design.md"
```

---

### Task 2: Store toast + ToastHost

**Files:**
- Create: `frontend/src/lib/stores/toast.svelte.js`
- Create: `frontend/src/lib/components/ui/ToastHost.svelte`

**Interfaces:**
- Consumes: rien (convention classe runes de `preferences.svelte.js`).
- Produces: `toast.success(title, msg)`, `toast.error(title, msg)`, `toast.warning(title, msg)`, `toast.info(title, msg)`, `toast.dismiss(id)`, `toast.items`. `<ToastHost/>` sans props, a monter une fois dans le layout (T3).

- [ ] **Step 1: Creer le store**

```js
/**
 * @typedef {'success' | 'error' | 'warning' | 'info'} ToastType
 * @typedef {{ id: string, type: ToastType, title: string, message: string }} ToastItem
 */
class ToastStore {
	/** @type {ToastItem[]} */
	items = $state([]);

	/** @param {ToastType} type @param {string} title @param {string} message */
	show(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		this.items = [...this.items, { id, type, title, message }];
		setTimeout(() => this.dismiss(id), 5000);
	}

	/** @param {string} title @param {string} message */
	success(title, message) { this.show('success', title, message); }
	/** @param {string} title @param {string} message */
	error(title, message) { this.show('error', title, message); }
	/** @param {string} title @param {string} message */
	warning(title, message) { this.show('warning', title, message); }
	/** @param {string} title @param {string} message */
	info(title, message) { this.show('info', title, message); }

	/** @param {string} id */
	dismiss(id) {
		this.items = this.items.filter((t) => t.id !== id);
	}
}

export const toast = new ToastStore();
```

- [ ] **Step 2: Creer ToastHost.svelte**

```svelte
<script>
	import { toast } from '$lib/stores/toast.svelte.js';
	import { CheckCircle2, AlertCircle, AlertTriangle, Info, X } from 'lucide-svelte';
</script>

{#if toast.items.length > 0}
	<div class="toast-host" role="status" aria-live="polite">
		{#each toast.items as item (item.id)}
			<div class="toast-item toast-{item.type}">
				{#if item.type === 'success'}
					<CheckCircle2 size={16} class="toast-icon-ok" />
				{:else if item.type === 'error'}
					<AlertCircle size={16} class="toast-icon-err" />
				{:else if item.type === 'warning'}
					<AlertTriangle size={16} class="toast-icon-warn" />
				{:else}
					<Info size={16} class="toast-icon-info" />
				{/if}
				<div class="toast-copy">
					<span class="toast-head">{item.title}</span>
					<span class="toast-body">{item.message}</span>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={() => toast.dismiss(item.id)} aria-label="Fermer la notification">
					<X size={14} />
				</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.toast-host {
		position: fixed;
		bottom: 1.5rem;
		right: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		z-index: var(--z-toast);
	}
	.toast-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.875rem 1.25rem;
		border-radius: var(--radius-card);
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		box-shadow: var(--elevation-shadow);
		color: var(--text-primary);
		min-width: 320px;
	}
	.toast-success { border-left: 3px solid var(--accent-green); }
	.toast-error { border-left: 3px solid var(--danger); }
	.toast-warning { border-left: 3px solid var(--warning); }
	.toast-info { border-left: 3px solid var(--accent-blue); }
	.toast-copy { display: flex; flex-direction: column; gap: 0.15rem; }
	.toast-head { font-weight: var(--font-weight-bold); font-size: var(--font-size-sm); }
	.toast-body { font-size: var(--font-size-xs); color: var(--text-secondary); }
	.toast-icon-ok { color: var(--accent-green); }
	.toast-icon-err { color: var(--danger-text); }
	.toast-icon-warn { color: var(--warning); }
	.toast-icon-info { color: var(--accent-blue-text); }
</style>
```

- [ ] **Step 3: Verifier**

Run: `npm run check` dans `frontend/`
Expected: zero nouvelle erreur vs baseline (erreurs `any` pre-existantes inchangees).

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/stores/toast.svelte.js frontend/src/lib/components/ui/ToastHost.svelte
git commit -m "chipanel(ui): store toast + ToastHost central"
```

---

### Task 3: Monter ToastHost dans le layout

**Files:**
- Modify: `frontend/src/routes/+layout.svelte` (import + `<ToastHost />` dans `.main-body`)

**Interfaces:**
- Consumes: `ToastHost` de la Task 2.
- Produces: toasts globaux actifs sur toutes les routes (les pages migrees T9-T12 suppriment leur conteneur local).

- [ ] **Step 1: Ajouter l'import a cote de l'import ModeSwitch**

```svelte
import ToastHost from '$lib/components/ui/ToastHost.svelte';
```

- [ ] **Step 2: Monter apres le slot page**

```svelte
<main class="main-body">
	{@render children()}
	<ToastHost />
</main>
```

- [ ] **Step 3: Verifier visuellement (aucune page n'emet encore vers le store)**

Run: `podman build -t localhost/chipanel:latest -f Containerfile .` puis run `chipanel-local` sur `127.0.0.1:25501` (voir CLAUDE.md), ouvrir http://localhost:25501.
Expected: layout identique, zero erreur console.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/routes/+layout.svelte
git commit -m "chipanel(ui): monte ToastHost global dans le layout"
```

---

### Task 4: Callout.svelte

**Files:**
- Create: `frontend/src/lib/components/ui/Callout.svelte`

**Interfaces:**
- Produces: `<Callout tone="info|success|warning|danger" title="...">detail</Callout>` (`title` optionnel, corps via snippet `children`).

- [ ] **Step 1: Creer le composant**

```svelte
<script>
	import { Info, CheckCircle2, AlertTriangle, AlertCircle } from 'lucide-svelte';
	/** @type {{ tone?: 'info' | 'success' | 'warning' | 'danger', title?: string }} */
	let { tone = 'info', title = '', children } = $props();
</script>

<div class="callout callout-{tone}" role={tone === 'danger' ? 'alert' : 'status'}>
	{#if tone === 'success'}
		<CheckCircle2 size={16} />
	{:else if tone === 'warning'}
		<AlertTriangle size={16} />
	{:else if tone === 'danger'}
		<AlertCircle size={16} />
	{:else}
		<Info size={16} />
	{/if}
	<div>
		{#if title}<strong>{title}</strong>{/if}
		{@render children()}
	</div>
</div>

<style>
	.callout {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 0.875rem 1rem;
		border-radius: 10px;
		font-size: var(--font-size-sm);
	}
	.callout-info { background: var(--accent-blue-bg); border: 1px solid var(--accent-blue-border); color: var(--accent-blue-text); }
	.callout-success { background: var(--accent-green-bg); border: 1px solid var(--accent-green-border); color: var(--accent-green); }
	.callout-warning { background: var(--warning-bg); border: 1px solid var(--warning-border); color: var(--warning); }
	.callout-danger { background: var(--danger-bg); border: 1px solid var(--danger-border); color: var(--danger-text); }
	.callout strong { color: var(--text-primary); }
	.callout p { margin: 0.2rem 0 0 0; }
</style>
```

- [ ] **Step 2: Verifier puis commit**

Run: `npm run check` dans `frontend/`
Expected: zero nouvelle erreur.

```bash
git add frontend/src/lib/components/ui/Callout.svelte
git commit -m "chipanel(ui): composant Callout central"
```

---

### Task 5: PageHeader.svelte + SearchBar.svelte

**Files:**
- Create: `frontend/src/lib/components/ui/PageHeader.svelte`
- Create: `frontend/src/lib/components/ui/SearchBar.svelte`

**Interfaces:**
- Produces: `<PageHeader title subtitle actions={snippet} />`, `<SearchBar bind:value placeholder />`.

- [ ] **Step 1: Creer PageHeader.svelte**

```svelte
<script>
	/** @type {{ title: string, subtitle?: string, actions?: import('svelte').Snippet }} */
	let { title, subtitle = '', actions } = $props();
</script>

<header class="page-header">
	<div class="header-titles">
		<h1>{title}</h1>
		{#if subtitle}<p class="subtitle">{subtitle}</p>{/if}
	</div>
	{#if actions}{@render actions()}{/if}
</header>

<style>
	.page-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: 1rem;
	}
	.header-titles h1 {
		font-size: 1.75rem;
		font-weight: 700;
		letter-spacing: -0.02em;
		margin: 0;
	}
	.subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		margin: 0.25rem 0 0 0;
	}
</style>
```

- [ ] **Step 2: Creer SearchBar.svelte**

```svelte
<script>
	import { Search } from 'lucide-svelte';
	/** @type {{ value: string, placeholder?: string }} */
	let { value = $bindable(''), placeholder = 'Rechercher...' } = $props();
</script>

<div class="search-wrap">
	<Search size={15} class="search-icon" />
	<input type="text" class="search-input" {placeholder} bind:value />
	{#if value}
		<button class="clear-search-btn" onclick={() => (value = '')} aria-label="Effacer la recherche">
			Effacer
		</button>
	{/if}
</div>

<style>
	.search-wrap { position: relative; display: flex; align-items: center; width: 100%; }
	:global(.search-icon) { position: absolute; left: 1.125rem; color: var(--text-muted); pointer-events: none; }
	.search-input {
		width: 100%;
		padding: 0.75rem 4.5rem 0.75rem 2.85rem;
		background: var(--bg-surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
		outline: none;
	}
	.search-input:focus { border-color: var(--accent-indigo); box-shadow: 0 0 0 3px var(--accent-indigo-bg); }
	.clear-search-btn {
		position: absolute; right: 1rem;
		background: transparent; border: none;
		color: var(--text-secondary); cursor: pointer;
		font-size: var(--font-size-xs); font-weight: 500;
	}
</style>
```

- [ ] **Step 3: Verifier puis commit**

Run: `npm run check` dans `frontend/`
Expected: zero nouvelle erreur.

```bash
git add frontend/src/lib/components/ui/PageHeader.svelte frontend/src/lib/components/ui/SearchBar.svelte
git commit -m "chipanel(ui): PageHeader + SearchBar centraux"
```

---

### Task 6: CategoryTabs.svelte + CustomSelect.svelte

**Files:**
- Create: `frontend/src/lib/components/ui/CategoryTabs.svelte`
- Create: `frontend/src/lib/components/ui/CustomSelect.svelte`

**Interfaces:**
- Produces: `<CategoryTabs options={[{id,label,count?}]} bind:active variant="tabs|segmented" />`, `<CustomSelect bind:value options={[{value,label}]} label? disabled? onchange? />`.

- [ ] **Step 1: Creer CategoryTabs.svelte**

```svelte
<script>
	/** @type {{ options: { id: string, label: string, count?: number }[], active: string, variant?: 'tabs' | 'segmented' }} */
	let { options, active = $bindable(''), variant = 'tabs' } = $props();
</script>

<div class={variant === 'segmented' ? 'segmented' : 'category-tabs'} role="tablist">
	{#each options as opt (opt.id)}
		<button
			role="tab"
			aria-selected={active === opt.id}
			class="tab-item {active === opt.id ? 'tab-active' : ''}"
			onclick={() => (active = opt.id)}
		>
			{opt.label}{#if opt.count !== undefined} ({opt.count}){/if}
		</button>
	{/each}
</div>

<style>
	.category-tabs, .segmented {
		display: flex;
		gap: 0.5rem;
		background: var(--bg-surface-2);
		padding: 0.35rem;
		border-radius: var(--radius-card);
		border: 1px solid var(--border-subtle);
		overflow-x: auto;
	}
	.tab-item {
		padding: 0.5rem 1rem;
		font-size: var(--font-size-sm);
		font-weight: 600;
		border-radius: var(--radius-btn);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		white-space: nowrap;
	}
	.tab-item:hover { color: var(--text-primary); background: rgba(255, 255, 255, 0.04); }
	.tab-active { background: var(--accent-indigo) !important; color: #ffffff !important; }
	.tab-item:active:not(:disabled) { transform: scale(0.97); }
</style>
```

- [ ] **Step 2: Creer CustomSelect.svelte**

```svelte
<script>
	import { ChevronDown } from 'lucide-svelte';
	/** @type {{ value: string, options: { value: string, label: string }[], label?: string, disabled?: boolean, onchange?: (v: string) => void }} */
	let { value = $bindable(''), options, label = '', disabled = false, onchange } = $props();
</script>

{#if label}<span class="input-label">{label}</span>{/if}
<div class="select-wrapper">
	<select class="select-control" bind:value {disabled} onchange={() => onchange?.(value)}>
		{#each options as opt (opt.value)}
			<option value={opt.value}>{opt.label}</option>
		{/each}
	</select>
	<ChevronDown size={14} class="select-arrow" />
</div>

<style>
	.input-label { font-size: var(--font-size-sm); font-weight: 500; color: var(--text-secondary); }
	.select-wrapper { position: relative; display: flex; align-items: center; width: 100%; }
	.select-control {
		width: 100%;
		padding: 0.55rem 2rem 0.55rem 0.875rem;
		background: var(--bg-inset);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
		font-weight: 500;
		outline: none;
		appearance: none;
		cursor: pointer;
	}
	.select-control:focus { border-color: var(--accent-indigo); }
	.select-arrow { position: absolute; right: 0.75rem; color: var(--text-muted); pointer-events: none; }
</style>
```

- [ ] **Step 3: Verifier puis commit**

Run: `npm run check` dans `frontend/`
Expected: zero nouvelle erreur.

```bash
git add frontend/src/lib/components/ui/CategoryTabs.svelte frontend/src/lib/components/ui/CustomSelect.svelte
git commit -m "chipanel(ui): CategoryTabs + CustomSelect centraux"
```

---

### Task 7: KpiCard + KpiGrid + HeroBanner

**Files:**
- Create: `frontend/src/lib/components/ui/KpiCard.svelte`
- Create: `frontend/src/lib/components/ui/KpiGrid.svelte`
- Create: `frontend/src/lib/components/ui/HeroBanner.svelte`

**Interfaces:**
- Produces: `<KpiCard label>{valeur}</KpiCard>`, `<KpiGrid>{cards}</KpiGrid>`, `<HeroBanner badge title visual? meta? kpis? />` (logo via snippet `visual`, description/tags via `meta`, tuiles via `kpis`).

- [ ] **Step 1: Creer KpiCard.svelte et KpiGrid.svelte**

```svelte
<!-- KpiCard.svelte -->
<script>
	/** @type {{ label: string }} */
	let { label, children } = $props();
</script>

<div class="kpi-card">
	<span class="kpi-label">{label}</span>
	<div class="kpi-val">{@render children()}</div>
</div>

<style>
	.kpi-card {
		display: flex; flex-direction: column; justify-content: center;
		gap: 0.35rem;
		padding: 0.875rem 1.125rem;
		border-radius: var(--radius-card);
		background: var(--bg-inset);
		border: 1px solid var(--border-subtle);
	}
	.kpi-label { font-size: var(--font-size-xs); color: var(--text-muted); font-weight: 500; }
	.kpi-val {
		display: flex; align-items: center; gap: 0.45rem;
		font-size: 0.9375rem; font-weight: 600; color: var(--text-primary);
		font-variant-numeric: tabular-nums;
	}
</style>
```

```svelte
<!-- KpiGrid.svelte -->
<script>
	let { children } = $props();
</script>

<div class="kpi-grid">{@render children()}</div>

<style>
	.kpi-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 0.75rem; }
	@media (max-width: 960px) { .kpi-grid { grid-template-columns: 1fr; } }
</style>
```

- [ ] **Step 2: Creer HeroBanner.svelte**

```svelte
<script>
	/** @type {{ badge: string, title: string }} */
	let { badge, title, visual, meta, kpis } = $props();
</script>

<section class="banner-card">
	<div class="banner-primary">
		<div class="banner-badge"><span>{badge}</span></div>
		<div class="banner-identity">
			{#if visual}{@render visual()}{/if}
			<div class="identity-info">
				<span class="banner-title">{title}</span>
				{#if meta}{@render meta()}{/if}
			</div>
		</div>
	</div>
	{#if kpis}{@render kpis()}{/if}
</section>

<style>
	.banner-card {
		display: grid;
		grid-template-columns: 1.35fr 1fr;
		gap: 1.5rem;
		padding: 1.75rem;
		border-radius: var(--radius-lg);
		background: var(--bg-surface-2);
		border: 1px solid var(--border);
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
	}
	.banner-primary { display: flex; flex-direction: column; gap: 0.875rem; }
	.banner-badge {
		display: inline-flex; align-items: center; gap: 0.45rem;
		font-size: 0.6875rem; font-weight: 700; letter-spacing: 0.05em;
		color: var(--accent-indigo-text);
	}
	.banner-identity { display: flex; align-items: flex-start; gap: 1.25rem; }
	.identity-info { display: flex; flex-direction: column; gap: 0.5rem; }
	.banner-title { font-size: 1.75rem; font-weight: 800; letter-spacing: -0.02em; }
	@media (max-width: 960px) { .banner-card { grid-template-columns: 1fr; } }
</style>
```

- [ ] **Step 3: Verifier puis commit**

Run: `npm run check` dans `frontend/`
Expected: zero nouvelle erreur.

```bash
git add frontend/src/lib/components/ui/KpiCard.svelte frontend/src/lib/components/ui/KpiGrid.svelte frontend/src/lib/components/ui/HeroBanner.svelte
git commit -m "chipanel(ui): KpiCard/Grid + HeroBanner centraux"
```

---

### Task 8: EmptyState + LoadingState + ChecklistCard + FormModal

**Files:**
- Create: `frontend/src/lib/components/ui/EmptyState.svelte`
- Create: `frontend/src/lib/components/ui/LoadingState.svelte`
- Create: `frontend/src/lib/components/ui/ChecklistCard.svelte`
- Create: `frontend/src/lib/components/ui/FormModal.svelte`

**Interfaces:**
- Produces: `<EmptyState message action? />`, `<LoadingState message? />`, `<ChecklistCard title detail? badge? bind:checked? />` (`checked=null` = pas de toggle), `<FormModal open title subtitle? onClose footer?>{champs}</FormModal>`.

- [ ] **Step 1: Creer EmptyState.svelte et LoadingState.svelte**

```svelte
<!-- EmptyState.svelte -->
<script>
	import { Info } from 'lucide-svelte';
	/** @type {{ message: string }} */
	let { message, children } = $props();
</script>

<div class="state-block">
	<Info size={28} class="state-icon" />
	<p>{message}</p>
	{#if children}{@render children()}{/if}
</div>

<style>
	.state-block {
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		padding: 4rem 1rem; gap: 0.875rem; color: var(--text-muted); text-align: center;
	}
	.state-icon { color: var(--text-muted); }
</style>
```

```svelte
<!-- LoadingState.svelte -->
<script>
	import { RefreshCw } from 'lucide-svelte';
	/** @type {{ message?: string }} */
	let { message = 'Chargement...' } = $props();
</script>

<div class="state-block" role="status">
	<RefreshCw size={28} class="spin-icon" />
	<p>{message}</p>
</div>

<style>
	.state-block {
		display: flex; flex-direction: column; align-items: center; justify-content: center;
		padding: 4rem 1rem; gap: 0.875rem; color: var(--text-muted); text-align: center;
	}
	.spin-icon { animation: spin 1s linear infinite; color: var(--text-primary); }
	@keyframes spin { to { transform: rotate(360deg); } }
</style>
```

- [ ] **Step 2: Creer ChecklistCard.svelte**

```svelte
<script>
	/** @type {{ title: string, detail?: string, badge?: string, checked?: boolean | null }} */
	let { title, detail = '', badge = '', checked = $bindable(null) } = $props();
</script>

<div class="checklist-card">
	<div class="checklist-row">
		{#if checked !== null}
			<input type="checkbox" class="system-checkbox" bind:checked aria-label={title} />
		{/if}
		<span class="checklist-title">{title}</span>
		{#if badge}<span class="tag-auto">{badge}</span>{/if}
	</div>
	{#if detail}<p class="checklist-detail">{detail}</p>{/if}
</div>

<style>
	.checklist-card {
		display: flex; flex-direction: column; gap: 0.2rem;
		padding: 0.75rem 0.875rem;
		border-radius: 10px;
		background: var(--bg-inset);
		border: 1px solid var(--border-subtle);
	}
	.checklist-row { display: flex; align-items: center; gap: 0.5rem; }
	.checklist-title { display: inline-flex; align-items: center; gap: 0.4rem; font-size: var(--font-size-sm); font-weight: 600; }
	.checklist-detail { font-size: var(--font-size-xs); color: var(--text-muted); margin: 0; padding-left: 1.4rem; }
	.tag-auto {
		font-size: 0.625rem; font-weight: 700; text-transform: uppercase;
		background: var(--accent-green-bg); color: var(--accent-green);
		border: 1px solid var(--accent-green-border);
		padding: 0.1rem 0.35rem; border-radius: 4px;
	}
	.system-checkbox { width: 16px; height: 16px; accent-color: var(--accent-indigo); cursor: pointer; }
</style>
```

- [ ] **Step 3: Creer FormModal.svelte**

```svelte
<script>
	/** @type {{ open: boolean, title: string, subtitle?: string, onClose: () => void }} */
	let { open, title, subtitle = '', onClose, children, footer } = $props();
</script>

{#if open}
	<div class="modal-backdrop" onclick={onClose} onkeydown={(e) => e.key === 'Escape' && onClose()} role="button" tabindex="0" aria-label="Fermer">
		<div class="modal-dialog" onclick={(e) => e.stopPropagation()} onkeydown={() => {}} role="dialog" tabindex="-1">
			<header class="modal-header">
				<div>
					<h2>{title}</h2>
					{#if subtitle}<p class="modal-subtitle">{subtitle}</p>{/if}
				</div>
			</header>
			<div class="modal-content">{@render children()}</div>
			{#if footer}<footer class="modal-footer">{@render footer()}</footer>{/if}
		</div>
	</div>
{/if}

<style>
	.modal-backdrop {
		position: fixed; inset: 0;
		background: rgba(0, 0, 0, 0.8);
		backdrop-filter: blur(8px);
		display: flex; align-items: center; justify-content: center;
		z-index: var(--z-modal);
		padding: 1rem;
	}
	.modal-dialog {
		width: 100%; max-width: 600px;
		background: var(--bg-surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		padding: 1.75rem;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
		max-height: 90vh; overflow-y: auto;
	}
	.modal-header h2 { font-size: 1.25rem; font-weight: 700; margin: 0 0 1.25rem 0; }
	.modal-subtitle { font-size: var(--font-size-sm); color: var(--text-secondary); margin: 0.2rem 0 0 0; }
	.modal-content { display: flex; flex-direction: column; gap: 1rem; }
	.modal-footer {
		display: flex; justify-content: flex-end; gap: 0.75rem;
		margin-top: 1.25rem; padding-top: 1rem;
		border-top: 1px solid var(--border-subtle);
	}
</style>
```

- [ ] **Step 4: Verifier puis commit**

Run: `npm run check` dans `frontend/`
Expected: zero nouvelle erreur.

```bash
git add frontend/src/lib/components/ui/EmptyState.svelte frontend/src/lib/components/ui/LoadingState.svelte frontend/src/lib/components/ui/ChecklistCard.svelte frontend/src/lib/components/ui/FormModal.svelte
git commit -m "chipanel(ui): Empty/Loading states + ChecklistCard + FormModal"
```

---

## Table de correspondance toast (Tasks 9-12)

| Ancien appel local | Nouvel appel store |
|---|---|
| `addToast('success', t, m)` | `toast.success(t, m)` |
| `addToast('error', t, m)` | `toast.error(t, m)` |
| `addToast('warning', t, m)` | `toast.warning(t, m)` |
| `addToast('info', t, m)` | `toast.info(t, m)` |

Dans chaque page migree : supprimer `let toasts = $state([])`, `addToast`, `removeToast`, le bloc conteneur local (`.toast-container` / `.toast-stack`), tout le CSS `.toast*` du `<style>`, les imports d'icones devenus inutilises. Ajouter `import { toast } from '$lib/stores/toast.svelte.js';`.

---

### Task 9: Migrer players

**Files:**
- Modify: `frontend/src/routes/players/+page.svelte` (etat toast ~lignes 44-63, conteneur ~458-490, CSS `.toast*` ~677-783)

**Interfaces:**
- Consumes: `toast` (T2), `PageHeader` (T5).
- Produces: zero `addToast(`, zero CSS `.toast*` dans players.

- [ ] **Step 1: Basculer les appels**

Run: `grep -n "addToast(" frontend/src/routes/players/+page.svelte`
Remplacer chaque appel selon la table, supprimer `toasts`/`addToast`/`removeToast`, supprimer le bloc `{#if toasts.length > 0}<div class="toast-container">...</div>{/if}`, supprimer tout le CSS `.toast*`.

- [ ] **Step 2: Remplacer le header route par PageHeader (titre + actions), seulement si le header actuel n'a pas de logique specifique**

- [ ] **Step 3: Verifier**

Run: `npm run check` dans `frontend/` + `grep -n "addToast\|toast-container\|toast-stack" frontend/src/routes/players/+page.svelte`
Expected: zero match grep, zero nouvelle erreur check.

- [ ] **Step 4: Preview + commit**

Preview podman `chipanel-local:25501`, page /players : declencher une action emettant un toast (ex. kick sans backend = toast error via store global).

```bash
git add frontend/src/routes/players/+page.svelte
git commit -m "chipanel(ui): migre players sur kit (toast store + PageHeader)"
```

---

### Task 10: Migrer worlds

**Files:**
- Modify: `frontend/src/routes/worlds/+page.svelte` (toast x18 via `.toast-stack`, `.btn` local x1)

**Interfaces:**
- Consumes: `toast` (T2), `FormModal` ou `ConfirmDialog` pour `borderModal`, `EmptyState` (T8) pour les 2 `.empty-state-card`.
- Produces: zero `addToast(`, zero CSS `.toast*`, zero `.btn` local dans worlds.

- [ ] **Step 1: Basculer les toasts selon la table + supprimer CSS `.toast*`**

- [ ] **Step 2: Migrer `borderModal` : champs = FormModal, simple confirmation = ConfirmDialog existant**

- [ ] **Step 3: Remplacer les 2 `.empty-state-card.card` par `<EmptyState>`**

- [ ] **Step 4: Verifier**

Run: `npm run check` + `grep -n "addToast\|toast-stack" frontend/src/routes/worlds/+page.svelte`
Expected: zero match, zero nouvelle erreur.

- [ ] **Step 5: Preview + commit**

```bash
git add frontend/src/routes/worlds/+page.svelte
git commit -m "chipanel(ui): migre worlds sur kit (toast store + FormModal + EmptyState)"
```

---

### Task 11: Migrer permissions

**Files:**
- Modify: `frontend/src/routes/permissions/+page.svelte` (toast x19, `.badge-purple` x1)

**Interfaces:**
- Consumes: `toast` (T2), `.badge-blue` central de `app.css` pour remplacer `.badge-purple` (meme role informatif).
- Produces: zero `addToast(`, zero CSS `.toast*`, zero `.badge-purple`.

- [ ] **Step 1: Basculer les toasts selon la table + supprimer CSS `.toast*`**

- [ ] **Step 2: Remplacer `badge-purple` par `badge badge-blue` central, supprimer sa regle CSS**

- [ ] **Step 3: Verifier**

Run: `npm run check` + `grep -n "addToast\|toast-stack\|badge-purple" frontend/src/routes/permissions/+page.svelte`
Expected: zero match, zero nouvelle erreur.

- [ ] **Step 4: Preview + commit**

```bash
git add frontend/src/routes/permissions/+page.svelte
git commit -m "chipanel(ui): migre permissions sur kit (toast store + badge central)"
```

---

### Task 12: Migrer accounts

**Files:**
- Modify: `frontend/src/routes/accounts/+page.svelte` (toast x10, badges x7, `.modal-lg` x1)

**Interfaces:**
- Consumes: `toast` (T2), `FormModal` (T8) pour `.modal-lg` (largeur via `style="max-width:720px"` inline sur l'usage, pas de variante kit), badges centraux `app.css`.
- Produces: zero `addToast(`, zero CSS `.toast*`/`.modal*`/badges ad-hoc dans accounts.

- [ ] **Step 1: Basculer les toasts selon la table + supprimer CSS `.toast*`**

- [ ] **Step 2: Migrer `.modal-lg` vers FormModal (max-width 720px inline) et les 7 variantes badge vers les classes centrales**

- [ ] **Step 3: Verifier**

Run: `npm run check` + `grep -n "addToast\|toast-container\|modal-lg" frontend/src/routes/accounts/+page.svelte`
Expected: zero match, zero nouvelle erreur.

- [ ] **Step 4: Preview + commit + push de la vague**

```bash
git add frontend/src/routes/accounts/+page.svelte
git commit -m "chipanel(ui): migre accounts sur kit (toast store + FormModal + badges)"
git push
```

---

## Amendement A1 — Composants canoniques pre-existants (session Design Council parallele)

Une session parallele a deja centralise : `PageHeader.svelte` (`{title, subtitle, icon, badge, children}`, 15 pages), `EmptyState.svelte` (`{title, description, compact, card, dashed, class, icon, action}`), `Spinner.svelte` (`{size, class, label}`), `Modal.svelte` (shell a11y : `{open, onclose, class, backdropClass, role, ariaLabelledBy, initialFocus, closeOnBackdrop}` + snippet `content`, focus-trap + Escape integres). Regle : reutiliser, ne pas dupliquer.

- **T5 reduite a SearchBar uniquement.** Ne PAS creer `PageHeader.svelte`. Message de commit : `chipanel(ui): SearchBar central (PageHeader existant conserve)`. Les migrations utilisent le PageHeader existant.
- **EmptyState/LoadingState du plan ABANDONNES.** Utiliser `EmptyState` et `Spinner` existants.
- **FormModal devient un wrapper fin de `Modal.svelte`** (titre + actions standardises, zero duplication du shell a11y). Code exact :

```svelte
<script>
	import Modal from '$lib/components/ui/Modal.svelte';
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ open: boolean, title: string, subtitle?: string, onClose: () => void, children?: Snippet, footer?: Snippet }} */
	let { open, title, subtitle = '', onClose, children, footer } = $props();
</script>

<Modal {open} onclose={onClose} class="fm-panel">
	{#snippet content()}
		<header class="fm-header">
			<h2>{title}</h2>
			{#if subtitle}<p class="fm-subtitle">{subtitle}</p>{/if}
		</header>
		<div class="fm-content">
			{#if children}{@render children()}{/if}
		</div>
		{#if footer}<footer class="fm-footer">{@render footer()}</footer>{/if}
	{/snippet}
</Modal>

<style>
	.fm-panel {
		width: 100%; max-width: 600px;
		background: var(--bg-surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		padding: 1.75rem;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
		max-height: 90vh; overflow-y: auto;
	}
	.fm-header h2 { font-size: 1.25rem; font-weight: 700; margin: 0 0 1.25rem 0; }
	.fm-subtitle { font-size: var(--font-size-sm); color: var(--text-secondary); margin: 0.2rem 0 0 0; }
	.fm-content { display: flex; flex-direction: column; gap: 1rem; }
	.fm-footer {
		display: flex; justify-content: flex-end; gap: 0.75rem;
		margin-top: 1.25rem; padding-top: 1rem;
		border-top: 1px solid var(--border-subtle);
	}
</style>
```

- **T8 reduite a ChecklistCard (code inchange) + FormModal wrapper ci-dessus.** Commit : `chipanel(ui): ChecklistCard + FormModal (wrapper Modal)`.
- **T10 step 3** : mapper vers `EmptyState` existant (`title`/`description`/`action`), pas le composant du plan.
