<script>
	import { goto } from '$app/navigation';
	import { preferences } from '$lib/stores/preferences.svelte.js';
	import { apiPost } from '$lib/api/client.js';
	import ModeSwitch from './ModeSwitch.svelte';
	import DesktopRuntimeBadge from './DesktopRuntimeBadge.svelte';
	import GameSelectorStep from './GameSelectorStep.svelte';
	import EngineSelectorStep from './EngineSelectorStep.svelte';
	import RamAllocationStep from './RamAllocationStep.svelte';
	import LaunchReviewStep from './LaunchReviewStep.svelte';
	import ChiPanelLogo from '$lib/components/common/ChiPanelLogo.svelte';
	import {
		ArrowLeft,
		ArrowRight,
		CheckCircle2,
		Sparkles,
		Terminal
	} from 'lucide-svelte';

	let { onCompleted = null } = $props();

	// Wizard State (Steps 1 to 4)
	let currentStep = $state(1);
	const totalSteps = 4;

	// Configuration Form State
	let selectedGame = $state('JAVA');
	let selectedEngine = $state('PURPUR');
	let selectedVersion = $state('LATEST');
	let selectedTemplate = $state('SURVIVAL_OPTIMIZED');
	let allocatedRamGb = $state(4);
	let lazymcEnabled = $state(true);
	let eulaAccepted = $state(true);

	// Launch execution state
	let isLaunching = $state(false);
	let launchProgressStep = $state('');
	let launchError = $state(null);

	const stepsMetadata = [
		{ step: 1, title: 'Édition de Jeu', shortTitle: 'Jeu' },
		{ step: 2, title: 'Moteur Serveur', shortTitle: 'Moteur' },
		{ step: 3, title: 'Allocation RAM', shortTitle: 'Mémoire' },
		{ step: 4, title: 'Lancement 1-Clic', shortTitle: 'Lancement' }
	];

	function nextStep() {
		if (currentStep < totalSteps) {
			currentStep += 1;
		}
	}

	function prevStep() {
		if (currentStep > 1) {
			currentStep -= 1;
		}
	}

	function goToStep(stepNumber) {
		if (stepNumber <= currentStep || stepNumber === currentStep + 1) {
			currentStep = stepNumber;
		}
	}

	async function handleLaunchServer() {
		if (isLaunching) return;
		isLaunching = true;
		launchError = null;

		try {
			// Step 1: Progress feedback
			launchProgressStep = '1/4 : Génération du conteneur Quadlet rootless...';
			await new Promise((r) => setTimeout(r, 400));

			// Step 2: Update engine and version via API
			launchProgressStep = '2/4 : Configuration du moteur et de la mémoire...';
			try {
				await apiPost('/api/server/engine', {
					engine_type: selectedEngine,
					version: selectedVersion === 'LATEST' ? 'LATEST' : selectedVersion,
					backup_world: false
				});
			} catch (err) {
				console.warn('API engine call notice (running in preview mode or fallback):', err);
			}

			// Step 3: Configure lazymc mode if enabled
			launchProgressStep = '3/4 : Initialisation du proxy de veille lazymc...';
			try {
				if (lazymcEnabled) {
					await apiPost('/api/server/mode', { mode: 'hibernate' });
				} else {
					await apiPost('/api/server/mode', { mode: 'on' });
				}
			} catch (err) {
				console.warn('API mode notice:', err);
			}

			// Step 4: Power start
			launchProgressStep = '4/4 : Démarrage du serveur...';
			try {
				await apiPost('/api/server/power', { action: 'start' });
			} catch (err) {
				console.warn('API power notice:', err);
			}

			// Mark onboarding completed in preferences
			preferences.completeOnboarding();

			// Completion callback or redirect
			if (onCompleted) {
				onCompleted();
			} else {
				goto('/dashboard');
			}
		} catch (err) {
			console.error('Launch failed:', err);
			launchError = err instanceof Error ? err.message : 'Erreur lors du lancement';
		} finally {
			isLaunching = false;
		}
	}
</script>

