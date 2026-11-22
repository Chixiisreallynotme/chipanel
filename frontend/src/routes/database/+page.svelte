<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';
	import Callout from '$lib/components/ui/Callout.svelte';
	import KpiCard from '$lib/components/ui/KpiCard.svelte';
	import KpiGrid from '$lib/components/ui/KpiGrid.svelte';
	import {
		Database,
		HardDrive,
		Trash2,
		RefreshCw,
		AlertTriangle,
		CheckCircle2,
		ShieldAlert,
		FileArchive,
		Flame,
		Sparkles,
		RotateCw,
		Terminal,
		Info,
		Layers
	} from '$lib/icons.js';

	/** @type {{ total_storage_bytes: number, formatted_total: string, coreprotect_db_bytes: number, luckperms_bytes: number, logs_bytes: number, nether_bytes: number, end_bytes: number, tables: Array<{ name: string, plugin: string, description: string, size_bytes: number, formatted_size: string }>, db_type: string } | null} */
	let stats = $state(null);
	let loading = $state(true);
	let error = $state('');

	// Action modals & states
	let activeModal = $state(null); // 'coreprotect' | 'logs' | 'nether' | 'end' | null
	let purgeDays = $state(30);
	let isExecuting = $state(false);
	let executionResult = $state(null);

	onMount(() => {
		loadStats();
	});

	async function loadStats() {
		loading = true;
		error = '';
		try {
			stats = await apiGet('/api/maintenance/db-stats');
		} catch (err) {
			error = err.message || 'Erreur lors du chargement des données';
		} finally {
			loading = false;
		}
	}

	async function handlePurge() {
		if (!activeModal || isExecuting) return;
		isExecuting = true;
		executionResult = null;

		try {
			let payload = { target: activeModal };
			if (activeModal === 'coreprotect') {
				payload.days = purgeDays;
			} else if (activeModal === 'nether') {
				payload.target = 'dimension';
				payload.dimension = 'DIM-1';
			} else if (activeModal === 'end') {
				payload.target = 'dimension';
				payload.dimension = 'DIM1';
			}

			const data = await apiPost('/api/maintenance/purge-db', payload);
			if (!data.success) {
				throw new Error(data.message || data.error || 'Erreur lors de la purge');
			}

			executionResult = data;
			// Refresh stats
			await loadStats();
		} catch (err) {
			executionResult = {
				success: false,
				action: 'Erreur',
				output: err.message || 'Erreur lors de la purge'
			};
		} finally {
			isExecuting = false;
		}
	}

	function closeModal() {
		activeModal = null;
		executionResult = null;
	}
</script>

<svelte:head>
	<title>Base de Données & Maintenance | ChiPanel</title>
</svelte:head>

