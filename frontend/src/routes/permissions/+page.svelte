<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
	import GroupList from '$lib/components/permissions/GroupList.svelte';
	import GroupEditorModal from '$lib/components/permissions/GroupEditorModal.svelte';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import {
		ShieldCheck,
		Plus,
		CheckCircle2,
		AlertCircle,
		Info,
		X,
		RefreshCw,
		Layers,
		Key,
		Crown,
		Sparkles
	} from '$lib/icons.js';

	/**
	 * @typedef {Object} LuckPermsPermissionNode
	 * @property {string} permission
	 * @property {boolean} value
	 * @property {string} [expiry]
	 * @property {string} [context]
	 */

	/**
	 * @typedef {Object} LuckPermsGroup
	 * @property {string} name
	 * @property {number} weight
	 * @property {string} [prefix]
	 * @property {string} [suffix]
	 * @property {string[]} [parents]
	 * @property {number} [permissions_count]
	 * @property {LuckPermsPermissionNode[]} [permissions]
	 */

	/**
	 * @typedef {Object} ToastNotification
	 * @property {string} id
	 * @property {string} title
	 * @property {string} message
	 * @property {'success' | 'danger' | 'warning' | 'info'} type
	 */

	// Component Reactive State
	let groups = $state([]);
	let loading = $state(false);
	let loadError = $state('');
	let isModalOpen = $state(false);
	let editingGroup = $state(null);
	let toasts = $state([]);

	// Statistics Derivations
	let totalGroupsCount = $derived(groups.length);
	let totalPermissionsCount = $derived(
		groups.reduce((acc, g) => {
			const count =
				typeof g.permissions_count === 'number'
					? g.permissions_count
					: Array.isArray(g.permissions)
						? g.permissions.length
						: 0;
			return acc + count;
		}, 0)
	);
	let highestWeightGroup = $derived.by(() => {
		if (groups.length === 0) return null;
		return [...groups].sort((a, b) => (b.weight || 0) - (a.weight || 0))[0];
	});

	// Mount logic
	onMount(() => {
		loadGroups();
	});

	// Toast Notification Dispatcher
	function addToast(title, message, type = 'success') {
		const id = `toast-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`;
		const toast = { id, title, message, type };
		toasts = [toast, ...toasts];

		setTimeout(() => {
			removeToast(id);
		}, 4000);
	}

	function removeToast(id) {
		toasts = toasts.filter((t) => t.id !== id);
	}

	/**
	 * DELETE helper. `apiFetch` only throws on 401, so a rejected delete would otherwise
	 * fall through as a success.
	 * @param {string} path
	 * @param {any} [body]
	 */
	async function apiDelete(path, body) {
		const res = await apiFetch(path, { method: 'DELETE', ...(body ? { body } : {}) });
		if (!res.ok) {
			const data = await res.json().catch(() => ({}));
			throw new Error(data.error || data.message || `DELETE ${path} failed (${res.status})`);
		}
		return res.json().catch(() => null);
	}

	// Fetch groups from backend API. There is no local default set: LuckPerms groups only
	// exist on the server, so inventing them here would show an operator permissions that
	// nobody has.
	async function loadGroups() {
		loading = true;
		loadError = '';
		try {
			const res = await apiGet('/api/permissions/groups');
			groups = Array.isArray(res) ? res : [];
		} catch (err) {
			groups = [];
			loadError = err?.message || 'LuckPerms groups are unavailable (backend or RCON offline).';
		} finally {
			loading = false;
		}
	}

	// Open Modal for Creating Group
	function handleOpenCreateModal() {
		editingGroup = null;
		isModalOpen = true;
	}

	// Open Modal for Editing Group. The list endpoint only carries a permission *count*, so
	// the node list has to come from the server before the editor may open - opening it on
	// anything else would show invented nodes and write them back on save.
	async function handleOpenEditModal(group) {
		loading = true;
		try {
			const detail = await apiGet(`/api/permissions/groups/${encodeURIComponent(group.name)}`);
			editingGroup = {
				name: detail.name,
				weight: detail.weight ?? 0,
				prefix: detail.prefix || '',
				suffix: detail.suffix || '',
				parents: Array.isArray(detail.parents) ? detail.parents : [],
				permissions: Array.isArray(detail.permissions) ? detail.permissions : []
			};
			isModalOpen = true;
		} catch (err) {
			addToast(
				'Group Unavailable',
				err?.message || `Could not read group "${group.name}" from the server.`,
				'danger'
			);
		} finally {
			loading = false;
		}
	}

	/** Group awaiting delete confirmation; null when the dialog is closed. */
	let groupPendingDelete = $state(/** @type {any} */ (null));

	function confirmDeleteGroup() {
		const group = groupPendingDelete;
		groupPendingDelete = null;
		if (group) handleDeleteGroup(group);
	}

	// Handle Group Deletion
	async function handleDeleteGroup(groupToDelete) {
		const gName = groupToDelete.name;

		try {
			const res = await apiDelete(`/api/permissions/groups/${encodeURIComponent(gName)}`);
			addToast('Group Deleted', res?.message || `LuckPerms group "${gName}" was removed.`, 'danger');
		} catch (err) {
			// The group still exists on the server - it stays in the list.
			addToast('Deletion Failed', err?.message || `Failed to delete group "${gName}".`, 'danger');
		} finally {
			// Re-read: the list must reflect the server, never an assumed outcome.
			await loadGroups();
		}
	}

	/**
	 * Sends the difference between the group the server gave us and the edited one.
	 * Every call throws on failure, so the first rejected step aborts the rest and the
	 * caller reports it - no step is allowed to fail silently.
	 */
	async function applyGroupChanges(gName, prev, next) {
		const base = `/api/permissions/groups/${encodeURIComponent(gName)}`;

		if (prev.prefix !== next.prefix || prev.suffix !== next.suffix || prev.weight !== next.weight) {
			await apiPost(`${base}/meta`, {
				prefix: next.prefix,
				suffix: next.suffix,
				weight: next.weight
			});
		}

		const prevPerms = prev.permissions || [];
		const nextPerms = next.permissions || [];

		for (const np of nextPerms) {
			const oldMatch = prevPerms.find((op) => op.permission === np.permission);
			if (!oldMatch || oldMatch.value !== np.value) {
				await apiPost(`${base}/permission`, { permission: np.permission, value: np.value });
			}
		}

		for (const op of prevPerms) {
			if (!nextPerms.some((np) => np.permission === op.permission)) {
				await apiDelete(`${base}/permission`, { permission: op.permission });
			}
		}

		const prevParents = prev.parents || [];
		const nextParents = next.parents || [];

		for (const np of nextParents) {
			if (!prevParents.includes(np)) {
				await apiPost(`${base}/parent`, { parent_group: np });
			}
		}

		for (const op of prevParents) {
			if (!nextParents.includes(op)) {
				await apiDelete(`${base}/parent`, { parent_group: op });
			}
		}
	}

	// Save Group Handler (Create or Update)
	async function handleSaveGroup(updatedGroup) {
		const gName = updatedGroup.name;
		const isNew = !groups.some((g) => g.name === gName);

		try {
			if (isNew) {
				await apiPost('/api/permissions/groups/create', {
					name: gName,
					weight: updatedGroup.weight,
					prefix: updatedGroup.prefix
				});
				// The create endpoint only takes name/weight/prefix; the rest of what the
				// form collected still has to be applied, or "created successfully" would
				// cover permissions and parents that were never set.
				await applyGroupChanges(
					gName,
					{ weight: updatedGroup.weight, prefix: updatedGroup.prefix, suffix: '', permissions: [], parents: [] },
					updatedGroup
				);
			} else {
				// Diff against the detail the server returned when the editor was opened.
				if (!editingGroup || editingGroup.name !== gName) {
					throw new Error('The server state for this group is no longer loaded. Reopen the group and try again.');
				}
				await applyGroupChanges(gName, editingGroup, updatedGroup);
			}

			addToast(
				isNew ? 'Group Created' : 'Group Saved',
				`LuckPerms group "${gName}" was ${isNew ? 'created' : 'updated'} on the server.`,
				'success'
			);
			isModalOpen = false;
		} catch (err) {
			// The server rejected part of the change. Report its message verbatim, keep the
			// editor open, and never patch the list with an outcome that did not happen.
			addToast('Save Failed', err?.message || `Failed to save LuckPerms group "${gName}".`, 'danger');
		} finally {
			// Re-read: whatever did or did not apply, the list shows the server's state.
			await loadGroups();
		}
	}
