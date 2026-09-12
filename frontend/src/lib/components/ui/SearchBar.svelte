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
	:global(.search-icon) { position: absolute; left: 1.125rem; color: var(--engine-slate-500); pointer-events: none; }
	.search-input {
		width: 100%;
		padding: 0.75rem 4.5rem 0.75rem 2.85rem;
		background: var(--bg-surface-2);
		border: 1px solid var(--border-engine-08);
		border-radius: var(--radius-card);
		color: var(--engine-slate-50);
		font-size: var(--font-size-base);
		outline: none;
		transition: border-color 0.2s, box-shadow 0.2s;
	}
	.search-input:focus { border-color: var(--engine-indigo); box-shadow: 0 0 0 3px var(--accent-indigo-bg); }
	.clear-search-btn {
		position: absolute; right: 1rem;
		background: transparent; border: none;
		color: var(--engine-slate-400); cursor: pointer;
		font-size: var(--font-size-xs); font-weight: 500;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
	}
	.clear-search-btn:hover {
		color: var(--engine-white);
	}
</style>
