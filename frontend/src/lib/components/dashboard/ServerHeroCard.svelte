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

<div class="card hero-banner-card">
	<div class="hero-banner-bg"></div>

	<div class="hero-banner-content">
		<div class="hero-left-section">
			<div class="server-avatar-badge">
				<Server size={26} />
			</div>
			<div class="hero-server-meta">
				<div class="hero-title-row">
					<h2 class="hero-server-name">{serverTitle}</h2>
					<button
						class="copy-pill-badge"
						onclick={() => copyToClipboard(containerId, 'tag')}
						title="Cliquer pour copier le moteur"
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
				<span class="hero-metric-label">JOUEURS EN LIGNE</span>
				<div class="hero-metric-val">
					{srv.onlinePlayers ?? UNAVAILABLE}
					<span class="val-slash">/</span>
					{srv.maxPlayers ?? UNAVAILABLE}
				</div>
				<div class="hero-progress-track">
					<div class="hero-progress-fill" style="width: {playerPct}%"></div>
				</div>
			</div>

			<!-- Real Uptime Metric -->
			<div class="hero-metric-box">
				<span class="hero-metric-label">DURÉE DE FONCTIONNEMENT</span>
				<div class="hero-metric-val-row">
					<Clock size={16} class="text-green" />
					<span class="hero-metric-val">{formattedUptime}</span>
				</div>
			</div>
		</div>
	</div>
</div>

<style>
	.hero-banner-card {
		position: relative;
		min-height: 120px;
		border-radius: var(--radius-lg);
		background: linear-gradient(135deg, var(--bg-surface) 0%, var(--bg-elevated) 100%);
		border: 1px solid var(--border);
		overflow: hidden;
	}

	.hero-banner-bg {
		position: absolute;
		inset: 0;
		background: radial-gradient(circle at 90% 10%, rgba(16, 185, 129, 0.06) 0%, transparent 60%);
		pointer-events: none;
	}

	.hero-banner-content {
		position: relative;
		z-index: 10;
		padding: var(--space-6);
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
		width: 52px;
		height: 52px;
		border-radius: var(--radius-card);
		background: var(--accent-green-bg);
		border: 1px solid var(--accent-green-border);
		color: var(--accent-green);
		display: flex;
		align-items: center;
		justify-content: center;
		box-shadow: 0 0 16px rgba(16, 185, 129, 0.2);
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
		font-size: 1.375rem;
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		letter-spacing: -0.02em;
	}

	.copy-pill-badge {
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-badge);
		padding: 2px 10px;
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		display: inline-flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.copy-pill-badge:hover {
		background: rgba(255, 255, 255, 0.1);
		color: var(--text-primary);
	}

	.ip-copy-btn {
		background: transparent;
		border: none;
		color: var(--text-secondary);
		font-size: var(--font-size-sm);
		font-family: var(--font-mono);
		display: inline-flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
		padding: 0;
		transition: color var(--transition-fast);
	}

	.ip-copy-btn:hover {
		color: var(--text-primary);
	}

	.hero-right-metrics {
		display: flex;
		align-items: center;
		gap: var(--space-8);
	}

	.hero-metric-box {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.hero-metric-label {
		font-size: 0.6875rem;
		font-weight: var(--font-weight-bold);
		color: var(--text-muted);
		letter-spacing: 0.08em;
	}

	.hero-metric-val {
		font-size: 1.25rem;
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}

	.val-slash {
		color: var(--text-muted);
		font-weight: var(--font-weight-normal);
	}

	.hero-progress-track {
		width: 120px;
		height: 4px;
		background: var(--bg-base);
		border-radius: 2px;
		overflow: hidden;
	}

	.hero-progress-fill {
		height: 100%;
		background: var(--accent-green);
		border-radius: 2px;
		box-shadow: 0 0 8px rgba(16, 185, 129, 0.5);
		transition: width var(--transition-fast);
	}

	.hero-metric-val-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.text-green { color: var(--accent-green); }
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
