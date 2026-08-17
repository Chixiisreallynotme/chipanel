<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
	import {
		Archive,
		HardDrive,
		Download,
		RotateCcw,
		Trash2,
		Plus,
		Settings2,
		CloudUpload,
		ShieldCheck,
		CheckCircle2,
		AlertCircle,
		AlertTriangle,
		RefreshCw,
		Check,
		Copy,
		X,
		Layers,
		FileCode,
		Globe,
		Server
	} from 'lucide-svelte';

	/** @type {Array<{ filename: string, file_size_bytes: number, created_at_secs: number, scope: string, sha256: string, format: string, file_count: number }>} */
	let backups = $state([]);
	let loading = $state(true);
	let error = $state('');

	// Backup settings state
	let settings = $state({
		max_retention_count: 7,
		default_exclusions: [],
		compression_level: 3,
		s3_config: null
	});

	// Filters
	let scopeFilter = $state('ALL'); // 'ALL' | 'full' | 'world_only' | 'configs_only'

	// Modals
	let showCreateModal = $state(false);
	let showSettingsModal = $state(false);
	let showRestoreModal = $state(false);
	let selectedBackupForRestore = $state(null);

	// Create backup form
	let newBackupName = $state('');
	let newBackupScope = $state('full');
	let isCreating = $state(false);

	// Restore in flight
	let isRestoring = $state(false);
	let isExportingS3 = $state({});
	let isSavingSettings = $state(false);

	// Toasts
	let toasts = $state([]);
	function addToast(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		toasts = [...toasts, { id, type, title, message }];
		setTimeout(() => {
			toasts = toasts.filter((t) => t.id !== id);
		}, 4500);
	}

	onMount(() => {
		loadBackups();
		loadSettings();
	});

	async function loadBackups() {
		loading = true;
		error = '';
		try {
			const res = await apiGet('/api/backups');
			backups = res || [];
		} catch (err) {
			error = err.message || 'Impossible de charger la liste des sauvegardes.';
		} finally {
			loading = false;
		}
	}

	async function loadSettings() {
		try {
			const res = await apiGet('/api/backups/settings');
			if (res) {
				settings = res;
			}
		} catch (err) {
			console.error('Failed to load backup settings:', err);
		}
	}

	async function handleCreateBackup() {
		isCreating = true;
		try {
			const res = await apiPost('/api/backups/create', {
				name: newBackupName.trim() || undefined,
				scope: newBackupScope
			});
			addToast('success', 'Sauvegarde créée', `L'archive "${res.filename}" a été générée avec succès.`);
			showCreateModal = false;
			newBackupName = '';
			await loadBackups();
		} catch (err) {
			addToast('danger', 'Échec de la sauvegarde', err.message || 'Erreur lors de la création de la sauvegarde.');
		} finally {
			isCreating = false;
		}
	}

	async function handleRestoreBackup() {
		if (!selectedBackupForRestore) return;
		isRestoring = true;
		try {
			const res = await apiPost('/api/backups/restore', {
				filename: selectedBackupForRestore.filename
			});
			addToast('success', 'Restauration réussie', res.message);
			showRestoreModal = false;
			selectedBackupForRestore = null;
		} catch (err) {
			addToast('danger', 'Échec de la restauration', err.message || 'Erreur lors de la restauration.');
		} finally {
			isRestoring = false;
		}
	}

	async function handleDeleteBackup(filename) {
		if (!confirm(`Êtes-vous sûr de vouloir supprimer définitivement la sauvegarde "${filename}" ?`)) {
			return;
		}
		try {
			await apiFetch(`/api/backups/${encodeURIComponent(filename)}`, { method: 'DELETE' });
			addToast('success', 'Sauvegarde supprimée', `L'archive "${filename}" a été supprimée.`);
			backups = backups.filter((b) => b.filename !== filename);
		} catch (err) {
			addToast('danger', 'Échec de la suppression', err.message || 'Impossible de supprimer la sauvegarde.');
		}
	}

	async function handleExportS3(filename) {
		isExportingS3[filename] = true;
		try {
			const res = await apiPost('/api/backups/export/s3', { filename });
			addToast('success', 'Export S3 terminé', res.message);
		} catch (err) {
			addToast('danger', 'Échec de l\'export S3', err.message || 'Impossible d\'exporter vers S3/MinIO.');
		} finally {
			isExportingS3[filename] = false;
		}
	}

	async function handleSaveSettings() {
		isSavingSettings = true;
		try {
			await apiPost('/api/backups/settings', settings);
			addToast('success', 'Paramètres enregistrés', 'La politique de rétention et d\'exclusion a été mise à jour.');
			showSettingsModal = false;
		} catch (err) {
			addToast('danger', 'Erreur d\'enregistrement', err.message || 'Impossible de sauvegarder les paramètres.');
		} finally {
			isSavingSettings = false;
		}
	}

	function formatBytes(bytes) {
		if (!bytes || bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatDate(timestampSecs) {
		if (!timestampSecs) return '—';
		return new Date(timestampSecs * 1000).toLocaleString('fr-FR', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Computed KPIs
	let totalBytesUsed = $derived(backups.reduce((acc, b) => acc + (b.file_size_bytes || 0), 0));
	let filteredBackups = $derived(
		scopeFilter === 'ALL' ? backups : backups.filter((b) => b.scope === scopeFilter)
	);
</script>

<svelte:head>
	<title>Sauvegardes & Snapshots | ChiPanel</title>
</svelte:head>

<div class="backups-page-layout">
	<!-- Page Header Card -->
	<header class="page-header-card card">
		<div class="header-main">
			<div class="icon-badge">
				<Archive size={24} />
			</div>
			<div>
				<div class="title-with-badge">
					<h1 class="page-title">Sauvegardes & Snapshots</h1>
					<span class="badge badge-primary">Rétention & Exclusions</span>
				</div>
				<p class="page-subtitle">
					Gestion unifiée des archives du serveur (Monde, Plugins, Configurations) avec compression et export distant S3.
				</p>
			</div>
		</div>

		<div class="header-actions">
			<button class="btn btn-secondary btn-sm" onclick={() => (showSettingsModal = true)}>
				<Settings2 size={14} />
				<span>Paramètres & S3</span>
			</button>
			<button class="btn btn-primary btn-sm" onclick={() => (showCreateModal = true)}>
				<Plus size={14} />
				<span>Nouvelle Sauvegarde</span>
			</button>
		</div>
	</header>

	<!-- KPI Stats Grid -->
	<div class="kpi-grid">
		<div class="kpi-card card">
			<div class="kpi-icon-wrapper kpi-primary">
				<Archive size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Nombre de Sauvegardes</span>
				<span class="kpi-value">{backups.length}</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper kpi-success">
				<HardDrive size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Espace Disque Utilisé</span>
				<span class="kpi-value">{formatBytes(totalBytesUsed)}</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper kpi-info">
				<ShieldCheck size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Rétention Max</span>
				<span class="kpi-value">{settings.max_retention_count} archives max</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper {settings.s3_config?.enabled ? 'kpi-success' : 'kpi-warning'}">
				<CloudUpload size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Export Distant S3 / MinIO</span>
				<span class="kpi-value">{settings.s3_config?.enabled ? 'Actif' : 'Désactivé'}</span>
			</div>
		</div>
	</div>

	<!-- Main Backups Table Card -->
	<div class="card table-card">
		<div class="table-header-row">
			<div class="filter-pills">
				<button class="pill-btn {scopeFilter === 'ALL' ? 'active' : ''}" onclick={() => (scopeFilter = 'ALL')}>
					Toutes ({backups.length})
				</button>
				<button class="pill-btn {scopeFilter === 'full' ? 'active' : ''}" onclick={() => (scopeFilter = 'full')}>
					Complètes ({backups.filter((b) => b.scope === 'full').length})
				</button>
				<button class="pill-btn {scopeFilter === 'world_only' ? 'active' : ''}" onclick={() => (scopeFilter = 'world_only')}>
					Mondes seuls ({backups.filter((b) => b.scope === 'world_only').length})
				</button>
				<button class="pill-btn {scopeFilter === 'configs_only' ? 'active' : ''}" onclick={() => (scopeFilter = 'configs_only')}>
					Configs ({backups.filter((b) => b.scope === 'configs_only').length})
				</button>
			</div>

			<button class="btn btn-ghost btn-sm" onclick={loadBackups} disabled={loading}>
				<RefreshCw size={14} class={loading ? 'spin' : ''} />
				<span>Actualiser</span>
			</button>
		</div>

		{#if loading}
			<div class="loading-state">
				<RefreshCw size={24} class="spin text-muted" />
				<span>Chargement des archives...</span>
			</div>
		{:else if filteredBackups.length === 0}
			<div class="empty-state">
				<Archive size={40} class="text-muted" />
				<p>Aucune sauvegarde disponible pour ce filtre.</p>
				<button class="btn btn-primary btn-sm" onclick={() => (showCreateModal = true)}>
					Créer une première sauvegarde
				</button>
			</div>
		{:else}
			<div class="table-wrapper">
				<table class="data-table">
					<thead>
						<tr>
							<th>Nom de l'Archive</th>
							<th>Type</th>
							<th>Taille</th>
							<th>Date de Création</th>
							<th class="text-right">Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredBackups as backup}
							<tr>
								<td>
									<div class="backup-name-cell">
										<Archive size={16} class="text-muted flex-shrink-0" />
										<span class="backup-filename font-mono">{backup.filename}</span>
									</div>
								</td>
								<td>
									{#if backup.scope === 'full'}
										<span class="badge badge-primary">Complète</span>
									{:else if backup.scope === 'world_only'}
										<span class="badge badge-success">Monde seul</span>
									{:else}
										<span class="badge badge-warning">Configs & Plugins</span>
									{/if}
								</td>
								<td class="font-mono text-sm">{formatBytes(backup.file_size_bytes)}</td>
								<td class="text-sm text-muted">{formatDate(backup.created_at_secs)}</td>
								<td>
									<div class="actions-cell">
										{#if settings.s3_config?.enabled}
											<button
												class="btn btn-ghost btn-sm btn-icon"
												onclick={() => handleExportS3(backup.filename)}
												disabled={isExportingS3[backup.filename]}
												title="Exporter vers S3 / MinIO"
											>
												<CloudUpload size={15} class={isExportingS3[backup.filename] ? 'spin' : ''} />
											</button>
										{/if}

										<button
											class="btn btn-secondary btn-sm"
											onclick={() => {
												selectedBackupForRestore = backup;
												showRestoreModal = true;
											}}
											title="Restaurer cette sauvegarde"
										>
											<RotateCcw size={13} />
											<span>Restaurer</span>
										</button>

										<button
											class="btn btn-danger-ghost btn-sm btn-icon"
											onclick={() => handleDeleteBackup(backup.filename)}
											title="Supprimer"
										>
											<Trash2 size={15} />
										</button>
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</div>

<!-- Modal Créer Sauvegarde -->
{#if showCreateModal}
	<div class="modal-backdrop" onclick={() => (showCreateModal = false)} role="presentation">
		<div class="modal-card card" onclick={(e) => e.stopPropagation()} role="dialog">
			<div class="modal-header">
				<h2 class="modal-title">Créer une Sauvegarde</h2>
				<button class="btn btn-ghost btn-sm btn-icon" onclick={() => (showCreateModal = false)}>
					<X size={16} />
				</button>
			</div>

			<div class="modal-body">
				<div class="form-group">
					<label for="backup-name" class="form-label">Nom personnalisé (Optionnel) :</label>
					<input
						id="backup-name"
						type="text"
						class="input"
						placeholder="ex: pre_update_1_21"
						bind:value={newBackupName}
					/>
				</div>

				<div class="form-group">
					<span class="form-label">Périmètre de la Sauvegarde :</span>
					<div class="scope-options">
						<label class="scope-option card {newBackupScope === 'full' ? 'selected' : ''}">
							<input type="radio" name="scope" value="full" bind:group={newBackupScope} />
							<div class="scope-text">
								<strong>Sauvegarde Complète (Recommandé)</strong>
								<span>Monde, plugins, configurations, permissions et propriétés.</span>
							</div>
						</label>

						<label class="scope-option card {newBackupScope === 'world_only' ? 'selected' : ''}">
							<input type="radio" name="scope" value="world_only" bind:group={newBackupScope} />
							<div class="scope-text">
								<strong>Monde Uniquement</strong>
								<span>Uniquement les dossiers de mondes et régions de chunks.</span>
							</div>
						</label>

						<label class="scope-option card {newBackupScope === 'configs_only' ? 'selected' : ''}">
							<input type="radio" name="scope" value="configs_only" bind:group={newBackupScope} />
							<div class="scope-text">
								<strong>Configs & Plugins</strong>
								<span>Configurations, plugins et métadonnées sans les lourdes régions.</span>
							</div>
						</label>
					</div>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={() => (showCreateModal = false)}>Annuler</button>
				<button class="btn btn-primary" onclick={handleCreateBackup} disabled={isCreating}>
					{#if isCreating}
						<RefreshCw size={14} class="spin" />
						<span>Compression en cours...</span>
					{:else}
						<Archive size={14} />
						<span>Lancer la Sauvegarde</span>
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Modal Paramètres & S3 -->
{#if showSettingsModal}
	<div class="modal-backdrop" onclick={() => (showSettingsModal = false)} role="presentation">
		<div class="modal-card card" onclick={(e) => e.stopPropagation()} role="dialog">
			<div class="modal-header">
				<h2 class="modal-title">Paramètres de Rétention & Export S3</h2>
				<button class="btn btn-ghost btn-sm btn-icon" onclick={() => (showSettingsModal = false)}>
					<X size={16} />
				</button>
			</div>

			<div class="modal-body">
				<div class="form-group">
					<label for="retention-count" class="form-label">Nombre d'archives à conserver (Rétention) :</label>
					<input
						id="retention-count"
						type="number"
						class="input"
						min="1"
						max="50"
						bind:value={settings.max_retention_count}
					/>
					<span class="form-hint">Les archives les plus anciennes seront automatiquement purgées au-delà de cette limite.</span>
				</div>

				<div class="divider"></div>

				<div class="s3-section">
					<div class="s3-header">
						<CloudUpload size={20} class="text-primary" />
						<div>
							<h3 class="section-heading">Stockage Distant S3 / MinIO</h3>
							<span class="section-sub">Exportez vos archives vers un bucket Cloudflare R2, MinIO, Scaleway ou AWS S3.</span>
						</div>
					</div>

					<div class="form-group">
						<label class="checkbox-label">
							<input
								type="checkbox"
								checked={settings.s3_config?.enabled ?? false}
								onchange={(e) => {
									if (!settings.s3_config) {
										settings.s3_config = {
											endpoint: 'https://s3.eu-west-1.amazonaws.com',
											bucket: '',
											region: 'eu-west-1',
											access_key: '',
											secret_key: '',
											path_prefix: 'minecraft-backups',
											enabled: e.target.checked
										};
									} else {
										settings.s3_config.enabled = e.target.checked;
									}
								}}
							/>
							<span>Activer l'intégration S3 / MinIO</span>
						</label>
					</div>

					{#if settings.s3_config?.enabled}
						<div class="form-group">
							<label for="s3-endpoint" class="form-label">Endpoint S3 :</label>
							<input id="s3-endpoint" type="text" class="input" bind:value={settings.s3_config.endpoint} />
						</div>

						<div class="grid-2">
							<div class="form-group">
								<label for="s3-bucket" class="form-label">Nom du Bucket :</label>
								<input id="s3-bucket" type="text" class="input" bind:value={settings.s3_config.bucket} />
							</div>
							<div class="form-group">
								<label for="s3-region" class="form-label">Région :</label>
								<input id="s3-region" type="text" class="input" bind:value={settings.s3_config.region} />
							</div>
						</div>

						<div class="form-group">
							<label for="s3-prefix" class="form-label">Préfixe / Dossier dans le bucket :</label>
							<input id="s3-prefix" type="text" class="input" bind:value={settings.s3_config.path_prefix} />
						</div>
					{/if}
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={() => (showSettingsModal = false)}>Fermer</button>
				<button class="btn btn-primary" onclick={handleSaveSettings} disabled={isSavingSettings}>
					Enregistrer les paramètres
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Modal Confirmation Restauration -->
{#if showRestoreModal && selectedBackupForRestore}
	<div class="modal-backdrop" onclick={() => (showRestoreModal = false)} role="presentation">
		<div class="modal-card card" onclick={(e) => e.stopPropagation()} role="dialog">
			<div class="modal-header">
				<h2 class="modal-title text-warning flex items-center gap-2">
					<AlertTriangle size={18} />
					<span>Confirmer la Restauration</span>
				</h2>
				<button class="btn btn-ghost btn-sm btn-icon" onclick={() => (showRestoreModal = false)}>
					<X size={16} />
				</button>
			</div>

			<div class="modal-body">
				<p>
					Vous êtes sur le point de restaurer la sauvegarde suivante :
				</p>
				<div class="restore-target-box card">
					<strong>{selectedBackupForRestore.filename}</strong>
					<span>Créée le {formatDate(selectedBackupForRestore.created_at_secs)} ({formatBytes(selectedBackupForRestore.file_size_bytes)})</span>
				</div>
				<div class="alert alert-warning">
					<AlertTriangle size={18} class="flex-shrink-0" />
					<span>
						Attention : La restauration remplacera les fichiers actuels du serveur Minecraft par ceux de l'archive.
					</span>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={() => (showRestoreModal = false)}>Annuler</button>
				<button class="btn btn-primary" onclick={handleRestoreBackup} disabled={isRestoring}>
					{#if isRestoring}
						<RefreshCw size={14} class="spin" />
						<span>Restauration en cours...</span>
					{:else}
						<RotateCcw size={14} />
						<span>Confirmer la Restauration</span>
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Floating Toasts -->
{#if toasts.length > 0}
	<div class="toast-stack">
		{#each toasts as toast (toast.id)}
			<div class="toast toast-{toast.type} card">
				{#if toast.type === 'success'}<CheckCircle2 size={16} class="text-success" />{/if}
				{#if toast.type === 'danger'}<AlertCircle size={16} class="text-danger" />{/if}
				<div>
					<strong>{toast.title}</strong>
					<p>{toast.message}</p>
				</div>
			</div>
		{/each}
	</div>
{/if}

<style>
	.backups-page-layout {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		padding-bottom: var(--space-8);
	}

	.page-header-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
		background-color: var(--bg-surface);
	}

	.header-main {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.icon-badge {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
	}

	.title-with-badge {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.page-title {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		margin: 0;
	}

	.page-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		margin: 2px 0 0 0;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	/* KPI Grid */
	.kpi-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-4);
	}

	.kpi-card {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-4);
		background-color: var(--bg-surface);
	}

	.kpi-icon-wrapper {
		width: 42px;
		height: 42px;
		border-radius: var(--radius-md);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.kpi-primary { background-color: var(--accent-blue-bg); color: var(--accent-blue-text); }
	.kpi-success { background-color: rgba(16, 185, 129, 0.15); color: #10B981; }
	.kpi-info { background-color: rgba(59, 130, 246, 0.15); color: #60A5FA; }
	.kpi-warning { background-color: rgba(245, 158, 11, 0.15); color: #F59E0B; }

	.kpi-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.kpi-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.kpi-value {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}

	/* Table Card */
	.table-card {
		background-color: var(--bg-surface);
	}

	.table-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-6);
		border-bottom: 1px solid var(--border);
	}

	.filter-pills {
		display: flex;
		gap: var(--space-2);
	}

	.pill-btn {
		background: none;
		border: 1px solid var(--border);
		border-radius: 9999px;
		padding: 4px 12px;
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.pill-btn.active {
		background-color: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-semibold);
	}

	.table-wrapper {
		overflow-x: auto;
	}

	.data-table {
		width: 100%;
		border-collapse: collapse;
		text-align: left;
	}

	.data-table th {
		padding: var(--space-3) var(--space-6);
		font-size: var(--font-size-xs);
		text-transform: uppercase;
		color: var(--text-muted);
		border-bottom: 1px solid var(--border);
	}

	.data-table td {
		padding: var(--space-3) var(--space-6);
		border-bottom: 1px solid var(--border-subtle);
	}

	.backup-name-cell {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.actions-cell {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-2);
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-12);
		gap: var(--space-3);
		color: var(--text-muted);
	}

	/* Modals */
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.7);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal-card {
		width: 100%;
		max-width: 520px;
		background-color: var(--bg-surface);
		box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
	}

	.modal-title {
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

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-6);
		border-top: 1px solid var(--border);
	}

	.scope-options {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.scope-option {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		cursor: pointer;
		border: 1px solid var(--border);
	}

	.scope-option.selected {
		border-color: var(--accent-blue-border);
		background-color: var(--accent-blue-bg);
	}

	.scope-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.scope-text span {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.restore-target-box {
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-card);
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.divider {
		height: 1px;
		background-color: var(--border);
		margin: var(--space-2) 0;
	}

	.s3-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.s3-header {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}

	.section-heading {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		margin: 0;
	}

	.section-sub {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.grid-2 {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-3);
	}

	/* Toast stack */
	.toast-stack {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		z-index: 2000;
	}

	.toast {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-surface);
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4);
		border-left: 4px solid var(--border);
	}

	.toast-success { border-left-color: #10B981; }
	.toast-danger { border-left-color: #EF4444; }

	.toast p {
		margin: 2px 0 0 0;
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
