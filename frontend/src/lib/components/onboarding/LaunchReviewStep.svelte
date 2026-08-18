<script>
	import {
		Play,
		Moon,
		Code,
		ShieldCheck,
		Server,
		HardDrive,
		Sliders,
		ChevronDown,
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
					<span class="icon-wrap-blue"><Server size={14} /></span>
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
				<div class="summary-sub font-mono tabular-nums">
					{selectedGame === 'BEDROCK' ? 'Port 19132 UDP' : selectedGame === 'CROSSPLAY' ? 'Ports 25565 + 19132' : 'Port 25565 TCP'}
				</div>
			</div>

			<div class="summary-item">
				<div class="summary-label">
					<span class="icon-wrap-blue"><Sliders size={14} /></span>
					<span>Moteur & Version</span>
				</div>
				<div class="summary-value font-mono tabular-nums">
					{selectedEngine} {selectedVersion}
				</div>
				<div class="summary-sub">
					<span class="badge badge-success">Optimisé Aikar Flags</span>
				</div>
			</div>

			<div class="summary-item">
				<div class="summary-label">
					<span class="icon-wrap-blue"><HardDrive size={14} /></span>
					<span>Mémoire Dédiée</span>
				</div>
				<div class="summary-value font-mono tabular-nums">
					{allocatedRamGb} Go RAM
				</div>
				<div class="summary-sub">
					Allocated JVM Heap
				</div>
			</div>

			<div class="summary-item">
				<div class="summary-label">
					<span class="icon-wrap-orange"><Moon size={14} /></span>
					<span>Économie d'Énergie</span>
				</div>
				<div class="summary-value">
					{lazymcEnabled ? 'Veille lazymc Active' : 'Toujours Allumé'}
				</div>
				<div class="summary-sub font-mono tabular-nums">
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
					<span class="icon-wrap-orange"><Moon size={16} /></span>
					<span class="option-title">Veille automatique intelligente (lazymc)</span>
					<span class="badge badge-success">Recommandé</span>
				</div>
				<p class="option-description">
					Économise 100% de la RAM serveur lorsqu'aucun joueur n'est connecté (après 10 minutes). Le serveur se réveille instantanément dès qu'un ami se connecte.
				</p>
			</div>
			<input type="checkbox" bind:checked={lazymcEnabled} class="checkbox-input" />
		</label>

		<!-- EULA Agreement -->
		<label class="option-row">
			<div class="option-info">
				<div class="option-title-group">
					<span class="icon-wrap-blue"><ShieldCheck size={16} /></span>
					<span class="option-title">Acceptation de l'EULA Minecraft (Mojang)</span>
				</div>
				<p class="option-description">
					En démarrant le serveur, vous acceptez le contrat de licence utilisateur final (EULA) de Mojang / Microsoft.
				</p>
			</div>
			<input type="checkbox" bind:checked={eulaAccepted} class="checkbox-input" />
		</label>
	</div>

	<!-- Power User Quadlet Preview Accordion -->
	<div class="power-user-preview-card">
		<button
			type="button"
			class="preview-toggle-btn"
			onclick={() => (showQuadletPreview = !showQuadletPreview)}
			aria-expanded={showQuadletPreview}
		>
			<div class="preview-toggle-left">
				<span class="preview-icon"><Code size={15} /></span>
				<span class="preview-title">Aperçu technique Quadlet Podman</span>
				<span class="badge font-mono">minecraft.container</span>
			</div>
			<span class="chevron-wrapper" class:rotated={showQuadletPreview}>
				<ChevronDown size={15} />
			</span>
		</button>

		{#if showQuadletPreview}
			<div class="quadlet-code-block">
				<pre class="font-mono tabular-nums">{generatedQuadletCode}</pre>
			</div>
		{/if}
	</div>

	<!-- Action Footer -->
	<div class="action-footer-container">
		{#if isLaunching}
			<div class="launch-progress-banner">
				<span class="spin-wrapper icon-wrap-blue">
					<RotateCw size={16} />
				</span>
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
				<span class="spin-wrapper">
					<RotateCw size={18} />
				</span>
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
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04), 0 4px 16px rgba(0, 0, 0, 0.2);
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
		transition: transform 160ms var(--ease-out),
					border-color 150ms var(--ease-out),
					background-color 150ms var(--ease-out),
					box-shadow 150ms var(--ease-out);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04), 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.option-row:hover {
		border-color: var(--border-focus);
		background-color: rgba(255, 255, 255, 0.02);
	}

	.option-row:active {
		transform: scale(0.99);
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

	.checkbox-input {
		width: 18px;
		height: 18px;
		accent-color: var(--accent-blue-solid);
		cursor: pointer;
		margin-top: 2px;
		flex-shrink: 0;
	}

	.power-user-preview-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		overflow: hidden;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
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
		transition: color 150ms var(--ease-out), background-color 150ms var(--ease-out);
	}

	.preview-toggle-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.03);
	}

	.preview-toggle-btn:active {
		transform: scale(0.99);
	}

	.preview-toggle-left {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.preview-icon {
		color: var(--text-muted);
	}

	.preview-title {
		font-weight: var(--font-weight-medium);
	}

	.chevron-wrapper {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		transition: transform 160ms var(--ease-out);
	}

	.chevron-wrapper.rotated {
		transform: rotate(180deg);
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
		transition: transform 160ms var(--ease-out), background-color 150ms var(--ease-out), box-shadow 150ms var(--ease-out);
	}

	.launch-button:hover:not(:disabled) {
		background-color: #0D965B;
		border-color: #0D965B;
		box-shadow: 0 6px 20px rgba(15, 169, 104, 0.35);
	}

	.launch-button:active:not(:disabled) {
		transform: scale(0.97);
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

	.spin-wrapper {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.tabular-nums {
		font-variant-numeric: tabular-nums;
	}
</style>

