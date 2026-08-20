<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Boxes } from 'lucide-svelte';
	import RedirectState from '$lib/components/ui/RedirectState.svelte';

	onMount(() => {
		goto('/addons?tab=modpacks', { replaceState: true });
	});
</script>

<svelte:head>
	<title>Redirection vers Addons - ChiPanel</title>
</svelte:head>

<RedirectState
	title="Modpacks"
	description="Les modpacks sont centralisés dans le Centre des Addons pour garder les versions et moteurs synchronisés."
	href="/addons?tab=modpacks"
	actionLabel="Ouvrir les modpacks"
>
	{#snippet children()}
		<Boxes size={22} />
	{/snippet}
</RedirectState>
