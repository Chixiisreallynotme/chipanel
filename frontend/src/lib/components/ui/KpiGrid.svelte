<script>
	/** @import { Snippet } from 'svelte'; */
	/**
	 * The outer machined frame around a row of KpiCard panels. `framed` is on by
	 * default so every route gets the /engine nested-panel treatment; set it to
	 * false only when the grid already sits inside another bezel.
	 * @type {{ children?: Snippet, framed?: boolean }}
	 */
	let { children, framed = true } = $props();
</script>

<div class="kpi-grid {framed ? 'bezel-frame' : ''}">{@render children()}</div>

<style>
	.kpi-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-4);
	}
</style>
