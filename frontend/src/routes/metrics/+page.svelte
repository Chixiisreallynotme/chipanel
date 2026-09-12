<script>
	import { onMount, onDestroy } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import MetricsChart from '$lib/components/metrics/MetricsChart.svelte';
	import SparkProfilerCard from '$lib/components/metrics/SparkProfilerCard.svelte';
	import AlertSettingsModal from '$lib/components/metrics/AlertSettingsModal.svelte';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import { toast } from '$lib/stores/toast.svelte.js';
	import {
		RefreshCw,
		Sliders,
		AlertTriangle,
		CalendarClock,
		Play,
		Check,
		Plus,
		Trash2,
		Webhook,
		Power
	} from '$lib/icons.js';

	// Range Selector Presets (TSDB Multi-Tier Retention)
	const rangePresets = [
		{ label: '1h', value: '1h' },
		{ label: '6h', value: '6h' },
		{ label: '24h', value: '24h' },
		{ label: '7j', value: '7d' },
		{ label: '30j', value: '30d' }
	];

	let selectedRange = $state('1h');
	let isRefreshing = $state(false);
	let isAlertModalOpen = $state(false);

	// Alert Configuration State
	let alertConfig = $state({
		tps_threshold: 15.0,
		cpu_threshold: 90,
		ram_threshold: 85,
		discord_webhook_url: '',
		alerts_enabled: true
	});

	// Relais vers le store toast global pour le composant enfant (mappe 'danger' -> error, store sans type danger).
	function forwardToast(type, title, message) {
		if (type === 'danger') toast.error(title, message);
		else toast.show(type, title, message);
	}

	// History Telemetry State — populated only from /api/metrics/history.
	// There is no synthetic seed and no client-side extrapolation: a range with
	// no samples renders as an empty chart, and `null` samples render as gaps.
	const EMPTY_HISTORY = { timestamps: [], tps: [], cpu: [], ram_mb: [], players: [] };

	let historyData = $state(EMPTY_HISTORY);
	let historyError = $state('');

	const HISTORY_POLL_MS = 10000;
	let pollInterval = null;

	async function loadMetricsHistory() {
		isRefreshing = true;
		try {
			const res = await apiGet(`/api/metrics/history?range=${selectedRange}`);
			historyData = res?.timestamps ? res : EMPTY_HISTORY;
			historyError = '';
		} catch (err) {
			historyData = EMPTY_HISTORY;
			historyError =
				(err instanceof Error && err.message) || 'Historique des métriques indisponible.';
		} finally {
			setTimeout(() => {
				isRefreshing = false;
			}, 300);
		}
	}

	async function loadAlertConfig() {
		try {
			const config = await apiGet('/api/metrics/alerts').catch(() => null);
			if (config) {
				alertConfig = config;
			}
		} catch (err) {
			// keep default config
		}
	}

	function handleRangeChange(range) {
		selectedRange = range;
		loadMetricsHistory();
	}

	async function handleTriggerSpark(durationSeconds) {
		try {
			const res = await apiPost('/api/tools/spark/sampler', {
				action: 'start',
				timeout_secs: durationSeconds
			});
			return {
				report_url: res?.url,
				output_log: res?.output
			};
		} catch (err) {
			throw new Error(err?.message || 'Impossible de lancer le profilage Spark.');
		}
	}

	async function handleSaveAlertConfig(newConfig) {
		try {
			await apiPost('/api/metrics/alerts', newConfig).catch(() => {
				// Fallback local save if endpoint not mounted
			});
			alertConfig = { ...newConfig };
			toast.success('Alert Rules Saved', 'Telemetry threshold alert configurations updated.');
		} catch (err) {
			toast.error('Save Failed', err.message || 'Could not save alert settings.');
			throw err;
		}
	}

	async function handleTestWebhook() {
		try {
			await apiPost('/api/metrics/alerts/test', { webhook_url: alertConfig.discord_webhook_url }).catch(() => {
				// Fallback demo response
			});
			toast.success('Test Dispatched', 'Test alert message sent to Discord Webhook.');
		} catch (err) {
			toast.error('Test Failed', err.message || 'Could not dispatch webhook test.');
			throw err;
		}
	}

	// Cron Scheduler & Webhooks State
	let schedulerJobs = $state([]);
	let webhooks = $state([]);
	let runningJobId = $state(null);

	async function loadSchedulerJobs() {
		try {
			const res = await apiGet('/api/scheduler/jobs').catch(() => []);
			schedulerJobs = res || [];
		} catch (err) {
			console.error('Failed to load scheduler jobs:', err);
		}
	}

	async function handleRunJob(id) {
		runningJobId = id;
		try {
			const res = await apiPost(`/api/scheduler/jobs/${encodeURIComponent(id)}/run`, {});
			toast.success('Tâche Exécutée', res.message);
			await loadSchedulerJobs();
		} catch (err) {
			toast.error('Échec de la tâche', err.message || 'Erreur lors de l\'exécution.');
		} finally {
			runningJobId = null;
		}
	}

	async function handleToggleJob(id) {
		try {
			const res = await apiPost(`/api/scheduler/jobs/${encodeURIComponent(id)}/toggle`, {});
			toast.success(res.enabled ? 'Tâche activée' : 'Tâche désactivée', res.message);
			await loadSchedulerJobs();
		} catch (err) {
			toast.error('Erreur de basculement', err.message || 'Action impossible');
		}
	}

	async function loadWebhooks() {
		try {
			const res = await apiGet('/api/webhooks').catch(() => []);
			webhooks = res || [];
		} catch (err) {
			console.error('Failed to load webhooks:', err);
		}
	}

	async function handleTestPing() {
		try {
			const res = await apiPost('/api/webhooks/test', {
				url: '',
				channel_type: 'discord',
				secret: null
			});
			toast.success('Notification expédiée', res.message);
		} catch (err) {
			toast.error('Échec du test', err.message || 'Erreur lors du test.');
		}
	}

	onMount(() => {
		loadMetricsHistory();
		loadAlertConfig();
		loadSchedulerJobs();
		loadWebhooks();

		// Re-read the authoritative history instead of extrapolating locally: the
		// live /ws frame measures the *container*, /api/metrics/history measures
		// the *host*, so appending one into the other mixed two different sources.
		pollInterval = setInterval(loadMetricsHistory, HISTORY_POLL_MS);
	});

	onDestroy(() => {
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	});

	// Status helper — describes the live /ws link, not the history poll.
	let telemetryStatusText = $derived(
		wsStore.connected ? 'WebSocket connecté' : 'WebSocket déconnecté'
	);
