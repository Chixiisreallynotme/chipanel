<script>
	import { serverState, fmtNum, barWidth, UNAVAILABLE } from './serverState.js';
	import { Activity, Users, Cpu, HardDrive, Gauge, Check, AlertTriangle, HelpCircle } from '$lib/icons.js';

	// Every tile below reads this one snapshot, so they cannot contradict each
	// other the way "Arrêté" + "En ligne" + "Podman : Actif" used to (B1).
	let srv = $derived(serverState());

	let status = $derived.by(() => {
		if (srv.containerRunning === true) {
			return { text: 'En ligne', valueClass: 'text-green', badgeClass: 'badge-green' };
		}
		if (srv.containerRunning === false) {
			return { text: 'Hors ligne', valueClass: 'text-danger', badgeClass: 'badge-danger' };
		}
		return { text: 'Indisponible', valueClass: 'text-muted', badgeClass: 'badge-muted' };
	});

	let statusDetail = $derived.by(() => {
		if (srv.podmanAvailable === false) return 'Podman injoignable — état inconnu';
		if (srv.containerRunning === false) return 'En attente de démarrage';
		if (srv.rconAvailable === true) return 'RCON actif et réactif';
		if (srv.rconAvailable === false) return 'RCON injoignable';
		return 'État RCON inconnu';
	});

	let formattedRam = $derived.by(() => {
		if (srv.memoryMb == null && srv.memoryLimitMb == null) return `${UNAVAILABLE} Go`;
		const used = srv.memoryMb == null ? UNAVAILABLE : (srv.memoryMb / 1024).toFixed(1);
		const total = srv.memoryLimitMb == null ? UNAVAILABLE : (srv.memoryLimitMb / 1024).toFixed(0);
		return `${used} Go / ${total} Go`;
	});

	let tpsStatus = $derived.by(() => {
		if (srv.tps == null) {
			const text =
				srv.rconAvailable === true
					? 'TPS non fourni par ce moteur'
					: 'Mesure TPS indisponible';
			return { text, colorClass: 'text-muted', ok: false, unknown: true };
		}
		if (srv.tps >= 18.0) return { text: 'Optimal', colorClass: 'text-green', ok: true, unknown: false };
		if (srv.tps >= 14.0) return { text: 'Légère baisse', colorClass: 'text-warning', ok: false, unknown: false };
		return { text: 'Baisse sévère', colorClass: 'text-danger', ok: false, unknown: false };
	});
</script>

