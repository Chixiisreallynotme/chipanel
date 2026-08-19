<script>
	import { browser } from '$app/environment';
	import { serverState, UNAVAILABLE } from './serverState.js';
	import { Copy, Check, Clock, Server, Globe } from 'lucide-svelte';

	let {
		serverTitle = 'Survie',
		containerId = `Moteur ${UNAVAILABLE}`,
		/** Real uptime from /api/server/status; null when Podman is unreachable. */
		uptimeSeconds = null
	} = $props();

	let copiedTag = $state(false);
	let copiedIp = $state(false);

	let srv = $derived(serverState());
	let playerPct = $derived(
		srv.onlinePlayers != null && srv.maxPlayers ? (srv.onlinePlayers / srv.maxPlayers) * 100 : 0
	);

	let ledState = $derived.by(() => {
		if (srv.containerRunning === true) return 'active';
		if (srv.containerRunning === false) return 'stopped';
		return 'crash';
	});

	let hostAddress = $derived.by(() => {
		if (browser) {
			return `${window.location.hostname}:25565`;
		}
		return '127.0.0.1:25565';
	});

	let formattedUptime = $derived.by(() => {
		if (uptimeSeconds == null) return UNAVAILABLE;
		if (uptimeSeconds <= 0) return 'Arrêté';
		const days = Math.floor(uptimeSeconds / 86400);
		const hrs = Math.floor((uptimeSeconds % 86400) / 3600);
		const mins = Math.floor((uptimeSeconds % 3600) / 60);
		const secs = uptimeSeconds % 60;
		if (days > 0) return `${days}j ${hrs}h ${mins}m`;
		if (hrs > 0) return `${hrs}h ${mins}m`;
		return `${mins}m ${secs}s`;
	});

	function copyToClipboard(text, type) {
		if (navigator.clipboard) {
			navigator.clipboard.writeText(text);
			if (type === 'tag') {
				copiedTag = true;
				setTimeout(() => (copiedTag = false), 2000);
			} else {
				copiedIp = true;
				setTimeout(() => (copiedIp = false), 2000);
			}
		}
	}
</script>

