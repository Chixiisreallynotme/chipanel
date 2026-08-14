<script>
	import {
		Search,
		Filter,
		RefreshCw,
		Trash2,
		Power,
		Package,
		Boxes,
		CheckCircle2,
		XCircle,
		AlertTriangle,
		X,
		FileText,
		HardDrive,
		Loader2
	} from 'lucide-svelte';

	/**
	 * @typedef {Object} InstalledPlugin
	 * @property {string} name
	 * @property {string} version
	 * @property {boolean} enabled
	 * @property {string} filename
	 * @property {number} file_size_bytes
	 * @property {string} loader_type
	 * @property {string} target_dir
	 */

	let {
		plugins = [],
		loading = false,
		onToggle = async () => {},
		onDelete = async () => {},
		onRefresh = () => {},
		initialTargetDir = 'all',
		lockFolderFilter = false
	} = $props();

	// Local filtering and modal states
	let searchQuery = $state('');
	let filterTargetDir = $state(initialTargetDir); // 'all', 'plugins', 'mods'
	let filterStatus = $state('all'); // 'all', 'enabled', 'disabled'
	let filterLoader = $state('all'); // 'all', 'paper', 'spigot', 'fabric', 'forge', 'purpur'

	// Action tracking states
	let togglingFilename = $state(null);
	let deleteModalOpen = $state(false);
	let pluginToDelete = $state(null);
	let isDeleting = $state(false);

	// Computed statistics
	let stats = $derived.by(() => {
		const total = plugins.length;
		const enabledCount = plugins.filter((p) => p.enabled).length;
		const disabledCount = total - enabledCount;
		const pluginsCount = plugins.filter((p) => p.target_dir === 'plugins').length;
		const modsCount = plugins.filter((p) => p.target_dir === 'mods').length;
		return { total, enabledCount, disabledCount, pluginsCount, modsCount };
	});

	// Filtered plugins array
	let filteredPlugins = $derived.by(() => {
		return plugins.filter((p) => {
			// Search query match
			if (searchQuery.trim()) {
				const q = searchQuery.toLowerCase().trim();
				const nameMatch = (p.name || '').toLowerCase().includes(q);
				const fileMatch = (p.filename || '').toLowerCase().includes(q);
				if (!nameMatch && !fileMatch) return false;
			}

			// Target dir match
			if (filterTargetDir !== 'all' && p.target_dir !== filterTargetDir) {
				return false;
			}

			// Status match
			if (filterStatus === 'enabled' && !p.enabled) return false;
			if (filterStatus === 'disabled' && p.enabled) return false;

			// Loader type match
			if (filterLoader !== 'all') {
				const loader = (p.loader_type || '').toLowerCase();
				if (filterLoader === 'spigot') {
					if (loader !== 'spigot' && loader !== 'paper' && loader !== 'purpur') return false;
				} else if (loader !== filterLoader) {
					return false;
				}
			}

			return true;
		});
	});

	// Formatting Helpers
	function formatFileSize(bytes) {
		if (!bytes || bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatLoaderBadge(loader) {
		const l = (loader || '').toLowerCase();
		if (l === 'paper' || l === 'spigot' || l === 'purpur') return 'Paper/Spigot';
		if (l === 'fabric') return 'Fabric';
		if (l === 'forge') return 'Forge';
		return loader ? loader.charAt(0).toUpperCase() + loader.slice(1) : 'Spigot';
	}

	function getLoaderBadgeClass(loader) {
		const l = (loader || '').toLowerCase();
		if (l === 'fabric') return 'badge-blue';
		if (l === 'forge') return 'badge-warning';
		if (l === 'paper' || l === 'spigot' || l === 'purpur') return 'badge-success';
		return 'badge-secondary';
	}

	// Action Handlers
	async function handleToggleClick(plugin) {
		if (togglingFilename) return;
		togglingFilename = plugin.filename;
		try {
			await onToggle(plugin);
		} finally {
			togglingFilename = null;
		}
	}

	function openDeleteModal(plugin) {
		pluginToDelete = plugin;
		deleteModalOpen = true;
	}

	function closeDeleteModal() {
		if (isDeleting) return;
		deleteModalOpen = false;
		pluginToDelete = null;
	}

	async function confirmDelete() {
		if (!pluginToDelete || isDeleting) return;
		isDeleting = true;
		try {
			await onDelete(pluginToDelete);
			closeDeleteModal();
		} catch (err) {
			console.error('Delete error:', err);
		} finally {
			isDeleting = false;
		}
	}

	function handleWindowKeydown(e) {
		if (e.key === 'Escape' && deleteModalOpen) {
			closeDeleteModal();
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="installed-plugins-container">
	<!-- Summary Stats Bar -->
	<div class="stats-row">
		<div class="stat-card">
			<div class="stat-icon icon-blue">
				<Package size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.total}</span>
				<span class="stat-label">Total Installed</span>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon icon-green">
				<CheckCircle2 size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.enabledCount}</span>
				<span class="stat-label">Enabled</span>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon icon-danger">
				<XCircle size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.disabledCount}</span>
				<span class="stat-label">Disabled</span>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon icon-purple">
				<Boxes size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.pluginsCount} P / {stats.modsCount} M</span>
				<span class="stat-label">Plugins vs Mods</span>
			</div>
		</div>
	</div>

	<!-- Filter & Search Controls Header -->
	<div class="controls-card card">
		<div class="controls-body">
			<div class="search-input-wrapper">
				<Search size={18} class="search-icon" />
				<input
					type="text"
					class="input search-input"
					placeholder="Search plugins and mods by name or filename..."
					bind:value={searchQuery}
					aria-label="Search plugins and mods"
				/>
				{#if searchQuery}
					<button
						class="btn btn-ghost btn-icon btn-sm clear-search-btn"
						onclick={() => (searchQuery = '')}
						title="Clear search"
					>
						<X size={14} />
					</button>
				{/if}
			</div>

			<div class="filter-group">
				<!-- Target Directory Filter -->
				<div class="filter-select-wrapper">
					<span class="filter-label">Folder:</span>
					<select class="select select-sm" bind:value={filterTargetDir} aria-label="Filter by folder">
						<option value="all">All Folders</option>
						<option value="plugins">plugins/</option>
						<option value="mods">mods/</option>
					</select>
				</div>

				<!-- Status Filter -->
				<div class="filter-select-wrapper">
					<span class="filter-label">Status:</span>
					<select class="select select-sm" bind:value={filterStatus} aria-label="Filter by status">
						<option value="all">All Statuses</option>
						<option value="enabled">Enabled Only</option>
						<option value="disabled">Disabled Only</option>
					</select>
				</div>

				<!-- Loader Filter -->
				<div class="filter-select-wrapper">
					<span class="filter-label">Loader:</span>
					<select class="select select-sm" bind:value={filterLoader} aria-label="Filter by loader">
						<option value="all">All Loaders</option>
						<option value="spigot">Paper/Spigot</option>
						<option value="fabric">Fabric</option>
						<option value="forge">Forge</option>
					</select>
				</div>

				<button
					class="btn btn-secondary btn-sm refresh-btn {loading ? 'btn-loading' : ''}"
					onclick={onRefresh}
					disabled={loading}
					title="Refresh list"
				>
					{#if !loading}
						<RefreshCw size={14} />
					{/if}
					<span>Refresh</span>
				</button>
			</div>
		</div>
	</div>

	<!-- Plugins Data Table -->
	<div class="table-container shadow-card">
		{#if loading && plugins.length === 0}
			<div class="loading-state">
				<Loader2 size={32} class="spinner" />
				<p>Scanning server plugins and mods directories...</p>
			</div>
		{:else if filteredPlugins.length === 0}
			<div class="table-empty">
				<Package size={40} class="empty-icon" />
				<h3>No Plugins or Mods Found</h3>
				<p>
					{#if searchQuery || filterTargetDir !== 'all' || filterStatus !== 'all' || filterLoader !== 'all'}
						No installed extensions match your current filter criteria.
					{:else}
						You don't have any plugins or mods installed on your server yet. Use the Modrinth Store Catalog tab to discover and install add-ons!
					{/if}
				</p>
			</div>
		{:else}
			<table class="table">
				<thead>
					<tr>
						<th>Extension Name & File</th>
						<th>Version</th>
						<th>Loader</th>
						<th>Directory</th>
						<th>Size</th>
						<th>Status</th>
						<th class="text-right">Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredPlugins as plugin (plugin.filename)}
						<tr class={!plugin.enabled ? 'row-disabled' : ''}>
							<!-- Name & Filename -->
							<td>
								<div class="plugin-name-cell">
									<div class="file-icon-box {plugin.enabled ? 'icon-active' : 'icon-inactive'}">
										<FileText size={18} />
									</div>
									<div class="name-info">
										<span class="plugin-title">{plugin.name}</span>
										<code class="plugin-filename">{plugin.filename}</code>
									</div>
								</div>
							</td>

							<!-- Version -->
							<td>
								<span class="version-tag">v{plugin.version}</span>
							</td>

							<!-- Loader Badge -->
							<td>
								<span class="badge {getLoaderBadgeClass(plugin.loader_type)}">
									{formatLoaderBadge(plugin.loader_type)}
								</span>
							</td>

							<!-- Target Directory -->
							<td>
								<span class="badge badge-secondary dir-badge">
									<HardDrive size={12} />
									{plugin.target_dir}/
								</span>
							</td>

							<!-- File Size -->
							<td>
								<span class="file-size">{formatFileSize(plugin.file_size_bytes)}</span>
							</td>

							<!-- Status Badge -->
							<td>
								{#if plugin.enabled}
									<span class="badge badge-success">
										<span class="status-dot status-dot-success status-dot-pulse"></span>
										Enabled
									</span>
								{:else}
									<span class="badge badge-danger">
										<span class="status-dot status-dot-danger"></span>
										Disabled
									</span>
								{/if}
							</td>

							<!-- Actions -->
							<td class="text-right">
								<div class="action-buttons">
									<!-- Toggle Enable / Disable -->
									<button
										class="btn btn-sm {plugin.enabled ? 'btn-secondary' : 'btn-primary'} {togglingFilename === plugin.filename ? 'btn-loading' : ''}"
										disabled={togglingFilename === plugin.filename}
										onclick={() => handleToggleClick(plugin)}
										title={plugin.enabled ? 'Disable plugin' : 'Enable plugin'}
									>
										{#if togglingFilename !== plugin.filename}
											<Power size={14} />
										{/if}
										<span>{plugin.enabled ? 'Disable' : 'Enable'}</span>
									</button>

									<!-- Delete Button -->
									<button
										class="btn btn-ghost btn-icon btn-sm delete-action-btn"
										onclick={() => openDeleteModal(plugin)}
										title="Delete plugin file"
									>
										<Trash2 size={16} />
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

<!-- Delete Confirmation Modal -->
{#if deleteModalOpen && pluginToDelete}
	<div
		class="modal-backdrop"
		onclick={closeDeleteModal}
		aria-hidden="true"
	>
		<div
			class="modal"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="delete-modal-title"
		>
			<div class="modal-header">
				<div class="modal-title-row">
					<AlertTriangle size={20} class="text-danger" />
					<h3 id="delete-modal-title" class="modal-title">Delete Extension</h3>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeDeleteModal} aria-label="Close modal">
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				<p class="delete-warning-text">
					Are you sure you want to permanently delete <strong>{pluginToDelete.name}</strong>?
				</p>
				<div class="delete-details-card">
					<div class="detail-row">
						<span class="detail-label">File:</span>
						<code class="detail-value">{pluginToDelete.filename}</code>
					</div>
					<div class="detail-row">
						<span class="detail-label">Location:</span>
						<span class="detail-value">{pluginToDelete.target_dir}/</span>
					</div>
					<div class="detail-row">
						<span class="detail-label">Size:</span>
						<span class="detail-value">{formatFileSize(pluginToDelete.file_size_bytes)}</span>
					</div>
				</div>
				<p class="delete-note">This file will be permanently removed from disk. This action cannot be undone.</p>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeDeleteModal} disabled={isDeleting}>
					Cancel
				</button>
				<button
					class="btn btn-danger {isDeleting ? 'btn-loading' : ''}"
					onclick={confirmDelete}
					disabled={isDeleting}
				>
					{#if !isDeleting}
						<Trash2 size={16} />
					{/if}
					<span>Delete File</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.installed-plugins-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	/* Stats Row */
	.stats-row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-4);
	}

	.stat-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-4);
		display: flex;
		align-items: center;
		gap: var(--space-4);
		box-shadow: var(--card-shadow);
	}

	.stat-icon {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-btn);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.icon-blue {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border: 1px solid var(--accent-blue-border);
	}

	.icon-green {
		background-color: var(--accent-green-bg);
		color: var(--accent-green);
		border: 1px solid var(--accent-green-border);
	}

	.icon-danger {
		background-color: var(--danger-bg);
		color: var(--danger-text);
		border: 1px solid var(--danger-border);
	}

	.icon-purple {
		background-color: rgba(168, 85, 247, 0.12);
		color: var(--accent-purple-text);
		border: 1px solid rgba(168, 85, 247, 0.3);
	}

	.stat-content {
		display: flex;
		flex-direction: column;
	}

	.stat-value {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		line-height: 1.2;
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	/* Controls Header */
	.controls-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
	}

	.controls-body {
		padding: var(--space-4);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.search-input-wrapper {
		position: relative;
		flex: 1;
		min-width: 260px;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: var(--space-3);
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		padding-left: var(--space-8);
		padding-right: var(--space-8);
	}

	.clear-search-btn {
		position: absolute;
		right: 4px;
		color: var(--text-muted);
	}

	.filter-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.filter-select-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.filter-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		white-space: nowrap;
	}

	.select-sm {
		height: 32px;
		padding-top: 0;
		padding-bottom: 0;
		font-size: var(--font-size-xs);
		min-width: 120px;
	}

	.refresh-btn {
		height: 32px;
	}

	/* Data Table Styling */
	.table-container {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		overflow-x: auto;
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-16) var(--space-6);
		gap: var(--space-4);
		color: var(--text-muted);
	}

	:global(.spinner) {
		animation: spin 1s linear infinite;
		color: var(--accent-blue-text);
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.empty-icon {
		color: var(--text-muted);
		opacity: 0.5;
		margin-bottom: var(--space-3);
	}

	.table-empty h3 {
		font-size: var(--font-size-lg);
		color: var(--text-primary);
		margin-bottom: var(--space-2);
	}

	.table-empty p {
		font-size: var(--font-size-sm);
		max-width: 480px;
		margin: 0 auto;
	}

	.plugin-name-cell {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.file-icon-box {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.icon-active {
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		color: var(--accent-blue-text);
	}

	.icon-inactive {
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		color: var(--text-muted);
	}

	.name-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.plugin-title {
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		font-size: var(--font-size-base);
	}

	.plugin-filename {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 1px;
	}

	.version-tag {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		background-color: var(--bg-base);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
	}

	.dir-badge {
		font-family: var(--font-mono);
	}

	.file-size {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.text-right {
		text-align: right;
	}

	.action-buttons {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}

	.delete-action-btn:hover {
		color: var(--danger-text);
		background-color: var(--danger-bg);
	}

	.row-disabled {
		opacity: 0.7;
		background-color: rgba(0, 0, 0, 0.15);
	}

	/* Modal Extras */
	.modal-title-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.text-danger {
		color: var(--danger-text);
	}

	.delete-warning-text {
		font-size: var(--font-size-base);
		color: var(--text-primary);
		margin-bottom: var(--space-4);
	}

	.delete-details-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		padding: var(--space-3) var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		margin-bottom: var(--space-4);
	}

	.detail-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: var(--font-size-sm);
	}

	.detail-label {
		color: var(--text-muted);
	}

	.detail-value {
		font-family: var(--font-mono);
		color: var(--text-primary);
	}

	.delete-note {
		font-size: var(--font-size-xs);
		color: var(--danger-text);
	}
</style>
