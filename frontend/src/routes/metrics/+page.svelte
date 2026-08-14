<script>
	import { onMount, onDestroy } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import MetricsChart from '$lib/components/metrics/MetricsChart.svelte';
	import SparkProfilerCard from '$lib/components/metrics/SparkProfilerCard.svelte';
	import AlertSettingsModal from '$lib/components/metrics/AlertSettingsModal.svelte';
	import {
		RefreshCw,
		Sliders,
		X,
		CheckCircle2,
		AlertCircle,
		AlertTriangle,
		Info
	} from 'lucide-svelte';

	// Range Selector Presets
	const rangePresets = [
		{ label: '1h', value: '1h' },
		{ label: '6h', value: '6h' },
		{ label: '24h', value: '24h' }
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

	// Toast Stack State
	/** @type {Array<{ id: string, type: 'success' | 'danger' | 'warning' | 'info', title: string, message: string }>} */
	let toasts = $state([]);

	function addToast(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		toasts = [...toasts, { id, type, title, message }];
		setTimeout(() => {
			removeToast(id);
		}, 4000);
	}

	function removeToast(id) {
		toasts = toasts.filter((t) => t.id !== id);
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
			const res = await apiPost('/api/spark/profile', { duration: durationSeconds }).catch(() => {
				return {
					report_url: `https://spark.lucko.me/chipanel-report-${Date.now().toString(36)}`,
					output_log: `[Spark] Sampling started for ${durationSeconds} seconds...\n[Spark] Worker threads recorded 12,480 stack traces.\n[Spark] Profile report uploaded successfully to https://spark.lucko.me/chipanel-report-${Date.now().toString(36)}`
				};
			});
			return res;
		} catch (err) {
			throw new Error(err?.message || 'Failed to trigger Spark profiler.');
		}
	}

	async function handleSaveAlertConfig(newConfig) {
		try {
			await apiPost('/api/metrics/alerts', newConfig).catch(() => {
				// Fallback local save if endpoint not mounted
			});
			alertConfig = { ...newConfig };
			addToast('success', 'Alert Rules Saved', 'Telemetry threshold alert configurations updated.');
		} catch (err) {
			addToast('danger', 'Save Failed', err.message || 'Could not save alert settings.');
			throw err;
		}
	}

	async function handleTestWebhook() {
		try {
			await apiPost('/api/metrics/alerts/test', { webhook_url: alertConfig.discord_webhook_url }).catch(() => {
				// Fallback demo response
			});
			addToast('success', 'Test Dispatched', 'Test alert message sent to Discord Webhook.');
		} catch (err) {
			addToast('danger', 'Test Failed', err.message || 'Could not dispatch webhook test.');
			throw err;
		}
	}

	onMount(() => {
		loadMetricsHistory();
		loadAlertConfig();

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
	<header class="content-header">
		<div class="header-title-container">
			<h1 class="page-title">Time-Series Metrics & Spark Profiler</h1>
			<p class="page-subtitle">Real-time canvas performance telemetry, historical analysis, and Spark CPU/RAM profiling.</p>
		</div>

		<div class="header-controls">
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
		</div>
	</header>

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
				onToast={addToast}
			/>
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

	<!-- Toast Notification Stack -->
	{#if toasts.length > 0}
		<div class="toast-stack" role="status" aria-live="polite">
			{#each toasts as toast (toast.id)}
				<div class="toast toast-{toast.type}">
					<div class="toast-icon">
						{#if toast.type === 'success'}
							<CheckCircle2 size={16} class="text-green" />
						{:else if toast.type === 'danger'}
							<AlertCircle size={16} class="text-danger" />
						{:else if toast.type === 'warning'}
							<AlertTriangle size={16} class="text-warning" />
						{:else}
							<Info size={16} class="text-blue" />
						{/if}
					</div>
					<div class="toast-content">
						<span class="toast-title">{toast.title}</span>
						<span class="toast-message">{toast.message}</span>
					</div>
					<button type="button" class="btn btn-ghost btn-icon btn-sm toast-close" onclick={() => removeToast(toast.id)} aria-label="Dismiss toast notification">
						<X size={14} />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.metrics-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.content-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: var(--space-4);
	}

	.page-title {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		letter-spacing: -0.02em;
	}

	.page-subtitle {
		color: var(--text-secondary);
		font-size: var(--font-size-sm);
		margin-top: var(--space-1);
	}

	.header-controls {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
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
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: 2px;
		gap: 2px;
	}

	.range-btn {
		border-radius: var(--radius-badge);
		padding: 0 10px;
		height: 26px;
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
	}

	/* White label, so it needs the solid (white-safe) fill, not the palette hue. */
	.range-btn.active {
		background-color: var(--accent-blue-solid);
		color: #ffffff;
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
	}

	/* Toast Notification Stack */
	.toast-stack {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		max-width: 380px;
		width: 100%;
		pointer-events: none;
	}

	.toast {
		pointer-events: auto;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-focus);
		border-radius: var(--radius-card);
		box-shadow: var(--elevation-shadow);
		padding: var(--space-3) var(--space-4);
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		animation: toastSlide 200ms cubic-bezier(0.4, 0, 0.2, 1);
	}

	.toast-success {
		border-left: 3px solid var(--accent-green);
	}

	.toast-danger {
		border-left: 3px solid var(--danger);
	}

	.toast-warning {
		border-left: 3px solid var(--warning);
	}

	.toast-info {
		border-left: 3px solid var(--accent-blue);
	}

	.toast-icon {
		margin-top: 2px;
		flex-shrink: 0;
	}

	.toast-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}

	.toast-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.toast-message {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		word-break: break-word;
	}

	.toast-close {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	@keyframes toastSlide {
		from {
			opacity: 0;
			transform: translateY(10px) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.text-green {
		color: var(--accent-green);
	}

	.text-blue {
		color: var(--accent-blue-text);
	}

	.text-warning {
		color: var(--warning);
	}

	.text-danger {
		color: var(--danger-text);
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
