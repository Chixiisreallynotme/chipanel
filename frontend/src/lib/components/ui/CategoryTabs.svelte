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
		border: 1px solid var(--border-engine-06);
		overflow-x: auto;
	}
	.tab-item {
		padding: 0.5rem 1rem;
		font-size: var(--font-size-xs);
		font-weight: 600;
		border-radius: var(--radius-btn);
		border: none;
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
		white-space: nowrap;
	}
	.tab-item:hover { color: var(--text-primary); background: rgba(255, 255, 255, 0.04); }
	.tab-active { background: var(--accent-blue-solid) !important; color: #ffffff !important; }
	.tab-item:active:not(:disabled) { transform: scale(0.97); }
</style>
