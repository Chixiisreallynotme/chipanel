<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
	import FileTree from '$lib/components/files/FileTree.svelte';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import Spinner from '$lib/components/ui/Spinner.svelte';
	import {
		FileCode,
		Settings,
		Sparkles,
		CheckCircle2,
		AlertCircle,
		Info,
		X,
		Plus,
		Trash2,
		AlertTriangle,
		RefreshCw,
		FileText,
		Folder
	} from '$lib/icons.js';

	// Main Tree & File State
	let fileTree = $state(null);
	let treeError = $state('');
	let selectedPath = $state('server.properties');
	let fileData = $state(null);
	let fileError = $state('');
	let isLoadingTree = $state(false);
	let isLoadingFile = $state(false);
	let isSaving = $state(false);

	// Editor dirty tracking reference
	let isEditorDirty = $state(false);

	// Quick Presets List
	const PRESET_CHIPS = [
		{ name: 'server.properties', path: 'server.properties', icon: Settings },
		{ name: 'paper-global.yml', path: 'paper-global.yml', icon: FileCode },
		{ name: 'spigot.yml', path: 'spigot.yml', icon: FileCode },
		{ name: 'bukkit.yml', path: 'bukkit.yml', icon: FileCode },
		{ name: 'LuckPerms/config.yml', path: 'plugins/LuckPerms/config.yml', icon: Settings }
	];

	// Modals State
	let createModalOpen = $state(false);
	let createParentPath = $state('');
	let newItemName = $state('');
	let newItemIsDir = $state(false);
	let isCreating = $state(false);

	let deleteModalOpen = $state(false);
	let deleteTargetPath = $state('');
	let isDeleting = $state(false);

	let unsavedModalOpen = $state(false);
	let pendingSwitchPath = $state('');

	// Toast Notification Stack
	/**
	 * @typedef {Object} ToastItem
	 * @property {string} id
	 * @property {'success' | 'error' | 'info'} type
	 * @property {string} title
	 * @property {string} message
	 */
	let toasts = $state([]);

	function addToast(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		const toast = { id, type, title, message };
		toasts = [...toasts, toast];

		setTimeout(() => {
			removeToast(id);
		}, 4500);
	}

	function removeToast(id) {
		toasts = toasts.filter((t) => t.id !== id);
	}

	/**
	 * DELETE helper. `apiFetch` only throws on 401, so a failed delete would otherwise
	 * fall through as a success.
	 * @param {string} path
	 */
	async function apiDelete(path) {
		const res = await apiFetch(path, { method: 'DELETE' });
		if (!res.ok) {
			const data = await res.json().catch(() => ({}));
			throw new Error(data.error || data.message || `DELETE ${path} failed (${res.status})`);
		}
		return res.json().catch(() => null);
	}

	// Fetch Tree Data. There is no fallback tree: a fabricated directory listing would be
	// indistinguishable from the real server's, and every path in it would be a lie.
	async function loadTree() {
		isLoadingTree = true;
		treeError = '';
		try {
			fileTree = await apiGet('/api/files/tree');
		} catch (err) {
			fileTree = null;
			treeError = err?.message || 'Failed to load the file tree.';
		} finally {
			isLoadingTree = false;
		}
	}

	// Fetch File Content. On failure `fileData` stays null so no editable buffer exists --
	// saving content that was never read from disk would overwrite the real file with it.
	async function loadFileContent(path) {
		if (!path) return;
		isLoadingFile = true;
		fileError = '';
		try {
			const data = await apiGet(`/api/files/read?path=${encodeURIComponent(path)}`);
			fileData = data;
			selectedPath = path;
			isEditorDirty = false;
		} catch (err) {
			fileData = null;
			selectedPath = path;
			isEditorDirty = false;
			fileError = err?.message || `Failed to read "${path}".`;
		} finally {
			isLoadingFile = false;
		}
	}

	function retryLoadFile() {
		if (selectedPath) loadFileContent(selectedPath);
	}

	// File Selection Request with Unsaved Guard
	function requestSelectFile(path) {
		if (selectedPath === path && fileData) return;

		if (isEditorDirty) {
			pendingSwitchPath = path;
			unsavedModalOpen = true;
		} else {
			loadFileContent(path);
		}
	}

	// Save File Action
	async function handleSaveFile(content, triggerReload) {
		// Guard: `fileData` is only set from a successful read, so a buffer that was never
		// read from disk can never reach the write endpoint.
		if (!fileData || !selectedPath) return;
		isSaving = true;
		try {
			// Field name must match the backend's `FileSaveRequest.trigger_reload`; the old
			// `reload_rcon` key was dropped by serde, so "reloaded via RCON" was never true.
			const res = await apiPost('/api/files/save', {
				path: selectedPath,
				content,
				trigger_reload: triggerReload
			});

			fileData = {
				...fileData,
				content,
				size_bytes: content.length
			};
			isEditorDirty = false;

			addToast('success', 'File Saved', res?.message || `Saved "${selectedPath}".`);
		} catch (err) {
			// No in-memory "save": the buffer stays dirty and the file on disk is unchanged.
			addToast('error', 'Save Failed', err?.message || `Failed to save "${selectedPath}". The file on disk was not modified.`);
		} finally {
			isSaving = false;
		}
	}

	// Create Modal Handlers
	function openCreateModal(parentPath = '') {
		createParentPath = parentPath;
		newItemName = '';
		newItemIsDir = false;
		createModalOpen = true;
	}

	async function handleCreateItemSubmit() {
		if (!newItemName.trim()) {
			addToast('error', 'Invalid Name', 'Please enter a valid filename or directory name.');
			return;
		}

		isCreating = true;
		const fullPath = createParentPath ? `${createParentPath}/${newItemName.trim()}` : newItemName.trim();

		try {
			const res = await apiPost('/api/files/create', {
				path: fullPath,
				is_dir: newItemIsDir
			});

			addToast('success', 'Item Created', res?.message || `Created ${newItemIsDir ? 'folder' : 'file'} "${fullPath}"`);
			createModalOpen = false;
			await loadTree();

			if (!newItemIsDir) {
				requestSelectFile(fullPath);
			}
		} catch (err) {
			// Nothing was created on disk, so nothing is added to the tree or opened.
			addToast('error', 'Create Failed', err?.message || `Failed to create "${fullPath}".`);
		} finally {
			isCreating = false;
		}
	}

	// Delete Modal Handlers
	function openDeleteModal(path) {
		deleteTargetPath = path;
		deleteModalOpen = true;
	}

	async function handleDeleteItemSubmit() {
		if (!deleteTargetPath) return;
		isDeleting = true;

		try {
			const res = await apiDelete(`/api/files/delete?path=${encodeURIComponent(deleteTargetPath)}`);

			addToast('success', 'Item Deleted', res?.message || `Deleted "${deleteTargetPath}"`);
			deleteModalOpen = false;

			if (selectedPath === deleteTargetPath) {
				selectedPath = '';
				fileData = null;
				fileError = '';
			}

			await loadTree();
		} catch (err) {
			// The file is still on disk - leave it in the tree and say so.
			addToast('error', 'Deletion Failed', err?.message || `Failed to delete "${deleteTargetPath}".`);
		} finally {
			isDeleting = false;
		}
	}

	// Unsaved Modal Actions
	function confirmDiscardAndSwitch() {
		unsavedModalOpen = false;
		isEditorDirty = false;
		if (pendingSwitchPath) {
			loadFileContent(pendingSwitchPath);
			pendingSwitchPath = '';
		}
	}

	function closeModal() {
		createModalOpen = false;
		deleteModalOpen = false;
		unsavedModalOpen = false;
		pendingSwitchPath = '';
	}

	onMount(() => {
		loadTree();
		loadFileContent(selectedPath);
	});
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }} />

