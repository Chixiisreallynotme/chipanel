<script>
	import { apiPost, apiFetch } from '$lib/api/client.js';
	import Modal from '$lib/components/ui/Modal.svelte';
	import {
		X,
		Archive,
		Plus,
		RotateCcw,
		Trash2,
		Search,
		HardDrive,
		Calendar,
		Globe,
		AlertTriangle,
		CheckCircle2,
		RefreshCw,
		FileArchive,
		ShieldAlert
	} from '$lib/icons.js';

	let {
		isOpen = false,
		backups = [],
		worlds = [],
		loading = false,
		onClose = () => {},
		onRefresh = () => {},
		onToast = () => {}
	} = $props();

	// Local Form & Filter State
	let searchQuery = $state('');
	let selectedWorldToBackup = $state('');
	let isCreating = $state(false);

	// Double-Confirmation Restore Dialog State
	let backupToRestore = $state(null);
	let isRestoring = $state(false);

	// Delete Confirmation State
	let backupToDelete = $state(null);
	let isDeleting = $state(false);

	// Set initial world selection when modal opens
	$effect(() => {
		if (isOpen) {
			searchQuery = '';
			backupToRestore = null;
			backupToDelete = null;
			if (worlds.length > 0 && !selectedWorldToBackup) {
				selectedWorldToBackup = worlds[0].folder_name;
			}
		}
	});

	// Derived Filtered Backups
	let filteredBackups = $derived.by(() => {
		const q = searchQuery.trim().toLowerCase();
		if (!q) return backups;
		return backups.filter(
			(b) =>
				(b.filename || '').toLowerCase().includes(q) ||
				(b.world_name || '').toLowerCase().includes(q)
		);
	});

	// Format disk size
	function formatBytes(bytes) {
		if (!bytes || bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i];
	}

	// Format Created Date
	function formatDate(isoStr) {
		if (!isoStr) return 'Unknown Date';
		try {
			const d = new Date(isoStr);
			if (isNaN(d.getTime())) return isoStr;
			return d.toLocaleString(undefined, {
				year: 'numeric',
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch (e) {
			return isoStr;
		}
	}

	// Create New Backup Action
	async function handleCreateBackup(e) {
		if (e) e.preventDefault();
		if (!selectedWorldToBackup) {
			onToast('error', 'Validation Error', 'Please select a world to back up.');
			return;
		}

		isCreating = true;
		try {
			const res = await apiPost('/api/worlds/backups/create', {
				world_name: selectedWorldToBackup
			});
			onToast('success', 'Backup Created', res.message || `Successfully created backup for '${selectedWorldToBackup}'.`);
			await onRefresh();
		} catch (err) {
			console.error('Failed to create backup:', err);
			onToast('error', 'Backup Creation Failed', err.message || 'Could not zip world directory.');
		} finally {
			isCreating = false;
		}
	}

	// Open Restore Confirmation Modal
	function promptRestore(backup) {
		backupToRestore = backup;
	}

	function cancelRestore() {
		if (isRestoring) return;
		backupToRestore = null;
	}

	// Execute Restore Backup
	async function confirmRestore() {
		if (!backupToRestore || isRestoring) return;
		isRestoring = true;

		const filename = backupToRestore.filename;
		const targetWorld = backupToRestore.world_name;

		try {
			const res = await apiPost('/api/worlds/backups/restore', { filename });
			onToast(
				'success',
				'Backup Restored',
				res.message || `Successfully restored '${filename}' into '${targetWorld}'.`
			);
			backupToRestore = null;
			await onRefresh();
		} catch (err) {
			console.error('Failed to restore backup:', err);
			onToast('error', 'Restore Failed', err.message || `Failed to restore '${filename}'.`);
		} finally {
			isRestoring = false;
		}
	}

	// Open Delete Confirmation Modal
	function promptDelete(backup) {
		backupToDelete = backup;
	}

	function cancelDelete() {
		if (isDeleting) return;
		backupToDelete = null;
	}

	// Execute Delete Backup
	async function confirmDelete() {
		if (!backupToDelete || isDeleting) return;
		isDeleting = true;

		const filename = backupToDelete.filename;

		try {
			const res = await apiFetch(`/api/worlds/backups/${encodeURIComponent(filename)}`, {
				method: 'DELETE'
			});

			if (!res.ok) {
				const errorData = await res.json().catch(() => ({ message: 'Delete failed' }));
				throw new Error(errorData.error || errorData.message || `DELETE failed (${res.status})`);
			}

			const data = await res.json().catch(() => ({}));
			onToast('success', 'Backup Deleted', data.message || `Successfully deleted backup '${filename}'.`);
			backupToDelete = null;
			await onRefresh();
		} catch (err) {
			console.error('Failed to delete backup:', err);
			onToast('error', 'Delete Failed', err.message || `Could not remove backup file '${filename}'.`);
		} finally {
			isDeleting = false;
		}
	}

	function handleKeydown(e) {
		if (e.key === 'Escape') {
			if (backupToRestore) {
				cancelRestore();
			} else if (backupToDelete) {
				cancelDelete();
			} else if (isOpen) {
				onClose();
			}
		}
	}
</script>

{#if isOpen}
	<div
		class="modal-backdrop"
		onclick={(e) => e.target === e.currentTarget && onClose()}
		onkeydown={handleKeydown}
		role="dialog"
		aria-modal="true"
		aria-labelledby="backup-modal-title"
		tabindex="-1"
	>
		<div class="modal backup-modal">
			<!-- Modal Header -->
			<div class="modal-header">
				<div class="modal-title-group">
					<div class="modal-icon-box">
						<Archive size={22} />
					</div>
					<div>
						<h3 id="backup-modal-title" class="modal-title">World Backup Manager</h3>
						<p class="modal-subtitle">Create ZIP archives of server dimensions, restore snapshots, or purge old backups</p>
					</div>
				</div>

				<button
					type="button"
					class="btn btn-ghost btn-icon btn-sm"
					onclick={onClose}
					aria-label="Close Backup Manager Modal"
				>
					<X size={18} />
				</button>
			</div>

			<!-- Modal Body -->
			<div class="modal-body backup-modal-body">
				<!-- Create New Backup Bar -->
				<div class="create-backup-card card">
					<div class="card-header create-header">
						<div class="header-left-title">
							<Plus size={16} class="icon-blue" />
							<h4 class="card-title text-sm">Create New World Backup</h4>
						</div>
					</div>
					<div class="card-body create-body">
						<form onsubmit={handleCreateBackup} class="create-backup-form">
							<div class="form-row-flex">
								<div class="form-group flex-1 mb-0">
									<label for="backup-world-select" class="label">Select Target Dimension</label>
									<select
										id="backup-world-select"
										class="select"
										bind:value={selectedWorldToBackup}
										disabled={isCreating}
									>
										{#if worlds.length === 0}
											<option value="world">world</option>
										{:else}
											{#each worlds as w}
												<option value={w.folder_name}>
													{w.level_name} ({w.folder_name})
												</option>
											{/each}
										{/if}
									</select>
								</div>

								<button
									type="submit"
									class="btn btn-primary create-btn {isCreating ? 'btn-loading' : ''}"
									disabled={isCreating}
								>
									{#if !isCreating}
										<Archive size={16} />
									{/if}
									<span>Create Backup ZIP</span>
								</button>
							</div>
						</form>
					</div>
				</div>

				<!-- Search Filter & List Header -->
				<div class="list-controls-bar">
					<div class="search-box">
						<Search size={14} class="search-icon" />
						<input
							type="text"
							class="input search-input"
							placeholder="Search backups by filename or world..."
							bind:value={searchQuery}
							aria-label="Filter backup archives"
						/>
					</div>

					<div class="list-meta">
						<span class="badge badge-secondary">
							{filteredBackups.length} of {backups.length} Backups
						</span>
						<button
							type="button"
							class="btn btn-ghost btn-icon btn-sm"
							onclick={onRefresh}
							title="Refresh Backups List"
							aria-label="Refresh Backups List"
						>
							<RefreshCw size={14} class={loading ? 'spin-slow' : ''} />
						</button>
					</div>
				</div>

				<!-- Backups List Table -->
				<div class="table-container backups-table-wrapper">
					{#if loading && backups.length === 0}
						<div class="empty-state-sub">
							<RefreshCw size={28} class="spin-slow empty-icon" />
							<p>Loading backup archives...</p>
						</div>
					{:else if backups.length === 0}
						<div class="empty-state-sub">
							<FileArchive size={32} class="empty-icon" />
							<p>No world backups found. Create your first snapshot above!</p>
						</div>
					{:else if filteredBackups.length === 0}
						<div class="empty-state-sub">
							<Search size={32} class="empty-icon" />
							<p>No backups matching filter "{searchQuery}".</p>
						</div>
					{:else}
						<table class="table backups-table">
							<thead>
								<tr>
									<th>World / Dimension</th>
									<th>Filename</th>
									<th>Created At</th>
									<th>Size</th>
									<th class="text-right">Actions</th>
								</tr>
							</thead>
							<tbody>
								{#each filteredBackups as backup (backup.filename)}
									<tr>
										<!-- World Name Badge -->
										<td class="world-name-cell">
											<span class="badge badge-blue">
												<Globe size={11} />
												<span>{backup.world_name}</span>
											</span>
										</td>

										<!-- Filename -->
										<td class="filename-cell">
											<code class="backup-filename font-mono" title={backup.filename}>
												{backup.filename}
											</code>
										</td>

										<!-- Date Created -->
										<td class="date-cell text-muted">
											<span class="date-text">
												<Calendar size={13} class="icon-subtle" />
												<span>{formatDate(backup.created_at)}</span>
											</span>
										</td>

										<!-- File Size -->
										<td class="size-cell font-mono">
											<span class="size-text">
												<HardDrive size={13} class="icon-subtle" />
												<span>{formatBytes(backup.size_bytes)}</span>
											</span>
										</td>

										<!-- Action Buttons -->
										<td class="actions-cell text-right">
											<div class="action-buttons-group">
												<button
													type="button"
													class="btn btn-secondary btn-sm restore-btn"
													onclick={() => promptRestore(backup)}
													title="Restore this backup snapshot"
												>
													<RotateCcw size={14} />
													<span>Restore</span>
												</button>

												<button
													type="button"
													class="btn btn-ghost btn-sm btn-icon danger-icon-btn"
													onclick={() => promptDelete(backup)}
													title="Delete backup file"
													aria-label="Delete backup file"
												>
													<Trash2 size={14} />
												</button>
											</div>
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					{/if}
				</div>
			</div>

			<!-- Modal Footer -->
			<div class="modal-footer">
				<button type="button" class="btn btn-secondary" onclick={onClose}>
					Close
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Double Confirmation Restore Dialog Overlay -->
<Modal
		open={!!backupToRestore}
		onclose={() => (isRestoring ? null : cancelRestore())}
		class="modal confirm-modal"
		backdropClass="sub-modal-backdrop"
		ariaLabelledBy="restore-confirm-title"
	>
		{#snippet content()}
			<div class="modal-header danger-header">
				<div class="confirm-title-row">
					<ShieldAlert size={22} class="icon-danger" />
					<h3 id="restore-confirm-title" class="modal-title text-danger">Confirm World Restoration</h3>
				</div>
				<button
					type="button"
					class="btn btn-ghost btn-icon btn-sm"
					onclick={cancelRestore}
					disabled={isRestoring}
					aria-label="Close Restore Confirmation"
				>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body confirm-body">
				<div class="alert alert-danger warning-alert-box">
					<AlertTriangle size={20} class="flex-shrink-0" />
					<div>
						<strong>CRITICAL WARNING: OVERWRITE HAZARD</strong>
						<p class="warning-text">
							Restoring this backup snapshot will completely overwrite existing world files for
							<strong class="font-mono text-primary">{backupToRestore.world_name}</strong>. Any changes or player builds made after this backup date will be permanently replaced.
						</p>
					</div>
				</div>

				<div class="backup-details-summary">
					<div class="summary-row">
						<span class="summary-label">Target World:</span>
						<span class="summary-val font-mono">{backupToRestore.world_name}</span>
					</div>
					<div class="summary-row">
						<span class="summary-label">Backup File:</span>
						<span class="summary-val font-mono">{backupToRestore.filename}</span>
					</div>
					<div class="summary-row">
						<span class="summary-label">Backup Date:</span>
						<span class="summary-val">{formatDate(backupToRestore.created_at)}</span>
					</div>
					<div class="summary-row">
						<span class="summary-label">Archive Size:</span>
						<span class="summary-val font-mono">{formatBytes(backupToRestore.size_bytes)}</span>
					</div>
				</div>
			</div>

			<div class="modal-footer">
				<button
					type="button"
					class="btn btn-secondary"
					onclick={cancelRestore}
					disabled={isRestoring}
				>
					Cancel
				</button>
				<button
					type="button"
					class="btn btn-danger {isRestoring ? 'btn-loading' : ''}"
					onclick={confirmRestore}
					disabled={isRestoring}
				>
					{#if !isRestoring}
						<RotateCcw size={16} />
					{/if}
					<span>Overwrite & Restore Backup</span>
				</button>
			</div>
		{/snippet}
	</Modal>

<!-- Delete Confirmation Dialog Overlay -->
<Modal
		open={!!backupToDelete}
		onclose={() => (isDeleting ? null : cancelDelete())}
		class="modal confirm-modal"
		backdropClass="sub-modal-backdrop"
		ariaLabelledBy="delete-confirm-title"
	>
		{#snippet content()}
			<div class="modal-header">
				<div class="confirm-title-row">
					<Trash2 size={20} class="icon-danger" />
					<h3 id="delete-confirm-title" class="modal-title">Delete Backup Archive</h3>
				</div>
				<button
					type="button"
					class="btn btn-ghost btn-icon btn-sm"
					onclick={cancelDelete}
					disabled={isDeleting}
					aria-label="Close Delete Confirmation"
				>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body confirm-body">
				<p class="confirm-prompt-text">
					Are you sure you want to permanently delete the backup archive
					<strong class="font-mono text-primary">{backupToDelete.filename}</strong>?
				</p>
				<p class="field-hint mt-2">This action cannot be undone and will free up {formatBytes(backupToDelete.size_bytes)} of disk space.</p>
			</div>

			<div class="modal-footer">
				<button
					type="button"
					class="btn btn-secondary"
					onclick={cancelDelete}
					disabled={isDeleting}
				>
					Cancel
				</button>
				<button
					type="button"
					class="btn btn-danger {isDeleting ? 'btn-loading' : ''}"
					onclick={confirmDelete}
					disabled={isDeleting}
				>
					{#if !isDeleting}
						<Trash2 size={16} />
					{/if}
					<span>Confirm Delete</span>
				</button>
			</div>
		{/snippet}
	</Modal>

<style>
	.backup-modal {
		max-width: 840px;
		width: 92vw;
		max-height: 88vh;
		display: flex;
		flex-direction: column;
	}

	.modal-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.modal-icon-box {
		width: 42px;
		height: 42px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	.backup-modal-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		overflow-y: auto;
		flex: 1;
		padding: var(--space-6);
	}

	/* Create Backup Box */
	.create-backup-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
	}

	.create-header {
		padding: var(--space-3) var(--space-4);
	}

	.header-left-title {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.create-body {
		padding: var(--space-4);
	}

	.create-backup-form {
		width: 100%;
	}

	.form-row-flex {
		display: flex;
		align-items: flex-end;
		gap: var(--space-4);
	}

	.create-btn {
		height: 38px;
		white-space: nowrap;
	}

	/* List Controls & Search */
	.list-controls-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
	}

	.search-box {
		position: relative;
		flex: 1;
		max-width: 400px;
	}

	.search-icon {
		position: absolute;
		left: 12px;
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		padding-left: 36px;
	}

	.list-meta {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	/* Table Wrapper */
	.backups-table-wrapper {
		min-height: 240px;
		max-height: 380px;
		overflow-y: auto;
	}

	.backups-table th {
		position: sticky;
		top: 0;
		z-index: 10;
		background-color: var(--bg-base);
	}

	.backup-filename {
		font-size: var(--font-size-xs);
		color: var(--text-primary);
		word-break: break-all;
	}

	.date-text, .size-text {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--font-size-xs);
	}

	.icon-subtle {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.actions-cell {
		width: 160px;
	}

	.action-buttons-group {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-2);
	}

	.danger-icon-btn {
		color: var(--text-muted);
	}

	.danger-icon-btn:hover {
		color: var(--danger-text);
		background-color: var(--danger-bg);
	}

	.empty-state-sub {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-12) var(--space-6);
		gap: var(--space-3);
		color: var(--text-muted);
		text-align: center;
	}

	.spin-slow {
		animation: spin 3s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* Sub-Modal / Double Confirmation Overlays */
	.sub-modal-backdrop {
		z-index: calc(var(--z-modal) + 1);
		background: rgba(0, 0, 0, 0.85);
	}

	.confirm-modal {
		max-width: 500px;
		width: 90vw;
	}

	.danger-header {
		border-bottom-color: var(--danger-border);
	}

	.confirm-title-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.icon-danger {
		color: var(--danger-text);
	}

	.text-danger {
		color: var(--danger-text);
	}

	.confirm-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.warning-alert-box {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-btn);
		padding: var(--space-4);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
	}

	.warning-text {
		color: var(--text-secondary);
		font-size: var(--font-size-xs);
		margin-top: 4px;
		line-height: 1.4;
	}

	.backup-details-summary {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		padding: var(--space-3) var(--space-4);
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-size: var(--font-size-xs);
	}

	.summary-row {
		display: flex;
		justify-content: space-between;
		gap: var(--space-4);
	}

	.summary-label {
		color: var(--text-muted);
	}

	.summary-val {
		color: var(--text-primary);
		font-weight: var(--font-weight-medium);
		word-break: break-all;
	}

	.confirm-prompt-text {
		font-size: var(--font-size-base);
		color: var(--text-primary);
	}

	.mb-0 { margin-bottom: 0; }
	.mt-2 { margin-top: var(--space-2); }

	@media (max-width: 600px) {
		.form-row-flex {
			flex-direction: column;
			align-items: stretch;
		}

		.create-btn {
			width: 100%;
		}

		.list-controls-bar {
			flex-direction: column;
			align-items: stretch;
		}

		.search-box {
			max-width: none;
		}
	}
</style>
