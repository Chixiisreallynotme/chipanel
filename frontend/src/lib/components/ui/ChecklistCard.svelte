<script>
	/** @import { Snippet } from 'svelte'; */
	/**
	 * Checkable configuration card (onboarding). `checked === null` renders a
	 * non-interactive row.
	 * @type {{ title: string, detail?: string, badge?: string, checked?: boolean | null, icon?: Snippet }}
	 */
	let { title, detail = '', badge = '', checked = $bindable(null), icon } = $props();
</script>

<div class="checklist-card {checked === true ? 'card-checked' : ''}">
	<div class={checked !== null ? 'checklist-toggle' : 'checklist-row'}>
		{#if checked !== null}
			<input type="checkbox" class="system-checkbox" bind:checked aria-label={title} />
		{/if}
		<span class="checklist-title">{#if icon}{@render icon()}{/if}<span>{title}</span></span>
		{#if badge}<span class="micro-stamp micro-stamp-active">{badge}</span>{/if}
	</div>
	{#if detail}<p class="checklist-detail">{detail}</p>{/if}
</div>

<style>
	.checklist-card {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-md);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
	}

	.card-checked { border-color: var(--accent-green-border); }

	.checklist-toggle,
	.checklist-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}

	.checklist-toggle { justify-content: flex-start; }

	.checklist-title {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		cursor: pointer;
	}

	.checklist-detail {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin: 0;
		padding-left: calc(16px + var(--space-2));
	}

	.system-checkbox {
		width: 16px;
		height: 16px;
		accent-color: var(--accent-blue);
		cursor: pointer;
	}
</style>
