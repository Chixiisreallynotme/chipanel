<script>
	import { Search } from '$lib/icons.js';
	/**
	 * Canonical search field: icon, clear affordance, optional input callback.
	 * @type {{
	 *   value?: string,
	 *   placeholder?: string,
	 *   label?: string,
	 *   compact?: boolean,
	 *   oninput?: (value: string) => void,
	 *   onclear?: () => void
	 * }}
	 */
	let { value = $bindable(''), placeholder = 'Rechercher…', label = 'Rechercher', compact = false, oninput, onclear } = $props();
</script>

<div class="search-wrap {compact ? 'search-compact' : ''}">
	<Search size={15} class="search-icon" />
	<input
		type="text"
		class="search-input"
		{placeholder}
		aria-label={label}
		bind:value
		oninput={() => oninput?.(value)}
	/>
	{#if value}
		<button type="button" class="clear-search-btn" onclick={() => { value = ''; onclear?.(); oninput?.(''); }} aria-label="Effacer la recherche">
			Effacer
		</button>
	{/if}
</div>

<style>
	.search-wrap {
		position: relative;
		display: flex;
		align-items: center;
		width: 100%;
	}

	.search-wrap :global(.search-icon) {
		position: absolute;
		left: var(--space-4);
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: var(--space-3) 4.5rem var(--space-3) 2.75rem;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		box-shadow: var(--bevel-top);
		color: var(--text-primary);
		font-size: var(--font-size-base);
		outline: none;
		transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
	}

	.search-compact .search-input {
		padding: var(--space-2) var(--space-10) var(--space-2) var(--space-8);
		border-radius: var(--radius-btn);
		font-size: var(--font-size-sm);
	}

	.search-compact :global(.search-icon) { left: var(--space-3); }
	.search-compact .clear-search-btn { right: var(--space-1); }

	.search-input:focus {
		border-color: var(--accent-blue);
		box-shadow: 0 0 0 3px var(--accent-blue-bg);
	}

	.clear-search-btn {
		position: absolute;
		right: var(--space-3);
		background: transparent;
		border: none;
		color: var(--text-secondary);
		cursor: pointer;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		padding: var(--space-1) var(--space-2);
		border-radius: var(--radius-sm);
	}

	.clear-search-btn:hover { color: var(--text-primary); }
</style>
