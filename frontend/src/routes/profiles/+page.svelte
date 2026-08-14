<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		goto('/addons?tab=profiles', { replaceState: true });
	});
</script>

<svelte:head>
	<title>Redirection vers Addons - ChiPanel</title>
</svelte:head>

<div class="redirect-container">
	<p>Redirection vers les Profils dans Addons...</p>
</div>

<style>
	.redirect-container {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 50vh;
		color: var(--text-muted, #94a3b8);
		font-family: var(--font-ui);
	}
</style>
