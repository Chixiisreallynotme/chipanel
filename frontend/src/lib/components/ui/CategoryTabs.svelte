<script>
	/**
	 * Segmented category filter used by catalogue pages.
	 * @type {{
	 *   options: { id: string, label: string, count?: number }[],
	 *   active?: string,
	 *   variant?: 'tabs' | 'segmented'
	 * }}
	 */
	let { options, active = $bindable(''), variant = 'tabs' } = $props();
</script>

<div class={variant === 'segmented' ? 'segmented' : 'category-tabs'} role="tablist">
	{#each options as opt (opt.id)}
		<button
			type="button"
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
	.category-tabs,
	.segmented {
		display: flex;
		gap: var(--space-1);
		background-color: var(--bg-surface);
		padding: var(--space-1);
		border-radius: var(--radius-card);
		border: 1px solid var(--border);
		overflow-x: auto;
	}

	.tab-item {
		padding: var(--space-2) var(--space-4);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		border-radius: var(--radius-btn);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
		white-space: nowrap;
	}

	.tab-item:hover { color: var(--text-primary); background-color: rgba(255, 255, 255, 0.04); }
	.tab-active { background-color: var(--accent-blue-solid); color: var(--text-on-solid); }
	.tab-item:active:not(:disabled) { transform: scale(0.97); }
</style>
