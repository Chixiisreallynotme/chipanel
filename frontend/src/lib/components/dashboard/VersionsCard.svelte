<script>
	import { serverState } from './serverState.js';
	import { Sliders, CheckCircle2, XCircle, HelpCircle } from 'lucide-svelte';

	const UNKNOWN = 'inconnu';

	let {
		/** `null` when /api/server/engine could not read the quadlet — never defaulted. */
		engineType = null,
		engineVersion = null
	} = $props();

	let srv = $derived(serverState());
</script>

<div class="card versions-card">
	<div class="card-header">
		<h3 class="card-title">Moteur & Spécifications</h3>
		<a href="/engine" class="btn btn-secondary btn-sm flex-align">
			<Sliders size={14} />
			<span>Modifier</span>
		</a>
	</div>

	<div class="card-body versions-body">
		<div class="version-row">
			<span class="version-label">Moteur du serveur</span>
			<span class="version-val {engineType ? '' : 'text-muted'}">{engineType ?? UNKNOWN}</span>
		</div>

		<div class="version-row">
			<span class="version-label">Version de Minecraft</span>
			<span class="version-val {engineVersion ? '' : 'text-muted'}">{engineVersion ?? UNKNOWN}</span>
		</div>

		<div class="version-row">
			<span class="version-label">Port de Jeu (TCP)</span>
			<span class="version-val code-val">25565</span>
		</div>

		<div class="version-row">
			<span class="version-label">Port RCON</span>
			<span class="version-val code-val">25575</span>
		</div>

		<div class="version-row">
			<span class="version-label">Statut du Processus</span>
			<span class="version-val flex-align">
				{#if srv.containerRunning === true}
					<CheckCircle2 size={14} class="text-green" />
					<span class="text-green">En cours d'exécution</span>
				{:else if srv.containerRunning === false}
					<XCircle size={14} class="text-danger" />
					<span class="text-danger">Arrêté</span>
				{:else}
					<HelpCircle size={14} class="text-muted" />
					<span class="text-muted">Indisponible</span>
				{/if}
			</span>
		</div>
	</div>
</div>

<style>
	.versions-card {
		height: 100%;
	}

	.versions-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-6);
	}

	.version-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: var(--space-2);
		border-bottom: 1px solid var(--border-subtle);
		font-size: var(--font-size-sm);
	}

	.version-row:last-child {
		border-bottom: none;
		padding-bottom: 0;
	}

	.version-label {
		color: var(--text-muted);
	}

	.version-val {
		color: var(--text-primary);
		font-weight: var(--font-weight-medium);
	}

	.code-val {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		background: var(--bg-base);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
	}

	.flex-align {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.text-green { color: var(--accent-green); }
	.text-danger { color: var(--danger-text); }
	.text-muted { color: var(--text-muted); }
</style>
