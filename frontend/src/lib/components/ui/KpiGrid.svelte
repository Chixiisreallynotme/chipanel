<!-- KpiGrid.svelte -->
<script>
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ children?: Snippet }} */
	let { children } = $props();
</script>

<div class="kpi-grid">{@render children()}</div>

<style>
	.kpi-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 0.75rem; }
	@media (max-width: 960px) { .kpi-grid { grid-template-columns: 1fr; } }
</style>
