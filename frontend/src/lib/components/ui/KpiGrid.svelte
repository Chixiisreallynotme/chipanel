<script>
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ children?: Snippet }} */
	let { children } = $props();
</script>

<div class="kpi-grid">{@render children()}</div>

<style>
	.kpi-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-4);
	}
</style>
