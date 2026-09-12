<script>
	import { onMount } from 'svelte';
	import { apiFetch, apiGet, apiPost } from '$lib/api/client.js';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import {
		Network,
		Smartphone,
		Gamepad2,
		ShieldCheck,
		Key,
		CheckCircle2,
		AlertCircle,
		AlertTriangle,
		RefreshCw,
		Copy,
		ExternalLink,
		Sliders,
		Check
	} from '$lib/icons.js';

	/** @type {{ is_geyser_installed: boolean, is_floodgate_installed: boolean, geyser_jar_name: string|null, floodgate_jar_name: string|null, bedrock_port: number, auth_type: string, has_encryption_key: boolean, is_running: boolean, recommended_engine: string } | null} */
	let geyserStatus = $state(null);
	let loading = $state(true);
	let error = $state('');

	// Setup form state
	let bedrockPort = $state(19132);
	let authType = $state('floodgate');
	let isSavingConfig = $state(false);
	let setupResult = $state(null);
	let copiedField = $state('');

	onMount(() => {
		loadGeyserStatus();
	});

	async function loadGeyserStatus() {
		loading = true;
		error = '';
		try {
			const res = await apiGet('/api/geyser/status');
			geyserStatus = res;
			if (res) {
				bedrockPort = res.bedrock_port || 19132;
				authType = res.auth_type || 'floodgate';
			}
		} catch (err) {
			error = err.message || 'Impossible de charger le statut Geyser';
		} finally {
			loading = false;
		}
	}

	async function handleSetupGeyser() {
		isSavingConfig = true;
		setupResult = null;
		try {
			const res = await apiPost('/api/geyser/setup', {
				bedrock_port: Number(bedrockPort),
				auth_type: authType
			});
			setupResult = res;
			await loadGeyserStatus();
		} catch (err) {
			error = err.message || 'Erreur lors de la configuration Geyser';
		} finally {
			isSavingConfig = false;
		}
	}

	function copyToClipboard(text, field) {
		navigator.clipboard.writeText(text);
		copiedField = field;
		setTimeout(() => {
			copiedField = '';
		}, 2000);
	}
</script>

<svelte:head>
	<title>Cross-Play Bedrock & Réseau | ChiPanel</title>
</svelte:head>

