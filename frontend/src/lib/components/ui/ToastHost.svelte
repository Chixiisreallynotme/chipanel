<script>
	import { toast } from '$lib/stores/toast.svelte.js';
	import { CheckCircle2, AlertCircle, AlertTriangle, Info, X } from 'lucide-svelte';
</script>

{#if toast.items.length > 0}
	<div class="toast-host" role="status" aria-live="polite">
		{#each toast.items as item (item.id)}
			<div class="toast-item toast-{item.type}">
				{#if item.type === 'success'}
					<CheckCircle2 size={16} class="toast-icon-ok" />
				{:else if item.type === 'error'}
					<AlertCircle size={16} class="toast-icon-err" />
				{:else if item.type === 'warning'}
					<AlertTriangle size={16} class="toast-icon-warn" />
				{:else}
					<Info size={16} class="toast-icon-info" />
				{/if}
				<div class="toast-copy">
					<span class="toast-head">{item.title}</span>
					<span class="toast-body">{item.message}</span>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={() => toast.dismiss(item.id)} aria-label="Fermer la notification">
					<X size={14} />
				</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.toast-host {
		position: fixed;
		bottom: 1.5rem;
		right: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		z-index: var(--z-toast);
	}
	.toast-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.875rem 1.25rem;
		border-radius: var(--radius-card);
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		box-shadow: var(--elevation-shadow);
		color: var(--text-primary);
		min-width: 320px;
	}
	.toast-success { border-left: 3px solid var(--accent-green); }
	.toast-error { border-left: 3px solid var(--danger); }
	.toast-warning { border-left: 3px solid var(--warning); }
	.toast-info { border-left: 3px solid var(--accent-blue); }
	.toast-copy { display: flex; flex-direction: column; gap: 0.15rem; }
	.toast-head { font-weight: var(--font-weight-bold); font-size: var(--font-size-sm); }
	.toast-body { font-size: var(--font-size-xs); color: var(--text-secondary); }
	.toast-icon-ok { color: var(--accent-green); }
	.toast-icon-err { color: var(--danger-text); }
	.toast-icon-warn { color: var(--warning); }
	.toast-icon-info { color: var(--accent-blue-text); }
</style>
