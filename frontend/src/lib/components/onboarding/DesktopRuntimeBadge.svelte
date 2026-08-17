<script>
	import { preferences } from '$lib/stores/preferences.svelte.js';
	import { Laptop, Server, Cpu, CheckCircle2, ShieldCheck, Activity } from 'lucide-svelte';

	let { showDetails = false } = $props();

	let runtimeLabel = $derived.by(() => {
		if (preferences.isDesktop) return 'Desktop App (Tauri v2)';
		if (preferences.runtimeStatus.podman) return 'Homelab (Podman Rootless)';
		if (preferences.runtimeStatus.docker) return 'Conteneur Docker';
		return 'Localhost (Bare-Metal Java)';
	});
</script>

<div class="runtime-badge-container">
	<div class="runtime-pill" title={`Environnement détecté : ${runtimeLabel}`}>
		{#if preferences.isDesktop}
			<Laptop size={13} class="text-blue" />
		{:else}
			<Server size={13} class="text-green" />
		{/if}
		<span class="runtime-text">{runtimeLabel}</span>
		<span class="status-dot status-dot-success"></span>
	</div>

	{#if showDetails}
		<div class="runtime-details-grid">
			<div class="detail-item">
				<div class="detail-label">
					<Cpu size={12} class="detail-icon" />
					<span>Moteur Conteneur</span>
				</div>
				<div class="detail-value">
					{#if preferences.runtimeStatus.podman}
						<span class="badge badge-success">Podman Rootless (Actif)</span>
					{:else if preferences.runtimeStatus.docker}
						<span class="badge badge-blue">Docker Engine</span>
					{:else}
						<span class="badge">Non requis</span>
					{/if}
				</div>
			</div>

			<div class="detail-item">
				<div class="detail-label">
					<Activity size={12} class="detail-icon" />
					<span>Java Détecté</span>
				</div>
				<div class="detail-value font-mono">
					{preferences.runtimeStatus.javaVersion || 'Non détecté'}
				</div>
			</div>

			<div class="detail-item">
				<div class="detail-label">
					<ShieldCheck size={12} class="detail-icon" />
					<span>Isolation & Veille</span>
				</div>
				<div class="detail-value">
					<span class="badge badge-success">lazymc proxy 25565</span>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.runtime-badge-container {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.runtime-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		background-color: rgba(255, 255, 255, 0.03);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-badge);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		user-select: none;
		width: fit-content;
	}

	.runtime-text {
		font-weight: var(--font-weight-medium);
	}

	.text-blue {
		color: var(--accent-blue-text);
	}

	.text-green {
		color: var(--accent-green);
	}

	.runtime-details-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: 8px;
		padding: 10px;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
	}

	.detail-item {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.detail-label {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 11px;
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.detail-icon {
		color: var(--text-muted);
	}

	.detail-value {
		font-size: var(--font-size-xs);
		color: var(--text-primary);
	}
</style>
