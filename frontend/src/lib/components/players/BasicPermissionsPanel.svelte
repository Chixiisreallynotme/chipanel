<script>
	import { apiGet, apiPost } from '$lib/api/client.js';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import {
		Shield,
		ShieldAlert,
		ShieldCheck,
		Crown,
		User,
		Sparkles,
		Search,
		CheckCircle2,
		AlertCircle,
		Loader2,
		RefreshCw,
		Sliders,
		Key,
		Clock,
		ArrowRight,
		X
	} from '$lib/icons.js';

	/**
	 * @typedef {Object} PermissionNodeInfo
	 * @property {string} permission
	 * @property {boolean} value
	 * @property {string|null} expiry
	 * @property {string|null} context
	 */

	/**
	 * @typedef {Object} PlayerPermissionsResponse
	 * @property {string|null} primary_group
	 * @property {PermissionNodeInfo[]} permissions
	 * @property {string} raw_output
	 */

	let { uuid = '' } = $props();

	// State
	let permissionsData = $state(null);
	let loading = $state(true);
	let error = $state('');
	let actionMessage = $state('');
	let settingGroup = $state(false);

	let PrimaryIconComp = $derived(getGroupIconComponent(permissionsData?.primary_group));

	// Form & Search inputs
	let selectedGroup = $state('default');
	let searchQuery = $state('');

	const groupOptions = [
		{ value: 'default', label: 'default' },
		{ value: 'vip', label: 'vip' },
		{ value: 'mod', label: 'mod' },
		{ value: 'admin', label: 'admin' },
		{ value: 'owner', label: 'owner' }
	];

	// Fetch permissions data whenever uuid changes or component mounts
	$effect(() => {
		if (uuid) {
			fetchPermissions();
		}
	});

	async function fetchPermissions() {
		if (!uuid) return;
		loading = true;
		error = '';
		try {
			const res = await apiGet(`/api/players/${uuid}/permissions`);
			permissionsData = res;
			if (res?.primary_group) {
				const norm = res.primary_group.toLowerCase();
				if (['default', 'vip', 'mod', 'admin', 'owner'].includes(norm)) {
					selectedGroup = norm;
				} else if (norm === 'moderator') {
					selectedGroup = 'mod';
				} else {
					selectedGroup = norm;
				}
			}
		} catch (err) {
			console.error(`Failed to fetch permissions for player ${uuid}:`, err);
			error = err.message || 'Failed to load player permissions';
		} finally {
			loading = false;
		}
	}

	async function handleSetGroup(e) {
		if (e) e.preventDefault();
		if (!uuid || settingGroup) return;
		settingGroup = true;
		error = '';
		actionMessage = '';

		try {
			const res = await apiPost(`/api/players/${uuid}/permissions/group`, {
				group_name: selectedGroup
			});
			actionMessage = res.message || `Primary LuckPerms group changed to '${selectedGroup}'.`;
			await fetchPermissions();
		} catch (err) {
			console.error(`Failed to set player group:`, err);
			error = err.message || 'Failed to update player group';
		} finally {
			settingGroup = false;
		}
	}

	// Filtered permission nodes based on search query
	let filteredPermissions = $derived.by(() => {
		const list = permissionsData?.permissions || [];
		const q = searchQuery.trim().toLowerCase();
		if (!q) return list;
		return list.filter(
			(item) =>
				item.permission.toLowerCase().includes(q) ||
				(item.context && item.context.toLowerCase().includes(q))
		);
	});

	function getGroupBadgeClass(groupStr) {
		const g = (groupStr || '').toLowerCase();
		if (g === 'owner') return 'badge-danger';
		if (g === 'admin') return 'badge-purple';
		if (g === 'mod' || g === 'moderator') return 'badge-warning';
		if (g === 'vip') return 'badge-blue';
		return 'badge-secondary';
	}

	function getGroupIconComponent(groupStr) {
		const g = (groupStr || '').toLowerCase();
		if (g === 'owner') return Crown;
		if (g === 'admin') return ShieldAlert;
		if (g === 'mod' || g === 'moderator') return ShieldCheck;
		if (g === 'vip') return Sparkles;
		return User;
	}

	function formatContext(contextStr) {
		if (!contextStr) return 'Direct user permission';
		if (contextStr.toLowerCase().startsWith('inherited')) {
			return contextStr;
		}
		return `Inherited from ${contextStr}`;
	}
