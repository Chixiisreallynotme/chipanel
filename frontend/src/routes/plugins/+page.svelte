<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Package } from 'lucide-svelte';
	import RedirectState from '$lib/components/ui/RedirectState.svelte';

	onMount(() => {
		goto('/addons?tab=plugins', { replaceState: true });
	});
</script>

<svelte:head>
	<title>Redirection vers Addons - ChiPanel</title>
</svelte:head>

<RedirectState
	title="Plugins installés"
	description="Les plugins sont désormais gérés depuis le Centre des Addons."
	href="/addons?tab=plugins"
	actionLabel="Ouvrir les plugins"
>
	{#snippet children()}
		<Package size={22} />
	{/snippet}
</RedirectState>
