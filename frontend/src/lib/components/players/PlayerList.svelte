<script>
	import {
		Users,
		Search,
		LayoutGrid,
		Table as TableIcon,
		Heart,
		Utensils,
		Globe,
		UserX,
		ShieldAlert,
		Eye,
		RefreshCw,
		Crown
	} from 'lucide-svelte';
	import { UNAVAILABLE } from '$lib/components/dashboard/serverState.js';

	/**
	 * @typedef {Object} PlayerSummary
	 * @property {string} uuid
	 * @property {string} username
	 * @property {boolean} is_online
	 * @property {number} [ping]
	 * @property {number} [health]
	 * @property {number} [food]
	 * @property {string} [dimension]
	 * @property {number} [last_seen]
	 * @property {boolean} [is_op]
	 * @property {boolean} [is_banned]
	 */

	let {
		players = [],
		loading = false,
		onViewProfile = (player) => {},
		onKickPlayer = (player) => {},
		onBanPlayer = (player) => {},
		onRefresh = () => {}
	} = $props();

	// Local state
	let viewMode = $state('grid'); // 'grid' | 'table'
	let searchQuery = $state('');
	let statusFilter = $state('all'); // 'all' | 'online' | 'banned' | 'op'

	// Computed filtered players
	let filteredPlayers = $derived.by(() => {
		return players.filter((player) => {
			// Search filter
			if (searchQuery.trim()) {
				const q = searchQuery.toLowerCase().trim();
				const nameMatch = (player.username || '').toLowerCase().includes(q);
				const uuidMatch = (player.uuid || '').toLowerCase().includes(q);
				if (!nameMatch && !uuidMatch) return false;
			}

			// Status dropdown filter
			if (statusFilter === 'online' && !player.is_online) return false;
			if (statusFilter === 'banned' && !player.is_banned) return false;
			if (statusFilter === 'op' && !player.is_op) return false;

			return true;
		});
	});

	// Helper for UUID formatting
	function formatShortUuid(uuid) {
		if (!uuid) return 'Unknown UUID';
		if (uuid.length <= 12) return uuid;
		return `${uuid.substring(0, 8)}...${uuid.substring(uuid.length - 4)}`;
	}

	/**
	 * Dimension badge. An unreported dimension is unknown, NOT the Overworld —
	 * the old `(dimension || '')` fallthrough branded every null as a green
	 * "Overworld" badge (B1). An unrecognised dimension string is shown verbatim.
	 */
	function getDimensionBadge(dimension) {
		if (!dimension) return { label: UNAVAILABLE, badgeClass: 'badge-secondary' };
		const d = dimension.toLowerCase();
		if (d.includes('nether')) {
			return { label: 'Nether', badgeClass: 'badge-danger' };
		}
		if (d.includes('end')) {
			return { label: 'The End', badgeClass: 'badge-purple' };
		}
		if (d.includes('overworld')) {
			return { label: 'Overworld', badgeClass: 'badge-success' };
		}
		// Modded dimension: show it rather than guessing, but drop the namespace so a
		// long id cannot stretch this nowrap badge past its card.
		return { label: dimension.split(':').pop(), badgeClass: 'badge-secondary' };
	}

	/**
	 * Health colour. No numeric default: an unsampled player must read as muted
	 * "unknown", never as a full green bar (B1).
	 * @param {number | null | undefined} health
	 */
	function getHealthColor(health) {
		if (health == null) return 'var(--text-muted)';
		if (health > 12) return 'var(--accent-green)';
		if (health > 5) return 'var(--warning)';
		return 'var(--danger)';
	}

	/** Bar width for a 0..max vital, or null when there is nothing to draw. */
	function vitalPercent(value, max = 20) {
		if (value == null || !max) return null;
		return Math.min(100, Math.max(0, (value / max) * 100));
	}

	// Helper for Last Seen timestamp. A missing timestamp is unknown, not "Never".
	function formatLastSeen(timestamp) {
		if (timestamp == null) return UNAVAILABLE;
		if (!timestamp) return 'Never';
		const date = new Date(timestamp * 1000);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / 60000);

		if (diffMins < 1) return 'Just now';
		if (diffMins < 60) return `${diffMins}m ago`;

		const diffHours = Math.floor(diffMins / 60);
		if (diffHours < 24) return `${diffHours}h ago`;

		const diffDays = Math.floor(diffHours / 24);
		if (diffDays < 30) return `${diffDays}d ago`;

		return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}
</script>