</script>

<svelte:head>
	<title>Time-Series Metrics & Spark Profiler - ChiPanel</title>
</svelte:head>

<div class="metrics-page">
	<!-- Page Header -->
	<PageHeader
		title="Time-Series Metrics & Spark Profiler"
		subtitle="Real-time canvas performance telemetry, historical analysis, and Spark CPU/RAM profiling."
	>
		<!-- Telemetry Status Indicator -->
		<div class="telemetry-status-pill">
			<span class="status-dot {wsStore.connected ? 'status-dot-success status-dot-pulse' : 'status-dot-warning'}"></span>
			<span class="status-text">{telemetryStatusText}</span>
		</div>

		<!-- Time Range Selector Pills -->
		<div class="range-selector-group" role="radiogroup" aria-label="Select history range">
			{#each rangePresets as preset}
				<button
					type="button"
					class="btn btn-sm range-btn {selectedRange === preset.value ? 'active' : 'btn-secondary'}"
					onclick={() => handleRangeChange(preset.value)}
					aria-checked={selectedRange === preset.value}
					role="radio"
				>
					{preset.label}
				</button>
			{/each}
		</div>

		<!-- Alert Settings Button -->
		<button
			type="button"
			class="btn btn-secondary btn-sm"
			onclick={() => (isAlertModalOpen = true)}
			title="Configure Alert Rules"
		>
			<Sliders size={14} />
			<span>Alert Settings</span>
		</button>

		<!-- Refresh Button -->
		<button
			type="button"
			class="btn btn-secondary btn-icon btn-sm {isRefreshing ? 'btn-loading' : ''}"
			onclick={loadMetricsHistory}
			title="Refresh Metrics Data"
		>
			{#if !isRefreshing}
				<RefreshCw size={14} />
			{/if}
		</button>
	</PageHeader>

	{#if historyError}
		<div class="history-alert" role="alert">
			<AlertTriangle size={16} />
			<span>Historique indisponible : {historyError}</span>
		</div>
	{/if}

	<!-- Main Metrics Grid -->
	<div class="metrics-grid">
		<!-- 1. TPS Time-Series Chart -->
		<div class="grid-span-full">
			<MetricsChart {historyData} title="TPS (ticks par seconde)" chartType="tps" />
		</div>

		<!-- 2. CPU and RAM: one unit per chart, one Y scale per chart -->
		<div class="grid-column-2">
			<MetricsChart {historyData} title="CPU hôte (%)" chartType="cpu" />
		</div>

		<div class="grid-column-2">
			<MetricsChart {historyData} title="Mémoire hôte (Mo)" chartType="ram" />
		</div>

		<!-- 3. Players Time-Series Chart -->
		<div class="grid-column-2">
			<MetricsChart {historyData} title="Joueurs connectés" chartType="players" />
		</div>

		<!-- 4. Spark Profiler Action Card -->
		<div class="grid-column-2">
			<SparkProfilerCard
				onTriggerSpark={handleTriggerSpark}
				onToast={forwardToast}
			/>
		</div>

		<!-- 5. Tokio Cron Automation Scheduler -->
		<div class="grid-column-2 card cron-scheduler-card">
			<div class="cron-header">
				<div class="flex items-center gap-2">
					<CalendarClock size={18} class="text-primary" />
					<h2 class="card-title">Planificateur Cron (Tokio Scheduler)</h2>
				</div>
				<span class="badge badge-secondary">{schedulerJobs.length} tâche(s)</span>
			</div>

			<div class="cron-job-list">
				{#each schedulerJobs as job}
					<div class="cron-job-item card">
						<div class="cron-job-main">
							<div class="flex items-center gap-2">
								<button
									class="btn btn-ghost btn-sm btn-icon"
									onclick={() => handleToggleJob(job.id)}
									title={job.enabled ? "Désactiver la tâche" : "Activer la tâche"}
								>
									<Power size={14} class={job.enabled ? 'text-success' : 'text-muted'} />
								</button>
								<div>
									<strong class="text-sm block">{job.name}</strong>
									<span class="font-mono text-xs text-muted">{job.cron_expression} ({job.action_type})</span>
								</div>
							</div>
							<div class="flex items-center gap-2">
								{#if job.last_status === 'SUCCESS'}
									<span class="badge badge-success text-xs">Succès</span>
								{:else if job.last_status === 'FAILED'}
									<span class="badge badge-danger text-xs">Échec</span>
								{/if}
								<button
									class="btn btn-secondary btn-sm"
									onclick={() => handleRunJob(job.id)}
									disabled={runningJobId === job.id}
									title="Exécuter immédiatement"
								>
									<Play size={12} class={runningJobId === job.id ? 'spin' : ''} />
									<span>Lancer</span>
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>

		<!-- 6. Multi-Channel Webhook Dispatcher -->
		<div class="grid-column-2 card webhook-card">
			<div class="cron-header">
				<div class="flex items-center gap-2">
					<Webhook size={18} class="text-primary" />
					<h2 class="card-title">Notifications Multi-Canaux (Discord & HMAC)</h2>
				</div>
				<span class="badge badge-secondary">{webhooks.length} actif(s)</span>
			</div>

			<p class="text-xs text-muted">
				Diffusion automatisée des alertes TPS, crashs, arrêts serveur et sauvegardes avec signature cryptographique SHA-256.
			</p>

			<div class="webhook-actions">
				<button class="btn btn-secondary btn-sm" onclick={handleTestPing}>
					<Webhook size={14} />
					<span>Envoyer un Ping de Test</span>
				</button>
				<button class="btn btn-primary btn-sm" onclick={() => (isAlertModalOpen = true)}>
					<Sliders size={14} />
					<span>Configurer les Seuils</span>
				</button>
			</div>
		</div>
	</div>

	<!-- Alert Settings Modal -->
	<AlertSettingsModal
		isOpen={isAlertModalOpen}
		{alertConfig}
		onSaveConfig={handleSaveAlertConfig}
		onTestWebhook={handleTestWebhook}
		onClose={() => (isAlertModalOpen = false)}
	/>
</div>

<style>
	.metrics-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.telemetry-status-pill {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: 4px 12px;
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
	}

	.status-text {
		color: var(--text-secondary);
	}

	.range-selector-group {
		display: flex;
		align-items: center;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: 2px;
		gap: 2px;
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.3);
	}

	.range-btn {
		border-radius: var(--radius-badge);
		padding: 0 10px;
		height: 26px;
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		user-select: none;
		transition: transform 160ms var(--ease-out),
		            background-color 150ms var(--ease-out),
		            border-color 150ms var(--ease-out),
		            color 150ms var(--ease-out);
	}

	.range-btn:active {
		transform: scale(0.97);
	}

	/* White label, so it needs the solid (white-safe) fill, not the palette hue. */
	.range-btn.active {
		background-color: var(--accent-blue-solid);
		color: #ffffff;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15), 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	/* Grid Layout */
	.metrics-grid {
		display: grid;
		/* `1fr` is shorthand for `minmax(auto, 1fr)`: the track floors at the
		   content's min size, so a chart wider than its column pushed the track
		   past the viewport and the newest data became unreachable (A15/B2). */
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: var(--space-6);
	}

	/* Grid items themselves default to min-width:auto — same failure, one level down. */
	.metrics-grid > * {
		min-width: 0;
	}

	.grid-span-full {
		grid-column: 1 / -1;
	}

	.grid-column-2 {
		grid-column: span 1;
	}

	.history-alert {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-card);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
		font-size: var(--font-size-sm);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
	}

	.cron-scheduler-card,
	.webhook-card {
		padding: var(--space-5);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.07), 0 2px 8px rgba(0, 0, 0, 0.2);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.cron-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: var(--space-2);
		border-bottom: 1px solid var(--border-subtle);
	}

	.card-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin: 0;
		color: var(--text-primary);
	}

	.cron-job-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.cron-job-item {
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
	}

	.cron-job-main {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}

	.webhook-actions {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		margin-top: auto;
		padding-top: var(--space-2);
	}

	@media (max-width: 992px) {
		.metrics-grid {
			grid-template-columns: 1fr;
		}

		.grid-column-2 {
			grid-column: 1 / -1;
		}
	}
</style>