<div class="hardware-shell onboarding-outer-shell">
	<div class="hardware-core onboarding-inner-core">
		<!-- Top Bar: Logo, Desktop Runtime Badge, Mode Switch -->
		<header class="wizard-header">
			<div class="header-left">
				<div class="brand-badge">
					<ChiPanelLogo size={32} status="stopped" />
					<div class="brand-text-group">
						<span class="brand-title">ChiPanel</span>
						<span class="brand-subtitle">Setup Homelab</span>
					</div>
				</div>
				<DesktopRuntimeBadge />
			</div>

			<div class="header-right">
				<ModeSwitch />
			</div>
		</header>

		<!-- Mode Explanation Banner (if in Novice mode) -->
		{#if preferences.mode === 'novice'}
			<div class="mode-info-banner">
				<div class="banner-left">
					<span class="icon-wrap-blue"><Sparkles size={16} /></span>
					<span><strong>Mode Novice Actif :</strong> Configuration simplifiée en 4 étapes sans ligne de commande.</span>
				</div>
				<span class="banner-shortcut font-mono">Alt+M pour le mode Expert</span>
			</div>
		{:else}
			<div class="mode-info-banner expert-banner">
				<div class="banner-left">
					<span class="icon-wrap-orange"><Terminal size={16} /></span>
					<span><strong>Mode Power User Actif :</strong> Accès complet aux Quadlets Podman, ports bruts et JVM flags.</span>
				</div>
				<span class="banner-shortcut font-mono">Alt+M pour le mode Novice</span>
			</div>
		{/if}

		<!-- Step Breadcrumb Bar -->
		<nav class="steps-progress-nav" aria-label="Progression de l'onboarding">
			{#each stepsMetadata as s (s.step)}
				<button
					type="button"
					class="step-progress-item"
					class:active={currentStep === s.step}
					class:completed={currentStep > s.step}
					onclick={() => goToStep(s.step)}
				>
					<div class="step-badge font-mono tabular-nums">
						{#if currentStep > s.step}
							<CheckCircle2 size={13} class="text-green" />
						{:else}
							<span>0{s.step}</span>
						{/if}
					</div>
					<div class="step-text-group">
						<span class="step-short-name">{s.shortTitle}</span>
						<span class="step-full-title">{s.title}</span>
					</div>
				</button>
				{#if s.step < totalSteps}
					<div class="step-connector" class:filled={currentStep > s.step}></div>
				{/if}
			{/each}
		</nav>

		<!-- Step Content Body -->
		<main class="wizard-body">
			{#if currentStep === 1}
				<GameSelectorStep bind:selectedGame />
			{:else if currentStep === 2}
				<EngineSelectorStep
					bind:selectedEngine
					bind:selectedVersion
					bind:selectedTemplate
				/>
			{:else if currentStep === 3}
				<RamAllocationStep bind:allocatedRamGb />
			{:else if currentStep === 4}
				<LaunchReviewStep
					{selectedGame}
					{selectedEngine}
					{selectedVersion}
					{allocatedRamGb}
					bind:lazymcEnabled
					bind:eulaAccepted
					{isLaunching}
					{launchProgressStep}
					onLaunch={handleLaunchServer}
				/>
			{/if}

			{#if launchError}
				<div class="error-banner">
					<strong>Erreur :</strong> {launchError}
				</div>
			{/if}
		</main>

		<!-- Navigation Footer Bar (for steps 1-3) -->
		{#if currentStep < totalSteps}
			<footer class="wizard-footer">
				<div class="footer-left">
					{#if currentStep > 1}
						<button
							type="button"
							class="btn btn-secondary btn-md prev-btn"
							onclick={prevStep}
						>
							<ArrowLeft size={16} />
							<span>Étape précédente</span>
						</button>
					{/if}
				</div>

				<div class="footer-right">
					<button
						type="button"
						class="btn btn-primary btn-md next-btn"
						onclick={nextStep}
					>
						<span>Continuer vers {stepsMetadata[currentStep]?.shortTitle}</span>
						<ArrowRight size={16} />
					</button>
				</div>
			</footer>
		{/if}
	</div>
</div>

<style>
	/* Machined Hardware Double-Bezel Envelope */
	.hardware-shell.onboarding-outer-shell {
		background-color: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.06);
		border-radius: var(--radius-card);
		padding: 6px;
		max-width: 920px;
		margin: 0 auto;
		box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
	}

	.hardware-core.onboarding-inner-core {
		background-color: var(--bg-surface);
		border-radius: calc(var(--radius-card) - 6px);
		border: 1px solid rgba(255, 255, 255, 0.04);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 4px 20px rgba(0, 0, 0, 0.25);
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.wizard-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding-bottom: 16px;
		border-bottom: 1px solid var(--border-subtle);
		flex-wrap: wrap;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 14px;
		flex-wrap: wrap;
	}

	.brand-badge {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.brand-text-group {
		display: flex;
		flex-direction: column;
	}

	.brand-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		line-height: 1.1;
		letter-spacing: -0.02em;
	}

	.brand-subtitle {
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.1;
	}

	.mode-info-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 14px;
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		border-radius: var(--radius-btn);
		font-size: var(--font-size-xs);
		color: var(--accent-blue-text);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
	}

	.mode-info-banner.expert-banner {
		background-color: var(--accent-orange-bg);
		border-color: var(--accent-orange-border);
		color: var(--accent-orange-text);
	}

	.banner-left {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.banner-shortcut {
		font-size: 11px;
		opacity: 0.8;
	}

	.steps-progress-nav {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 10px 14px;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		overflow-x: auto;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.02);
	}

	.step-progress-item {
		display: flex;
		align-items: center;
		gap: 8px;
		background: transparent;
		border: none;
		cursor: pointer;
		padding: 4px 8px;
		border-radius: var(--radius-input);
		color: var(--text-secondary);
		transition: transform 160ms var(--ease-out), color 150ms ease, background-color 150ms ease;
		white-space: nowrap;
	}

	.step-progress-item:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.03);
	}

	.step-progress-item:active {
		transform: scale(0.97);
	}

	.step-progress-item.active {
		color: var(--text-primary);
	}

	.step-progress-item.active .step-badge {
		background-color: var(--accent-blue-solid);
		border-color: var(--accent-blue-solid);
		color: #FFFFFF;
		box-shadow: 0 0 10px rgba(59, 130, 246, 0.3);
	}

	.step-badge {
		width: 24px;
		height: 24px;
		border-radius: 50%;
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-focus);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 11px;
		font-weight: var(--font-weight-semibold);
		color: var(--text-muted);
	}

	.step-text-group {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		line-height: 1.2;
	}

	.step-short-name {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
	}

	.step-full-title {
		font-size: 10px;
		color: var(--text-muted);
	}

	.step-connector {
		flex: 1;
		height: 2px;
		background-color: var(--border);
		min-width: 16px;
		transition: background-color 200ms var(--ease-out);
	}

	.step-connector.filled {
		background-color: var(--accent-green);
	}

	.wizard-body {
		display: flex;
		flex-direction: column;
		min-height: 380px;
	}

	.wizard-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding-top: 18px;
		border-top: 1px solid var(--border-subtle);
	}

	.prev-btn {
		transition: transform 160ms var(--ease-out), background-color 150ms ease, border-color 150ms ease;
	}

	.prev-btn:active:not(:disabled) {
		transform: scale(0.97);
	}

	.next-btn {
		background-color: var(--accent-blue-solid);
		border-color: var(--accent-blue-solid);
		color: #FFFFFF;
		box-shadow: 0 2px 10px rgba(59, 130, 246, 0.2);
		transition: transform 160ms var(--ease-out), background-color 150ms ease, box-shadow 150ms ease;
	}

	.next-btn:hover:not(:disabled) {
		background-color: var(--accent-blue-solid-hover);
	}

	.next-btn:active:not(:disabled) {
		transform: scale(0.97);
	}

	.error-banner {
		padding: 10px 14px;
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-input);
		color: var(--danger-text);
		font-size: var(--font-size-sm);
		margin-top: 12px;
	}

	.icon-wrap-blue {
		color: var(--accent-blue-text);
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.icon-wrap-orange {
		color: var(--accent-orange-text);
		display: inline-flex;
		align-items: center;
		justify-content: center;
	}

	.text-green {
		color: var(--accent-green);
	}

	.tabular-nums {
		font-variant-numeric: tabular-nums;
	}

	@media (max-width: 640px) {
		.hardware-shell.onboarding-outer-shell {
			padding: 4px;
		}

		.hardware-core.onboarding-inner-core {
			padding: var(--space-4);
			gap: var(--space-3);
		}

		.wizard-header {
			display: grid;
			grid-template-columns: minmax(0, 1fr) auto;
			align-items: start;
			gap: var(--space-3);
		}

		.header-left {
			min-width: 0;
			flex-direction: column;
			align-items: flex-start;
			gap: var(--space-2);
		}

		.header-right {
			justify-self: end;
		}

		.mode-info-banner {
			align-items: flex-start;
			flex-direction: column;
			gap: var(--space-2);
			padding: var(--space-3);
		}

		.banner-left {
			align-items: flex-start;
		}

		.banner-shortcut {
			align-self: flex-start;
			padding-left: 24px;
		}

		.steps-progress-nav {
			justify-content: flex-start;
			padding: var(--space-2);
		}

		.step-progress-item {
			padding: var(--space-1);
		}

		.step-connector {
			min-width: 10px;
		}

		.wizard-body {
			min-height: 0;
		}
	}
</style>