<div class="files-page-container">
	<!-- Page Header with Preset Chips -->
	<PageHeader
		title="Éditeur de Fichiers & Configurations"
		subtitle="Inspectez et modifiez les fichiers de configuration du serveur Minecraft avec coloration syntaxique"
	>
		{#snippet icon()}
			<FileCode size={20} />
		{/snippet}
		<!-- Quick Presets Chips Bar (Double-Bezel Milled Capsule) -->
		<div class="preset-chips-wrapper" aria-label="Fichiers de configuration prédéfinis">
			<span class="presets-label">
				<Sparkles size={13} class="preset-sparkle-icon" />
				<span>Presets :</span>
			</span>
			<div class="chips-list">
				{#each PRESET_CHIPS as preset}
					{@const IconComponent = preset.icon}
					<button
						type="button"
						class="preset-chip {selectedPath === preset.path ? 'active' : ''}"
						onclick={() => requestSelectFile(preset.path)}
						title="Ouvrir {preset.name}"
						aria-pressed={selectedPath === preset.path}
					>
						<IconComponent size={13} />
						<span>{preset.name}</span>
					</button>
				{/each}
			</div>
		</div>
	</PageHeader>

	<!-- Dual-Pane Layout in Hardware Double-Bezel Shell -->
	<div class="hardware-shell files-dual-pane-shell">
		<div class="hardware-core files-dual-pane">
			<!-- Left Pane: File Tree Sidebar (300px width) -->
			<div class="file-tree-pane">
				<FileTree
					tree={fileTree}
					error={treeError}
					loading={isLoadingTree}
					{selectedPath}
					onSelectFile={requestSelectFile}
					onRefreshTree={loadTree}
					onCreateItem={openCreateModal}
					onDeleteItem={openDeleteModal}
				/>
			</div>

			<!-- Right Pane: CodeMirror Editor (lazy-loaded, CodeMirror ~320K only fetched when needed) -->
			<div class="config-editor-pane">
				{#await import('$lib/components/files/ConfigEditor.svelte')}
					<div class="editor-loading-state">
						<Spinner size={40} />
						<h3>Chargement de l'éditeur…</h3>
					</div>
				{:then { default: ConfigEditor }}
					<ConfigEditor
						{fileData}
						{isSaving}
						error={fileError}
						isLoading={isLoadingFile}
						onSave={handleSaveFile}
						onRetry={retryLoadFile}
						onDirtyChange={(dirty) => (isEditorDirty = dirty)}
					/>
				{/await}
			</div>
		</div>
	</div>
</div>

<!-- Modal 1: Create File/Folder -->
{#if createModalOpen}
	<div class="modal-backdrop" onclick={() => (createModalOpen = false)} onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }} role="presentation">
		<div
			class="modal"
			onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			tabindex="-1"
			aria-labelledby="create-modal-title"
			aria-modal="true"
		>
			<header class="modal-header">
				<h3 id="create-modal-title" class="modal-title">
					<Plus size={18} />
					Create New File or Folder
				</h3>
				<button class="btn btn-ghost btn-icon" onclick={() => (createModalOpen = false)} aria-label="Close modal">
					<X size={18} />
				</button>
			</header>

			<form onsubmit={(e) => { e.preventDefault(); handleCreateItemSubmit(); }}>
				<div class="modal-body">
					<div class="form-group">
						<span class="label">Target Directory</span>
						<span class="badge badge-secondary parent-path-badge">
							{createParentPath || 'root (/)'}
						</span>
					</div>

					<div class="form-group">
						<span class="label label-required" id="item-type-label">Item Type</span>
						<div class="type-radio-group" id="item-type" role="radiogroup" aria-labelledby="item-type-label">
							<label class="radio-label { !newItemIsDir ? 'selected-radio' : '' }" for="item-type-file">
								<input id="item-type-file" type="radio" name="item-type" value={false} bind:group={newItemIsDir} />
								<FileText size={16} />
								File
							</label>
							<label class="radio-label { newItemIsDir ? 'selected-radio' : '' }" for="item-type-folder">
								<input id="item-type-folder" type="radio" name="item-type" value={true} bind:group={newItemIsDir} />
								<Folder size={16} />
								Folder
							</label>
						</div>
					</div>

					<div class="form-group">
						<label class="label label-required" for="item-name">
							{newItemIsDir ? 'Folder Name' : 'File Name (with extension)'}
						</label>
						<input
							id="item-name"
							type="text"
							class="input"
							placeholder={newItemIsDir ? 'my-plugin-data' : 'custom-config.yml'}
							bind:value={newItemName}
							required
						/>
					</div>
				</div>

				<footer class="modal-footer">
					<button type="button" class="btn btn-secondary" onclick={() => (createModalOpen = false)}>
						Cancel
					</button>
					<button
						type="submit"
						class="btn btn-primary {isCreating ? 'btn-loading' : ''}"
						disabled={isCreating || !newItemName.trim()}
					>
						Create {newItemIsDir ? 'Folder' : 'File'}
					</button>
				</footer>
			</form>
		</div>
	</div>
{/if}

<!-- Modal 2: Delete Confirmation -->
{#if deleteModalOpen}
	<div class="modal-backdrop" onclick={() => (deleteModalOpen = false)} onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }} role="presentation">
		<div
			class="modal"
			onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			tabindex="-1"
			aria-labelledby="delete-modal-title"
			aria-modal="true"
		>
			<header class="modal-header">
				<h3 id="delete-modal-title" class="modal-title text-danger">
					<Trash2 size={18} />
					Confirm Deletion
				</h3>
				<button class="btn btn-ghost btn-icon" onclick={() => (deleteModalOpen = false)} aria-label="Close modal">
					<X size={18} />
				</button>
			</header>

			<div class="modal-body">
				<div class="delete-warning-box">
					<AlertTriangle size={24} class="warning-icon" />
					<p>
						Are you sure you want to delete <code>{deleteTargetPath}</code>? This action cannot be undone and may cause server configuration issues.
					</p>
				</div>
			</div>

			<footer class="modal-footer">
				<button type="button" class="btn btn-secondary" onclick={() => (deleteModalOpen = false)}>
					Cancel
				</button>
				<button
					type="button"
					class="btn btn-danger {isDeleting ? 'btn-loading' : ''}"
					disabled={isDeleting}
					onclick={handleDeleteItemSubmit}
				>
					Delete Item
				</button>
			</footer>
		</div>
	</div>
{/if}

<!-- Modal 3: Unsaved Changes Guard -->
{#if unsavedModalOpen}
	<div class="modal-backdrop" onclick={() => (unsavedModalOpen = false)} onkeydown={(e) => { if (e.key === 'Escape') closeModal(); }} role="presentation">
		<div
			class="modal"
			onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			tabindex="-1"
			aria-labelledby="unsaved-modal-title"
			aria-modal="true"
		>
			<header class="modal-header">
				<h3 id="unsaved-modal-title" class="modal-title">
					<AlertCircle size={18} class="text-warning" />
					Unsaved Changes
				</h3>
				<button class="btn btn-ghost btn-icon" onclick={() => (unsavedModalOpen = false)} aria-label="Close modal">
					<X size={18} />
				</button>
			</header>

			<div class="modal-body">
				<p>
					You have unsaved changes in <code>{selectedPath}</code>. Switching files will discard your unsaved modifications.
				</p>
			</div>

			<footer class="modal-footer">
				<button type="button" class="btn btn-secondary" onclick={() => (unsavedModalOpen = false)}>
					Cancel
				</button>
				<button type="button" class="btn btn-danger" onclick={confirmDiscardAndSwitch}>
					Discard & Switch
				</button>
			</footer>
		</div>
	</div>
{/if}

<!-- Toast Notification Stack -->
<div class="toast-stack" role="region" aria-label="Notifications Stack">
	{#each toasts as toast (toast.id)}
		<div class="toast toast-{toast.type}">
			{#if toast.type === 'success'}
				<CheckCircle2 size={18} class="toast-icon text-success" />
			{:else if toast.type === 'error'}
				<AlertCircle size={18} class="toast-icon text-danger" />
			{:else}
				<Info size={18} class="toast-icon text-info" />
			{/if}

			<div class="toast-content">
				<strong class="toast-title">{toast.title}</strong>
				<p class="toast-message">{toast.message}</p>
			</div>

			<button
				type="button"
				class="btn btn-ghost btn-icon toast-close-btn"
				onclick={() => removeToast(toast.id)}
				aria-label="Close notification"
			>
				<X size={14} />
			</button>
		</div>
	{/each}
</div>

<style>
	.files-page-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		flex: 1;
		min-height: calc(100dvh - 60px - 48px);
	}

	/* Preset Chips (Milled Capsule) */
	.preset-chips-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-btn);
		padding: 3px 6px;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
	}

	.presets-label {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--text-muted);
		white-space: nowrap;
		padding: 0 4px;
	}

	:global(.preset-sparkle-icon) {
		color: var(--warning);
	}

	.chips-list {
		display: flex;
		align-items: center;
		gap: 4px;
		overflow-x: auto;
	}

	.preset-chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 10px;
		background-color: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.05);
		border-radius: var(--radius-sm);
		color: var(--text-secondary);
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		cursor: pointer;
		white-space: nowrap;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
		transition: transform 160ms var(--ease-out),
			background-color 150ms var(--ease-out),
			border-color 150ms var(--ease-out),
			color 150ms var(--ease-out);
	}

	.preset-chip:hover {
		border-color: var(--border-focus);
		color: var(--text-primary);
		background-color: var(--bg-elevated);
	}

	.preset-chip:active {
		transform: scale(0.97);
	}

	.preset-chip.active {
		background-color: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-semibold);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
	}

	/* Dual Pane Layout */
	.files-dual-pane-shell {
		padding: 4px;
		flex: 1;
		display: flex;
		min-height: 0;
	}

	.files-dual-pane {
		display: flex;
		flex: 1;
		background-color: var(--bg-surface);
		border-radius: calc(var(--radius-card) - 4px);
		overflow: hidden;
		min-height: 0;
		padding: 0;
	}

	.file-tree-pane {
		width: 300px;
		flex-shrink: 0;
		height: 100%;
	}

	.config-editor-pane {
		flex: 1;
		height: 100%;
		min-width: 0;
	}

	.editor-loading-state {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		padding: var(--space-8);
		text-align: center;
		color: var(--text-muted);
	}

	.editor-loading-state h3 {
		font-size: var(--font-size-lg);
		color: var(--text-primary);
	}

	/* Form & Modal Styling */
	.parent-path-badge {
		font-family: var(--font-mono);
		margin-top: 4px;
	}

	.type-radio-group {
		display: flex;
		gap: var(--space-3);
		margin-top: 4px;
	}

	.radio-label {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-4);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		background-color: var(--bg-base);
		cursor: pointer;
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		transition: border-color var(--transition-fast), background-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
	}

	.radio-label:active {
		transform: scale(0.97);
	}

	.radio-label.selected-radio {
		border-color: var(--accent-blue);
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
	}

	.delete-warning-box {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-btn);
		padding: var(--space-4);
		color: var(--text-primary);
	}

	:global(.warning-icon) {
		color: var(--danger-text);
		flex-shrink: 0;
	}

	/* Toast Notification Stack */
	.toast-stack {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		max-width: 400px;
		pointer-events: none;
	}

	.toast {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-focus);
		border-radius: var(--radius-card);
		box-shadow: var(--elevation-shadow);
		color: var(--text-primary);
		pointer-events: auto;
		animation: toastSlideIn 200ms ease-out;
	}

	@keyframes toastSlideIn {
		from {
			transform: translateY(20px);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}

	.toast-success {
		border-color: var(--accent-green-border);
	}

	.toast-error {
		border-color: var(--danger-border);
	}

	:global(.text-success) { color: var(--accent-green); }
	:global(.text-danger) { color: var(--danger-text); }
	:global(.text-info) { color: var(--accent-blue-text); }
	:global(.text-warning) { color: var(--warning); }

	.toast-content {
		flex: 1;
	}

	.toast-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		display: block;
	}

	.toast-message {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin-top: 2px;
	}

	:global(.toast-close-btn) {
		width: 24px !important;
		height: 24px !important;
		color: var(--text-muted);
	}

	@media (max-width: 900px) {
		.files-dual-pane {
			flex-direction: column;
		}

		.file-tree-pane {
			width: 100%;
			height: 280px;
		}

		.files-page-container {
			height: auto;
		}
	}
</style>
