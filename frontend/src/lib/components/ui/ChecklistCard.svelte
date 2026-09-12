<script>
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ title: string, detail?: string, badge?: string, checked?: boolean | null, icon?: Snippet }} */
	let { title, detail = '', badge = '', checked = $bindable(null), icon } = $props();
</script>

<div class="checklist-card {checked === true ? 'card-checked' : ''}">
	<div class={checked !== null ? 'checklist-toggle' : 'checklist-row'}>
		{#if checked !== null}
			<input type="checkbox" class="system-checkbox" bind:checked aria-label={title} />
		{/if}
		<span class="checklist-title">{#if icon}{@render icon()}{/if}<span>{title}</span></span>
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
		border: 1px solid var(--border-engine-05);
	}
	.card-checked {
		border-color: var(--accent-indigo-border);
	}
	.checklist-toggle, .checklist-row { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; }
	.checklist-toggle {
		justify-content: flex-start;
	}
	.checklist-title { display: inline-flex; align-items: center; gap: 0.4rem; font-size: var(--font-size-sm); font-weight: 600; color: var(--engine-slate-100); cursor: pointer; }
	.checklist-detail { font-size: var(--font-size-xs); color: var(--engine-slate-500); margin: 0; padding-left: 1.4rem; }
	.tag-auto {
		font-size: 0.625rem; font-weight: 700; text-transform: uppercase;
		background: var(--engine-emerald-bg); color: var(--accent-emerald-text);
		border: 1px solid var(--engine-emerald-border);
		padding: 0.1rem 0.35rem; border-radius: 4px;
	}
	.system-checkbox { width: 16px; height: 16px; accent-color: var(--engine-indigo); cursor: pointer; }
</style>
