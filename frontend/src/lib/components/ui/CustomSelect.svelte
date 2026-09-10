<script>
	import { ChevronDown } from 'lucide-svelte';
	/** @type {{ value: string, options: { value: string, label: string }[], label?: string, disabled?: boolean, onchange?: (v: string) => void }} */
	let { value = $bindable(''), options, label = '', disabled = false, onchange } = $props();
</script>

{#if label}<span class="input-label">{label}</span>{/if}
<div class="select-wrapper">
	<select class="select-control" bind:value {disabled} onchange={() => onchange?.(value)}>
		{#each options as opt (opt.value)}
			<option value={opt.value}>{opt.label}</option>
		{/each}
	</select>
	<ChevronDown size={14} class="select-arrow" />
</div>

<style>
	.input-label { font-size: var(--font-size-sm); font-weight: 500; color: var(--text-secondary); }
	.select-wrapper { position: relative; display: flex; align-items: center; width: 100%; }
	.select-control {
		width: 100%;
		padding: 0.55rem 2rem 0.55rem 0.875rem;
		background: var(--bg-inset);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
		font-weight: 500;
		outline: none;
		appearance: none;
		cursor: pointer;
	}
	.select-control:focus { border-color: var(--accent-indigo); }
	.select-arrow { position: absolute; right: 0.75rem; color: var(--text-muted); pointer-events: none; }
</style>
