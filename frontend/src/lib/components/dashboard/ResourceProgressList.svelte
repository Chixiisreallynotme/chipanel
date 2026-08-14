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
				<span class="resource-value">{fmtNum(srv.cpuPercent, 1)}{srv.cpuPercent == null ? '' : '%'}</span>
			</div>
			<div class="progress-track">
				<div class="progress-bar bar-blue" style="width: {barWidth(srv.cpuPercent)}%"></div>
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
					<span class="resource-value">
						{fmtNum(srv.memoryPercent, 0)}{srv.memoryPercent == null ? '' : '%'}
					</span>
					<span class="resource-sub">{formattedRam}</span>
				</div>
			</div>
			<div class="progress-track">
				<div class="progress-bar bar-purple" style="width: {barWidth(srv.memoryPercent)}%"></div>
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
				<div class="progress-bar bar-green" style="width: {podman.width}%"></div>
			</div>
		</div>
	</div>
</div>

<style>
	.resources-card {
		height: 100%;
	}

	.resources-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-5);
		padding: var(--space-6);
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
		border-radius: var(--radius-badge);
		transition: width var(--transition-fast) ease-out;
	}

	.bar-blue {
		background-color: var(--accent-blue);
		box-shadow: 0 0 10px rgba(59, 130, 246, 0.4);
	}

	.bar-purple {
		background-color: var(--accent-purple);
		box-shadow: 0 0 10px rgba(139, 92, 246, 0.4);
	}

	.bar-green {
		background-color: var(--accent-green);
		box-shadow: 0 0 10px rgba(16, 185, 129, 0.4);
	}

	.text-blue { color: var(--accent-blue-text); }
	.text-purple { color: var(--accent-purple-text); }
	.text-green { color: var(--accent-green); }
	.text-danger { color: var(--danger-text); }
	.text-muted { color: var(--text-muted); }
</style>