<div class="player-list-container">
	<!-- Control Bar: Search, Status Filter & View Toggle -->
	<div class="controls-card card">
		<div class="controls-body">
			<!-- Search Bar -->
			<div class="search-box">
				<Search size={16} class="search-icon" />
				<input
					type="text"
					class="input search-input"
					placeholder="Search players by username or UUID..."
					bind:value={searchQuery}
					aria-label="Search players"
				/>
			</div>

			<div class="controls-right">
				<!-- Status Filter Selector Dropdown -->
				<div class="filter-wrapper">
					<span class="filter-label">Filter:</span>
					<select
						class="select select-sm status-select"
						bind:value={statusFilter}
						aria-label="Filter players by status"
					>
						<option value="all">All Players</option>
						<option value="online">Online Only</option>
						<option value="banned">Banned Only</option>
						<option value="op">Operators Only</option>
					</select>
				</div>

				<!-- View Mode Selector Buttons -->
				<div class="view-toggle-group">
					<button
						class="btn btn-sm view-btn {viewMode === 'grid' ? 'active' : 'btn-ghost'}"
						onclick={() => (viewMode = 'grid')}
						title="Grid View"
						aria-label="Grid view"
					>
						<LayoutGrid size={16} />
					</button>
					<button
						class="btn btn-sm view-btn {viewMode === 'table' ? 'active' : 'btn-ghost'}"
						onclick={() => (viewMode = 'table')}
						title="Table View"
						aria-label="Table view"
					>
						<TableIcon size={16} />
					</button>
				</div>

				<!-- Refresh Button -->
				<button
					class="btn btn-secondary btn-sm {loading ? 'btn-loading' : ''}"
					onclick={onRefresh}
					disabled={loading}
					title="Refresh Player List"
				>
					{#if !loading}
						<RefreshCw size={14} />
					{/if}
					<span class="refresh-label">Refresh</span>
				</button>
			</div>
		</div>
	</div>

	<!-- Main Body Display (Grid or Table) -->
	{#if loading && players.length === 0}
		<div class="loading-state card">
			<RefreshCw size={32} class="spinner" />
			<p>Loading player data from Minecraft server...</p>
		</div>
	{:else if filteredPlayers.length === 0}
		<div class="empty-state card">
			<Users size={48} class="empty-icon" />
			<h3 class="empty-title">No Players Found</h3>
			<p class="empty-subtitle">
				{#if searchQuery.trim() || statusFilter !== 'all'}
					No players match your active search query or status filter.
				{:else}
					No player data files have been recorded on this server yet.
				{/if}
			</p>
		</div>
	{:else if viewMode === 'grid'}
		<!-- Grid View -->
		<div class="player-grid">
			{#each filteredPlayers as player (player.uuid || player.username)}
				{@const dim = getDimensionBadge(player.dimension)}
				{@const healthPct = vitalPercent(player.health)}
				{@const foodPct = vitalPercent(player.food)}
				<div class="card player-card {player.is_online ? 'online-card' : ''}">
					<!-- Card Top Header -->
					<div class="player-card-header">
						<div class="status-badges-row">
							{#if player.is_online}
								<span class="badge badge-success">
									<span class="status-dot status-dot-success status-dot-pulse"></span>
									ONLINE
								</span>
							{:else}
								<span class="badge badge-secondary">
									<span class="status-dot"></span>
									OFFLINE
								</span>
							{/if}

							{#if player.is_banned}
								<span class="badge badge-danger">BANNED</span>
							{/if}

							{#if player.is_op}
								<span class="badge badge-op">
									<Crown size={12} />
									OP
								</span>
							{/if}
						</div>

						<!-- Dimension Badge -->
						<span class="badge {dim.badgeClass}">
							<Globe size={12} />
							{dim.label}
						</span>
					</div>

					<!-- Card Core Body -->
					<div class="player-card-body">
						<div class="avatar-container">
							<img
								src="https://mc-heads.net/avatar/{encodeURIComponent(player.uuid || player.username)}/64"
								alt="{player.username}'s 3D head avatar"
								class="player-head-avatar"
								width="64"
								height="64"
								loading="lazy"
							/>
						</div>

						<div class="player-identity">
							<h3 class="player-username" title={player.username}>{player.username}</h3>
							<span class="player-uuid" title={player.uuid}>{formatShortUuid(player.uuid)}</span>
						</div>

						<!-- Vital Gauges (Health & Food) -->
						<div class="vitals-container">
							<div class="vital-row">
								<div class="vital-meta">
									<Heart size={14} style="color: {getHealthColor(player.health)};" />
									<span class="vital-label">Health</span>
								</div>
								{#if healthPct == null}
									<div class="gauge-bar gauge-unknown" title="Health not reported for this player"></div>
								{:else}
									<div class="gauge-bar">
										<div
											class="gauge-fill"
											style="width: {healthPct}%; background-color: {getHealthColor(player.health)};"
										></div>
									</div>
								{/if}
								<span class="vital-value"
									>{player.health == null ? UNAVAILABLE : `${player.health.toFixed(0)}/20`}</span
								>
							</div>

							<div class="vital-row">
								<div class="vital-meta">
									<Utensils
										size={14}
										style="color: {player.food == null ? 'var(--text-muted)' : 'var(--warning)'};"
									/>
									<span class="vital-label">Food</span>
								</div>
								{#if foodPct == null}
									<div class="gauge-bar gauge-unknown" title="Food level not reported for this player"></div>
								{:else}
									<div class="gauge-bar">
										<div class="gauge-fill gauge-fill-warning" style="width: {foodPct}%;"></div>
									</div>
								{/if}
								<span class="vital-value"
									>{player.food == null ? UNAVAILABLE : `${player.food}/20`}</span
								>
							</div>
						</div>

						{#if !player.is_online}
							<div class="last-seen-row">
								<span class="last-seen-label">Last Seen:</span>
								<span class="last-seen-value">{formatLastSeen(player.last_seen)}</span>
							</div>
						{/if}
					</div>

					<!-- Card Action Menu -->
					<div class="player-card-footer">
						<button
							class="btn btn-secondary btn-sm view-profile-btn"
							onclick={() => onViewProfile(player)}
							title="View full player profile and moderation panel"
							aria-label="View profile for {player.username}"
						>
							<Eye size={14} />
							<span>Profile</span>
						</button>

						<div class="quick-actions">
							{#if player.is_online}
								<button
									class="btn btn-secondary btn-sm kick-btn"
									onclick={() => onKickPlayer(player)}
									title="Kick player from server"
									aria-label="Kick {player.username}"
								>
									<UserX size={14} />
									<span>Kick</span>
								</button>
							{/if}

							<button
								class="btn btn-danger btn-sm ban-btn"
								onclick={() => onBanPlayer(player)}
								title="Ban player from server"
								aria-label="Ban {player.username}"
							>
								<ShieldAlert size={14} />
								<span>Ban</span>
							</button>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<!-- Table View -->
		<div class="table-container shadow-card">
			<table class="table">
				<thead>
					<tr>
						<th>Player</th>
						<th>Status</th>
						<th>Dimension</th>
						<th>Health & Food</th>
						<th>Last Seen</th>
						<th class="text-right">Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredPlayers as player (player.uuid || player.username)}
						{@const dim = getDimensionBadge(player.dimension)}
						{@const healthTxt =
							player.health == null ? UNAVAILABLE : `${player.health.toFixed(0)}/20`}
						{@const foodTxt = player.food == null ? UNAVAILABLE : `${player.food}/20`}
						<tr>
							<!-- Player Identity -->
							<td>
								<div class="table-player-cell">
									<img
										src="https://mc-heads.net/avatar/{encodeURIComponent(player.uuid || player.username)}/32"
										alt="{player.username}'s 3D head"
										class="table-head-avatar"
										width="32"
										height="32"
										loading="lazy"
									/>
									<div class="table-player-info">
										<div class="table-name-row">
											<span class="table-username">{player.username}</span>
											{#if player.is_op}
												<span class="badge badge-op badge-xs">
													<Crown size={10} />
													OP
												</span>
											{/if}
										</div>
										<span class="table-uuid" title={player.uuid}>{formatShortUuid(player.uuid)}</span>
									</div>
								</div>
							</td>

							<!-- Status -->
							<td>
								{#if player.is_online}
									<span class="badge badge-success">
										<span class="status-dot status-dot-success status-dot-pulse"></span>
										ONLINE
									</span>
								{:else if player.is_banned}
									<span class="badge badge-danger">BANNED</span>
								{:else}
									<span class="badge badge-secondary">OFFLINE</span>
								{/if}
							</td>

							<!-- Dimension -->
							<td>
								<span class="badge {dim.badgeClass}">
									<Globe size={12} />
									{dim.label}
								</span>
							</td>

							<!-- Health & Food -->
							<td>
								<div class="table-vitals-cell">
									<div class="table-vital-item" title="Health: {healthTxt}">
										<Heart size={13} style="color: {getHealthColor(player.health)};" />
										<span>{healthTxt}</span>
									</div>
									<div class="table-vital-item" title="Food: {foodTxt}">
										<Utensils
											size={13}
											style="color: {player.food == null ? 'var(--text-muted)' : 'var(--warning)'};"
										/>
										<span>{foodTxt}</span>
									</div>
								</div>
							</td>

							<!-- Last Seen -->
							<td>
								<span class="table-last-seen">{player.is_online ? 'Active now' : formatLastSeen(player.last_seen)}</span>
							</td>

							<!-- Actions -->
							<td class="text-right">
								<div class="table-actions">
									<button
										class="btn btn-secondary btn-sm"
										onclick={() => onViewProfile(player)}
										title="View Full Profile"
										aria-label="View profile for {player.username}"
									>
										<Eye size={14} />
										<span>Profile</span>
									</button>

									{#if player.is_online}
										<button
											class="btn btn-secondary btn-sm"
											onclick={() => onKickPlayer(player)}
											title="Kick Player"
											aria-label="Kick {player.username}"
										>
											<UserX size={14} />
											<span>Kick</span>
										</button>
									{/if}

									<button
										class="btn btn-danger btn-sm"
										onclick={() => onBanPlayer(player)}
										title="Ban Player"
										aria-label="Ban {player.username}"
									>
										<ShieldAlert size={14} />
										<span>Ban</span>
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

<style>
	.player-list-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	/* Control Header */
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

	.search-box {
		position: relative;
		display: flex;
		align-items: center;
		flex: 1;
		min-width: 260px;
	}

	.search-icon {
		position: absolute;
		left: var(--space-3);
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		padding-left: var(--space-8);
	}

	.controls-right {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.filter-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.filter-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		white-space: nowrap;
	}

	.status-select {
		height: 34px;
		font-size: var(--font-size-xs);
		min-width: 130px;
	}

	.view-toggle-group {
		display: flex;
		align-items: center;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		padding: 2px;
	}

	.view-btn {
		height: 30px;
		width: 34px;
		padding: 0;
		border-radius: var(--radius-input);
		color: var(--text-muted);
	}

	.view-btn.active {
		background-color: var(--bg-elevated);
		color: var(--accent-blue-text);
		border: 1px solid var(--border-focus);
	}

	.refresh-label {
		font-size: var(--font-size-xs);
	}

	/* Loading & Empty States */
	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-16) var(--space-6);
		text-align: center;
		gap: var(--space-3);
	}

	.spinner {
		animation: spin 1s linear infinite;
		color: var(--accent-blue-text);
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.empty-icon {
		color: var(--text-muted);
		opacity: 0.5;
	}

	.empty-title {
		font-size: var(--font-size-lg);
		color: var(--text-primary);
	}

	.empty-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		max-width: 420px;
	}

	/* Grid View Styling */
	.player-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-4);
	}

	.player-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		transition: border-color var(--transition-fast), transform var(--transition-fast), box-shadow var(--transition-fast);
	}

	.player-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px -4px rgba(0, 0, 0, 0.4);
	}

	.player-card.online-card {
		border-left: 3px solid var(--accent-green);
	}

	.player-card-header {
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}

	.status-badges-row {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		flex-wrap: wrap;
	}

	.badge-op {
		background-color: rgba(168, 85, 247, 0.15);
		color: var(--accent-purple-text);
		border-color: rgba(168, 85, 247, 0.3);
	}

	.badge-purple {
		background-color: rgba(168, 85, 247, 0.12);
		color: var(--accent-purple-text);
		border-color: rgba(168, 85, 247, 0.25);
	}

	.badge-xs {
		padding: 1px 6px;
		font-size: 10px;
	}

	.player-card-body {
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: var(--space-3);
	}

	.avatar-container {
		width: 64px;
		height: 64px;
		border-radius: 8px;
		overflow: hidden;
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-focus);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.player-head-avatar {
		width: 64px;
		height: 64px;
		image-rendering: pixelated;
		object-fit: contain;
	}

	.player-identity {
		display: flex;
		flex-direction: column;
		gap: 2px;
		width: 100%;
	}

	.player-username {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.player-uuid {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.vitals-container {
		width: 100%;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		background-color: var(--bg-base);
		padding: var(--space-3);
		border-radius: var(--radius-input);
		border: 1px solid var(--border-subtle);
	}

	.vital-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.vital-meta {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		width: 65px;
		flex-shrink: 0;
	}

	.vital-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.vital-value {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		width: 38px;
		text-align: right;
		flex-shrink: 0;
	}

	.last-seen-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
	}

	.last-seen-label {
		color: var(--text-muted);
	}

	.last-seen-value {
		color: var(--text-secondary);
		font-weight: var(--font-weight-medium);
	}

	.player-card-footer {
		padding: var(--space-3) var(--space-4);
		border-top: 1px solid var(--border-subtle);
		background-color: rgba(0, 0, 0, 0.15);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}

	.view-profile-btn {
		flex: 1;
	}

	.quick-actions {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}

	/* Table View Styling */
	.table-player-cell {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.table-head-avatar {
		width: 32px;
		height: 32px;
		border-radius: 4px;
		border: 1px solid var(--border-subtle);
		image-rendering: pixelated;
		background-color: var(--bg-elevated);
		flex-shrink: 0;
	}

	.table-player-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.table-name-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.table-username {
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.table-uuid {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.table-vitals-cell {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.table-vital-item {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.table-last-seen {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.table-actions {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
	}

	.text-right {
		text-align: right;
	}
</style>
