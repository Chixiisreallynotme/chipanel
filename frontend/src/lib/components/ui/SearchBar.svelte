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
