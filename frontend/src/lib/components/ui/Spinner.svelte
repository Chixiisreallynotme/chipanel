<script>
	import { Loader2 } from '$lib/icons.js';

	let { size = 24, class: klass = '', label = 'Chargement…' } = $props();
</script>

<span class="spinner {klass}" role="status" aria-label={label}>
	<Loader2 size={size} />
</span>

<style>
	.spinner {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
	}
	.spinner :global(svg) {
		animation: spin 1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>