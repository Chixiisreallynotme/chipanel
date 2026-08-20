<script>
	import {
		Shield,
		Crown,
		User,
		ShieldCheck,
		ShieldAlert,
		Sparkles,
		Key,
		GitFork,
		Edit,
		Trash2,
		Plus,
		Search,
		LayoutGrid,
		List,
		X,
		Layers,
		ArrowUpRight
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

	let {
		groups = [],
		loading = false,
		error = '',
		onEditGroup = (group) => {},
		onDeleteGroup = (group) => {},
		onCreateGroup = () => {},
		onRetry = () => {}
	} = $props();

	// Component State
	let viewMode = $state('card'); // 'card' | 'list'
	let searchQuery = $state('');

	// Derived filtered group list
	let filteredGroups = $derived.by(() => {
		const q = searchQuery.trim().toLowerCase();
		if (!q) return groups;
		return groups.filter((g) => {
			const nameMatch = g.name.toLowerCase().includes(q);
			const prefixMatch = g.prefix && g.prefix.toLowerCase().includes(q);
			const suffixMatch = g.suffix && g.suffix.toLowerCase().includes(q);
			const parentMatch = g.parents && g.parents.some((p) => p.toLowerCase().includes(q));
			return nameMatch || prefixMatch || suffixMatch || parentMatch;
		});
	});

	function getWeightBadgeClass(weight) {
		const w = weight || 0;
		if (w >= 100) return 'badge-danger';
		if (w >= 50) return 'badge-purple';
		if (w >= 20) return 'badge-blue';
		if (w > 0) return 'badge-warning';
		return 'badge-secondary';
	}

	function getGroupIconComponent(groupName, weight = 0) {
		const name = (groupName || '').toLowerCase();
		if (name === 'owner' || weight >= 100) return Crown;
		if (name === 'admin' || weight >= 80) return ShieldAlert;
		if (name === 'mod' || name === 'moderator' || weight >= 50) return ShieldCheck;
		if (name === 'vip' || name === 'premium' || weight >= 20) return Sparkles;
		return User;
	}

	function getNodeCount(group) {
		if (typeof group.permissions_count === 'number') return group.permissions_count;
		if (Array.isArray(group.permissions)) return group.permissions.length;
		return 0;
	}
</script>

<div class="group-list-container">
	<!-- Control Bar: Search, View Mode Switcher, Create Group Button -->
	<div class="list-control-bar">
		<div class="search-box">
			<Search size={16} class="search-icon" />
			<input
				type="text"
				class="input search-input"
				placeholder="Search groups by name, prefix, or parents..."
				aria-label="Search groups by name, prefix, or parents"
				bind:value={searchQuery}
			/>
			{#if searchQuery}
				<button
					class="btn btn-ghost btn-icon btn-sm clear-btn"
					onclick={() => (searchQuery = '')}
					title="Clear search"
				>
					<X size={14} />
				</button>
			{/if}
		</div>

		<div class="actions-group">
			<!-- View Switcher -->
			<div class="view-switcher" role="radiogroup" aria-label="Group view mode">
				<button
					class="switch-btn {viewMode === 'card' ? 'active' : ''}"
					onclick={() => (viewMode = 'card')}
					title="Grid Card View"
					aria-label="Grid Card View"
					role="radio"
					aria-checked={viewMode === 'card'}
				>
					<LayoutGrid size={16} />
				</button>
				<button
					class="switch-btn {viewMode === 'list' ? 'active' : ''}"
					onclick={() => (viewMode = 'list')}
					title="Table List View"
					aria-label="Table List View"
					role="radio"
					aria-checked={viewMode === 'list'}
				>
					<List size={16} />
				</button>
			</div>

			<!-- Create Group Button -->
			<button class="btn btn-primary" onclick={onCreateGroup}>
				<Plus size={16} />
				<span>Create Group</span>
			</button>
		</div>
	</div>

	<!-- Main Content Area -->
	{#if error}
		<!-- Groups could not be read from the server: show nothing in their place. -->
		<div class="empty-state card" role="alert">
			<ShieldAlert size={40} class="unavailable-icon" />
			<h3>Groups Unavailable</h3>
			<p class="unavailable-detail">{error}</p>
			<p>The server's group list could not be read, so none is shown.</p>
			<button class="btn btn-secondary btn-sm" onclick={onRetry} disabled={loading}>
				<span>Retry</span>
			</button>
		</div>
	{:else if loading && groups.length === 0}
		<div class="loading-state card">
			<div class="spinner-ring"></div>
			<p>Loading LuckPerms group configurations...</p>
		</div>
	{:else if filteredGroups.length === 0}
		<div class="empty-state-shell hardware-shell">
			<div class="empty-state hardware-core card">
				<div class="empty-icon-box">
					<Layers size={36} class="empty-icon" />
				</div>
				{#if searchQuery}
					<h3 class="empty-state-title">No Matching LuckPerms Groups</h3>
					<p class="empty-state-desc">No permission group matches search query "{searchQuery}".</p>
					<button class="btn btn-secondary btn-sm" onclick={() => (searchQuery = '')}>
						Clear Filter
					</button>
				{:else}
					<h3 class="empty-state-title">No Groups Configured</h3>
					<p class="empty-state-desc">Get started by creating your first LuckPerms permission group.</p>
					<button class="btn btn-primary btn-md create-first-btn" onclick={onCreateGroup}>
						<Plus size={16} />
						<span>Create First Group</span>
					</button>
				{/if}
			</div>
		</div>
	{:else if viewMode === 'card'}
		<!-- Grid / Card View -->
		<div class="cards-grid">
			{#each filteredGroups as group (group.name)}
				{@const IconComponent = getGroupIconComponent(group.name, group.weight)}
				{@const nodeCount = getNodeCount(group)}
				<div class="card group-card">
					<div class="card-header group-card-top">
						<div class="group-identity">
							<div class="group-avatar {getWeightBadgeClass(group.weight)}">
								<IconComponent size={20} />
							</div>
							<div class="group-title-box">
								<h3 class="group-name">{group.name}</h3>
								<span class="badge {getWeightBadgeClass(group.weight)} weight-badge">
									Weight: {group.weight ?? 0}
								</span>
							</div>
						</div>
					</div>

					<div class="card-body group-card-details">
						<!-- Prefix & Suffix Preview -->
						<div class="meta-preview-section">
							<span class="section-label">Chat Formatting</span>
							<div class="prefix-suffix-box">
								{#if group.prefix}
									<span class="prefix-tag" title="Group Prefix">{group.prefix}</span>
								{:else}
									<span class="no-meta-text">No prefix</span>
								{/if}
								{#if group.suffix}
									<span class="suffix-tag" title="Group Suffix">{group.suffix}</span>
								{/if}
							</div>
						</div>

						<!-- Nodes Count & Inheritance Row -->
						<div class="group-meta-stats">
							<div class="stat-pill" title="Permission Node Count">
								<Key size={14} class="stat-icon" />
								<span>{nodeCount} {nodeCount === 1 ? 'Permission' : 'Permissions'}</span>
							</div>

							<div class="stat-pill" title="Inherited Parent Groups">
								<GitFork size={14} class="stat-icon" />
								<span>{group.parents?.length || 0} Parents</span>
							</div>
						</div>

						<!-- Parent Inheritance Tags -->
						{#if group.parents && group.parents.length > 0}
							<div class="parents-tags-wrapper">
								<span class="section-label">Inherits From:</span>
								<div class="parents-tags-list">
									{#each group.parents as parent}
										<span class="badge badge-secondary parent-tag">{parent}</span>
									{/each}
								</div>
							</div>
						{:else}
							<div class="parents-tags-wrapper">
								<span class="section-label">Inherits From:</span>
								<span class="no-meta-text">Direct group (no parents)</span>
							</div>
						{/if}
					</div>

					<div class="card-footer group-card-actions">
						<button
							class="btn btn-secondary btn-sm edit-btn"
							onclick={() => onEditGroup(group)}
						>
							<Edit size={14} />
							<span>Edit Group</span>
						</button>
						<button
							class="btn btn-danger btn-sm delete-btn"
							onclick={() => onDeleteGroup(group)}
							title="Delete group '{group.name}'"
						>
							<Trash2 size={14} />
							<span>Delete</span>
						</button>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<!-- Table / List View -->
		<div class="table-container">
			<table class="table groups-table">
				<thead>
					<tr>
						<th>Group Name</th>
						<th>Weight</th>
						<th>Prefix Preview</th>
						<th>Permission Nodes</th>
						<th>Inherited Parents</th>
						<th class="text-right">Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredGroups as group (group.name)}
						{@const IconComponent = getGroupIconComponent(group.name, group.weight)}
						{@const nodeCount = getNodeCount(group)}
						<tr>
							<td class="name-cell">
								<div class="table-group-identity">
									<div class="table-avatar {getWeightBadgeClass(group.weight)}">
										<IconComponent size={16} />
									</div>
									<span class="table-group-name">{group.name}</span>
								</div>
							</td>

							<td>
								<span class="badge {getWeightBadgeClass(group.weight)}">
									{group.weight ?? 0}
								</span>
							</td>

							<td>
								{#if group.prefix}
									<code class="prefix-code">{group.prefix}</code>
								{:else}
									<span class="no-meta-text">—</span>
								{/if}
							</td>

							<td>
								<div class="stat-pill inline-pill">
									<Key size={13} class="stat-icon" />
									<span>{nodeCount}</span>
								</div>
							</td>

							<td>
								{#if group.parents && group.parents.length > 0}
									<div class="parents-inline-list">
										{#each group.parents as parent}
											<span class="badge badge-secondary parent-tag">{parent}</span>
										{/each}
									</div>
								{:else}
									<span class="no-meta-text">None</span>
								{/if}
							</td>

							<td class="text-right actions-cell">
								<button
									class="btn btn-secondary btn-sm"
									onclick={() => onEditGroup(group)}
								>
									<Edit size={14} />
									<span>Edit</span>
								</button>
								<button
									class="btn btn-ghost btn-sm btn-icon danger-icon-btn"
									onclick={() => onDeleteGroup(group)}
									title="Delete {group.name}"
								>
									<Trash2 size={14} />
								</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.group-list-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	/* Control Bar */
	.list-control-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.search-box {
		position: relative;
		display: flex;
		align-items: center;
		flex: 1;
		min-width: 280px;
		max-width: 480px;
	}

	.search-icon {
		position: absolute;
		left: 12px;
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		padding-left: 38px;
		padding-right: 32px;
		height: 40px;
	}

	.clear-btn {
		position: absolute;
		right: 4px;
		width: 28px;
		height: 28px;
		padding: 0;
	}

	.actions-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	/* View Switcher */
	.view-switcher {
		display: inline-flex;
		align-items: center;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		padding: 3px;
		gap: 2px;
	}

	.switch-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		border-radius: var(--radius-input);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast);
	}

	.switch-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.05);
	}

	.switch-btn.active {
		background-color: var(--bg-elevated);
		color: var(--accent-blue-text);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	/* Cards Grid */
	.cards-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: var(--space-4);
	}

	.group-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		transition: border-color var(--transition-fast), transform var(--transition-fast), box-shadow var(--transition-fast);
	}

	.group-card:hover {
		border-color: var(--border-focus);
		box-shadow: 0 8px 24px -6px rgba(0, 0, 0, 0.5);
	}

	.group-card-top {
		padding: var(--space-4) var(--space-5);
		border-bottom: 1px solid var(--border-subtle);
	}

	.group-identity {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.group-avatar {
		width: 38px;
		height: 38px;
		border-radius: var(--radius-card);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.group-avatar.badge-danger {
		background-color: var(--danger-bg);
		color: var(--danger-text);
		border: 1px solid var(--danger-border);
	}

	.group-avatar.badge-purple {
		background-color: rgba(139, 92, 246, 0.15);
		color: var(--accent-purple-text);
		border: 1px solid rgba(139, 92, 246, 0.3);
	}

	.group-avatar.badge-blue {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border: 1px solid var(--accent-blue-border);
	}

	.group-avatar.badge-warning {
		background-color: var(--warning-bg);
		color: var(--warning);
		border: 1px solid var(--warning-border);
	}

	.group-avatar.badge-secondary {
		background-color: var(--bg-elevated);
		color: var(--text-secondary);
		border: 1px solid var(--border);
	}

	.group-title-box {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.group-name {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		line-height: 1.2;
	}

	.weight-badge {
		font-size: var(--font-size-xs);
		align-self: flex-start;
		font-family: var(--font-mono);
	}

	.group-card-details {
		padding: var(--space-4) var(--space-5);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.meta-preview-section {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.section-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.prefix-suffix-box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background-color: var(--bg-base);
		padding: 6px var(--space-3);
		border-radius: var(--radius-input);
		border: 1px solid var(--border-subtle);
		min-height: 32px;
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
	}

	.prefix-tag {
		color: var(--accent-green);
		font-weight: var(--font-weight-semibold);
	}

	.suffix-tag {
		color: var(--warning);
		font-weight: var(--font-weight-semibold);
	}

	.no-meta-text {
		color: var(--text-muted);
		font-size: var(--font-size-xs);
		font-style: italic;
	}

	.group-meta-stats {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.stat-pill {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		padding: 4px 10px;
		border-radius: var(--radius-badge);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.stat-icon {
		color: var(--accent-blue-text);
	}

	.parents-tags-wrapper {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.parents-tags-list {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.parent-tag {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
	}

	.group-card-actions {
		padding: var(--space-3) var(--space-5);
		justify-content: space-between;
	}

	.edit-btn {
		flex: 1;
	}

	/* Table View */
	.table-group-identity {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.table-avatar {
		width: 28px;
		height: 28px;
		border-radius: var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.table-group-name {
		font-weight: var(--font-weight-semibold);
		font-size: var(--font-size-base);
	}

	.prefix-code {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--accent-green);
		background-color: var(--accent-green-bg);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
	}

	.inline-pill {
		padding: 2px 8px;
	}

	.parents-inline-list {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.actions-cell {
		white-space: nowrap;
	}

	.danger-icon-btn {
		color: var(--text-muted);
	}

	.danger-icon-btn:hover {
		color: var(--danger-text);
		background-color: var(--danger-bg);
	}

	.text-right {
		text-align: right;
	}

	/* Loading & Empty States */
	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-12) var(--space-6);
		gap: var(--space-3);
		text-align: center;
		color: var(--text-muted);
	}

	.empty-state-shell {
		width: 100%;
	}

	.empty-icon-box {
		width: 64px;
		height: 64px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: var(--space-2);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
	}

	.empty-icon-box .empty-icon {
		color: var(--accent-blue-text);
	}

	.empty-icon {
		color: var(--text-muted);
	}

	.empty-state-title {
		color: var(--text-primary);
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		letter-spacing: -0.01em;
	}

	.empty-state-desc {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		max-width: 42ch;
	}

	.create-first-btn {
		margin-top: var(--space-2);
		height: 38px;
		padding: 0 var(--space-5);
		font-size: var(--font-size-sm);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15), 0 2px 8px rgba(0, 0, 0, 0.25);
		transition: transform 160ms var(--ease-out),
		            background-color 150ms var(--ease-out),
		            border-color 150ms var(--ease-out),
		            box-shadow 150ms var(--ease-out);
	}

	.create-first-btn:active {
		transform: scale(0.97);
	}

	.empty-state h3 {
		color: var(--text-primary);
		font-size: var(--font-size-lg);
	}

	:global(.unavailable-icon) {
		color: var(--danger-text);
	}

	.unavailable-detail {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-primary);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-btn);
		padding: var(--space-2) var(--space-4);
		max-width: 560px;
		word-break: break-word;
	}

	.spinner-ring {
		width: 32px;
		height: 32px;
		border: 3px solid var(--border-focus);
		border-top-color: var(--accent-blue);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
