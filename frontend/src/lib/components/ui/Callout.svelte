<script>
	import { Info, CheckCircle2, AlertTriangle, AlertCircle } from '$lib/icons.js';
	/** @import { Snippet } from 'svelte'; */
	/**
	 * Inline status message (replaces per-page `.alert` blocks).
	 * `tone` picks icon + semantic colours; `action` hosts a retry/dismiss button.
	 * @type {{
	 *   tone?: 'info' | 'success' | 'warning' | 'danger',
	 *   title?: string,
	 *   action?: Snippet,
	 *   children?: Snippet
	 * }}
	 */
	let { tone = 'info', title = '', action, children } = $props();
</script>

<div class="callout callout-{tone}" role={tone === 'danger' ? 'alert' : 'status'}>
	<span class="callout-icon">
		{#if tone === 'success'}
			<CheckCircle2 size={18} />
		{:else if tone === 'warning'}
			<AlertTriangle size={18} />
		{:else if tone === 'danger'}
			<AlertCircle size={18} />
		{:else}
			<Info size={18} />
		{/if}
	</span>
	<div class="callout-copy">
		{#if title}<strong>{title}</strong>{/if}
		{#if children}<div class="callout-body">{@render children()}</div>{/if}
	</div>
	{#if action}<div class="callout-action">{@render action()}</div>{/if}
</div>

<style>
	.callout {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-card);
		box-shadow: var(--bevel-top);
		font-size: var(--font-size-sm);
	}

	.callout-info { background-color: var(--accent-blue-bg); border: 1px solid var(--accent-blue-border); color: var(--accent-blue-text); }
	.callout-success { background-color: var(--accent-green-bg); border: 1px solid var(--accent-green-border); color: var(--accent-green-text); }
	.callout-warning { background-color: var(--warning-bg); border: 1px solid var(--warning-border); color: var(--warning-text); }
	.callout-danger { background-color: var(--danger-bg); border: 1px solid var(--danger-border); color: var(--danger-text); }

	.callout-icon { display: inline-flex; flex-shrink: 0; margin-top: 1px; }
	.callout-copy { display: flex; flex-direction: column; gap: var(--space-1); min-width: 0; }
	.callout-copy strong { color: var(--text-primary); }
	.callout-body { color: var(--text-secondary); }
	.callout-action { margin-left: auto; flex-shrink: 0; }
</style>
