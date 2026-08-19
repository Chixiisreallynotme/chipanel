<script>
	import { onDestroy } from 'svelte';
	import { Zap, Play, Clock, Copy, Check, ExternalLink, FileText, CheckCircle2, Loader2 } from 'lucide-svelte';

	let { onTriggerSpark, onToast } = $props();

	const presets = [
		{ label: '30s', seconds: 30 },
		{ label: '60s', seconds: 60 },
		{ label: '3m', seconds: 180 },
		{ label: '5m', seconds: 300 }
	];

	let selectedDuration = $state(60);
	let isProfiling = $state(false);
	let elapsedSeconds = $state(0);
	let timerInterval = null;

	/** @type {{ report_url?: string, output_log?: string } | null} */
	let reportResult = $state(null);
	let copied = $state(false);
	let showOutputLog = $state(false);

	function selectPreset(seconds) {
		if (isProfiling) return;
		selectedDuration = seconds;
	}

	async function handleStartProfiling() {
		if (isProfiling || !onTriggerSpark) return;

		isProfiling = true;
		elapsedSeconds = 0;
		reportResult = null;
		copied = false;

		timerInterval = setInterval(() => {
			if (elapsedSeconds < selectedDuration) {
				elapsedSeconds += 1;
			}
		}, 1000);

		try {
			const res = await onTriggerSpark(selectedDuration);
			reportResult = res || { output_log: 'Profiler process completed.' };

			if (onToast) {
				onToast(
					'success',
					'Spark Profile Completed',
					res?.report_url ? `Report generated: ${res.report_url}` : 'Spark profiler completed successfully.'
				);
			}
		} catch (err) {
			const errorMessage = err?.message || 'Failed to execute Spark profiler command.';
			if (onToast) {
				onToast('danger', 'Spark Profiler Error', errorMessage);
			}
		} finally {
			stopTimer();
			isProfiling = false;
		}
	}

	function stopTimer() {
		if (timerInterval) {
			clearInterval(timerInterval);
			timerInterval = null;
		}
	}

	async function copyReportUrl() {
		if (!reportResult?.report_url) return;
		try {
			await navigator.clipboard.writeText(reportResult.report_url);
			copied = true;
			if (onToast) {
				onToast('success', 'Copied to Clipboard', 'Spark report link copied.');
			}
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			if (onToast) {
				onToast('danger', 'Copy Failed', 'Failed to copy URL to clipboard.');
			}
		}
	}

	onDestroy(() => {
		stopTimer();
	});

	let progressPercentage = $derived(
		selectedDuration > 0 ? Math.min(100, Math.round((elapsedSeconds / selectedDuration) * 100)) : 0
	);
	let remainingSeconds = $derived(Math.max(0, selectedDuration - elapsedSeconds));
</script>

