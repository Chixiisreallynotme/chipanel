<script>
	import { Info, CheckCircle2, AlertTriangle, AlertCircle } from 'lucide-svelte';
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ tone?: 'info' | 'success' | 'warning' | 'danger', title?: string, children?: Snippet }} */
	let { tone = 'info', title = '', children } = $props();
</script>

<div class="callout callout-{tone}" role={tone === 'danger' ? 'alert' : 'status'}>
	{#if tone === 'success'}
		<CheckCircle2 size={16} />
	{:else if tone === 'warning'}
		<AlertTriangle size={16} />
	{:else if tone === 'danger'}
		<AlertCircle size={16} />
	{:else}
		<Info size={16} />
	{/if}
	<div>
		{#if title}<strong>{title}</strong>{/if}
		{@render children()}
	</div>
</div>

<style>
	.callout {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 0.875rem 1rem;
		border-radius: 10px;
		font-size: var(--font-size-sm);
	}
	.callout-info { background: var(--accent-blue-bg); border: 1px solid var(--accent-blue-border); color: var(--accent-blue-text); }
	.callout-success { background: var(--accent-green-bg); border: 1px solid var(--accent-green-border); color: var(--accent-green); }
	.callout-warning { background: var(--warning-bg); border: 1px solid var(--warning-border); color: var(--warning); }
	.callout-danger { background: var(--danger-bg); border: 1px solid var(--danger-border); color: var(--danger-text); }
	.callout strong { color: var(--text-primary); }
	.callout p { margin: 0.2rem 0 0 0; }
</style>
