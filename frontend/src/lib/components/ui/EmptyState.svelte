<script>
	/**
	 * @typedef {Object} EmptyStateProps
	 * @property {string} [title]
	 * @property {string} [description]
	 * @property {boolean} [compact]
	 * @property {boolean} [card]
	 * @property {boolean} [dashed]
	 * @property {boolean} [framed]
	 */
	let { title = '', description = '', compact = false, card = false, dashed = false, framed = false, class: klass = '', icon, action } = $props();
</script>

<div class="empty-state {card ? 'empty-state-card' : ''} {compact ? 'empty-state-compact' : ''} {dashed ? 'empty-state-dashed' : ''} {framed ? 'empty-state-framed bezel-panel' : ''} {klass}">
	{#if icon}
		<div class="empty-icon-box">
			{@render icon?.()}
		</div>
	{/if}
	{#if title}
		<h3 class="empty-title">{title}</h3>
	{/if}
	{#if description}
		<p class="empty-description">
			{#if typeof description === 'string'}
				{description}
			{:else}
				{@render description?.()}
			{/if}
		</p>
	{/if}
	{#if action}
		<div class="empty-action">
			{@render action?.()}
		</div>
	{/if}
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-12);
		gap: var(--space-3);
		text-align: center;
		color: var(--text-muted);
	}

	/* Page-level empty state: the frame comes from the global .bezel-panel;
	   only the padding and the icon medallion are specific here. */
	.empty-state-framed {
		padding: var(--space-10) var(--space-6);
	}

	.empty-state-framed .empty-icon-box {
		width: 56px;
		height: 56px;
		border-radius: var(--radius-card);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		box-shadow: var(--bevel-top);
	}
	.empty-state-card {
		border: 1px solid var(--border);
		background-color: var(--bg-surface);
		border-radius: var(--radius-card);
		padding: var(--space-16) var(--space-6);
	}
	.empty-state-dashed {
		padding: var(--space-8);
		background-color: var(--bg-base);
		border: 1px dashed var(--border-focus);
		border-radius: var(--radius-card);
		gap: var(--space-2);
	}
	.empty-state-compact {
		padding: var(--space-8) var(--space-4);
		gap: var(--space-2);
	}
	.empty-state-compact .empty-icon-box {
		color: var(--text-muted);
		opacity: 0.5;
		margin-bottom: var(--space-1);
	}
	.empty-state-compact .empty-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-medium);
		color: var(--text-secondary);
	}
	.empty-state-compact .empty-description {
		font-size: var(--font-size-xs);
		margin-top: var(--space-1);
	}
	.empty-icon-box {
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
	}
	.empty-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}
	.empty-description {
		font-size: var(--font-size-sm);
		max-width: 420px;
	}
	.empty-description :global(code) {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--accent-blue-text);
		background: var(--bg-elevated);
		padding: 1px 6px;
		border-radius: var(--radius-input);
		border: 1px solid var(--border-subtle);
	}
	.empty-action {
		margin-top: var(--space-1);
	}
</style>