</script>

<ConfirmDialog
	open={groupPendingDelete !== null}
	title="Delete group"
	message={`Are you sure you want to delete LuckPerms group "${groupPendingDelete?.name ?? ''}"?`}
	confirmLabel="Delete"
	cancelLabel="Cancel"
	onconfirm={confirmDeleteGroup}
	oncancel={() => (groupPendingDelete = null)}
/>

<div class="permissions-page">
	<!-- Toast Notifications Container -->
	<div class="toast-container" aria-live="polite" aria-atomic="true">
		{#each toasts as toast (toast.id)}
			<div class="toast toast-{toast.type}">
				<div class="toast-icon">
					{#if toast.type === 'success'}
						<CheckCircle2 size={18} />
					{:else if toast.type === 'danger'}
						<AlertCircle size={18} />
					{:else if toast.type === 'warning'}
						<AlertCircle size={18} />
					{:else}
						<Info size={18} />
					{/if}
				</div>

				<div class="toast-body">
					<h4 class="toast-title">{toast.title}</h4>
					<p class="toast-message">{toast.message}</p>
				</div>

				<button
					class="btn btn-ghost btn-icon btn-sm toast-close"
					onclick={() => removeToast(toast.id)}
					aria-label="Close notification"
				>
					<X size={14} />
				</button>
			</div>
		{/each}
	</div>

	<!-- Page Header -->
	<PageHeader
		title="LuckPerms Visual Editor"
		subtitle="Visual management interface for Minecraft LuckPerms groups, prefixes, permission nodes & inheritance"
	>
		{#snippet icon()}
			<ShieldCheck size={26} />
		{/snippet}
		{#snippet badge()}
			{#if loadError}
				<span class="badge badge-danger font-mono">Unavailable</span>
			{:else}
				<span class="badge badge-purple font-mono">{totalGroupsCount} Groups</span>
			{/if}
		{/snippet}
		<button class="btn btn-ghost btn-sm" onclick={loadGroups} disabled={loading} title="Refresh LuckPerms groups">
			<RefreshCw size={14} class={loading ? 'spinner' : ''} />
			<span>Refresh</span>
		</button>

		<button class="btn btn-primary" onclick={handleOpenCreateModal}>
			<Plus size={16} />
			<span>Create Group</span>
		</button>
	</PageHeader>

	{#if loadError}
		<!-- No groups could be read. Nothing is shown in their place. -->
		<div class="alert alert-danger unavailable-banner" role="alert">
			<AlertCircle size={18} />
			<div>
				<strong>LuckPerms groups unavailable</strong>
				<p class="unavailable-detail">{loadError}</p>
				<p class="unavailable-note">
					No group data is shown while the server cannot be reached — the panel does not keep a
					local copy of your permission setup.
				</p>
			</div>
			<button class="btn btn-secondary btn-sm" onclick={loadGroups} disabled={loading}>
				<RefreshCw size={14} />
				<span>Retry</span>
			</button>
		</div>
	{/if}

	<!-- Quick Telemetry & Stats Cards Bar -->
	<div class="stats-grid">
		<div class="hardware-shell">
			<div class="hardware-core stat-card">
				<div class="stat-card-body">
					<div class="stat-icon-wrapper blue">
						<Layers size={20} />
					</div>
					<div class="stat-details">
						<span class="stat-label">Total Groups</span>
						<span class="stat-value font-mono tabular-nums">{loadError ? '—' : totalGroupsCount}</span>
					</div>
				</div>
			</div>
		</div>

		<div class="hardware-shell">
			<div class="hardware-core stat-card">
				<div class="stat-card-body">
					<div class="stat-icon-wrapper green">
						<Key size={20} />
					</div>
					<div class="stat-details">
						<span class="stat-label">Assigned Permission Nodes</span>
						<span class="stat-value font-mono tabular-nums">{loadError ? '—' : totalPermissionsCount}</span>
					</div>
				</div>
			</div>
		</div>

		<div class="hardware-shell">
			<div class="hardware-core stat-card">
				<div class="stat-card-body">
					<div class="stat-icon-wrapper purple">
						<Crown size={20} />
					</div>
					<div class="stat-details">
						<span class="stat-label">Highest Priority Group</span>
						<span class="stat-value font-mono">
							{#if loadError}
								—
							{:else if highestWeightGroup}
								{highestWeightGroup.name} <span class="weight-sub tabular-nums">(w: {highestWeightGroup.weight})</span>
							{:else}
								None
							{/if}
						</span>
					</div>
				</div>
			</div>
		</div>
	</div>

	<!-- Main Group List Component -->
	<GroupList
		{groups}
		{loading}
		error={loadError}
		onEditGroup={handleOpenEditModal}
		onDeleteGroup={(group) => (groupPendingDelete = group)}
		onCreateGroup={handleOpenCreateModal}
		onRetry={loadGroups}
	/>

	<!-- Group Editor Modal -->
	<GroupEditorModal
		isOpen={isModalOpen}
		group={editingGroup}
		allGroups={groups}
		onSave={handleSaveGroup}
		onClose={() => (isModalOpen = false)}
	/>
</div>

<style>
	.permissions-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		position: relative;
	}

	/* Toast Notifications */
	.toast-container {
		position: fixed;
		top: var(--space-6);
		right: var(--space-6);
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		max-width: 380px;
		width: calc(100vw - 32px);
		pointer-events: none;
	}

	.toast {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: var(--elevation-shadow);
		pointer-events: auto;
		animation: toastSlideIn var(--transition-fast) cubic-bezier(0.16, 1, 0.3, 1);
	}

	@keyframes toastSlideIn {
		from {
			opacity: 0;
			transform: translateX(30px) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateX(0) scale(1);
		}
	}

	.toast-success {
		border-color: var(--accent-green-border);
	}

	.toast-success .toast-icon {
		color: var(--accent-green-text);
	}

	.toast-danger {
		border-color: var(--danger-border);
	}

	.toast-danger .toast-icon {
		color: var(--danger-text);
	}

	.toast-warning {
		border-color: var(--accent-orange-border);
	}

	.toast-warning .toast-icon {
		color: var(--accent-orange-text);
	}

	.toast-info {
		border-color: var(--accent-blue-border);
	}

	.toast-info .toast-icon {
		color: var(--accent-blue-text);
	}

	.toast-body {
		flex: 1;
		min-width: 0;
	}

	.toast-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		line-height: 1.3;
	}

	.toast-message {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin-top: 2px;
	}

	.toast-close {
		color: var(--text-muted);
		padding: 0;
		width: 24px;
		height: 24px;
	}

	/* Quick Stats Bar */
	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-4);
	}

	.stat-card {
		background-color: var(--bg-surface);
		border-radius: calc(var(--radius-card) - 6px);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.stat-card-body {
		padding: var(--space-4) var(--space-5);
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.stat-icon-wrapper {
		width: 42px;
		height: 42px;
		border-radius: var(--radius-card);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.stat-icon-wrapper.blue {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border: 1px solid var(--accent-blue-border);
	}

	.stat-icon-wrapper.green {
		background-color: var(--accent-green-bg);
		color: var(--accent-green);
		border: 1px solid var(--accent-green-border);
	}

	.stat-icon-wrapper.purple {
		background-color: rgba(139, 92, 246, 0.15);
		color: var(--accent-purple-text);
		border: 1px solid rgba(139, 92, 246, 0.3);
	}

	.stat-details {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.stat-value {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		line-height: 1.2;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.weight-sub {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: normal;
	}

	.unavailable-banner {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
	}

	.unavailable-banner div {
		flex: 1;
		min-width: 0;
	}

	.unavailable-detail {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		margin-top: 2px;
		word-break: break-word;
	}

	.unavailable-note {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: var(--space-1);
	}

	.badge-purple {
		background-color: rgba(139, 92, 246, 0.12);
		color: var(--accent-purple-text);
		border-color: rgba(139, 92, 246, 0.25);
	}

	.font-mono {
		font-family: var(--font-mono);
	}

	.spinner {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
