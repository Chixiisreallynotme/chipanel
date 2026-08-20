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
		min-height: calc(100dvh - 60px);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-6) var(--space-4);
		box-sizing: border-box;
		overflow-x: hidden;
	}

	.setup-container {
		width: 100%;
		max-width: 920px;
	}

	@media (max-width: 768px) {
		.setup-page-wrapper {
			align-items: flex-start;
			padding: var(--space-4) var(--space-3) var(--space-8);
		}
	}
</style>
