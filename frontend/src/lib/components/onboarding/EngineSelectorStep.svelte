<script>
	import { Zap, Cpu, Box, Sparkles, Sliders, ChevronDown, Check, Info } from 'lucide-svelte';

	let {
		selectedEngine = $bindable('PURPUR'),
		selectedVersion = $bindable('LATEST'),
		selectedTemplate = $bindable('SURVIVAL_OPTIMIZED')
	} = $props();

	let showCustomVersion = $state(false);

	const standardVersions = [
		{ id: 'LATEST', label: 'Dernière Version Stable (1.21.4)', tag: 'Recommandé' },
		{ id: '1.21.4', label: '1.21.4 (Tricky Trials Update)' },
		{ id: '1.21.1', label: '1.21.1 (Patch correctif)' },
		{ id: '1.20.4', label: '1.20.4 (Trails & Tales)' },
		{ id: '1.20.1', label: '1.20.1 (Idéal gros modpacks)' },
		{ id: '1.19.4', label: '1.19.4 (The Wild Update)' },
		{ id: '1.18.2', label: '1.18.2 (Caves & Cliffs II)' }
	];

	const enginePresets = [
		{
			id: 'PURPUR',
			name: 'Purpur / PaperMC',
			category: 'Performance & Plugins',
			description: 'Moteur ultra-optimisé garantissant 20 TPS stables avec 0 saccade. Compatible avec tous les plugins Bukkit/Spigot/Paper.',
			ramRequirement: '2 à 4 Go',
			badge: 'Recommandé Production',
			badgeType: 'badge-success',
			icon: Zap,
			pros: ['Anti-lag agressif & async chunks', 'Support 100% plugins', 'Outils Spark & Chunky inclus'],
			modsSupport: false,
			pluginsSupport: true
		},
		{
			id: 'FABRIC',
			name: 'Fabric',
			category: 'Moddé Moderne',
			description: 'Chargeur de mods nouvelle génération, extrêmement léger et performant. Idéal pour les mods de gameplay et d’optimisation (Lithium, FerriteCore).',
			ramRequirement: '3 à 6 Go',
			badge: 'Mods Légers & Rapides',
			badgeType: 'badge-blue',
			icon: Cpu,
			pros: ['Mises à jour instantanées', 'Empreinte CPU/RAM minimale', 'Compatible Lithium & Sodium'],
			modsSupport: true,
			pluginsSupport: false
		},
		{
			id: 'PAPER',
			name: 'PaperMC Standard',
			category: 'Standard Industrie',
			description: 'La référence éprouvée de l’écosystème Minecraft. Corrige les exploits Vanilla et offre une excellente stabilité réseau.',
			ramRequirement: '2 à 4 Go',
			badge: 'Standard Robuste',
			badgeType: 'badge',
			icon: Sliders,
			pros: ['Fiabilité éprouvée 10+ ans', 'Écosystème plugins géant', 'Sécurité anti-duplication'],
			modsSupport: false,
			pluginsSupport: true
		},
		{
			id: 'VANILLA',
			name: 'Vanilla Officiel Mojang',
			category: 'Jeu Pur',
			description: 'Le serveur officiel distribué par Mojang sans aucune altération de code. Recommandé pour tester les snapshots et mécaniques pures.',
			ramRequirement: '2 à 4 Go',
			badge: '100% Officiel',
			badgeType: 'badge',
			icon: Box,
			pros: ['Mécaniques Redstone exactes', 'Accès direct aux snapshots', 'Comportement 100% Vanilla'],
			modsSupport: false,
			pluginsSupport: false
		}
	];

	const itzgTemplates = [
		{
			id: 'SURVIVAL_OPTIMIZED',
			name: 'Survie Homelab Optimisée',
			description: 'Purpur 1.21.4 pré-configuré avec Spark (diagnostic), Chunky (pré-génération) et LuckPerms.',
			engine: 'PURPUR'
		},
		{
			id: 'MODDED_PERF',
			name: 'Pack Moddé Performance',
			description: 'Fabric 1.21.4 avec Fabric API, Lithium, FerriteCore et FastSuite prêts à l’emploi.',
			engine: 'FABRIC'
		},
		{
			id: 'LOW_RAM_ECO',
			name: 'Profil Éco Haswell / Low-RAM',
			description: 'Paramétrage Aikar Flags optimisé pour CPU 2C/4T et mémoire contrainte à 2-3 Go.',
			engine: 'PURPUR'
		}
	];

	function applyTemplate(tpl) {
		selectedTemplate = tpl.id;
		selectedEngine = tpl.engine;
	}
