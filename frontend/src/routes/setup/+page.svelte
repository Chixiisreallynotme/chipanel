<script>
	import OnboardingWizard from '$lib/components/onboarding/OnboardingWizard.svelte';
	import { goto } from '$app/navigation';

	function handleCompleted() {
		goto('/dashboard');
	}
</script>

<svelte:head>
	<title>Assistant de Configuration 1-Click — ChiPanel</title>
</svelte:head>

<div class="setup-page-wrapper">
	<div class="setup-container">
		<OnboardingWizard onCompleted={handleCompleted} />
	</div>
</div>

<style>
	.setup-page-wrapper {
		min-height: calc(100vh - 120px);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 24px 16px;
	}

	.setup-container {
		width: 100%;
		max-width: 920px;
	}
</style>
