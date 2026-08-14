<script>
	import {
		X,
		Sliders,
		Key,
		GitFork,
		Plus,
		Trash2,
		Search,
		Check,
		AlertCircle,
		MessageSquare,
		CheckCircle2,
		Sparkles,
		Tag,
		ToggleLeft,
		ToggleRight
	} from 'lucide-svelte';

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
	 * @property {LuckPermsPermissionNode[]} [permissions]
	 */

	let {
		isOpen = false,
		group = null,
		allGroups = [],
		onSave = (updatedGroup) => {},
		onClose = () => {}
	} = $props();

	// Active tab: 'metadata' | 'nodes' | 'parents'
	let activeTab = $state('metadata');

	// Local Form State
	let name = $state('');
	let weight = $state(0);
	let prefix = $state('');
	let suffix = $state('');
	let permissions = $state([]);
	let parents = $state([]);

	// Chat Preview State
	let previewUser = $state('PlayerName');
	let previewMessage = $state('Hello world! Testing LuckPerms chat formatting.');

	// Permission Nodes Tab State
	let nodeSearch = $state('');
	let newNodeId = $state('');
	let newNodeValue = $state(true);

	// Parents Tab State
	let selectedParentToAdd = $state('');

	// Error & Validation
	let errorMessage = $state('');

	// Watch for modal opening or group changes to reset internal form state
	$effect(() => {
		if (isOpen) {
			errorMessage = '';
			activeTab = 'metadata';
			nodeSearch = '';
			newNodeId = '';
			newNodeValue = true;
			selectedParentToAdd = '';

			if (group) {
				name = group.name || '';
				weight = group.weight ?? 0;
				prefix = group.prefix || '';
				suffix = group.suffix || '';
				permissions = Array.isArray(group.permissions)
					? group.permissions.map((n) => ({ ...n }))
					: [];
				parents = Array.isArray(group.parents) ? [...group.parents] : [];
			} else {
				// Default state for new group creation
				name = '';
				weight = 0;
				prefix = '';
				suffix = '';
				permissions = [];
				parents = [];
			}
		}
	});

	// Derived filtered candidate parents for dropdown (exclude self & already added)
	let availableParents = $derived.by(() => {
		const parentSet = new Set(parents);
		const result = [];
		for (const g of allGroups) {
			if (g.name !== name && !parentSet.has(g.name)) {
				result.push(g.name);
			}
		}
		return result;
	});

	// Derived filtered permission nodes
	let filteredNodes = $derived.by(() => {
		const q = nodeSearch.trim().toLowerCase();
		if (!q) return permissions;
		return permissions.filter((n) => n.permission.toLowerCase().includes(q));
	});

	/* Minecraft Color Code Parser for Chat Preview */
	const MC_COLORS = {
		'0': '#000000',
		'1': '#0000AA',
		'2': '#00AA00',
		'3': '#00AAAA',
		'4': '#AA0000',
		'5': '#AA00AA',
		'6': '#FFAA00',
		'7': '#AAAAAA',
		'8': '#555555',
		'9': '#5555FF',
		a: '#55FF55',
		b: '#55FFFF',
		c: '#FF5555',
		d: '#FF55FF',
		e: '#FFFF55',
		f: '#FFFFFF'
	};

	const DARK_MC_COLORS = new Set(['#000000', '#0000AA', '#AA0000', '#555555']);
	const COLOR_CODE_REGEX = /(?:&|§)([0-9a-fA-Fk-oK-ORr])/g;

	let prefixSegments = $derived(renderFormattedMinecraftText(prefix));
	let suffixSegments = $derived(renderFormattedMinecraftText(suffix));

	function renderFormattedMinecraftText(text) {
		if (!text) return [];
		COLOR_CODE_REGEX.lastIndex = 0;
		const segments = [];
		let currentColor = '#E8E8F0';
		let isBold = false;
		let isItalic = false;

		let lastIndex = 0;
		let match;

		while ((match = COLOR_CODE_REGEX.exec(text)) !== null) {
			const index = match.index;
			if (index > lastIndex) {
				segments.push({
					text: text.substring(lastIndex, index),
					color: currentColor,
					bold: isBold,
					italic: isItalic
				});
			}

			const code = match[1].toLowerCase();
			if (MC_COLORS[code]) {
				currentColor = MC_COLORS[code];
				isBold = false;
				isItalic = false;
			} else if (code === 'l') {
				isBold = true;
			} else if (code === 'o') {
				isItalic = true;
			} else if (code === 'r') {
				currentColor = '#E8E8F0';
				isBold = false;
				isItalic = false;
			}
			lastIndex = COLOR_CODE_REGEX.lastIndex;
		}

		if (lastIndex < text.length) {
			segments.push({
				text: text.substring(lastIndex),
				color: currentColor,
				bold: isBold,
				italic: isItalic
			});
		}

		return segments;
	}

	/* Permission Node Handlers */
	function handleAddNode() {
		errorMessage = '';
		const trimmedNode = newNodeId.trim();

		if (!trimmedNode) {
			errorMessage = 'Permission node cannot be empty.';
			return;
		}

		if (trimmedNode.includes(' ') || trimmedNode.includes('/')) {
			errorMessage = 'Invalid permission node format. Spaces and slashes are not allowed.';
			return;
		}

		const exists = permissions.some((n) => n.permission === trimmedNode);
		if (exists) {
			errorMessage = `Permission node "${trimmedNode}" is already assigned to this group.`;
			return;
		}

		permissions.push({
			permission: trimmedNode,
			value: newNodeValue
		});

		newNodeId = '';
		newNodeValue = true;
	}

	function handleToggleNodeValue(permString) {
		permissions = permissions.map((n) => {
			if (n.permission === permString) {
				return { ...n, value: !n.value };
			}
			return n;
		});
	}

	function handleRemoveNode(permString) {
		permissions = permissions.filter((n) => n.permission !== permString);
	}

	/* Parent Inheritance Handlers */
	function handleAddParent() {
		errorMessage = '';
		if (!selectedParentToAdd) {
			errorMessage = 'Please select a parent group to add.';
			return;
		}

		if (!parents.includes(selectedParentToAdd)) {
			parents.push(selectedParentToAdd);
		}
		selectedParentToAdd = '';
	}

	function handleRemoveParent(parentName) {
		parents = parents.filter((p) => p !== parentName);
	}

	/* Submit Handler */
	function handleSubmit(e) {
		if (e) e.preventDefault();
		errorMessage = '';

		const cleanName = name.trim();
		if (!cleanName) {
			errorMessage = 'Group name is required.';
			activeTab = 'metadata';
			return;
		}

		if (!/^[a-zA-Z0-9_.-]+$/.test(cleanName)) {
			errorMessage = 'Group name can only contain letters, numbers, underscores, hyphens, and dots.';
			activeTab = 'metadata';
			return;
		}

		const updatedGroup = {
			name: cleanName,
			weight: Number(weight) || 0,
			prefix: prefix.trim(),
			suffix: suffix.trim(),
			permissions: [...permissions],
			parents: [...parents]
		};

		onSave(updatedGroup);
	}