<div class="network-page-layout">
	<!-- Page Header -->
	<PageHeader
		title="Cross-Play Bedrock & Réseau"
		subtitle="Pont d'interconnexion pour joueurs Minecraft Bedrock (iOS, Android, Windows 10/11, Xbox, Switch, PS4/PS5)"
	>
		{#snippet icon()}
			<Network size={22} />
		{/snippet}
		{#snippet badge()}
			<span class="badge">Geyser + Floodgate</span>
		{/snippet}
		<button class="btn btn-secondary btn-sm" onclick={loadGeyserStatus} disabled={loading}>
			<RefreshCw size={14} class={loading ? 'spin' : ''} />
			<span>Actualiser</span>
		</button>
	</PageHeader>

	{#if error}
		<div class="alert alert-danger card">
			<AlertCircle size={20} />
			<span>{error}</span>
			<button class="btn btn-secondary btn-sm" onclick={loadGeyserStatus}>Réessayer</button>
		</div>
	{/if}

	<!-- Status KPI Grid -->
	<div class="status-grid">
		<div class="kpi-card card">
			<div class="kpi-icon-wrapper {geyserStatus?.is_geyser_installed ? 'kpi-success' : 'kpi-warning'}">
				<Smartphone size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Plugin Geyser</span>
				<span class="kpi-value">
					{#if loading}
						—
					{:else if geyserStatus?.is_geyser_installed}
						Installé ({geyserStatus.geyser_jar_name})
					{:else}
						Non Détecté
					{/if}
				</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper {geyserStatus?.is_floodgate_installed ? 'kpi-success' : 'kpi-info'}">
				<ShieldCheck size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Floodgate (Auth Bedrock)</span>
				<span class="kpi-value">
					{#if loading}
						—
					{:else if geyserStatus?.is_floodgate_installed}
						Installé ({geyserStatus.floodgate_jar_name})
					{:else}
						Non Détecté
					{/if}
				</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper {geyserStatus?.has_encryption_key ? 'kpi-success' : 'kpi-warning'}">
				<Key size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Clé de Chiffrement (key.pem)</span>
				<span class="kpi-value">
					{#if loading}
						—
					{:else if geyserStatus?.has_encryption_key}
						Générée & Active
					{:else}
						En attente de boot
					{/if}
				</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper kpi-primary">
				<Gamepad2 size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Port UDP Bedrock</span>
				<span class="kpi-value font-mono">
					{geyserStatus?.bedrock_port ?? 19132} / UDP
				</span>
			</div>
		</div>
	</div>

	<!-- Two Column Layout: Wizard & Client Connection Card -->
	<div class="columns-grid">
		<!-- Setup Configuration Card -->
		<div class="card config-card">
			<div class="card-header">
				<h2 class="card-title">Assistant de Configuration Cross-Play</h2>
				<span class="card-subtitle">Génération automatique des paramètres optimisés de Geyser</span>
			</div>

			<div class="card-body">
				<div class="form-group">
					<label for="bedrock-port" class="form-label">Port UDP Bedrock (Hôte & Routeur) :</label>
					<input
						id="bedrock-port"
						type="number"
						class="input"
						bind:value={bedrockPort}
						min="1024"
						max="65535"
					/>
					<span class="form-hint">Port standard Bedrock : <code>19132</code> (protocole UDP).</span>
				</div>

				<div class="form-group">
					<label for="auth-mode" class="form-label">Mode d'Authentification Joueurs Bedrock :</label>
					<select id="auth-mode" class="select" bind:value={authType}>
						<option value="floodgate">Floodgate (Recommandé - Sans compte Java obligatoire)</option>
						<option value="online">Online (Compte Microsoft Java requis)</option>
						<option value="offline">Offline / Insecure</option>
					</select>
					<span class="form-hint">
						Avec <strong>Floodgate</strong>, les joueurs Bedrock se connectent avec leur compte Xbox/Microsoft directement, sans devoir posséder le jeu sur PC (Java Edition).
					</span>
				</div>

				<div class="action-row">
					<button class="btn btn-primary" onclick={handleSetupGeyser} disabled={isSavingConfig}>
						{#if isSavingConfig}
							<RefreshCw size={14} class="spin" />
							<span>Génération en cours...</span>
						{:else}
							<Sliders size={14} />
							<span>Générer / Réinitialiser Config Geyser</span>
						{/if}
					</button>
				</div>

				{#if setupResult}
					<div class="result-alert alert-success card">
						<CheckCircle2 size={20} class="text-success flex-shrink-0" />
						<div>
							<strong>{setupResult.message}</strong>
							<ul class="instructions-list">
								{#each setupResult.instructions as step}
									<li>{step}</li>
								{/each}
							</ul>
						</div>
					</div>
				{/if}
			</div>
		</div>

		<!-- Client Connection Guide Card -->
		<div class="card connection-card">
			<div class="card-header">
				<h2 class="card-title">Fiche de Connexion Joueurs Bedrock</h2>
				<span class="card-subtitle">Paramètres à saisir dans le menu Serveurs de Minecraft Bedrock</span>
			</div>

			<div class="card-body">
				<div class="param-box">
					<span class="param-label">Nom du serveur :</span>
					<div class="param-value-row">
						<span class="param-val">ChiServ Minecraft</span>
						<button
							class="btn btn-ghost btn-sm btn-icon"
							onclick={() => copyToClipboard('ChiServ Minecraft', 'name')}
							title="Copier"
						>
							{#if copiedField === 'name'}<Check size={14} class="text-success" />{:else}<Copy size={14} />{/if}
						</button>
					</div>
				</div>

				<div class="param-box">
					<span class="param-label">Adresse du serveur (IP / Domaine) :</span>
					<div class="param-value-row">
						<span class="param-val font-mono">192.168.1.109 / 100.115.204.85</span>
						<button
							class="btn btn-ghost btn-sm btn-icon"
							onclick={() => copyToClipboard('192.168.1.109', 'ip')}
							title="Copier"
						>
							{#if copiedField === 'ip'}<Check size={14} class="text-success" />{:else}<Copy size={14} />{/if}
						</button>
					</div>
				</div>

				<div class="param-box">
					<span class="param-label">Port :</span>
					<div class="param-value-row">
						<span class="param-val font-mono">{bedrockPort}</span>
						<button
							class="btn btn-ghost btn-sm btn-icon"
							onclick={() => copyToClipboard(String(bedrockPort), 'port')}
							title="Copier"
						>
							{#if copiedField === 'port'}<Check size={14} class="text-success" />{:else}<Copy size={14} />{/if}
						</button>
					</div>
				</div>

				<div class="console-note-box">
					<Gamepad2 size={18} class="text-info flex-shrink-0" />
					<div class="note-text">
						<strong>Astuce pour Consoles (Xbox, Switch, PlayStation) :</strong>
						<p>
							Les consoles bloquent la saisie d'IP personnalisées. Utilisez l'application mobile <em>BedrockTogether</em> ou configurez un DNS local (ex: MC Server Connector) pour faire apparaître ChiServ dans la liste des serveurs LAN.
						</p>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	.network-page-layout {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		padding-bottom: var(--space-8);
	}

	/* Status Grid */
	.status-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-4);
	}

	.kpi-card {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-4);
		background-color: var(--bg-surface);
	}

	.kpi-icon-wrapper {
		width: 42px;
		height: 42px;
		border-radius: var(--radius-md);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.kpi-primary {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
	}

	.kpi-success {
		background-color: rgba(16, 185, 129, 0.15);
		color: var(--engine-emerald);
	}

	.kpi-warning {
		background-color: rgba(245, 158, 11, 0.15);
		color: var(--engine-amber);
	}

	.kpi-info {
		background-color: rgba(59, 130, 246, 0.15);
		color: var(--accent-blue-text);
	}

	.kpi-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.kpi-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.kpi-value {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}

	/* Columns Grid */
	.columns-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-6);
	}

	@media (max-width: 900px) {
		.columns-grid {
			grid-template-columns: 1fr;
		}
	}

	.config-card,
	.connection-card {
		background-color: var(--bg-surface);
	}

	.card-header {
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
	}

	.card-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin: 0;
	}

	.card-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.card-body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.form-label {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--text-muted);
	}

	.form-hint {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		line-height: 1.4;
	}

	.action-row {
		display: flex;
		justify-content: flex-end;
		margin-top: var(--space-2);
	}

	.result-alert {
		display: flex;
		gap: var(--space-3);
		padding: var(--space-4);
		background-color: rgba(16, 185, 129, 0.1);
		border: 1px solid rgba(16, 185, 129, 0.3);
		border-radius: var(--radius-sm);
		font-size: var(--font-size-sm);
	}

	.instructions-list {
		margin: var(--space-2) 0 0 0;
		padding-left: var(--space-4);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	/* Param boxes */
	.param-box {
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-card);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.param-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.param-value-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.param-val {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.console-note-box {
		display: flex;
		gap: var(--space-3);
		padding: var(--space-4);
		background-color: rgba(59, 130, 246, 0.1);
		border: 1px solid rgba(59, 130, 246, 0.3);
		border-radius: var(--radius-sm);
	}

	.note-text {
		font-size: var(--font-size-xs);
		color: #93C5FD;
		line-height: 1.4;
	}

	.note-text p {
		margin: 4px 0 0 0;
		color: var(--text-secondary);
	}

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
