<!-- KpiCard.svelte -->
<script>
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ label: string, children?: Snippet }} */
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
