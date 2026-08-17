<script>
	import { Monitor, Gamepad2, Layers, CheckCircle2, ShieldCheck, Zap } from 'lucide-svelte';

	let { selectedGame = $bindable('JAVA') } = $props();

	const gameOptions = [
		{
			id: 'JAVA',
			title: 'Minecraft Java Edition',
			subtitle: 'PC (Windows, macOS, Linux)',
			description: 'La référence historique du jeu. Compatible avec tous les plugins et mods (Paper, Purpur, Fabric, Forge).',
			badge: 'Port 25565 TCP',
			icon: Monitor,
			recommended: true,
			highlights: ['Plugins & Mods', 'Hautes performances', 'Mises à jour rapides']
		},
		{
			id: 'BEDROCK',
			title: 'Minecraft Bedrock Edition',
			subtitle: 'Consoles & Mobiles (Xbox, PS, Switch, iOS, Android)',
			description: 'Version optimisée pour le jeu multiplateforme natif sur consoles et appareils tactiles.',
			badge: 'Port 19132 UDP',
			icon: Gamepad2,
			recommended: false,
			highlights: ['Cross-device natif', 'Intégration manette', 'Faible latence réseau']
		},
		{
			id: 'CROSSPLAY',
			title: 'Passerelle Cross-Play Hybride',
			subtitle: 'Java Edition + Passerelle Geyser & Floodgate',
			description: 'Permet aux joueurs PC (Java) et Consoles/Mobiles (Bedrock) de rejoindre ensemble le même serveur.',
			badge: 'Ports 25565 + 19132',
			icon: Layers,
			recommended: false,
			highlights: ['Joueurs PC + Consoles', 'Moteur Paper optimisé', 'Authentification unifiée']
		}
	];
</script>

<div class="game-step-container">
	<div class="step-header">
		<h2 class="step-title">1. Choisissez votre édition de jeu</h2>
		<p class="step-subtitle">
			Sélectionnez l'écosystème cible. ChiPanel configure automatiquement les ports d'écoute et les protocoles réseau correspondants.
		</p>
	</div>

	<div class="game-cards-grid">
		{#each gameOptions as option (option.id)}
			<button
				type="button"
				class="game-card"
				class:selected={selectedGame === option.id}
				onclick={() => (selectedGame = option.id)}
				aria-pressed={selectedGame === option.id}
			>
				<div class="card-top-row">
					<div class="icon-avatar" class:icon-selected={selectedGame === option.id}>
						<option.icon size={22} />
					</div>

					<div class="badges-row">
						{#if option.recommended}
							<span class="badge badge-success">Recommandé</span>
						{/if}
						<span class="badge font-mono">{option.badge}</span>
					</div>
				</div>

				<div class="card-content">
					<div class="card-title-group">
						<h3 class="card-title">{option.title}</h3>
						<span class="card-subtitle-text">{option.subtitle}</span>
					</div>
					<p class="card-description">{option.description}</p>
				</div>

				<div class="highlights-list">
					{#each option.highlights as item}
						<div class="highlight-item">
							<CheckCircle2 size={12} class="highlight-icon" />
							<span>{item}</span>
						</div>
					{/each}
				</div>

				<div class="selection-indicator">
					{#if selectedGame === option.id}
						<div class="radio-circle selected">
							<div class="radio-inner"></div>
						</div>
						<span class="select-label active">Sélectionné</span>
					{:else}
						<div class="radio-circle"></div>
						<span class="select-label">Cliquer pour choisir</span>
					{/if}
				</div>
			</button>
		{/each}
	</div>
</div>

<style>
	.game-step-container {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.step-header {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.step-title {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		letter-spacing: -0.015em;
	}

	.step-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		line-height: var(--line-height-normal);
		max-width: 65ch;
	}

	.game-cards-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: 14px;
	}

	.game-card {
		display: flex;
		flex-direction: column;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: 16px;
		text-align: left;
		cursor: pointer;
		position: relative;
		user-select: none;
		transition: transform 160ms var(--ease-out),
					border-color 150ms var(--ease-out),
					background-color 150ms var(--ease-out),
					box-shadow 150ms var(--ease-out);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04), 0 4px 16px rgba(0, 0, 0, 0.2);
	}

	.game-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-2px);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 8px 24px rgba(0, 0, 0, 0.35);
	}

	.game-card:active {
		transform: scale(0.97);
	}

	.game-card.selected {
		border-color: var(--accent-blue);
		background-color: #171B26;
		box-shadow: inset 0 1px 0 rgba(59, 130, 246, 0.2), 0 0 0 1px var(--accent-blue), 0 8px 24px rgba(0, 0, 0, 0.4);
	}

	.card-top-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 14px;
	}

	.icon-avatar {
		width: 42px;
		height: 42px;
		border-radius: var(--radius-btn);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-secondary);
		transition: color 150ms ease, background-color 150ms ease;
	}

	.icon-avatar.icon-selected {
		background-color: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
		color: var(--accent-blue-text);
	}

	.badges-row {
		display: flex;
		align-items: center;
		gap: 6px;
		flex-wrap: wrap;
	}

	.card-content {
		display: flex;
		flex-direction: column;
		gap: 6px;
		flex: 1;
		margin-bottom: 16px;
	}

	.card-title-group {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.card-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.card-subtitle-text {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.card-description {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		line-height: 1.45;
		margin-top: 4px;
	}

	.highlights-list {
		display: flex;
		flex-direction: column;
		gap: 5px;
		padding: 10px;
		background-color: rgba(0, 0, 0, 0.2);
		border-radius: var(--radius-input);
		border: 1px solid var(--border-subtle);
		margin-bottom: 16px;
	}

	.highlight-item {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: var(--text-secondary);
	}

	.highlight-icon {
		color: var(--accent-green);
		flex-shrink: 0;
	}

	.selection-indicator {
		display: flex;
		align-items: center;
		gap: 8px;
		padding-top: 10px;
		border-top: 1px solid var(--border-subtle);
	}

	.radio-circle {
		width: 16px;
		height: 16px;
		border-radius: 50%;
		border: 1px solid var(--border-focus);
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--bg-base);
		transition: border-color 150ms ease;
	}

	.radio-circle.selected {
		border-color: var(--accent-blue);
		background-color: var(--accent-blue-solid);
	}

	.radio-inner {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background-color: #FFFFFF;
	}

	.select-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.select-label.active {
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-semibold);
	}
</style>