</script>

<div class="permissions-panel-container">
	<!-- Top Alerts -->
	{#if error}
		<div class="alert alert-danger" role="alert">
			<AlertCircle size={18} />
			<span>{error}</span>
		</div>
	{/if}

	{#if actionMessage}
		<div class="alert alert-success" role="alert">
			<CheckCircle2 size={18} />
			<span>{actionMessage}</span>
		</div>
	{/if}

	<!-- Primary Group & Group Change Card -->
	<div class="card group-card">
		<div class="card-header group-card-header">
			<div class="group-info-main">
				<div class="group-icon-avatar">
					<PrimaryIconComp size={20} />
				</div>
				<div>
					<div class="primary-group-title-row">
						<span class="group-label-text">LuckPerms Primary Group:</span>
						<span class="badge {getGroupBadgeClass(permissionsData?.primary_group)} group-primary-badge">
							{#if permissionsData?.primary_group}
								{permissionsData.primary_group}
							{:else}
								default
							{/if}
						</span>
					</div>
					<p class="group-subtitle">Manages overall player permissions & role inheritance</p>
				</div>
			</div>

			<button
				class="btn btn-ghost btn-sm"
				onclick={fetchPermissions}
				disabled={loading}
				title="Refresh LuckPerms state"
			>
				<RefreshCw size={14} class={loading ? 'spinner' : ''} />
				<span>Refresh</span>
			</button>
		</div>

		<div class="card-body group-change-body">
			<form class="group-change-form" onsubmit={handleSetGroup}>
				<div class="group-select-field">
					<label for="group-select" class="label">Change Group</label>
					<div class="select-inline-group">
						<select id="group-select" class="select" bind:value={selectedGroup}>
							{#each groupOptions as option}
								<option value={option.value}>{option.label}</option>
							{/each}
						</select>

						<button
							type="submit"
							class="btn btn-primary {settingGroup ? 'btn-loading' : ''}"
							disabled={settingGroup}
						>
							{#if !settingGroup}
								<ArrowRight size={16} />
							{/if}
							<span>Set Group</span>
						</button>
					</div>
				</div>
			</form>
		</div>
	</div>

	<!-- Permission Nodes Section -->
	<div class="card nodes-card">
		<div class="card-header nodes-card-header">
			<div class="nodes-header-title">
				<Key size={18} class="icon-blue" />
				<div>
					<h4 class="card-title">Permission Nodes</h4>
					<p class="card-subtitle">Active and inherited node rules parsed from LuckPerms</p>
				</div>
			</div>

			<!-- Search Filter -->
			<div class="search-input-wrapper">
				<Search size={14} class="search-icon" />
				<input
					type="text"
					class="input search-input"
					placeholder="Search permission nodes..."
					aria-label="Search permission nodes"
					bind:value={searchQuery}
				/>
				{#if searchQuery}
					<button class="btn btn-ghost btn-sm clear-search-btn" onclick={() => (searchQuery = '')} aria-label="Clear search">
						<X size={12} />
					</button>
				{/if}
			</div>
		</div>

		<div class="card-body nodes-card-body">
			{#if loading && !permissionsData}
				<div class="loading-state">
					<Loader2 size={28} class="spinner" />
					<span>Querying LuckPerms permissions database...</span>
				</div>
			{:else if filteredPermissions.length === 0}
				<EmptyState
						title={searchQuery ? 'No Matching Permissions' : 'No Permission Nodes Found'}
						description={searchQuery ? `No permission nodes match your search query "${searchQuery}".` : 'No explicit permission nodes registered for this player or LuckPerms info not returned.'}
					>
						{#snippet icon()}
							<Sliders size={36} class="icon-muted" />
						{/snippet}
					</EmptyState>
			{:else}
				<div class="table-container nodes-table-container">
					<table class="table nodes-table">
						<thead>
							<tr>
								<th>Permission Node</th>
								<th>State</th>
								<th>Origin Context</th>
								<th>Expiry</th>
							</tr>
						</thead>
						<tbody>
							{#each filteredPermissions as node (node.permission)}
								<tr>
									<td class="node-cell">
										<code class="node-code">{node.permission}</code>
									</td>
									<td class="state-cell">
										{#if node.value}
											<span class="badge badge-success state-badge">TRUE</span>
										{:else}
											<span class="badge badge-danger state-badge">FALSE</span>
										{/if}
									</td>
									<td class="context-cell">
										<span class="context-text">{formatContext(node.context)}</span>
									</td>
									<td class="expiry-cell">
										{#if node.expiry}
											<span class="badge badge-warning expiry-badge">
												<Clock size={11} />
												{node.expiry}
											</span>
										{:else}
											<span class="no-expiry">—</span>
										{/if}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.permissions-panel-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.alert {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-input);
		font-size: var(--font-size-sm);
	}

	.alert-danger {
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
	}

	.alert-success {
		background-color: var(--accent-green-bg);
		border: 1px solid var(--accent-green-border);
		color: var(--accent-green);
	}

	/* Group Card */
	.group-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
	}

	.group-card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
	}

	.group-info-main {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.group-icon-avatar {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-card);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-focus);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-purple-text);
	}

	.primary-group-title-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.group-label-text {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--text-secondary);
	}

	.group-primary-badge {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		text-transform: lowercase;
		font-family: var(--font-mono);
	}

	.group-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.group-change-body {
		padding: var(--space-4) var(--space-6);
		border-top: 1px solid var(--border-subtle);
		background-color: rgba(0, 0, 0, 0.1);
	}

	.group-change-form {
		display: flex;
		align-items: flex-end;
	}

	.group-select-field {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		width: 100%;
		max-width: 400px;
	}

	.select-inline-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	/* Nodes Card */
	.nodes-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
	}

	.nodes-card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.nodes-header-title {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.search-input-wrapper {
		position: relative;
		display: flex;
		align-items: center;
		width: 260px;
	}

	.search-icon {
		position: absolute;
		left: 10px;
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		padding-left: 32px;
		padding-right: 28px;
		height: 36px;
		font-size: var(--font-size-xs);
	}

	.clear-search-btn {
		position: absolute;
		right: 4px;
		padding: 0;
		width: 24px;
		height: 24px;
		color: var(--text-muted);
	}

	.nodes-card-body {
		padding: var(--space-4);
	}

	.loading-state,
	.nodes-table-container {
		border-radius: var(--radius-input);
	}

	.nodes-table th {
		background-color: var(--bg-surface);
	}

	.node-cell {
		max-width: 320px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.node-code {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--accent-blue-text);
		background-color: var(--accent-blue-bg);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
	}

	.state-badge {
		font-size: 0.7rem;
		font-weight: var(--font-weight-bold);
		padding: 2px 6px;
	}

	.context-text {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.expiry-badge {
		font-size: var(--font-size-xs);
	}

	.no-expiry {
		color: var(--text-muted);
		font-size: var(--font-size-xs);
	}

	.badge-purple {
		background-color: rgba(168, 85, 247, 0.12);
		color: var(--accent-purple-text);
		border-color: rgba(168, 85, 247, 0.25);
	}

	.icon-blue {
		color: var(--accent-blue-text);
	}

	.icon-muted {
		color: var(--text-muted);
	}

	.spinner {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
