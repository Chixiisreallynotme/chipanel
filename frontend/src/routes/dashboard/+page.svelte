<script>
	import { onMount } from 'svelte';
	import { apiGet, apiFetch } from '$lib/api/client.js';
	import { serverState } from '$lib/components/dashboard/serverState.js';
	import ServerHeroCard from '$lib/components/dashboard/ServerHeroCard.svelte';
	import TelemetryGauges from '$lib/components/dashboard/TelemetryGauges.svelte';
	import ResourceProgressList from '$lib/components/dashboard/ResourceProgressList.svelte';
	import RecentActivityFeed from '$lib/components/dashboard/RecentActivityFeed.svelte';
	import VersionsCard from '$lib/components/dashboard/VersionsCard.svelte';
	import MetricsChart from '$lib/components/metrics/MetricsChart.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import { Play, Moon, Power, RotateCw, AlertTriangle, X } from 'lucide-svelte';

	const UNKNOWN = 'inconnu';
	const EMPTY_HISTORY = { timestamps: [], tps: [], cpu: [], ram_mb: [], players: [] };

	/** Every status shown on this page derives from here, so none can contradict another (B1). */
	let srv = $derived(serverState());

	/** @type {'off' | 'on' | 'hibernate' | null} */
	let loadingMode = $state(null);
	/** @type {{ title: string, detail: string, serverMessage: string } | null} */
	let modeError = $state(null);

	/** Operator-selected power mode from /api/server/status. */
	let powerMode = $state(null);
	let lazymcActive = $state(false);

	let engineType = $state(null);
	let engineVersion = $state(null);
	let engineError = $state(null);
	let uptimeSeconds = $state(null);
	let metricsHistory = $state(EMPTY_HISTORY);

	let engineLabel = $derived(
		engineType && engineVersion ? `${engineType} ${engineVersion}` : `Moteur ${UNKNOWN}`
	);

	/** Live sub-state: the server is reachable but asleep (lazymc lobby only). */
	let serverAwake = $derived(srv.containerRunning === true);

	/** Short human label for the current live state, shown next to the title. */
	let liveStateLabel = $derived.by(() => {
		if (powerMode === 'off') return 'Hors ligne';
		if (serverAwake) return 'En ligne';
		return powerMode === 'hibernate' ? 'En veille' : 'Endormi';
	});

	async function fetchServerInfo() {
		const [status, engine, history] = await Promise.all([
			apiGet('/api/server/status').catch(() => null),
			apiGet('/api/server/engine').catch(() => null),
			apiGet('/api/metrics/history?range=1h').catch(() => null)
		]);

		// current_type / current_version are null when the quadlet is unreadable —
		// render that as unknown rather than guessing a default engine.
		engineType = engine?.current_type ?? null;
		engineVersion = engine?.current_version ?? null;
		engineError = engine?.config_error ?? null;
		uptimeSeconds = status?.container?.uptime_seconds ?? null;
		metricsHistory = history?.timestamps ? history : EMPTY_HISTORY;
		powerMode = status?.power_mode ?? null;
		lazymcActive = status?.lazymc_active ?? false;
	}

	/** @type {Record<number, string>} */
	const MODE_ERRORS = {
		403: "Accès refusé : cette action est réservée à l'administrateur.",
		500: "Erreur interne du serveur : l'action n'a pas pu être exécutée."
	};

	const MODE_LABELS = {
		off: 'Extinction du serveur échouée',
		on: 'Allumage du serveur échoué',
		hibernate: 'Mise en hibernation échouée'
	};

	let offConfirmOpen = $state(false);

	/** @param {'off' | 'on' | 'hibernate'} mode */
	function requestMode(mode) {
		if (loadingMode) return;
		if (mode === 'off') {
			offConfirmOpen = true;
			return;
		}
		runMode(mode);
	}

	function confirmOff() {
		offConfirmOpen = false;
		runMode('off');
	}

	/** @param {'off' | 'on' | 'hibernate'} mode */
	async function runMode(mode) {
		if (loadingMode) return;

		loadingMode = mode;
		modeError = null;
		try {
			// apiFetch serializes plain objects; the cast only satisfies RequestInit.
			const res = await apiFetch('/api/server/mode', {
				method: 'POST',
				body: /** @type {any} */ ({ action: mode })
			});
			if (!res.ok) {
				const data = await res.json().catch(() => ({}));
				modeError = {
					title: MODE_LABELS[mode],
					detail: MODE_ERRORS[res.status] ?? `La requête a échoué (HTTP ${res.status}).`,
					serverMessage: data?.error || data?.message || ''
				};
				return;
			}
			const data = await res.json().catch(() => ({}));
			powerMode = data?.mode ?? mode;
			lazymcActive = data?.lazymc_active ?? false;
			await fetchServerInfo();
		} catch (err) {
			modeError = {
				title: MODE_LABELS[mode],
				detail: 'Requête impossible — le panneau n’a pas pu joindre le backend.',
				serverMessage: err instanceof Error ? err.message : ''
			};
		} finally {
			loadingMode = null;
		}
	}

	onMount(() => {
		fetchServerInfo();
		const interval = setInterval(fetchServerInfo, 10000);
		return () => clearInterval(interval);
	});