<div class="kpi-cards-grid">
	<!-- 1. Statut -->
	<div class="card kpi-card">
		<div class="kpi-top">
			<div class="kpi-icon-badge {status.badgeClass}">
				<Activity size={18} />
			</div>
			<div class="kpi-meta">
				<span class="kpi-label">Statut du serveur</span>
				<span class="kpi-val {status.valueClass}">{status.text}</span>
			</div>
		</div>
		<div class="kpi-sub">
			{#if srv.containerRunning === true}
				<span class="telemetry-led telemetry-led-active"></span>
			{:else if srv.containerRunning === false}
				<span class="telemetry-led telemetry-led-stopped"></span>
			{:else}
				<span class="telemetry-led telemetry-led-crash"></span>
			{/if}
			<span>{statusDetail}</span>
		</div>
	</div>

	<!-- 2. Joueurs en ligne -->
	<div class="card kpi-card">
		<div class="kpi-top">
			<div class="kpi-icon-badge badge-purple">
				<Users size={18} />
			</div>
			<div class="kpi-meta">
				<span class="kpi-label">Joueurs en ligne</span>
				<span class="kpi-val tabular-nums">
					{srv.onlinePlayers ?? UNAVAILABLE}
					<span class="val-sub">/ {srv.maxPlayers ?? UNAVAILABLE}</span>
				</span>
			</div>
		</div>
		<div class="kpi-sub text-muted">
			<span class="tabular-nums">
				{srv.onlinePlayers == null
					? 'Liste des joueurs indisponible'
					: `${srv.onlinePlayers} connecté(s)`}
			</span>
		</div>
	</div>

	<!-- 3. Utilisation CPU -->
	<div class="card kpi-card">
		<div class="kpi-top">
			<div class="kpi-icon-badge badge-blue">
				<Cpu size={18} />
			</div>
			<div class="kpi-meta">
				<span class="kpi-label">Utilisation CPU</span>
				<span class="kpi-val tabular-nums">{fmtNum(srv.cpuPercent, 1)}{srv.cpuPercent == null ? '' : '%'}</span>
			</div>
		</div>
		<div class="kpi-progress-bar-track">
			<div
				class="kpi-progress-bar-fill bar-blue"
				style="--pct: {barWidth(srv.cpuPercent)}; transform: scaleX(calc(var(--pct) / 100));"
			></div>
		</div>
	</div>

	<!-- 4. Allocation RAM -->
	<div class="card kpi-card">
		<div class="kpi-top">
			<div class="kpi-icon-badge badge-purple">
				<HardDrive size={18} />
			</div>
			<div class="kpi-meta">
				<span class="kpi-label">Mémoire RAM</span>
				<span class="kpi-val tabular-nums">
					{fmtNum(srv.memoryPercent, 0)}{srv.memoryPercent == null ? '' : '%'}
				</span>
			</div>
		</div>
		<div class="kpi-progress-bar-track">
			<div
				class="kpi-progress-bar-fill bar-purple"
				style="--pct: {barWidth(srv.memoryPercent)}; transform: scaleX(calc(var(--pct) / 100));"
			></div>
		</div>
		<span class="kpi-sub-text tabular-nums">{formattedRam}</span>
	</div>

	<!-- 5. Performance TPS -->
	<div class="card kpi-card">
		<div class="kpi-top">
			<div class="kpi-icon-badge badge-green">
				<Gauge size={18} />
			</div>
			<div class="kpi-meta">
				<span class="kpi-label">TPS (Ticks/sec)</span>
				<span class="kpi-val tabular-nums">{srv.tps == null ? 'n/a' : srv.tps.toFixed(1)}</span>
			</div>
		</div>
		<div class="kpi-sub {tpsStatus.colorClass} flex-align">
			{#if tpsStatus.unknown}
				<HelpCircle size={14} />
			{:else if tpsStatus.ok}
				<Check size={14} />
			{:else}
				<AlertTriangle size={14} />
			{/if}
			<span>{tpsStatus.text}</span>
		</div>
	</div>
</div>

<style>
	.kpi-cards-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: var(--space-4);
	}

	.kpi-card {
		padding: var(--space-4);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 2px 8px rgba(0, 0, 0, 0.2);
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		gap: var(--space-3);
		min-height: 110px;
		transition: border-color var(--transition-fast), transform var(--transition-fast);
	}

	.kpi-card:hover {
		border-color: var(--border-focus);
	}

	.kpi-top {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.kpi-icon-badge {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
	}

	.badge-green {
		background-color: var(--accent-green-bg);
		color: var(--accent-green-text);
		border: 1px solid var(--accent-green-border);
	}

	.badge-purple {
		background-color: var(--accent-purple-bg);
		color: var(--accent-purple-text);
		border: 1px solid var(--accent-purple-border);
	}

	.badge-blue {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border: 1px solid var(--accent-blue-border);
	}

	.badge-danger {
		background-color: var(--danger-bg);
		color: var(--danger-text);
		border: 1px solid var(--danger-border);
	}

	.badge-muted {
		background-color: var(--bg-base);
		color: var(--text-muted);
		border: 1px solid var(--border);
	}

	.kpi-meta {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.kpi-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.kpi-val {
		font-size: 1.25rem;
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		line-height: 1.2;
		font-family: var(--font-mono);
	}

	.val-sub {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		font-weight: var(--font-weight-normal);
	}

	.kpi-sub {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.kpi-progress-bar-track {
		height: 4px;
		background: var(--bg-base);
		border-radius: var(--radius-badge);
		overflow: hidden;
		border: 1px solid var(--border-subtle);
		margin-top: 4px;
	}

	.kpi-progress-bar-fill {
		height: 100%;
		width: 100%;
		border-radius: var(--radius-badge);
		transform-origin: left;
		transition: transform var(--transition-fast);
	}

	.bar-blue {
		background-color: var(--accent-blue);
		box-shadow: 0 0 6px rgba(59, 130, 246, 0.4);
	}

	.bar-purple {
		background-color: var(--accent-purple);
		box-shadow: 0 0 6px rgba(139, 92, 246, 0.4);
	}

	.kpi-sub-text {
		font-size: 0.6875rem;
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.flex-align {
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}

	.tabular-nums {
		font-variant-numeric: tabular-nums;
	}

	.text-green { color: var(--accent-green-text); }
	.text-warning { color: var(--warning-text, #FBBF24); }
	.text-danger { color: var(--danger-text); }
	.text-muted { color: var(--text-muted); }
</style>
