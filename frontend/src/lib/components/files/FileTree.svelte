<script>
	import {
		Folder,
		FolderOpen,
		FileText,
		FileCode,
		Settings,
		Terminal,
		Search,
		Plus,
		Trash2,
		RefreshCw,
		ChevronRight,
		ChevronDown,
		X,
		FilePlus,
		AlertCircle,
		AlertTriangle
	} from 'lucide-svelte';

	/**
	 * @typedef {Object} FileTreeNode
	 * @property {string} name
	 * @property {string} path
	 * @property {boolean} is_dir
	 * @property {number} [size_bytes]
	 * @property {FileTreeNode[]} [children]
	 * @property {boolean} [truncated] listing incomplete (item cap, depth limit or I/O error); bubbles up
	 * @property {string|null} [error] set only for a real I/O failure; bubbles up
	 */

	let {
		tree = null,
		error = '',
		loading = false,
		selectedPath = '',
		onSelectFile = (path) => {},
		onRefreshTree = () => {},
		onCreateItem = (parentPath) => {},
		onDeleteItem = (path) => {}
	} = $props();

	// Search filter query
	let searchQuery = $state('');

	// Expanded directories path set ('' is the root container)
	let expandedPaths = $state(new Set(['', 'plugins', 'config']));

	// Keyboard focus state
	let focusedPath = $state('');

	function toggleExpand(path, event) {
		if (event) event.stopPropagation();
		const next = new Set(expandedPaths);
		if (next.has(path)) {
			next.delete(path);
		} else {
			next.add(path);
		}
		expandedPaths = next;
	}

	function expandPath(path) {
		if (!expandedPaths.has(path)) {
			const next = new Set(expandedPaths);
			next.add(path);
			expandedPaths = next;
		}
	}

	function collapsePath(path) {
		if (expandedPaths.has(path)) {
			const next = new Set(expandedPaths);
			next.delete(path);
			expandedPaths = next;
		}
	}

	function clearSearch() {
		searchQuery = '';
	}

	// Filter helper for tree search (expects lowercased query for efficiency)
	function nodeMatchesSearch(node, lowerQuery) {
		if (!lowerQuery) return true;
		if (node.name && node.name.toLowerCase().includes(lowerQuery)) return true;
		if (node.is_dir && node.children) {
			return node.children.some((child) => nodeMatchesSearch(child, lowerQuery));
		}
		return false;
	}

	// Get corresponding Lucide icon component
	function getFileIcon(node, isExpanded) {
		if (node.is_dir) {
			return isExpanded ? FolderOpen : Folder;
		}
		const name = (node.name || '').toLowerCase();
		if (
			name.endsWith('.properties') ||
			name.endsWith('.yml') ||
			name.endsWith('.yaml') ||
			name.endsWith('.json') ||
			name.endsWith('.toml') ||
			name.endsWith('.conf') ||
			name.endsWith('.ini') ||
			name.endsWith('.cfg')
		) {
			return Settings;
		}
		if (
			name.endsWith('.sh') ||
			name.endsWith('.cmd') ||
			name.endsWith('.bat') ||
			name.endsWith('.log')
		) {
			return Terminal;
		}
		if (
			name.endsWith('.js') ||
			name.endsWith('.ts') ||
			name.endsWith('.html') ||
			name.endsWith('.css') ||
			name.endsWith('.sk') ||
			name.endsWith('.java') ||
			name.endsWith('.py')
		) {
			return FileCode;
		}
		return FileText;
	}

	// Flatten visible tree nodes for accessible ARIA keyboard navigation
	function flattenTree(node, depth = 0, result = [], lowerQuery = '') {
		if (!node) return result;

		// Handle root container node. The backend names the root after the data directory
		// itself and gives it an empty path; its children are the server root's entries.
		if (node.is_dir && node.path === '' && node.children) {
			for (const child of node.children) {
				flattenTree(child, depth, result, lowerQuery);
			}
			return result;
		}

		if (lowerQuery && !nodeMatchesSearch(node, lowerQuery)) {
			return result;
		}

		result.push({ key: node.path, node, depth, note: '' });

		const isExpanded = expandedPaths.has(node.path) || Boolean(lowerQuery);

		if (node.is_dir && isExpanded) {
			const children = node.children || [];
			for (const child of children) {
				flattenTree(child, depth + 1, result, lowerQuery);
			}

			// A directory that really is empty and one whose listing was cut short (depth
			// limit, item cap, or I/O error) both arrive as `children: []`. Say which it is
			// instead of showing both as an empty folder.
			if (children.length === 0 && !lowerQuery) {
				result.push({
					key: `${node.path}::note`,
					node: null,
					depth: depth + 1,
					note: node.error ? 'error' : node.truncated ? 'truncated' : 'empty'
				});
			}
		}

		return result;
	}

	let visibleNodes = $derived(flattenTree(tree, 0, [], searchQuery.toLowerCase()));
	// Note rows are not selectable, so keyboard navigation skips them.
	let navNodes = $derived(visibleNodes.filter((item) => item.node));

	// Keyboard Navigation for ARIA tree
	function handleKeyDown(e) {
		const visibleNodes = navNodes;
		if (visibleNodes.length === 0) return;

		const currentIndex = visibleNodes.findIndex((item) => item.node.path === focusedPath);

		if (e.key === 'ArrowDown') {
			e.preventDefault();
			const nextIdx = currentIndex < 0 ? 0 : Math.min(currentIndex + 1, visibleNodes.length - 1);
			focusedPath = visibleNodes[nextIdx].node.path;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			const prevIdx = currentIndex < 0 ? 0 : Math.max(currentIndex - 1, 0);
			focusedPath = visibleNodes[prevIdx].node.path;
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			if (currentIndex >= 0) {
				const { node } = visibleNodes[currentIndex];
				if (node.is_dir) {
					if (!expandedPaths.has(node.path)) {
						expandPath(node.path);
					} else if (currentIndex + 1 < visibleNodes.length) {
						focusedPath = visibleNodes[currentIndex + 1].node.path;
					}
				}
			}
		} else if (e.key === 'ArrowLeft') {
			e.preventDefault();
			if (currentIndex >= 0) {
				const { node, depth } = visibleNodes[currentIndex];
				if (node.is_dir && expandedPaths.has(node.path)) {
					collapsePath(node.path);
				} else {
					for (let i = currentIndex - 1; i >= 0; i--) {
						if (visibleNodes[i].depth < depth && visibleNodes[i].node.is_dir) {
							focusedPath = visibleNodes[i].node.path;
							break;
						}
					}
				}
			}
		} else if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			if (currentIndex >= 0) {
				const { node } = visibleNodes[currentIndex];
				if (node.is_dir) {
					toggleExpand(node.path);
				} else {
					onSelectFile(node.path);
				}
			}
		}
	}

	function handleNodeClick(node) {
		focusedPath = node.path;
		if (node.is_dir) {
			toggleExpand(node.path);
		} else {
			onSelectFile(node.path);
		}
	}
