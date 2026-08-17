<script>
	import {
		Play,
		Moon,
		CheckCircle2,
		Code,
		ShieldCheck,
		Server,
		HardDrive,
		Sliders,
		Terminal,
		ChevronDown,
		ExternalLink,
		RotateCw
	} from 'lucide-svelte';

	let {
		selectedGame = 'JAVA',
		selectedEngine = 'PURPUR',
		selectedVersion = '1.21.4',
		allocatedRamGb = 4,
		lazymcEnabled = $bindable(true),
		eulaAccepted = $bindable(true),
		isLaunching = false,
		launchProgressStep = '',
		onLaunch = () => {}
	} = $props();

	let showQuadletPreview = $state(false);

	let generatedQuadletCode = $derived(`[Unit]
Description=Minecraft Server (${selectedEngine} ${selectedVersion})
After=network-online.target local-fs.target
Wants=network-online.target

[Container]
ContainerName=minecraft-server
Image=docker.io/itzg/minecraft-server:latest
Environment=TYPE=${selectedEngine}
Environment=VERSION=${selectedVersion}
Environment=MEMORY=${allocatedRamGb}G
Environment=EULA=TRUE
Environment=USE_AIKAR_FLAGS=true
Environment=ENABLE_RCON=true
Environment=RCON_PORT=25575
Volume=%h/minecraft/data:/data:Z
PublishPort=127.0.0.1:25566:25565
PublishPort=127.0.0.1:25575:25575
${selectedGame === 'CROSSPLAY' ? 'PublishPort=0.0.0.0:19132:19132/udp\n' : ''}UserNS=keep-id

[Service]
Restart=on-failure
RestartSec=10s
TimeoutStartSec=300

[Install]
WantedBy=default.target`);
</script>