</script>

<svelte:head>
	<title>Vue d'ensemble - ChiServ</title>
</svelte:head>

<ConfirmDialog
	open={offConfirmOpen}
	title="Éteindre le serveur"
	message="Êtes-vous sûr de vouloir éteindre le serveur ? Il ne sera plus joignable tant que vous ne le rallumerez pas."
	confirmLabel="Éteindre"
	onconfirm={confirmOff}
	oncancel={() => (offConfirmOpen = false)}
/>

<div class="dashboard-page">
	<!-- Top Main Header & Controls -->
	<header class="dashboard-top-header">
		<div>
			<h1 class="page-title">Vue d'ensemble</h1>
			<p class="page-subtitle">
				Gérez et surveillez votre serveur Minecraft en temps réel.
				{#if powerMode}
					<span class="mode-badge mode-badge-{powerMode}">{liveStateLabel}</span>
				{/if}
			</p>
		</div>

		<div class="header-actions">
			<!-- Three-mode selector. The active mode is highlighted; the live sub-state
			     (awake/asleep) is shown as a hint next to the label. -->
			<div class="mode-selector" role="group" aria-label="Mode du serveur">
				<button
					class="btn mode-btn mode-off {powerMode === 'off' ? 'mode-active' : ''} {loadingMode === 'off' ? 'btn-loading' : ''}"
					onclick={() => requestMode('off')}
					disabled={loadingMode !== null}
				>
					<Power size={16} />
					<span>Éteint</span>
				</button>

				<button
					class="btn mode-btn mode-on {powerMode === 'on' ? 'mode-active' : ''} {loadingMode === 'on' ? 'btn-loading' : ''}"
					onclick={() => requestMode('on')}
					disabled={loadingMode !== null}
				>
					<Play size={16} />
					<span>Allumé</span>
				</button>

				<button
					class="btn mode-btn mode-hibernate {powerMode === 'hibernate' ? 'mode-active' : ''} {loadingMode === 'hibernate' ? 'btn-loading' : ''}"
					onclick={() => requestMode('hibernate')}
					disabled={loadingMode !== null}
				>
					<Moon size={16} />
					<span>Hibernation</span>
				</button>
			</div>

			<button class="btn btn-secondary" onclick={fetchServerInfo} title="Actualiser la télémétrie">
				<RotateCw size={16} />
				<span>Actualiser</span>
			</button>
		</div>
	</header>

	{#if modeError}
		<div class="page-alert alert-danger" role="alert">
			<AlertTriangle size={18} class="alert-icon" />
			<div class="alert-body">
				<span class="alert-title">{modeError.title}</span>
				<span class="alert-detail">{modeError.detail}</span>
				{#if modeError.serverMessage}
					<span class="alert-server-msg">Réponse du serveur : {modeError.serverMessage}</span>
				{/if}
			</div>
			<button
				class="btn btn-ghost btn-icon btn-sm"
				onclick={() => (modeError = null)}
				aria-label="Masquer l'erreur"
			>
				<X size={14} />
			</button>
		</div>
	{/if}

	{#if engineError}
		<div class="page-alert alert-warning" role="alert">
			<AlertTriangle size={18} class="alert-icon" />
			<div class="alert-body">
				<span class="alert-title">Configuration du moteur illisible</span>
				<span class="alert-detail">{engineError}</span>
			</div>
		</div>
	{/if}

	<!-- 1. Server Hero Banner -->
	<ServerHeroCard serverTitle="Survie" containerId={engineLabel} {uptimeSeconds} />

	<!-- 2. KPI Cards Row (5 Cards - Real Live Metrics) -->
	<TelemetryGauges />

	<!-- 3. Middle Section (3 Grid Columns) -->
	<div class="middle-grid">
		<!-- Left Column: Joueurs en ligne (real history, uPlot) -->
		<MetricsChart
			historyData={metricsHistory}
			title="Joueurs en ligne (historique)"
			chartType="players"
			height={160}
		/>

		<!-- Middle Column: Ressources Progress List -->
		<ResourceProgressList />

		<!-- Right Column: Événements Console RCON -->
		<RecentActivityFeed />
	</div>

	<!-- 4. Bottom Section (3 Grid Columns) -->
	<div class="bottom-grid">
		<MetricsChart
			historyData={metricsHistory}
			title="TPS (stabilité)"
			chartType="tps"
			height={160}
		/>

		<MetricsChart
			historyData={metricsHistory}
			title="Mémoire hôte (Mo)"
			chartType="ram"
			height={160}
		/>

		<!-- Right Column: Versions Card -->
		<VersionsCard {engineType} {engineVersion} />
	</div>
</div>

<style>
	.dashboard-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.dashboard-top-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: var(--space-4);
	}

	.page-title {
		font-size: 1.75rem;
		font-weight: var(--font-weight-bold);
		letter-spacing: -0.02em;
		color: var(--text-primary);
	}

	.page-subtitle {
		color: var(--text-secondary);
		font-size: var(--font-size-sm);
		margin-top: 2px;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.mode-selector {
		display: inline-flex;
		align-items: stretch;
		gap: var(--space-1);
		padding: var(--space-1);
		background: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
	}

	.mode-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		background: transparent;
		color: var(--text-secondary);
		border: 1px solid transparent;
		border-radius: calc(var(--radius-badge) - 2px);
		font-weight: var(--font-weight-semibold);
		transition: all var(--transition-fast);
	}

	.mode-btn:hover:not(:disabled) {
		color: var(--text-primary);
	}

	.mode-active.mode-off {
		background: var(--danger-bg);
		border-color: var(--danger-border);
		color: var(--danger-text);
	}

	.mode-active.mode-on {
		background: var(--accent-green-bg);
		border-color: var(--accent-green-border);
		color: var(--accent-green);
	}

	.mode-active.mode-hibernate {
		background: var(--warning-bg, rgba(245, 158, 11, 0.12));
		border-color: var(--warning-border, rgba(245, 158, 11, 0.3));
		color: var(--warning);
	}

	.mode-badge {
		display: inline-block;
		margin-left: var(--space-2);
		padding: 1px 8px;
		border-radius: var(--radius-badge);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		border: 1px solid transparent;
		vertical-align: middle;
	}

	.mode-badge-off {
		background: var(--danger-bg);
		border-color: var(--danger-border);
		color: var(--danger-text);
	}

	.mode-badge-on {
		background: var(--accent-green-bg);
		border-color: var(--accent-green-border);
		color: var(--accent-green);
	}

	.mode-badge-hibernate {
		background: var(--warning-bg, rgba(245, 158, 11, 0.12));
		border-color: var(--warning-border, rgba(245, 158, 11, 0.3));
		color: var(--warning);
	}

	/* Persistent inline alerts — a failed power action must stay on screen. */
	.page-alert {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-card);
		border: 1px solid transparent;
	}

	.alert-danger {
		background-color: var(--danger-bg);
		border-color: var(--danger-border);
		color: var(--danger-text);
	}

	.alert-warning {
		background-color: var(--warning-bg, rgba(245, 158, 11, 0.12));
		border-color: var(--warning-border, rgba(245, 158, 11, 0.3));
		color: var(--warning);
	}

	.alert-body {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}

	.alert-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
	}

	.alert-detail {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.alert-server-msg {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--text-muted);
		word-break: break-word;
	}

	/* Middle & Bottom Grids */
	.middle-grid,
	.bottom-grid {
		display: grid;
		/* minmax(0, …) so a chart can never widen its own column (A15/B2). */
		grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr) minmax(0, 1fr);
		gap: var(--space-6);
	}

	/* :global(*) because the grid children are components — Svelte cannot scope a
	   bare `> *` onto them, and the rule would be silently dropped. */
	.middle-grid > :global(*),
	.bottom-grid > :global(*) {
		min-width: 0;
	}

	@media (max-width: 1100px) {
		.middle-grid,
		.bottom-grid {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