<div class="card spark-profiler-card">
	<div class="card-header">
		<div class="header-title-group">
			<div class="zap-icon-bg">
				<Zap size={18} class="zap-icon" />
			</div>
			<div>
				<h3 class="card-title">Spark CPU & Memory Profiler</h3>
				<p class="card-subtitle">Run low-overhead asynchronous tick & thread sampling profiling.</p>
			</div>
		</div>
		<span class="badge badge-purple">
			<Clock size={12} />
			<span>Preset Sampling</span>
		</span>
	</div>

	<div class="card-body spark-card-body">
		<!-- Profiler Duration Selector Presets -->
		<div class="preset-section">
			<span class="label">Profiler Sampling Duration:</span>
			<div class="segmented-control" role="radiogroup" aria-label="Profiler sampling duration">
				{#each presets as preset}
					<button
						type="button"
						class="segmented-btn {selectedDuration === preset.seconds ? 'active' : ''}"
						disabled={isProfiling}
						onclick={() => selectPreset(preset.seconds)}
						role="radio"
						aria-checked={selectedDuration === preset.seconds}
					>
						{preset.label}
					</button>
				{/each}
			</div>
		</div>

		<!-- Profiler Action Button -->
		<div class="action-row">
			<button
				type="button"
				class="btn btn-primary start-profiler-btn {isProfiling ? 'btn-loading' : ''}"
				disabled={isProfiling}
				onclick={handleStartProfiling}
			>
				{#if isProfiling}
					<Loader2 size={16} class="spin-icon" />
					<span>Profiling ({remainingSeconds}s remaining)...</span>
				{:else}
					<Play size={16} />
					<span>Start Spark Profiler ({selectedDuration}s)</span>
				{/if}
			</button>
		</div>

		<!-- Active Profiling Progress State -->
		{#if isProfiling}
			<div class="profiling-progress-card">
				<div class="progress-info-row">
					<div class="progress-status">
						<span class="status-dot status-dot-warning status-dot-pulse"></span>
						<span class="status-text">Sampling server threads...</span>
					</div>
					<span class="progress-counter">{elapsedSeconds}s / {selectedDuration}s ({progressPercentage}%)</span>
				</div>

				<div
					class="gauge-bar spark-progress-bar"
					role="progressbar"
					aria-valuenow={progressPercentage}
					aria-valuemin="0"
					aria-valuemax="100"
					aria-label="Spark profiling progress"
				>
					<div class="gauge-fill gauge-fill-warning" style="transform: scaleX({progressPercentage / 100});"></div>
				</div>
			</div>
		{/if}

		<!-- Report Results Display Card -->
		{#if reportResult}
			<div class="report-result-card">
				<div class="report-header">
					<div class="report-title-group">
						<CheckCircle2 size={18} class="text-green" />
						<span class="report-title">Spark Profiler Report Ready</span>
					</div>
					<span class="badge badge-success">Success</span>
				</div>

				{#if reportResult.report_url}
					<div class="url-box">
						<span class="url-label">Report URL:</span>
						<a href={reportResult.report_url} target="_blank" rel="noopener noreferrer" class="spark-url-link">
							{reportResult.report_url}
						</a>
						<div class="url-actions">
							<button type="button" class="btn btn-secondary btn-sm" onclick={copyReportUrl} title="Copy Spark Link">
								{#if copied}
									<Check size={14} class="text-green" />
									<span>Copied!</span>
								{:else}
									<Copy size={14} />
									<span>Copy Link</span>
								{/if}
							</button>
							<a
								href={reportResult.report_url}
								target="_blank"
								rel="noopener noreferrer"
								class="btn btn-primary btn-sm"
							>
								<ExternalLink size={14} />
								<span>Open Report</span>
							</a>
						</div>
					</div>
				{/if}

				{#if reportResult.output_log}
					<div class="log-accordion">
						<button
							type="button"
							class="btn btn-ghost btn-sm log-toggle-btn"
							onclick={() => (showOutputLog = !showOutputLog)}
						>
							<FileText size={14} />
							<span>{showOutputLog ? 'Hide Console Output' : 'View Console Output'}</span>
						</button>

						{#if showOutputLog}
							<pre class="log-output-box">{reportResult.output_log}</pre>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.spark-profiler-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
	}

	.header-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.zap-icon-bg {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-input);
		background-color: rgba(139, 92, 246, 0.12);
		border: 1px solid rgba(139, 92, 246, 0.25);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-purple-text);
		flex-shrink: 0;
	}

	.badge-purple {
		background-color: rgba(139, 92, 246, 0.12);
		color: var(--accent-purple-text);
		border-color: rgba(139, 92, 246, 0.25);
	}

	.spark-card-body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
	}

	.preset-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.segmented-control {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		padding: 3px;
		gap: 3px;
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4);
	}

	.segmented-btn {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		font-variant-numeric: tabular-nums;
		height: 32px;
		border: 1px solid transparent;
		border-radius: calc(var(--radius-btn) - 3px);
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		user-select: none;
		transition: transform 160ms var(--ease-out),
		            background-color 150ms var(--ease-out),
		            border-color 150ms var(--ease-out),
		            color 150ms var(--ease-out),
		            box-shadow 150ms var(--ease-out);
	}

	.segmented-btn:hover:not(:disabled):not(.active) {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
	}

	.segmented-btn:active:not(:disabled) {
		transform: scale(0.97);
	}

	.segmented-btn.active {
		background-color: var(--bg-elevated);
		border-color: var(--accent-blue-border);
		color: var(--accent-blue-text);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.09), 0 2px 6px rgba(0, 0, 0, 0.35);
	}

	.start-profiler-btn {
		width: 100%;
		height: 42px;
		font-size: var(--font-size-md);
	}

	.spin-icon {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* Profiling Progress State */
	.profiling-progress-card {
		background-color: var(--bg-base);
		border: 1px solid var(--warning-border);
		border-radius: var(--radius-btn);
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.progress-info-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: var(--font-size-sm);
	}

	.progress-status {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-weight: var(--font-weight-medium);
		color: var(--warning);
	}

	.progress-counter {
		font-family: var(--font-mono);
		color: var(--text-secondary);
		font-size: var(--font-size-xs);
	}

	.spark-progress-bar {
		height: 6px;
	}

	/* Report Results Card */
	.report-result-card {
		background-color: var(--bg-base);
		border: 1px solid var(--accent-green-border);
		border-radius: var(--radius-card);
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.report-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.report-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.report-title {
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		font-size: var(--font-size-base);
	}

	.url-box {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		padding: var(--space-3) var(--space-4);
	}

	.url-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.spark-url-link {
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		word-break: break-all;
		color: var(--accent-blue-text);
	}

	.url-actions {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-2);
		margin-top: var(--space-1);
	}

	.log-accordion {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.log-toggle-btn {
		align-self: flex-start;
		color: var(--text-secondary);
	}

	.log-output-box {
		background-color: #060609;
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		padding: var(--space-3);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-primary);
		max-height: 200px;
		overflow-y: auto;
		white-space: pre-wrap;
		word-break: break-all;
	}

	.text-green {
		color: var(--accent-green);
	}
</style>