<div class="page-stack">
	<!-- Header Banner -->
	<PageHeader
		title="Base de Données & Maintenance"
		subtitle="Diagnostic d'empreinte disque, compaction et purge guidée SQLite / CoreProtect / Logs"
	>
		{#snippet icon()}
			<Database size={22} />
		{/snippet}
		{#snippet badge()}
			{#if stats}
				<span class="badge">{stats.db_type}</span>
			{/if}
		{/snippet}
		<button class="btn btn-secondary btn-sm" onclick={loadStats} disabled={loading}>
			<RefreshCw size={14} class={loading ? 'spin' : ''} />
			<span>Actualiser</span>
		</button>
	</PageHeader>

	{#if error}
		<Callout tone="danger" title="Base de données inaccessible">
			{error}
			{#snippet action()}
				<button class="btn btn-secondary btn-sm" onclick={loadStats}>Réessayer</button>
			{/snippet}
		</Callout>
	{/if}

	<!-- Overview KPI Grid -->
	<KpiGrid>
		<KpiCard label="Empreinte Totale Détectée" tone="info">
			{#snippet icon()}<HardDrive size={20} />{/snippet}
			{stats ? stats.formatted_total : '—'}
		</KpiCard>
		<KpiCard label="CoreProtect (Logs Blocs/Coffres)" tone="warning">
			{#snippet icon()}<Layers size={20} />{/snippet}
			{stats?.tables?.[0]?.formatted_size ?? '—'}
		</KpiCard>
		<KpiCard label="Archives Logs (.log.gz)" tone="info">
			{#snippet icon()}<FileArchive size={20} />{/snippet}
			{stats?.tables?.[2]?.formatted_size ?? '—'}
		</KpiCard>
		<KpiCard label="Dimensions (Nether + End)" tone="danger">
			{#snippet icon()}<Flame size={20} />{/snippet}
			{#if stats?.tables}
				{stats.tables[3]?.formatted_size ?? '—'} + {stats.tables[4]?.formatted_size ?? '—'}
			{:else}
				—
			{/if}
		</KpiCard>
	</KpiGrid>

	<!-- Storage Breakdown Table -->
	<div class="table-card card">
		<div class="card-header">
			<h2 class="card-title">Consommation par Module de Stockage</h2>
			<span class="card-subtitle">Analyse détaillée des fichiers lourds et options de purge</span>
		</div>

		<div class="table-container">
			<table class="data-table">
				<thead>
					<tr>
						<th>Module / Fichier</th>
						<th>Catégorie</th>
						<th>Description</th>
						<th>Poids Disque</th>
						<th>Actions de Maintenance</th>
					</tr>
				</thead>
				<tbody>
					{#if loading && !stats}
						<tr>
							<td colspan="5" class="text-center py-8">
								<RotateCw size={24} class="spin inline text-muted" />
								<span class="ml-2 text-muted">Analyse du stockage en cours...</span>
							</td>
						</tr>
					{:else if stats}
						{#each stats.tables as table}
							<tr>
								<td class="font-semibold">{table.name}</td>
								<td>
									<span class="badge badge-secondary">{table.plugin}</span>
								</td>
								<td class="text-muted">{table.description}</td>
								<td class="font-mono font-medium">{table.formatted_size}</td>
								<td>
									{#if table.plugin === 'CoreProtect'}
										<button
											class="btn btn-secondary btn-sm"
											onclick={() => (activeModal = 'coreprotect')}
										>
											<Trash2 size={13} />
											<span>Purger l'historique</span>
										</button>
									{:else if table.plugin === 'Système'}
										<button
											class="btn btn-secondary btn-sm"
											onclick={() => (activeModal = 'logs')}
										>
											<Trash2 size={13} />
											<span>Nettoyer les logs</span>
										</button>
									{:else if table.name.includes('Nether')}
										<button
											class="btn btn-ghost btn-sm text-danger"
											onclick={() => (activeModal = 'nether')}
										>
											<Flame size={13} />
											<span>Régénérer Nether</span>
										</button>
									{:else if table.name.includes('End')}
										<button
											class="btn btn-ghost btn-sm text-danger"
											onclick={() => (activeModal = 'end')}
										>
											<Sparkles size={13} />
											<span>Régénérer End</span>
										</button>
									{:else}
										<span class="text-muted text-xs">Automatique</span>
									{/if}
								</td>
							</tr>
						{/each}
					{/if}
				</tbody>
			</table>
		</div>
	</div>
</div>

<!-- Purge Modal Dialogs -->
{#if activeModal}
	<Modal
		open={activeModal !== null}
		onclose={() => { if (!isExecuting) closeModal(); }}
		class="modal-dialog"
		ariaLabelledBy="purge-modal-title"
	>
		{#snippet content()}
			<div class="modal-header">
				<div class="modal-header-left">
					{#if activeModal === 'coreprotect'}
						<ShieldAlert size={22} class="text-warning" />
						<h2 id="purge-modal-title">Purger l'historique CoreProtect</h2>
					{:else if activeModal === 'logs'}
						<FileArchive size={22} class="text-info" />
						<h2 id="purge-modal-title">Nettoyer les archives de logs</h2>
					{:else}
						<AlertTriangle size={22} class="text-danger" />
						<h2 id="purge-modal-title">Régénération de Dimension</h2>
					{/if}
				</div>
				<button class="btn btn-ghost btn-sm" onclick={closeModal} aria-label="Fermer">&times;</button>
			</div>

			<div class="modal-body">
				{#if executionResult}
					<div class="result-box {executionResult.success ? 'result-success' : 'result-error'}">
						<div class="result-header">
							{#if executionResult.success}
								<CheckCircle2 size={18} class="text-success" />
								<span>{executionResult.action}</span>
							{:else}
								<AlertTriangle size={18} class="text-danger" />
								<span>Échec de l'opération</span>
							{/if}
						</div>
						<pre class="result-output font-mono">{executionResult.output}</pre>
					</div>
				{:else}
					{#if activeModal === 'coreprotect'}
						<p class="modal-desc">
							La purge supprime les données de blocs, interactions et transactions antérieures à la période sélectionnée via la commande RCON officielle <code>/co purge</code>.
						</p>

						<div class="form-group">
							<label for="purge-days-input" class="form-label">Conserver les données des :</label>
							<div class="input-with-suffix">
								<input
									id="purge-days-input"
									type="number"
									class="input"
									min="1"
									max="365"
									bind:value={purgeDays}
								/>
								<span class="suffix">derniers jours</span>
							</div>
							<span class="form-hint">Ex: 30 jours supprimera tout l'historique antérieur à un mois.</span>
						</div>
					{:else if activeModal === 'logs'}
						<p class="modal-desc">
							Cette opération supprimera définitivement tous les fichiers d'archives <code>.log.gz</code> compressés dans le dossier <code>logs/</code> pour libérer immédiatement de l'espace disque.
						</p>
					{:else if activeModal === 'nether' || activeModal === 'end'}
						<div class="warning-callout">
							<AlertTriangle size={20} class="text-danger flex-shrink-0" />
							<div>
								<strong>Attention : Opération Irréversible !</strong>
								<p>
									Toutes les constructions et coffres situés dans {activeModal === 'nether' ? 'le Nether' : "l'End"} seront supprimés. Les chunks seront régénérés selon la graine (seed) du monde à la prochaine visite d'un joueur.
								</p>
							</div>
						</div>
					{/if}
				{/if}
			</div>

			<div class="modal-footer">
				{#if executionResult}
					<button class="btn btn-secondary" onclick={closeModal}>Fermer</button>
				{:else}
					<button class="btn btn-secondary" onclick={closeModal} disabled={isExecuting}>
						Annuler
					</button>
					<button
						class="btn {activeModal === 'nether' || activeModal === 'end' ? 'btn-danger' : 'btn-primary'}"
						onclick={handlePurge}
						disabled={isExecuting}
					>
						{#if isExecuting}
							<RotateCw size={14} class="spin" />
							<span>Exécution en cours...</span>
						{:else}
							<span>Confirmer la Purge</span>
						{/if}
					</button>
				{/if}
			</div>
		{/snippet}
	</Modal>
{/if}

<style>

	/* Table Card */
	.table-card {
		background-color: var(--bg-surface);
		overflow: hidden;
	}

	.card-header {
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
	}

	.card-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin: 0;
	}

	.card-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.table-container {
		overflow-x: auto;
	}

	.data-table {
		width: 100%;
		border-collapse: collapse;
		text-align: left;
		font-size: var(--font-size-sm);
	}

	.data-table th {
		padding: var(--space-3) var(--space-6);
		background-color: var(--bg-surface);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
		border-bottom: 1px solid var(--border);
	}

	.data-table td {
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border-subtle);
		vertical-align: middle;
	}

	.data-table tr:hover td {
		background-color: rgba(255, 255, 255, 0.02);
	}

	/* Modals */

	.modal-dialog {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
		width: 100%;
		max-width: 550px;
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
		background-color: var(--bg-surface);
	}

	.modal-header-left {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.modal-header h2 {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin: 0;
	}

	.modal-body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.modal-desc {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		line-height: 1.5;
		margin: 0;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.form-label {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--text-muted);
	}

	.input-with-suffix {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.suffix {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}

	.form-hint {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.warning-callout {
		display: flex;
		gap: var(--space-3);
		padding: var(--space-4);
		background-color: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.3);
		border-radius: var(--radius-sm);
		font-size: var(--font-size-sm);
		color: var(--danger-text);
	}

	.warning-callout p {
		margin: var(--space-1) 0 0 0;
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.result-box {
		padding: var(--space-4);
		border-radius: var(--radius-sm);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.result-success {
		background-color: rgba(16, 185, 129, 0.1);
		border: 1px solid rgba(16, 185, 129, 0.3);
	}

	.result-error {
		background-color: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.3);
	}

	.result-header {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-weight: var(--font-weight-medium);
		font-size: var(--font-size-sm);
	}

	.result-output {
		background-color: var(--bg-code);
		padding: var(--space-3);
		border-radius: var(--radius-sm);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		max-height: 150px;
		overflow-y: auto;
		white-space: pre-wrap;
	}

	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-6);
		border-top: 1px solid var(--border);
		background-color: var(--bg-surface);
	}

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