</script>

<div class="engine-step-container">
	<div class="step-header">
		<h2 class="step-title">2. Choisissez le moteur du serveur</h2>
		<p class="step-subtitle">
			Le moteur détermine les performances, la consommation de mémoire et la compatibilité avec les plugins ou mods.
		</p>
	</div>

	<!-- Curated Presets Bar -->
	<div class="presets-section">
		<div class="presets-header">
			<Sparkles size={14} class="preset-header-icon" />
			<span class="presets-label">Modèles Pré-configurés 1-Click (Templates)</span>
		</div>

		<div class="templates-grid">
			{#each itzgTemplates as tpl (tpl.id)}
				<button
					type="button"
					class="template-card"
					class:active={selectedTemplate === tpl.id}
					onclick={() => applyTemplate(tpl)}
				>
					<div class="template-top">
						<span class="template-name">{tpl.name}</span>
						{#if selectedTemplate === tpl.id}
							<Check size={14} class="text-green" />
						{/if}
					</div>
					<p class="template-desc">{tpl.description}</p>
				</button>
			{/each}
		</div>
	</div>

	<!-- Main Engine Cards -->
	<div class="engine-cards-grid">
		{#each enginePresets as engine (engine.id)}
			<button
				type="button"
				class="engine-card"
				class:selected={selectedEngine === engine.id}
				onclick={() => {
					selectedEngine = engine.id;
					selectedTemplate = 'CUSTOM';
				}}
				aria-pressed={selectedEngine === engine.id}
			>
				<div class="engine-top-row">
					<div class="engine-icon-avatar" class:icon-selected={selectedEngine === engine.id}>
						<engine.icon size={20} />
					</div>

					<div class="badges-group">
						<span class="badge {engine.badgeType}">{engine.badge}</span>
						<span class="badge font-mono">RAM : {engine.ramRequirement}</span>
					</div>
				</div>

				<div class="engine-content">
					<div class="engine-heading">
						<h3 class="engine-title">{engine.name}</h3>
						<span class="engine-category">{engine.category}</span>
					</div>
					<p class="engine-description">{engine.description}</p>
				</div>

				<div class="engine-pros-list">
					{#each engine.pros as pro}
						<div class="pro-item">
							<span class="pro-dot"></span>
							<span>{pro}</span>
						</div>
					{/each}
				</div>

				<div class="engine-selection-footer">
					<div class="radio-pill" class:selected={selectedEngine === engine.id}>
						{#if selectedEngine === engine.id}
							<Check size={12} class="text-white" />
							<span>Moteur Actif</span>
						{:else}
							<span>Choisir ce moteur</span>
						{/if}
					</div>
				</div>
			</button>
		{/each}
	</div>

	<!-- Version Selector Dropdown -->
	<div class="version-selector-card">
		<div class="version-card-header">
			<div class="version-meta">
				<span class="version-label">Version de Minecraft :</span>
				<span class="version-selected-badge font-mono">
					{standardVersions.find((v) => v.id === selectedVersion)?.label || selectedVersion}
				</span>
			</div>

			<button
				type="button"
				class="btn btn-secondary btn-sm"
				onclick={() => (showCustomVersion = !showCustomVersion)}
			>
				<span>{showCustomVersion ? 'Masquer les versions' : 'Changer de version'}</span>
				<ChevronDown size={14} class={showCustomVersion ? 'rotate-180' : ''} />
			</button>
		</div>

		{#if showCustomVersion}
			<div class="version-options-list">
				{#each standardVersions as v (v.id)}
					<button
						type="button"
						class="version-option-btn"
						class:active={selectedVersion === v.id}
						onclick={() => {
							selectedVersion = v.id;
							showCustomVersion = false;
						}}
					>
						<span class="font-mono">{v.label}</span>
						{#if v.tag}
							<span class="badge badge-success">{v.tag}</span>
						{/if}
						{#if selectedVersion === v.id}
							<Check size={14} class="text-blue" />
						{/if}
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.engine-step-container {
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

	.presets-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
		background-color: rgba(255, 255, 255, 0.02);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
	}

	.presets-header {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--accent-blue-text);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.preset-header-icon {
		color: var(--accent-blue);
	}

	.templates-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: 8px;
	}

	.template-card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 10px 12px;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		text-align: left;
		cursor: pointer;
		transition: transform 150ms var(--ease-out), border-color 150ms ease, background-color 150ms ease;
	}

	.template-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-1px);
	}

	.template-card:active {
		transform: scale(0.97);
	}

	.template-card.active {
		border-color: var(--accent-blue);
		background-color: rgba(59, 130, 246, 0.08);
	}

	.template-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
	}

	.template-name {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.template-desc {
		font-size: 11px;
		color: var(--text-muted);
		line-height: 1.35;
	}

	.engine-cards-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 12px;
	}

	.engine-card {
		display: flex;
		flex-direction: column;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: 14px;
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

	.engine-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-2px);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 8px 24px rgba(0, 0, 0, 0.35);
	}

	.engine-card:active {
		transform: scale(0.97);
	}

	.engine-card.selected {
		border-color: var(--accent-blue);
		background-color: #171B26;
		box-shadow: inset 0 1px 0 rgba(59, 130, 246, 0.2), 0 0 0 1px var(--accent-blue), 0 8px 24px rgba(0, 0, 0, 0.4);
	}

	.engine-top-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 12px;
	}

	.engine-icon-avatar {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-btn);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-secondary);
	}

	.engine-icon-avatar.icon-selected {
		background-color: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
		color: var(--accent-blue-text);
	}

	.badges-group {
		display: flex;
		align-items: center;
		gap: 4px;
		flex-wrap: wrap;
	}

	.engine-content {
		display: flex;
		flex-direction: column;
		gap: 4px;
		flex: 1;
		margin-bottom: 12px;
	}

	.engine-heading {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.engine-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.engine-category {
		font-size: 11px;
		color: var(--text-muted);
	}

	.engine-description {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		line-height: 1.4;
		margin-top: 2px;
	}

	.engine-pros-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px 10px;
		background-color: rgba(0, 0, 0, 0.2);
		border-radius: var(--radius-input);
		border: 1px solid var(--border-subtle);
		margin-bottom: 12px;
	}

	.pro-item {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--text-secondary);
	}

	.pro-dot {
		width: 4px;
		height: 4px;
		border-radius: 50%;
		background-color: var(--accent-green);
		flex-shrink: 0;
	}

	.engine-selection-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		padding-top: 8px;
		border-top: 1px solid var(--border-subtle);
	}

	.radio-pill {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
		padding: 3px 8px;
		border-radius: var(--radius-sm);
	}

	.radio-pill.selected {
		background-color: var(--accent-blue-solid);
		color: #FFFFFF;
		font-weight: var(--font-weight-semibold);
	}

	.version-selector-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: 12px 16px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.version-card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.version-meta {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.version-label {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		font-weight: var(--font-weight-medium);
	}

	.version-selected-badge {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--accent-blue-text);
		background-color: var(--accent-blue-bg);
		padding: 2px 8px;
		border-radius: var(--radius-input);
		border: 1px solid var(--accent-blue-border);
	}

	.version-options-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding-top: 8px;
		border-top: 1px solid var(--border-subtle);
		max-height: 200px;
		overflow-y: auto;
	}

	.version-option-btn {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		color: var(--text-primary);
		font-size: var(--font-size-xs);
		cursor: pointer;
		transition: background-color 150ms ease, border-color 150ms ease;
	}

	.version-option-btn:hover {
		border-color: var(--border-focus);
		background-color: var(--bg-elevated);
	}

	.version-option-btn.active {
		border-color: var(--accent-blue);
		background-color: rgba(59, 130, 246, 0.1);
	}

	.text-blue {
		color: var(--accent-blue-text);
	}

	.text-green {
		color: var(--accent-green);
	}

	.text-white {
		color: #FFFFFF;
	}

	.rotate-180 {
		transform: rotate(180deg);
		transition: transform 150ms ease;
	}
</style>
