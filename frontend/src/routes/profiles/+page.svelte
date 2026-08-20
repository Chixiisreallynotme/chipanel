<script>
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { HardDrive } from 'lucide-svelte';
	import RedirectState from '$lib/components/ui/RedirectState.svelte';

	onMount(() => {
		goto('/addons?tab=profiles', { replaceState: true });
	});
</script>

<svelte:head>
	<title>Redirection vers Addons - ChiPanel</title>
</svelte:head>

<RedirectState
	title="Profils & presets"
	description="Les profils de configuration sont gérés dans le Centre des Addons, au même endroit que les modpacks."
	href="/addons?tab=profiles"
	actionLabel="Ouvrir les profils"
>
	{#snippet children()}
		<HardDrive size={22} />
	{/snippet}
</RedirectState>
