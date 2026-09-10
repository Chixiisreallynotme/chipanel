<script>
	/** @type {{ title: string, detail?: string, badge?: string, checked?: boolean | null }} */
	let { title, detail = '', badge = '', checked = $bindable(null) } = $props();
</script>

<div class="checklist-card">
	<div class="checklist-row">
		{#if checked !== null}
			<input type="checkbox" class="system-checkbox" bind:checked aria-label={title} />
		{/if}
		<span class="checklist-title">{title}</span>
		{#if badge}<span class="tag-auto">{badge}</span>{/if}
	</div>
	{#if detail}<p class="checklist-detail">{detail}</p>{/if}
</div>

<style>
	.checklist-card {
		display: flex; flex-direction: column; gap: 0.2rem;
		padding: 0.75rem 0.875rem;
		border-radius: 10px;
		background: var(--bg-inset);
		border: 1px solid var(--border-subtle);
	}
	.checklist-row { display: flex; align-items: center; gap: 0.5rem; }
	.checklist-title { display: inline-flex; align-items: center; gap: 0.4rem; font-size: var(--font-size-sm); font-weight: 600; }
	.checklist-detail { font-size: var(--font-size-xs); color: var(--text-muted); margin: 0; padding-left: 1.4rem; }
	.tag-auto {
		font-size: 0.625rem; font-weight: 700; text-transform: uppercase;
		background: var(--accent-green-bg); color: var(--accent-green);
		border: 1px solid var(--accent-green-border);
		padding: 0.1rem 0.35rem; border-radius: 4px;
	}
	.system-checkbox { width: 16px; height: 16px; accent-color: var(--accent-indigo); cursor: pointer; }
</style>
