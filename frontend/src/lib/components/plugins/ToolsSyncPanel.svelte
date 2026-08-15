<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import { RefreshCw, Zap, Loader2, CheckCircle2, AlertCircle, Wrench } from 'lucide-svelte';

	let { onToast } = $props();

	/** @type {{ engine?: string, version?: string, config_source?: string, supported?: boolean, target_dir?: string, managed?: string[] } | null} */
	let status = $state(null);
	let syncing = $state(false);
	/** @type {{ engine?: string, target_dir?: string, outcomes?: Array<{tool:string,status:string,version?:string}> , error?: string } | null} */
	let result = $state(null);

	async function loadStatus() {
		try {
			status = await apiGet('/api/tools/status');
		} catch (err) {
			status = { supported: false, engine: 'inconnu', version: '', config_source: 'error', managed: [] };
			onToast?.('error', 'Outils indisponibles', err?.message || 'Impossible de lire le statut des outils.');
		}
	}

	async function handleSync() {
		syncing = true;
		result = null;
		try {
			result = await apiPost('/api/tools/sync', {});
			const installed = result?.outcomes?.filter((o) => o.status === 'installed' || o.status === 'updated');
			onToast?.(
				'success',
				'Outils synchronisés',
				installed?.length ? `${installed.length} outil(s) installé(s)/mis à jour.` : 'Spark & Chunky sont déjà à jour.'
			);
		} catch (err) {
			onToast?.('error', 'Synchronisation échouée', err?.message || 'Échec de la synchronisation des outils.');
		} finally {
			syncing = false;
			await loadStatus();
		}
	}

	onMount(() => {
		loadStatus();
	});
</script>

<div class="tools-sync-card">
	<div class="tools-header">
		<div class="tools-icon-box">
			<Wrench size={18} />
		</div>
		<div>
			<h3 class="tools-title">Outils de performance — spark & chunky</h3>
			<p class="tools-subtitle">
				Installation et mise à jour automatiques selon le moteur et la version du serveur.
			</p>
		</div>
		<button type="button" class="btn btn-primary sync-btn" disabled={syncing} onclick={handleSync}>
			{#if syncing}
				<Loader2 size={16} class="spin" />
				<span>Synchronisation…</span>
			{:else}
				<RefreshCw size={16} />
				<span>Synchroniser</span>
			{/if}
		</button>
	</div>

	<div class="tools-body">
		{#if status === null}
			<p class="tools-muted">Chargement du statut…</p>
		{:else if !status.supported}
			<div class="status-row status-warn">
				<AlertCircle size={16} />
				<span>
					Moteur « {status.engine || 'inconnu'} » non pris en charge — spark/chunky indisponibles pour ce moteur.
				</span>
			</div>
		{:else}
			<div class="status-row status-ok">
				<CheckCircle2 size={16} />
				<span>
					Moteur <strong>{status.engine}</strong> (MC {status.version}) → cible
					<code>/{status.target_dir}/</code>
				</span>
			</div>
			<div class="managed-row">
				<span class="tools-muted">Géré automatiquement :</span>
				{#each (status.managed || []) as tool}
					<span class="tool-chip">{tool}</span>
				{/each}
			</div>
		{/if}

		{#if result?.outcomes?.length}
			<div class="outcomes">
				{#each result.outcomes as o}
					<div class="outcome-row">
						<Zap size={14} class={o.status === 'error' ? 'text-red' : o.status === 'up_to_date' ? 'text-muted' : 'text-green'} />
						<span class="outcome-tool">{o.tool}</span>
						<span class="outcome-status">{o.status}</span>
						{#if o.version}<span class="outcome-version">v{o.version}</span>{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.tools-sync-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-4) var(--space-5);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		margin-bottom: var(--space-4);
	}
	.tools-header {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.tools-icon-box {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-input);
		background-color: var(--accent-blue-bg, rgba(59, 130, 246, 0.12));
		border: 1px solid var(--accent-blue-border, rgba(59, 130, 246, 0.25));
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text, #60a5fa);
		flex-shrink: 0;
	}
	.tools-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		margin: 0;
	}
	.tools-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		margin: 0;
	}
	.sync-btn {
		margin-left: auto;
	}
	.tools-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.status-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
	}
	.status-ok {
		color: var(--accent-green-text, #34d399);
	}
	.status-warn {
		color: var(--warning, #f59e0b);
	}
	.status-row code {
		font-family: var(--font-mono);
	}
	.managed-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	.tool-chip {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: 2px 8px;
		color: var(--text-secondary);
	}
	.tools-muted {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}
	.outcomes {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		margin-top: var(--space-1);
	}
	.outcome-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
	}
	.outcome-tool {
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}
	.outcome-status {
		color: var(--text-secondary);
	}
	.outcome-version {
		color: var(--text-muted);
	}
	.text-green {
		color: var(--accent-green, #10b981);
	}
	.text-red {
		color: var(--danger, #ef4444);
	}
	.text-muted {
		color: var(--text-muted);
	}
	.spin {
		animation: spin 1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