<div class="review-step-container">
	<div class="step-header">
		<h2 class="step-title">4. Vérification et lancement en 1 clic</h2>
		<p class="step-subtitle">
			Tout est prêt. ChiPanel génère automatiquement les unités systemd et le conteneur Podman rootless.
		</p>
	</div>

	<!-- Configuration Summary Card -->
	<div class="summary-card">
		<div class="summary-grid">
			<div class="summary-item">
				<div class="summary-label">
					<Server size={14} class="text-blue" />
					<span>Édition & Jeu</span>
				</div>
				<div class="summary-value">
					{#if selectedGame === 'JAVA'}
						Minecraft Java Edition
					{:else if selectedGame === 'BEDROCK'}
						Minecraft Bedrock Edition
					{:else}
						Cross-Play (Java + Geyser)
					{/if}
				</div>
				<div class="summary-sub font-mono">
					{selectedGame === 'BEDROCK' ? 'Port 19132 UDP' : selectedGame === 'CROSSPLAY' ? 'Ports 25565 + 19132' : 'Port 25565 TCP'}
				</div>
			</div>

			<div class="summary-item">
				<div class="summary-label">
					<Sliders size={14} class="text-blue" />
					<span>Moteur & Version</span>
				</div>
				<div class="summary-value font-mono">
					{selectedEngine} {selectedVersion}
				</div>
				<div class="summary-sub">
					<span class="badge badge-success">Optimisé Aikar Flags</span>
				</div>
			</div>

			<div class="summary-item">
				<div class="summary-label">
					<HardDrive size={14} class="text-blue" />
					<span>Mémoire Dédiée</span>
				</div>
				<div class="summary-value font-mono">
					{allocatedRamGb} Go RAM
				</div>
				<div class="summary-sub">
					Allocated JVM Heap
				</div>
			</div>

			<div class="summary-item">
				<div class="summary-label">
					<Moon size={14} class="text-warning" />
					<span>Économie d'Énergie</span>
				</div>
				<div class="summary-value">
					{lazymcEnabled ? 'Veille lazymc Active' : 'Toujours Allumé'}
				</div>
				<div class="summary-sub">
					{lazymcEnabled ? '0 Mo RAM au repos' : 'Consommation continue'}
				</div>
			</div>
		</div>
	</div>

	<!-- Options Box -->
	<div class="options-container">
		<!-- lazymc Toggle -->
		<label class="option-row">
			<div class="option-info">
				<div class="option-title-group">
					<Moon size={16} class="text-warning" />
					<span class="option-title">Veille automatique intelligente (lazymc)</span>
					<span class="badge badge-success">Recommandé</span>
				</div>
				<p class="option-description">
					Économise 100% de la RAM serveur lorsqu'aucun joueur n'est connecté (après 10 minutes). Le serveur se réveille instantanément dès qu'un ami se connecte.
				</p>
			</div>
			<input type="checkbox" bind:checked={lazymcEnabled} class="checkbox" />
		</label>

		<!-- EULA Agreement -->
		<label class="option-row">
			<div class="option-info">
				<div class="option-title-group">
					<ShieldCheck size={16} class="text-blue" />
					<span class="option-title">Acceptation de l'EULA Minecraft (Mojang)</span>
				</div>
				<p class="option-description">
					En démarrant le serveur, vous acceptez le contrat de licence utilisateur final (EULA) de Mojang / Microsoft.
				</p>
			</div>
			<input type="checkbox" bind:checked={eulaAccepted} class="checkbox" />
		</label>
	</div>

	<!-- Power User Quadlet Preview Accordion -->
	<div class="power-user-preview-card">
		<button
			type="button"
			class="preview-toggle-btn"
			onclick={() => (showQuadletPreview = !showQuadletPreview)}
		>
			<div class="preview-toggle-left">
				<Code size={15} class="text-muted" />
				<span class="preview-title">Aperçu technique Quadlet Podman</span>
				<span class="badge font-mono">minecraft.container</span>
			</div>
			<ChevronDown size={15} class={showQuadletPreview ? 'rotate-180' : ''} />
		</button>

		{#if showQuadletPreview}
			<div class="quadlet-code-block">
				<pre class="font-mono">{generatedQuadletCode}</pre>
			</div>
		{/if}
	</div>

	<!-- Action Footer -->
	<div class="action-footer-container">
		{#if isLaunching}
			<div class="launch-progress-banner">
				<RotateCw size={16} class="spin-icon text-blue" />
				<div class="progress-details">
					<span class="progress-step-text font-mono">{launchProgressStep || 'Initialisation du conteneur...'}</span>
					<span class="progress-sub">Veuillez patienter pendant la génération et le premier démarrage.</span>
				</div>
			</div>
		{/if}

		<button
			type="button"
			class="btn btn-primary btn-lg launch-button"
			disabled={isLaunching || !eulaAccepted}
			onclick={onLaunch}
		>
			{#if isLaunching}
				<RotateCw size={18} class="spin-icon" />
				<span>Création du serveur en cours...</span>
			{:else}
				<Play size={18} />
				<span>Démarrer le serveur en 1 clic</span>
			{/if}
		</button>
	</div>
</div>

<style>
	.review-step-container {
		display: flex;
		flex-direction: column;
		gap: 18px;
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

	.summary-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: 16px;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
	}

	.summary-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
		gap: 14px;
	}

	.summary-item {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.summary-label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.summary-value {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.summary-sub {
		font-size: 11px;
		color: var(--text-muted);
	}

	.options-container {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.option-row {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 14px;
		padding: 14px;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		cursor: pointer;
		user-select: none;
		transition: border-color 150ms ease, background-color 150ms ease;
	}

	.option-row:hover {
		border-color: var(--border-focus);
		background-color: rgba(255, 255, 255, 0.02);
	}

	.option-info {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.option-title-group {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.option-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.option-description {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		line-height: 1.4;
	}

	.power-user-preview-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		overflow: hidden;
	}

	.preview-toggle-btn {
		width: 100%;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 14px;
		background: transparent;
		border: none;
		color: var(--text-secondary);
		cursor: pointer;
		font-size: var(--font-size-xs);
		transition: color 150ms ease, background-color 150ms ease;
	}

	.preview-toggle-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.02);
	}

	.preview-toggle-left {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.preview-title {
		font-weight: var(--font-weight-medium);
	}

	.quadlet-code-block {
		padding: 12px 14px;
		background-color: #0A0C10;
		border-top: 1px solid var(--border);
		overflow-x: auto;
	}

	.quadlet-code-block pre {
		font-size: 11px;
		color: #A5B4FC;
		line-height: 1.5;
		margin: 0;
	}

	.action-footer-container {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-top: 6px;
	}

	.launch-progress-banner {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 12px 16px;
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		border-radius: var(--radius-btn);
	}

	.progress-details {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.progress-step-text {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--accent-blue-text);
	}

	.progress-sub {
		font-size: 11px;
		color: var(--text-secondary);
	}

	.launch-button {
		width: 100%;
		height: 48px;
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		gap: 10px;
		border-radius: var(--radius-btn);
		background-color: var(--accent-green-solid);
		border-color: var(--accent-green-solid);
		box-shadow: 0 4px 16px rgba(15, 169, 104, 0.25);
		transition: transform 160ms var(--ease-out), background-color 150ms ease, box-shadow 150ms ease;
	}

	.launch-button:hover:not(:disabled) {
		background-color: #0D965B;
		border-color: #0D965B;
		box-shadow: 0 6px 20px rgba(15, 169, 104, 0.35);
	}

	.launch-button:active:not(:disabled) {
		transform: scale(0.97);
	}

	.text-blue {
		color: var(--accent-blue-text);
	}

	.text-warning {
		color: var(--accent-orange-text);
	}

	.spin-icon {
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.rotate-180 {
		transform: rotate(180deg);
		transition: transform 150ms ease;
	}
</style>