</script>

{#if isOpen}
	<div
		class="modal-backdrop"
		onclick={(e) => e.target === e.currentTarget && onClose()}
		onkeydown={(e) => e.key === 'Escape' && onClose()}
		role="dialog"
		aria-modal="true"
		aria-labelledby="group-editor-modal-title"
		tabindex="-1"
	>
		<div class="modal group-editor-modal">
			<!-- Modal Header -->
			<div class="modal-header">
				<div class="modal-title-group">
					<div class="modal-title-icon">
						<Sliders size={20} />
					</div>
					<div>
						<h3 class="modal-title" id="group-editor-modal-title">
							{#if group}
								Edit Group: <span class="group-name-highlight">{group.name}</span>
							{:else}
								Create New LuckPerms Group
							{/if}
						</h3>
						<p class="modal-subtitle">Configure group weight, chat prefix/suffix, permissions & inheritance</p>
					</div>
				</div>
				<button class="btn btn-ghost btn-icon" onclick={onClose} aria-label="Close Modal">
					<X size={18} />
				</button>
			</div>

			<!-- Tabs Navigation Header -->
			<div class="modal-tabs" role="tablist">
				<button
					class="tab-btn {activeTab === 'metadata' ? 'active' : ''}"
					onclick={() => (activeTab = 'metadata')}
					role="tab"
					aria-selected={activeTab === 'metadata'}
					aria-controls="tabpanel-metadata"
				>
					<Sliders size={16} />
					<span>Metadata & Chat Preview</span>
				</button>
				<button
					class="tab-btn {activeTab === 'nodes' ? 'active' : ''}"
					onclick={() => (activeTab = 'nodes')}
					role="tab"
					aria-selected={activeTab === 'nodes'}
					aria-controls="tabpanel-nodes"
				>
					<Key size={16} />
					<span>Permission Nodes ({permissions.length})</span>
				</button>
				<button
					class="tab-btn {activeTab === 'parents' ? 'active' : ''}"
					onclick={() => (activeTab = 'parents')}
					role="tab"
					aria-selected={activeTab === 'parents'}
					aria-controls="tabpanel-parents"
				>
					<GitFork size={16} />
					<span>Inheritance ({parents.length})</span>
				</button>
			</div>

			<!-- Alert Error Banner -->
			{#if errorMessage}
				<div class="alert alert-danger modal-alert" role="alert">
					<AlertCircle size={16} />
					<span>{errorMessage}</span>
				</div>
			{/if}

			<!-- Modal Body -->
			<div class="modal-body">
				{#if activeTab === 'metadata'}
					<!-- TAB 1: Metadata & Chat Preview -->
					<div class="tab-content metadata-tab" role="tabpanel" id="tabpanel-metadata">
						<form id="group-meta-form" onsubmit={handleSubmit}>
							<div class="form-grid">
								<!-- Group Name -->
								<div class="form-group">
									<label for="group-name-input" class="label label-required">Group Identifier</label>
									<input
										id="group-name-input"
										type="text"
										class="input"
										placeholder="e.g. admin, vip, moderator"
										bind:value={name}
										disabled={!!group}
										required
									/>
									{#if group}
										<span class="field-hint">Group identifier cannot be renamed after creation.</span>
									{/if}
								</div>

								<!-- Weight Input -->
								<div class="form-group">
									<label for="group-weight-input" class="label">Group Weight / Priority</label>
									<input
										id="group-weight-input"
										type="number"
										class="input"
										placeholder="0"
										bind:value={weight}
									/>
									<span class="field-hint">Higher weight takes priority in prefix display & permission overrides.</span>
								</div>

								<!-- Prefix Input -->
								<div class="form-group">
									<label for="group-prefix-input" class="label">Prefix</label>
									<input
										id="group-prefix-input"
										type="text"
										class="input font-mono"
										placeholder="e.g. &c[Admin] "
										bind:value={prefix}
									/>
									<span class="field-hint">Supports Minecraft color codes e.g. &c (Red), &a (Green), &b (Aqua), &e (Yellow).</span>
								</div>

								<!-- Suffix Input -->
								<div class="form-group">
									<label for="group-suffix-input" class="label">Suffix</label>
									<input
										id="group-suffix-input"
										type="text"
										class="input font-mono"
										placeholder="e.g.  &e[Staff]"
										bind:value={suffix}
									/>
									<span class="field-hint">Appended after player username in chat formats.</span>
								</div>
							</div>
						</form>

						<!-- Live In-Game Chat Preview Box -->
						<div class="chat-preview-container card">
							<div class="card-header preview-header">
								<div class="preview-header-left">
									<MessageSquare size={16} class="icon-blue" />
									<span class="card-title text-sm">Live In-Game Chat Preview</span>
								</div>
								<span class="badge badge-success text-xs">Minecraft Chat HUD</span>
							</div>

							<div class="card-body preview-body">
								<!-- Interactive Controls for Preview -->
								<div class="preview-controls-row">
									<div class="preview-input-field">
										<label for="preview-user-input" class="label-xs">Test Username:</label>
										<input
											id="preview-user-input"
											type="text"
											class="input input-sm preview-name-input"
											bind:value={previewUser}
											placeholder="PlayerName"
										/>
									</div>
									<div class="preview-input-field flex-1">
										<label for="preview-message-input" class="label-xs">Test Message:</label>
										<input
											id="preview-message-input"
											type="text"
											class="input input-sm preview-msg-input"
											bind:value={previewMessage}
											placeholder="Hello world!"
										/>
									</div>
								</div>

								<!-- Formatted Chat Window -->
								<div class="minecraft-chat-hud">
									<div class="chat-line">
										<span class="chat-bracket">&lt;</span>

										<!-- Formatted Prefix -->
										{#if prefix}
											<span class="formatted-prefix">
												{#each prefixSegments as seg}
													<span
														style="color: {seg.color}; font-weight: {seg.bold
															? 'bold'
															: 'normal'}; font-style: {seg.italic ? 'italic' : 'normal'};{DARK_MC_COLORS.has(seg.color) ? ' text-shadow: 0 0 2px rgba(255,255,255,0.4);' : ''}"
													>{seg.text}</span>
												{/each}
											</span>
										{/if}

										<!-- Player Name -->
										<span class="chat-player-name">{previewUser || 'PlayerName'}</span>

										<!-- Formatted Suffix -->
										{#if suffix}
											<span class="formatted-suffix">
												{#each suffixSegments as seg}
													<span
														style="color: {seg.color}; font-weight: {seg.bold
															? 'bold'
															: 'normal'}; font-style: {seg.italic ? 'italic' : 'normal'};{DARK_MC_COLORS.has(seg.color) ? ' text-shadow: 0 0 2px rgba(255,255,255,0.4);' : ''}"
													>{seg.text}</span>
												{/each}
											</span>
										{/if}

										<span class="chat-bracket">&gt;</span>
										<span class="chat-space"> </span>
										<span class="chat-message-text">{previewMessage || 'Hello world!'}</span>
									</div>
								</div>
							</div>
						</div>
					</div>

				{:else if activeTab === 'nodes'}
					<!-- TAB 2: Permission Nodes -->
					<div class="tab-content nodes-tab" role="tabpanel" id="tabpanel-nodes">
						<!-- Add Node Form Box -->
						<div class="card add-node-card">
							<div class="card-header add-node-header">
								<h4 class="card-title text-sm">Add Permission Node</h4>
							</div>
							<div class="card-body add-node-body">
								<div class="add-node-inputs">
									<div class="node-id-field">
										<input
											type="text"
											class="input font-mono"
											placeholder="e.g. minecraft.command.tp or essentials.fly"
											bind:value={newNodeId}
											onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAddNode())}
											aria-label="Add permission node"
										/>
									</div>

									<div class="node-value-toggle-group">
										<button
											type="button"
											class="btn btn-sm {newNodeValue ? 'btn-success-active' : 'btn-danger-active'}"
											onclick={() => (newNodeValue = !newNodeValue)}
											title="Click to toggle node state"
										>
											{#if newNodeValue}
												<Check size={14} />
												<span>TRUE</span>
											{:else}
												<X size={14} />
												<span>FALSE</span>
											{/if}
										</button>

										<button
											type="button"
											class="btn btn-primary btn-sm"
											onclick={handleAddNode}
										>
											<Plus size={14} />
											<span>Add Node</span>
										</button>
									</div>
								</div>
							</div>
						</div>

						<!-- Search Filter & Nodes Table -->
						<div class="assigned-nodes-section">
							<div class="search-and-count-bar">
								<div class="search-box">
									<Search size={14} class="search-icon" />
									<input
										type="text"
										class="input search-input"
										placeholder="Filter assigned permission nodes..."
										bind:value={nodeSearch}
										aria-label="Filter permission nodes"
									/>
								</div>
								<span class="badge badge-secondary">
									{filteredNodes.length} of {permissions.length} nodes
								</span>
							</div>

							{#if permissions.length === 0}
								<div class="empty-state-sub">
									<Key size={32} class="empty-icon" />
									<p>No permission nodes assigned to this group yet.</p>
								</div>
							{:else if filteredNodes.length === 0}
								<div class="empty-state-sub">
									<p>No permission nodes match filter "{nodeSearch}".</p>
								</div>
							{:else}
								<div class="table-container nodes-table-wrapper">
									<table class="table nodes-table">
										<thead>
											<tr>
												<th>Permission Node</th>
												<th>State Value</th>
												<th class="text-right">Actions</th>
											</tr>
										</thead>
										<tbody>
											{#each filteredNodes as node (node.permission)}
												<tr>
													<td class="node-permission-cell">
														<code class="node-code">{node.permission}</code>
													</td>
													<td class="node-value-cell">
														<button
															class="btn btn-sm state-toggle-btn {node.value ? 'badge-success' : 'badge-danger'}"
															onclick={() => handleToggleNodeValue(node.permission)}
															title="Click to toggle value between TRUE and FALSE"
														>
															{node.value ? 'TRUE' : 'FALSE'}
														</button>
													</td>
													<td class="text-right">
														<button
															class="btn btn-ghost btn-sm btn-icon danger-icon-btn"
															onclick={() => handleRemoveNode(node.permission)}
															title="Remove node"
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
					</div>

				{:else if activeTab === 'parents'}
					<!-- TAB 3: Inheritance & Parents -->
					<div class="tab-content parents-tab" role="tabpanel" id="tabpanel-parents">
						<!-- Add Parent Group Card -->
						<div class="card add-parent-card">
							<div class="card-header add-parent-header">
								<h4 class="card-title text-sm">Add Parent Inheritance Group</h4>
							</div>
							<div class="card-body add-parent-body">
								<div class="add-parent-row">
									<select class="select flex-1" bind:value={selectedParentToAdd} aria-label="Select parent group to inherit from">
										<option value="" disabled selected>Select parent group to inherit from...</option>
										{#each availableParents as pName}
											<option value={pName}>{pName}</option>
										{/each}
									</select>

									<button
										type="button"
										class="btn btn-primary"
										onclick={handleAddParent}
										disabled={!selectedParentToAdd}
									>
										<Plus size={16} />
										<span>Add Parent</span>
									</button>
								</div>
								{#if availableParents.length === 0}
									<p class="field-hint mt-2">No additional candidate parent groups available to add.</p>
								{/if}
							</div>
						</div>

						<!-- Current Parents List -->
						<div class="current-parents-section">
							<h4 class="section-title">Inherited Groups ({parents.length})</h4>

							{#if parents.length === 0}
								<div class="empty-state-sub">
									<GitFork size={32} class="empty-icon" />
									<p>This group does not inherit permissions from any parent groups.</p>
								</div>
							{:else}
								<div class="parents-cards-list">
									{#each parents as parentName (parentName)}
										<div class="parent-item-card">
											<div class="parent-item-info">
												<GitFork size={16} class="icon-blue" />
												<span class="parent-name">{parentName}</span>
												<span class="badge badge-secondary">Parent Group</span>
											</div>
											<button
												type="button"
												class="btn btn-ghost btn-sm remove-parent-btn"
												onclick={() => handleRemoveParent(parentName)}
												title="Remove parent {parentName}"
											>
												<Trash2 size={14} />
												<span>Remove Parent</span>
											</button>
										</div>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			<!-- Modal Footer -->
			<div class="modal-footer">
				<button type="button" class="btn btn-secondary" onclick={onClose}>
					Cancel
				</button>
				<button type="button" class="btn btn-primary" onclick={handleSubmit}>
					<CheckCircle2 size={16} />
					<span>{group ? 'Save Changes' : 'Create Group'}</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	/* Modal Layout */
	.group-editor-modal {
		max-width: 680px;
		width: 90vw;
		display: flex;
		flex-direction: column;
		max-height: 85vh;
	}

	.modal-header {
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
	}

	.modal-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.modal-title-icon {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
	}

	.group-name-highlight {
		color: var(--accent-blue-text);
		font-family: var(--font-mono);
	}

	.modal-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	/* Modal Tabs Header */
	.modal-tabs {
		display: flex;
		align-items: center;
		background-color: var(--bg-base);
		border-bottom: 1px solid var(--border);
		padding: 0 var(--space-6);
		gap: var(--space-2);
		overflow-x: auto;
	}

	.tab-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-4);
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--text-muted);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		white-space: nowrap;
		transition: color var(--transition-fast), border-color var(--transition-fast);
	}

	.tab-btn:hover {
		color: var(--text-primary);
	}

	.tab-btn.active {
		color: var(--accent-blue-text);
		border-bottom-color: var(--accent-blue);
	}

	.modal-alert {
		margin: var(--space-4) var(--space-6) 0 var(--space-6);
	}

	.modal-body {
		padding: var(--space-6);
		overflow-y: auto;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	/* Form Grid */
	.form-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-4);
	}

	@media (max-width: 600px) {
		.form-grid {
			grid-template-columns: 1fr;
		}
	}

	.field-hint {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.font-mono {
		font-family: var(--font-mono);
	}

	/* Chat Preview Card */
	.chat-preview-container {
		background-color: var(--bg-base);
		border: 1px solid var(--border-focus);
		margin-top: var(--space-4);
	}

	.preview-header {
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-surface);
	}

	.preview-header-left {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.text-sm {
		font-size: var(--font-size-sm);
	}

	.text-xs {
		font-size: var(--font-size-xs);
	}

	.preview-body {
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.preview-controls-row {
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.preview-input-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.flex-1 {
		flex: 1;
	}

	.label-xs {
		font-size: 0.75rem;
		color: var(--text-muted);
	}

	.input-sm {
		height: 32px;
		font-size: var(--font-size-xs);
	}

	.preview-name-input {
		width: 140px;
	}

	/* Minecraft Chat HUD */
	.minecraft-chat-hud {
		background-color: #0B0B10;
		border: 1px solid #1A1A28;
		border-radius: var(--radius-input);
		padding: var(--space-4);
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		color: #E8E8F0;
		box-shadow: inset 0 2px 8px rgba(0, 0, 0, 0.7);
		min-height: 48px;
		display: flex;
		align-items: center;
	}

	.chat-line {
		word-break: break-all;
		line-height: 1.5;
	}

	.chat-bracket {
		color: #AAAAAA;
	}

	.chat-player-name {
		color: #FFFFFF;
		font-weight: var(--font-weight-semibold);
	}

	.chat-space {
		white-space: pre;
	}

	.chat-message-text {
		color: #FFFFFF;
	}

	/* Permission Nodes Tab */
	.nodes-tab {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.add-node-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
	}

	.add-node-header {
		padding: var(--space-3) var(--space-4);
	}

	.add-node-body {
		padding: var(--space-4);
	}

	.add-node-inputs {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}

	.node-id-field {
		flex: 1;
	}

	.node-value-toggle-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.btn-success-active {
		background-color: var(--accent-green-bg);
		color: var(--accent-green);
		border: 1px solid var(--accent-green-border);
	}

	.btn-danger-active {
		background-color: var(--danger-bg);
		color: var(--danger-text);
		border: 1px solid var(--danger-border);
	}

	.search-and-count-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		margin-bottom: var(--space-3);
	}

	.nodes-table-wrapper {
		max-height: 240px;
		overflow-y: auto;
	}

	.node-permission-cell {
		font-family: var(--font-mono);
	}

	.node-code {
		color: var(--accent-blue-text);
		background-color: var(--accent-blue-bg);
		padding: 2px 6px;
		border-radius: var(--radius-sm);
		font-size: var(--font-size-xs);
	}

	.state-toggle-btn {
		font-size: 0.7rem;
		font-weight: var(--font-weight-bold);
		cursor: pointer;
	}

	/* Parents Tab */
	.parents-tab {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.add-parent-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
	}

	.add-parent-header {
		padding: var(--space-3) var(--space-4);
	}

	.add-parent-body {
		padding: var(--space-4);
	}

	.add-parent-row {
		display: flex;
		gap: var(--space-3);
		align-items: center;
	}

	.section-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-secondary);
		margin-bottom: var(--space-3);
	}

	.parents-cards-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.parent-item-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
	}

	.parent-item-info {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.parent-name {
		font-weight: var(--font-weight-semibold);
		font-family: var(--font-mono);
	}

	.remove-parent-btn {
		color: var(--text-muted);
	}

	.remove-parent-btn:hover {
		color: var(--danger-text);
		background-color: var(--danger-bg);
	}

	.empty-state-sub {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-8);
		gap: var(--space-2);
		color: var(--text-muted);
		text-align: center;
		font-size: var(--font-size-xs);
		background-color: var(--bg-base);
		border: 1px dashed var(--border);
		border-radius: var(--radius-card);
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

	.icon-blue {
		color: var(--accent-blue-text);
	}

	.mt-2 {
		margin-top: 8px;
	}
</style>
