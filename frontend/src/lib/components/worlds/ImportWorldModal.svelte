<script>
	import { CheckCircle2, X, Upload, Archive } from 'lucide-svelte';
	import { apiFetch, apiPost } from '$lib/api/client.js';

	let {
		isOpen = false,
		backups = [],
		onClose = () => {},
		onDone = () => {},
		onToast = () => {}
	} = $props();

	let tab = $state('upload');
	let uploadName = $state('');
	let uploadFile = $state(null);
	let backupFilename = $state('');
	let backupName = $state('');
	let submitting = $state(false);

	function reset() {
		uploadName = '';
		uploadFile = null;
		backupFilename = '';
		backupName = '';
	}

	function handleFileChange(e) {
		uploadFile = e.target.files?.[0] ?? null;
		if (uploadFile && !uploadName) {
			uploadName = uploadFile.name.replace(/\.zip$/i, '');
		}
	}

	async function submitUpload() {
		if (!uploadFile || submitting) return;
		submitting = true;
		try {
			const formData = new FormData();
			formData.append('file', uploadFile);
			if (uploadName.trim()) formData.append('name', uploadName.trim());

			const res = await apiFetch('/api/worlds/import', { method: 'POST', body: formData });
			const data = await res.json().catch(() => ({}));
			if (!res.ok) {
				throw new Error(data?.error || `Import failed (HTTP ${res.status})`);
			}
			onToast('success', 'World Imported', data?.message || 'World imported successfully.');
			if (data?.warning) onToast('info', 'Warning', data.warning);
			reset();
			onDone();
		} catch (err) {
			onToast('error', 'Import Failed', err.message || 'Could not import the world.');
		} finally {
			submitting = false;
		}
	}

	async function submitBackup() {
		if (!backupFilename || submitting) return;
		submitting = true;
		try {
			/** @type {Record<string, string>} */
			const payload = { filename: backupFilename };
			if (backupName.trim()) payload.name = backupName.trim();
			const res = await apiPost('/api/worlds/import/backup', payload);
			onToast('success', 'World Imported', res.message || 'World imported from backup.');
			if (res.warning) onToast('info', 'Warning', res.warning);
			reset();
			onDone();
		} catch (err) {
			onToast('error', 'Import Failed', err.message || 'Could not import the backup.');
		} finally {
			submitting = false;
		}
	}
</script>

{#if isOpen}
	<div
		class="modal-backdrop"
		onclick={() => (submitting ? null : onClose())}
		role="dialog"
		aria-modal="true"
		aria-labelledby="import-world-title"
		tabindex="-1"
	>
		<div class="modal world-modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<div class="m-title">
					<Upload size={20} class="icon-blue" />
					<h3 id="import-world-title" class="modal-title">Import World</h3>
				</div>
				<button
					type="button"
					class="btn btn-ghost btn-icon btn-sm"
					onclick={onClose}
					disabled={submitting}
					aria-label="Close"
				>
					<X size={18} />
				</button>
			</div>

			<div class="tabs">
				<button
					type="button"
					class="tab-btn {tab === 'upload' ? 'tab-active' : ''}"
					onclick={() => (tab = 'upload')}
				>
					<Upload size={14} />
					<span>Upload .zip</span>
				</button>
				<button
					type="button"
					class="tab-btn {tab === 'backup' ? 'tab-active' : ''}"
					onclick={() => (tab = 'backup')}
				>
					<Archive size={14} />
					<span>From backup</span>
				</button>
			</div>

			<div class="modal-body world-modal-body">
				{#if tab === 'upload'}
					<div class="form-group">
						<label class="label label-required" for="iw-file">World archive (.zip)</label>
						<input
							id="iw-file"
							type="file"
							accept=".zip"
							class="input"
							onchange={handleFileChange}
						/>
						<span class="field-hint">The archive must contain a `level.dat` world.</span>
					</div>
					<div class="form-group">
						<label class="label" for="iw-name">Target folder name (optional)</label>
						<input
							id="iw-name"
							class="input font-mono"
							bind:value={uploadName}
							placeholder="Defaults to the archive filename"
						/>
					</div>
					<div class="modal-footer">
						<button type="button" class="btn btn-secondary" onclick={onClose} disabled={submitting}>
							Cancel
						</button>
						<button
							type="button"
							class="btn btn-primary {submitting ? 'btn-loading' : ''}"
							onclick={submitUpload}
							disabled={submitting || !uploadFile}
						>
							{#if !submitting}
								<CheckCircle2 size={16} />
							{/if}
							<span>Import</span>
						</button>
					</div>
				{:else}
					<div class="form-group">
						<label class="label label-required" for="iw-backup">Backup</label>
						<select id="iw-backup" class="input" bind:value={backupFilename}>
							<option value="">Select a backup…</option>
							{#each backups as b (b.filename)}
								<option value={b.filename}>{b.filename}</option>
							{/each}
						</select>
					</div>
					<div class="form-group">
						<label class="label" for="iw-backup-name">Target folder name (optional)</label>
						<input
							id="iw-backup-name"
							class="input font-mono"
							bind:value={backupName}
							placeholder="Defaults to the backup's world name"
						/>
					</div>
					<div class="modal-footer">
						<button type="button" class="btn btn-secondary" onclick={onClose} disabled={submitting}>
							Cancel
						</button>
						<button
							type="button"
							class="btn btn-primary {submitting ? 'btn-loading' : ''}"
							onclick={submitBackup}
							disabled={submitting || !backupFilename}
						>
							{#if !submitting}
								<CheckCircle2 size={16} />
							{/if}
							<span>Import</span>
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.world-modal {
		max-width: 520px;
		width: 90vw;
	}

	.world-modal-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.m-title {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.icon-blue {
		color: var(--accent-blue-text);
	}

	.tabs {
		display: flex;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
	}

	.tab-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		background: transparent;
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: var(--space-2) var(--space-4);
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.tab-btn:hover {
		color: var(--text-primary);
	}

	.tab-active {
		background: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
		color: var(--accent-blue-text);
	}

	.field-hint {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 4px;
		display: block;
	}
</style>
