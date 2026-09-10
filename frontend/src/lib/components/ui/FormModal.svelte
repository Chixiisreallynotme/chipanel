<script>
	import Modal from '$lib/components/ui/Modal.svelte';
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ open: boolean, title: string, subtitle?: string, onClose: () => void, children?: Snippet, footer?: Snippet }} */
	let { open, title, subtitle = '', onClose, children, footer } = $props();
</script>

<Modal {open} onclose={onClose} class="fm-panel">
	{#snippet content()}
		<header class="fm-header">
			<h2>{title}</h2>
			{#if subtitle}<p class="fm-subtitle">{subtitle}</p>{/if}
		</header>
		<div class="fm-content">
			{#if children}{@render children()}{/if}
		</div>
		{#if footer}<footer class="fm-footer">{@render footer()}</footer>{/if}
	{/snippet}
</Modal>

<style>
	.fm-panel {
		width: 100%; max-width: 600px;
		background: var(--bg-surface-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		padding: 1.75rem;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
		max-height: 90vh; overflow-y: auto;
	}
	.fm-header h2 { font-size: 1.25rem; font-weight: 700; margin: 0 0 1.25rem 0; }
	.fm-subtitle { font-size: var(--font-size-sm); color: var(--text-secondary); margin: 0.2rem 0 0 0; }
	.fm-content { display: flex; flex-direction: column; gap: 1rem; }
	.fm-footer {
		display: flex; justify-content: flex-end; gap: 0.75rem;
		margin-top: 1.25rem; padding-top: 1rem;
		border-top: 1px solid var(--border-subtle);
	}
</style>
