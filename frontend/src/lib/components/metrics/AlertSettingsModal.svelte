<script>
	import { Bell, X, Send, Save, CheckCircle2, AlertTriangle, ShieldAlert, Loader2 } from 'lucide-svelte';

	let {
		isOpen = false,
		alertConfig = {
			tps_threshold: 15.0,
			cpu_threshold: 90,
			ram_threshold: 85,
			discord_webhook_url: '',
			alerts_enabled: true
		},
		onSaveConfig,
		onTestWebhook,
		onClose
	} = $props();

	// Local mutable state
	let localConfig = $state({
		tps_threshold: 15.0,
		cpu_threshold: 90,
		ram_threshold: 85,
		discord_webhook_url: '',
		alerts_enabled: true
	});

	let isSaving = $state(false);
	let isTesting = $state(false);
	let testSuccess = $state(false);

	// Sync local state when modal opens or config changes
	$effect(() => {
		if (isOpen && alertConfig) {
			localConfig = {
				tps_threshold: alertConfig.tps_threshold ?? 15.0,
				cpu_threshold: alertConfig.cpu_threshold ?? 90,
				ram_threshold: alertConfig.ram_threshold ?? 85,
				discord_webhook_url: alertConfig.discord_webhook_url ?? '',
				alerts_enabled: alertConfig.alerts_enabled ?? true
			};
			testSuccess = false;
		}
	});

	function handleKeyDown(event) {
		if (event.key === 'Escape' && isOpen) {
			onClose();
		}
	}

	async function handleSave() {
		if (!onSaveConfig) return;
		isSaving = true;
		try {
			await onSaveConfig(localConfig);
			onClose();
		} catch (err) {
			console.error('Failed to save alert settings:', err);
		} finally {
			isSaving = false;
		}
	}

	async function handleTest() {
		if (!onTestWebhook) return;
		isTesting = true;
		testSuccess = false;
		try {
			await onTestWebhook();
			testSuccess = true;
			setTimeout(() => {
				testSuccess = false;
			}, 3000);
		} catch (err) {
			console.error('Failed to test webhook:', err);
		} finally {
			isTesting = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
	<div
		class="modal-backdrop"
		onclick={(e) => e.target === e.currentTarget && onClose()}
		role="presentation"
	>
		<div
			class="modal alert-settings-modal"
			role="dialog"
			aria-modal="true"
			aria-labelledby="alert-settings-title"
		>
			<!-- Modal Header -->
			<div class="modal-header">
				<div class="modal-title-group">
					<div class="modal-icon-bg">
						<Bell size={20} class="text-blue" />
					</div>
					<div>
						<h2 id="alert-settings-title" class="modal-title">Telemetry Alert Rules</h2>
						<p class="modal-subtitle">Configure automated metric threshold alerts and Webhook dispatches.</p>
					</div>
				</div>
				<button type="button" class="btn btn-ghost btn-icon" onclick={onClose} aria-label="Close dialog">
					<X size={18} />
				</button>
			</div>

			<!-- Modal Body -->
			<div class="modal-body">
				<!-- Alerts Enable/Disable Switch -->
				<div class="form-group alert-toggle-card">
					<div class="toggle-text-group">
						<span class="label font-semibold">Enable Automated Alerts</span>
						<span class="subtext">Send notification triggers when performance limits are breached.</span>
					</div>
					<label class="toggle-switch">
						<input type="checkbox" bind:checked={localConfig.alerts_enabled} aria-label="Enable Automated Alerts" />
						<span class="toggle-slider"></span>
					</label>
				</div>

				<!-- Threshold Controls -->
				<div class="thresholds-container {localConfig.alerts_enabled ? '' : 'disabled-area'}">
					<!-- TPS Threshold -->
					<div class="form-group threshold-group">
						<div class="threshold-label-row">
							<span class="label flex-align">
								<ShieldAlert size={15} class="text-danger" />
								<span>TPS Warning Threshold (&lt; {localConfig.tps_threshold.toFixed(1)})</span>
							</span>
							<div class="threshold-input-wrapper">
								<input
									type="number"
									min="5.0"
									max="19.5"
									step="0.5"
									class="input input-sm number-input"
									bind:value={localConfig.tps_threshold}
									disabled={!localConfig.alerts_enabled}
									aria-label="TPS warning threshold"
								/>
								<span class="input-unit">TPS</span>
							</div>
						</div>
						<input
							type="range"
							min="5.0"
							max="19.5"
							step="0.5"
							class="range-slider range-tps"
							bind:value={localConfig.tps_threshold}
							disabled={!localConfig.alerts_enabled}
							aria-label="TPS warning threshold slider"
						/>
						<span class="field-hint">Triggers an alert when server TPS falls below {localConfig.tps_threshold.toFixed(1)}.</span>
					</div>

					<!-- CPU Threshold -->
					<div class="form-group threshold-group">
						<div class="threshold-label-row">
							<span class="label flex-align">
								<AlertTriangle size={15} class="text-warning" />
								<span>CPU Utilization Limit (&gt; {localConfig.cpu_threshold}%)</span>
							</span>
							<div class="threshold-input-wrapper">
								<input
									type="number"
									min="50"
									max="98"
									step="1"
									class="input input-sm number-input"
									bind:value={localConfig.cpu_threshold}
									disabled={!localConfig.alerts_enabled}
									aria-label="CPU utilization limit percentage"
								/>
								<span class="input-unit">%</span>
							</div>
						</div>
						<input
							type="range"
							min="50"
							max="98"
							step="1"
							class="range-slider range-cpu"
							bind:value={localConfig.cpu_threshold}
							disabled={!localConfig.alerts_enabled}
							aria-label="CPU utilization limit slider"
						/>
						<span class="field-hint">Triggers an alert when container CPU load exceeds {localConfig.cpu_threshold}%.</span>
					</div>

					<!-- RAM Threshold -->
					<div class="form-group threshold-group">
						<div class="threshold-label-row">
							<span class="label flex-align">
								<AlertTriangle size={15} class="text-warning" />
								<span>RAM Memory Limit (&gt; {localConfig.ram_threshold}%)</span>
							</span>
							<div class="threshold-input-wrapper">
								<input
									type="number"
									min="50"
									max="98"
									step="1"
									class="input input-sm number-input"
									bind:value={localConfig.ram_threshold}
									disabled={!localConfig.alerts_enabled}
									aria-label="RAM memory limit percentage"
								/>
								<span class="input-unit">%</span>
							</div>
						</div>
						<input
							type="range"
							min="50"
							max="98"
							step="1"
							class="range-slider range-ram"
							bind:value={localConfig.ram_threshold}
							disabled={!localConfig.alerts_enabled}
							aria-label="RAM memory limit slider"
						/>
						<span class="field-hint">Triggers an alert when RAM memory usage exceeds {localConfig.ram_threshold}%.</span>
					</div>

					<!-- Discord Webhook Input -->
					<div class="form-group webhook-group">
						<label for="discord-webhook-url" class="label">Discord Webhook Notification Endpoint</label>
						<div class="webhook-input-row">
							<input
								id="discord-webhook-url"
								type="url"
								class="input"
								placeholder="https://discord.com/api/webhooks/..."
								bind:value={localConfig.discord_webhook_url}
								disabled={!localConfig.alerts_enabled}
							/>
							<button
								type="button"
								class="btn btn-secondary test-webhook-btn {isTesting ? 'btn-loading' : ''}"
								disabled={!localConfig.alerts_enabled || !localConfig.discord_webhook_url || isTesting}
								onclick={handleTest}
							>
								{#if isTesting}
									<Loader2 size={14} class="spin-icon" />
								{:else if testSuccess}
									<CheckCircle2 size={14} class="text-green" />
									<span>Sent!</span>
								{:else}
									<Send size={14} />
									<span>Test Webhook</span>
								{/if}
							</button>
						</div>
						<span class="field-hint">JSON payload dispatches directly to Discord channel upon metric breaches.</span>
					</div>
				</div>
			</div>

			<!-- Modal Footer -->
			<div class="modal-footer">
				<button type="button" class="btn btn-ghost" onclick={onClose} disabled={isSaving}>
					Cancel
				</button>
				<button
					type="button"
					class="btn btn-primary {isSaving ? 'btn-loading' : ''}"
					onclick={handleSave}
					disabled={isSaving}
				>
					{#if !isSaving}
						<Save size={16} />
					{/if}
					<span>Save Alert Rules</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.alert-settings-modal {
		max-width: 560px;
	}

	.modal-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.modal-icon-bg {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-input);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.modal-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.alert-toggle-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		padding: var(--space-3) var(--space-4);
		margin-bottom: var(--space-4);
	}

	.toggle-text-group {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.subtext {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.thresholds-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		transition: opacity var(--transition-fast);
	}

	.thresholds-container.disabled-area {
		opacity: 0.45;
		pointer-events: none;
	}

	.threshold-group {
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-btn);
		padding: var(--space-3) var(--space-4);
		margin-bottom: 0;
	}

	.threshold-label-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--space-2);
	}

	.threshold-input-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}

	.number-input {
		width: 70px;
		text-align: right;
		font-family: var(--font-mono);
		padding-right: var(--space-2);
	}

	.input-unit {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.range-slider {
		width: 100%;
		accent-color: var(--accent-blue);
		cursor: pointer;
		height: 6px;
		background-color: var(--bg-elevated);
		border-radius: 3px;
		outline: none;
	}

	.range-tps {
		accent-color: var(--danger);
	}

	.range-cpu {
		accent-color: var(--accent-blue);
	}

	.range-ram {
		accent-color: var(--warning);
	}

	.webhook-group {
		margin-bottom: 0;
	}

	.webhook-input-row {
		display: flex;
		gap: var(--space-2);
		margin-top: var(--space-1);
	}

	.test-webhook-btn {
		flex-shrink: 0;
	}

	.field-hint {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: var(--space-1);
		display: block;
	}

	.text-blue {
		color: var(--accent-blue-text);
	}

	.text-green {
		color: var(--accent-green);
	}

	.text-warning {
		color: var(--warning);
	}

	.text-danger {
		color: var(--danger-text);
	}

	.flex-align {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}

	.spin-icon {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
