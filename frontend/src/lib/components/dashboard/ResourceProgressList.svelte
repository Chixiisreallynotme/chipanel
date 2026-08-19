<script>
	import { serverState, fmtNum, barWidth, UNAVAILABLE } from './serverState.js';
	import { Cpu, HardDrive, Layers } from 'lucide-svelte';

	let srv = $derived(serverState());

	let formattedRam = $derived.by(() => {
		if (srv.memoryMb == null && srv.memoryLimitMb == null) return UNAVAILABLE;
		const used = srv.memoryMb == null ? UNAVAILABLE : (srv.memoryMb / 1024).toFixed(1);
		const total = srv.memoryLimitMb == null ? UNAVAILABLE : (srv.memoryLimitMb / 1024).toFixed(0);
		return `${used} Go / ${total} Go`;
	});

	// "Podman : Actif" must never appear while podman_available is false/unknown (B1).
	let podman = $derived.by(() => {
		if (srv.podmanAvailable === true) return { text: 'Actif', cls: 'text-green', width: 100 };
		if (srv.podmanAvailable === false) return { text: 'Indisponible', cls: 'text-danger', width: 0 };
		return { text: 'Inconnu', cls: 'text-muted', width: 0 };
	});

	let feedBadge = $derived.by(() => {
		if (srv.cpuPercent != null || srv.memoryPercent != null) {
			return { text: 'Temps réel', cls: 'badge-success' };
		}
		return { text: 'Mesures indisponibles', cls: 'badge-danger' };
	});
</script>

<div class="card resources-card">
	<div class="card-header">
		<h3 class="card-title">Utilisation des Ressources</h3>
		<span class="badge {feedBadge.cls}">{feedBadge.text}</span>
	</div>

	<div class="card-body resources-body">
		<!-- CPU Progress Bar -->
		<div class="resource-item">
			<div class="resource-label-row">
				<span class="resource-name">
					<Cpu size={16} class="resource-icon text-blue" />
					<span>Processeur (CPU)</span>
				</span>
				<span class="resource-value tabular-nums">{fmtNum(srv.cpuPercent, 1)}{srv.cpuPercent == null ? '' : '%'}</span>
			</div>
			<div class="progress-track">
				<div
					class="progress-bar bar-blue"
					style="--pct: {barWidth(srv.cpuPercent)}; transform: scaleX(calc(var(--pct) / 100));"
				></div>
			</div>
		</div>

		<!-- RAM Progress Bar -->
		<div class="resource-item">
			<div class="resource-label-row">
				<span class="resource-name">
					<HardDrive size={16} class="resource-icon text-purple" />
					<span>Mémoire (RAM)</span>
				</span>
				<div class="resource-val-group">
					<span class="resource-value tabular-nums">
						{fmtNum(srv.memoryPercent, 0)}{srv.memoryPercent == null ? '' : '%'}
					</span>
					<span class="resource-sub tabular-nums">{formattedRam}</span>
				</div>
			</div>
			<div class="progress-track">
				<div
					class="progress-bar bar-purple"
					style="--pct: {barWidth(srv.memoryPercent)}; transform: scaleX(calc(var(--pct) / 100));"
				></div>
			</div>
		</div>

		<!-- Podman Socket State -->
		<div class="resource-item">
			<div class="resource-label-row">
				<span class="resource-name">
					<Layers size={16} class="resource-icon text-green" />
					<span>Conteneur Rootless Podman</span>
				</span>
				<span class="resource-value {podman.cls}">{podman.text}</span>
			</div>
			<div class="progress-track">
				<div
					class="progress-bar bar-green"
					style="--pct: {podman.width}; transform: scaleX(calc(var(--pct) / 100));"
				></div>
			</div>
		</div>
	</div>
</div>

<style>
	.resources-card {
		height: 100%;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.resources-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
		padding: var(--space-5) var(--space-6);
	}

	.resource-item {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.resource-label-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: var(--font-size-sm);
	}

	.resource-name {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-weight: var(--font-weight-medium);
		color: var(--text-primary);
	}

	.resource-icon {
		flex-shrink: 0;
	}

	.resource-value {
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		font-family: var(--font-mono);
	}

	.resource-val-group {
		display: flex;
		align-items: baseline;
		gap: var(--space-2);
	}

	.resource-sub {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.progress-track {
		height: 6px;
		background-color: var(--bg-base);
		border-radius: var(--radius-badge);
		overflow: hidden;
		border: 1px solid var(--border-subtle);
	}

	.progress-bar {
		height: 100%;
		width: 100%;
		border-radius: var(--radius-badge);
		transform-origin: left;
		transition: transform var(--transition-fast);
	}

	.bar-blue {
		background-color: var(--accent-blue);
	}

	.bar-purple {
		background-color: var(--accent-purple);
	}

	.bar-green {
		background-color: var(--accent-green);
	}

	.tabular-nums {
		font-variant-numeric: tabular-nums;
	}

	.text-blue { color: var(--accent-blue-text); }
	.text-purple { color: var(--accent-purple-text); }
	.text-green { color: var(--accent-green-text); }
	.text-danger { color: var(--danger-text); }
	.text-muted { color: var(--text-muted); }
</style>