</script>

<aside class="file-tree-sidebar">
	<!-- Top Search & Action Bar -->
	<div class="tree-toolbar">
		<div class="search-input-wrapper">
			<Search size={14} class="search-icon" />
			<input
				type="text"
				class="input tree-search-input"
				placeholder="Filter files..."
				bind:value={searchQuery}
				aria-label="Filter tree by filename"
			/>
			{#if searchQuery}
				<button
					type="button"
					class="btn btn-ghost btn-icon clear-search-btn"
					onclick={clearSearch}
					aria-label="Clear search filter"
				>
					<X size={12} />
				</button>
			{/if}
		</div>

		<div class="tree-header-actions">
			<button
				type="button"
				class="btn btn-secondary btn-icon btn-sm"
				onclick={() => onCreateItem('')}
				title="Create new file or folder in root"
				aria-label="Create file or folder in root"
			>
				<Plus size={16} />
			</button>
			<button
				type="button"
				class="btn btn-secondary btn-icon btn-sm"
				onclick={onRefreshTree}
				title="Refresh file tree"
				aria-label="Refresh file tree"
			>
				<RefreshCw size={15} />
			</button>
		</div>
	</div>

	<!-- Tree View Container -->
	<div
		class="file-tree-content"
		role="tree"
		aria-label="Server file directory tree"
		tabindex="0"
		onkeydown={handleKeyDown}
	>
		{#if error}
			<!-- The tree could not be fetched. Nothing is rendered in its place. -->
			<div class="tree-empty-state" role="alert">
				<AlertCircle size={28} class="tree-error-icon" />
				<p class="error-title">File tree unavailable</p>
				<p class="error-detail">{error}</p>
				<button type="button" class="btn btn-secondary btn-sm" onclick={onRefreshTree}>
					<RefreshCw size={13} />
					<span>Retry</span>
				</button>
			</div>
		{:else if loading && !tree}
			<div class="tree-empty-state">
				<p class="empty-text">Loading file tree…</p>
			</div>
		{:else if !tree}
			<div class="tree-empty-state">
				<p class="empty-text">File tree not loaded</p>
				<button type="button" class="btn btn-secondary btn-sm" onclick={onRefreshTree}>
					<RefreshCw size={13} />
					<span>Load tree</span>
				</button>
			</div>
		{:else}
			{#if tree.error}
				<div class="tree-banner banner-error" role="alert">
					<AlertCircle size={14} />
					<span>Listing failed part-way through: {tree.error}. What is shown below is incomplete.</span>
				</div>
			{:else if tree.truncated}
				<div class="tree-banner banner-warn">
					<AlertTriangle size={14} />
					<span>Listing incomplete — some entries are not shown (size/depth limit reached).</span>
				</div>
			{/if}

			{#if visibleNodes.length === 0}
				<div class="tree-empty-state">
					{#if searchQuery}
						<p class="empty-text">No files match "{searchQuery}"</p>
						<button type="button" class="btn btn-ghost btn-sm" onclick={clearSearch}>Clear Filter</button>
					{:else if tree.error || tree.truncated}
						<p class="empty-text">No entries could be listed</p>
					{:else}
						<p class="empty-text">Directory is empty</p>
					{/if}
				</div>
			{:else}
			<div class="tree-node-list" role="group">
				{#each visibleNodes as { node, depth, note, key } (key)}
					{#if note}
						<div
							class="tree-node-note note-{note}"
							style="padding-left: {depth * 16 + 28}px"
							role="treeitem"
							aria-selected="false"
							aria-disabled="true"
							tabindex="-1"
						>
							{#if note === 'error'}
								<AlertCircle size={12} />
								<span>Listing failed — contents unknown</span>
							{:else if note === 'truncated'}
								<AlertTriangle size={12} />
								<span>Not listed (depth or size limit) — contents unknown</span>
							{:else}
								<span>Empty folder</span>
							{/if}
						</div>
					{:else}
					{@const isExpanded = expandedPaths.has(node.path) || Boolean(searchQuery)}
					{@const isSelected = !node.is_dir && selectedPath === node.path}
					{@const isFocused = focusedPath === node.path}
					{@const IconComponent = getFileIcon(node, isExpanded)}

					<div
						class="tree-node-item {isSelected ? 'selected' : ''} {isFocused ? 'focused' : ''}"
						style="padding-left: {depth * 16 + 8}px"
						role="treeitem"
						aria-expanded={node.is_dir ? isExpanded : undefined}
						aria-selected={isSelected}
						aria-label={node.name}
						tabindex={isFocused ? 0 : -1}
						onclick={() => handleNodeClick(node)}
						onkeydown={(e) => {
							if (e.key === 'Enter' || e.key === ' ') {
								e.preventDefault();
								handleNodeClick(node);
							}
						}}
					>
						<!-- Toggle Arrow for directories -->
						{#if node.is_dir}
							<button
								type="button"
								class="chevron-btn"
								onclick={(e) => toggleExpand(node.path, e)}
								aria-label={isExpanded ? `Collapse ${node.name}` : `Expand ${node.name}`}
								tabindex="-1"
							>
								{#if isExpanded}
									<ChevronDown size={14} class="chevron-icon" />
								{:else}
									<ChevronRight size={14} class="chevron-icon" />
								{/if}
							</button>
						{:else}
							<span class="chevron-spacer"></span>
						{/if}

						<!-- File / Folder Icon -->
						<span class="node-icon-wrapper {node.is_dir ? 'folder-icon' : 'file-icon'}">
							<IconComponent size={15} />
						</span>

						<!-- Node Name -->
						<span class="node-name" title={node.path}>{node.name}</span>

						<!-- Incompleteness flags (both bubble up from descendants) -->
						{#if node.error}
							<span class="node-flag flag-error" title="Listing failed: {node.error}">
								<AlertCircle size={12} />
							</span>
						{:else if node.truncated}
							<span class="node-flag flag-warn" title="Listing incomplete — some entries under this folder are not shown.">
								<AlertTriangle size={12} />
							</span>
						{/if}

						<!-- Hover Quick Actions -->
						<div class="node-hover-actions">
							{#if node.is_dir}
								<button
									type="button"
									class="action-btn"
									title="Create item in folder"
									aria-label="Create item inside {node.name}"
									onclick={(e) => {
										e.stopPropagation();
										onCreateItem(node.path);
									}}
								>
									<FilePlus size={13} />
								</button>
							{/if}

							<button
								type="button"
								class="action-btn danger-action"
								title="Delete {node.is_dir ? 'folder' : 'file'}"
								aria-label="Delete {node.name}"
								onclick={(e) => {
									e.stopPropagation();
									onDeleteItem(node.path);
								}}
							>
								<Trash2 size={13} />
							</button>
						</div>
					</div>
					{/if}
				{/each}
			</div>
			{/if}
		{/if}
	</div>
</aside>

<style>
	.file-tree-sidebar {
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		background-color: var(--bg-surface);
		border-right: 1px solid var(--border);
		user-select: none;
		overflow: hidden;
	}

	.tree-toolbar {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3);
		border-bottom: 1px solid var(--border-subtle);
		background-color: var(--bg-base);
	}

	.search-input-wrapper {
		position: relative;
		flex: 1;
		display: flex;
		align-items: center;
	}

	:global(.search-input-wrapper .search-icon) {
		position: absolute;
		left: 10px;
		color: var(--text-muted);
		pointer-events: none;
	}

	.tree-search-input {
		height: 32px;
		padding-left: 30px;
		padding-right: 28px;
		font-size: var(--font-size-xs);
		background-color: var(--bg-surface);
		border-color: var(--border-subtle);
	}

	.tree-search-input:focus {
		border-color: var(--accent-blue);
		box-shadow: 0 0 0 2px var(--accent-blue-border);
	}

	:global(.clear-search-btn) {
		position: absolute;
		right: 4px;
		width: 22px !important;
		height: 22px !important;
		color: var(--text-muted);
	}

	.tree-header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}

	.file-tree-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-2) 0;
		outline: none;
	}

	.file-tree-content:focus-visible {
		box-shadow: inset 0 0 0 2px var(--accent-blue);
	}

	.tree-node-list {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.tree-node-item {
		display: flex;
		align-items: center;
		height: 30px;
		padding-right: var(--space-2);
		border-radius: var(--radius-sm);
		margin: 0 var(--space-2);
		cursor: pointer;
		color: var(--text-secondary);
		font-size: var(--font-size-sm);
		transition: background-color var(--transition-fast), color var(--transition-fast);
		position: relative;
	}

	.tree-node-item:hover {
		background-color: rgba(255, 255, 255, 0.04);
		color: var(--text-primary);
	}

	.tree-node-item.selected {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-medium);
		border-left: 2px solid var(--accent-blue);
	}

	.tree-node-item.focused {
		outline: 1px solid var(--accent-blue);
		outline-offset: -1px;
	}

	.chevron-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		border-radius: 3px;
		margin-right: 2px;
	}

	.chevron-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.1);
	}

	.chevron-spacer {
		width: 20px;
	}

	.node-icon-wrapper {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-right: 6px;
		flex-shrink: 0;
	}

	.folder-icon {
		color: var(--warning);
	}

	.file-icon {
		color: var(--text-muted);
	}

	.tree-node-item.selected .file-icon {
		color: var(--accent-blue-text);
	}

	.node-name {
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		font-family: var(--font-ui);
		line-height: 1;
	}

	.node-hover-actions {
		display: none;
		align-items: center;
		gap: 2px;
		margin-left: var(--space-1);
	}

	.tree-node-item:hover .node-hover-actions,
	.tree-node-item:focus-within .node-hover-actions {
		display: flex;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		border: none;
		background: transparent;
		color: var(--text-muted);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: color var(--transition-fast), background-color var(--transition-fast);
	}

	.action-btn:hover {
		background-color: rgba(255, 255, 255, 0.1);
		color: var(--text-primary);
	}

	.action-btn.danger-action:hover {
		background-color: var(--danger-bg);
		color: var(--danger-text);
	}

	.tree-empty-state {
		padding: var(--space-8) var(--space-4);
		text-align: center;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
	}

	.empty-text {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.error-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--danger-text);
	}

	.error-detail {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		word-break: break-word;
	}

	:global(.tree-error-icon) {
		color: var(--danger-text);
	}

	.tree-banner {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		margin: 0 var(--space-2) var(--space-2);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-sm);
		font-size: var(--font-size-xs);
		line-height: 1.4;
	}

	.banner-error {
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
	}

	.banner-warn {
		background-color: var(--warning-bg);
		border: 1px solid var(--warning-border);
		color: var(--warning);
	}

	.tree-node-note {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 26px;
		margin: 0 var(--space-2);
		font-size: var(--font-size-xs);
		font-style: italic;
		color: var(--text-muted);
	}

	.tree-node-note.note-error {
		color: var(--danger-text);
		font-style: normal;
	}

	.tree-node-note.note-truncated {
		color: var(--warning);
		font-style: normal;
	}

	.node-flag {
		display: flex;
		align-items: center;
		margin-left: 4px;
		flex-shrink: 0;
	}

	.node-flag.flag-error {
		color: var(--danger-text);
	}

	.node-flag.flag-warn {
		color: var(--warning);
	}
</style>