<div class="hardware-shell hero-banner-shell">
	<div class="hardware-core hero-banner-core">
		<div class="hero-banner-content">
			<div class="hero-left-section">
				<div class="server-avatar-badge">
					<Server size={24} />
					<span
						class="hero-led telemetry-led telemetry-led-{ledState}"
						title="État du serveur : {srv.containerRunning ? 'En ligne' : 'Arrêté'}"
					></span>
				</div>
				<div class="hero-server-meta">
					<div class="hero-title-row">
						<h2 class="hero-server-name">{serverTitle}</h2>
						<button
							class="copy-pill-badge"
							onclick={() => copyToClipboard(containerId, 'tag')}
							title="Cliquer pour copier le moteur"
							type="button"
						>
							<span>{containerId}</span>
							{#if copiedTag}
								<Check size={12} class="text-green" />
							{:else}
								<Copy size={12} />
							{/if}
						</button>
					</div>

					<div class="ip-address-row">
						<button
							class="ip-copy-btn"
							onclick={() => copyToClipboard(hostAddress, 'ip')}
							title="Cliquer pour copier l'adresse IP"
							type="button"
						>
							<Globe size={14} class="text-muted" />
							<span>{hostAddress}</span>
							{#if copiedIp}
								<Check size={14} class="text-green" />
							{:else}
								<Copy size={14} />
							{/if}
						</button>
					</div>
				</div>
			</div>

			<div class="hero-right-metrics">
				<!-- Online Players Metric -->
				<div class="hero-metric-box">
					<span class="hero-metric-label">Joueurs en ligne</span>
					<div class="hero-metric-val tabular-nums">
						{srv.onlinePlayers ?? UNAVAILABLE}
						<span class="val-slash">/</span>
						{srv.maxPlayers ?? UNAVAILABLE}
					</div>
					<div class="hero-progress-track">
						<div
							class="hero-progress-fill"
							style="--pct: {playerPct}; transform: scaleX(calc(var(--pct) / 100));"
						></div>
					</div>
				</div>

				<!-- Real Uptime Metric -->
				<div class="hero-metric-box">
					<span class="hero-metric-label">Durée de fonctionnement</span>
					<div class="hero-metric-val-row tabular-nums">
						<Clock size={15} class="text-green" />
						<span class="hero-metric-val">{formattedUptime}</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	.hero-banner-shell {
		width: 100%;
		background-color: rgba(255, 255, 255, 0.02);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		padding: 6px;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
	}

	.hero-banner-core {
		position: relative;
		min-height: 110px;
		border-radius: calc(var(--radius-card) - 6px);
		background-color: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.04);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.09), 0 2px 8px rgba(0, 0, 0, 0.2);
		overflow: hidden;
		padding: var(--space-5) var(--space-6);
	}

	.hero-banner-content {
		position: relative;
		z-index: 2;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-6);
		flex-wrap: wrap;
	}

	.hero-left-section {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.server-avatar-badge {
		position: relative;
		width: 48px;
		height: 48px;
		border-radius: var(--radius-input);
		background: var(--bg-base);
		border: 1px solid var(--border);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
		color: var(--accent-green-text);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.hero-led {
		position: absolute;
		top: -2px;
		right: -2px;
		border: 2px solid var(--bg-surface);
	}

	.hero-server-meta {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.hero-title-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.hero-server-name {
		font-size: 1.25rem;
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		letter-spacing: -0.02em;
	}

	.copy-pill-badge {
		background: rgba(255, 255, 255, 0.04);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-badge);
		padding: 2px 10px;
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		color: var(--text-secondary);
		display: inline-flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
		transition: transform var(--transition-fast), background-color var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
	}

	.copy-pill-badge:hover {
		background: rgba(255, 255, 255, 0.08);
		border-color: var(--border-focus);
		color: var(--text-primary);
	}

	.copy-pill-badge:active {
		transform: scale(0.97);
	}

	.ip-copy-btn {
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		color: var(--text-secondary);
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
		padding: 3px 8px;
		transition: transform var(--transition-fast), background-color var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
	}

	.ip-copy-btn:hover {
		background: rgba(255, 255, 255, 0.06);
		border-color: var(--border-focus);
		color: var(--text-primary);
	}

	.ip-copy-btn:active {
		transform: scale(0.97);
	}

	.hero-right-metrics {
		display: flex;
		align-items: center;
		gap: var(--space-8);
	}

	.hero-metric-box {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.hero-metric-label {
		font-size: 0.6875rem;
		font-weight: var(--font-weight-semibold);
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.hero-metric-val {
		font-size: 1.125rem;
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		font-family: var(--font-mono);
	}

	.val-slash {
		color: var(--text-muted);
		font-weight: var(--font-weight-normal);
		margin: 0 1px;
	}

	.hero-progress-track {
		width: 120px;
		height: 4px;
		background: var(--bg-base);
		border-radius: var(--radius-badge);
		overflow: hidden;
		border: 1px solid var(--border-subtle);
		margin-top: 2px;
	}

	.hero-progress-fill {
		height: 100%;
		width: 100%;
		background: var(--accent-green);
		border-radius: var(--radius-badge);
		transform-origin: left;
		transition: transform var(--transition-fast);
	}

	.hero-metric-val-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-family: var(--font-mono);
	}

	.tabular-nums {
		font-variant-numeric: tabular-nums;
	}

	.text-green { color: var(--accent-green-text); }
	.text-muted { color: var(--text-muted); }

	@media (max-width: 768px) {
		.hero-banner-content {
			flex-direction: column;
			align-items: flex-start;
		}

		.hero-right-metrics {
			width: 100%;
			justify-content: space-between;
		}
	}
</style>
