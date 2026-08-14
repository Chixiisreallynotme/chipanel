<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import { UNAVAILABLE } from '$lib/components/dashboard/serverState.js';
	import InventoryVisualizer from './InventoryVisualizer.svelte';
	import StatusEffectsPanel from './StatusEffectsPanel.svelte';
	import BasicPermissionsPanel from './BasicPermissionsPanel.svelte';
	import {
		X,
		Crown,
		UserX,
		ShieldAlert,
		ShieldCheck,
		Navigation,
		Heart,
		Utensils,
		Zap,
		Clock,
		MapPin,
		Globe,
		Calendar,
		Sparkles,
		CheckCircle2,
		AlertCircle,
		Loader2,
		Package,
		Activity,
		Key
	} from 'lucide-svelte';

	/**
	 * @typedef {Object} PlayerDetail
	 * @property {string} uuid
	 * @property {string} username
	 * @property {boolean} is_online
	 * @property {number} health
	 * @property {number} max_health
	 * @property {number} food
	 * @property {number} exp_level
	 * @property {number} exp_progress
	 * @property {number} position_x
	 * @property {number} position_y
	 * @property {number} position_z
	 * @property {string} dimension
	 * @property {number} playtime_seconds
	 * @property {number} first_joined_timestamp
	 * @property {number} last_joined_timestamp
	 * @property {boolean} is_op
	 * @property {boolean} is_banned
	 * @property {string|null} ban_reason
	 */

	let {
		open = false,
		player = null, // Initial summary player or object containing uuid
		onClose = () => {},
		onActionSuccess = (action, message) => {}
	} = $props();

	// Detailed state fetched from API
	let detail = $state(null);
	let loadingDetail = $state(false);
	let actionLoading = $state(null); // 'kick' | 'ban' | 'pardon' | 'teleport' | 'op' | 'deop'
	let errorMessage = $state('');

	// Inventory NBT state
	let inventoryData = $state(null);
	let loadingInventory = $state(false);
	let inventoryError = $state('');

	// Action Form inputs
	let kickReason = $state('Kicked by admin via ChiPanel');
	let banReason = $state('Banned by admin via ChiPanel');
	let tpX = $state(0);
	let tpY = $state(64);
	let tpZ = $state(0);
	let activeTab = $state('overview'); // 'overview' | 'inventory' | 'effects' | 'permissions' | 'moderation'

	// Fetch detailed player stats whenever modal opens or player UUID changes
	$effect(() => {
		if (open && player && player.uuid) {
			fetchPlayerDetail(player.uuid);
			if (activeTab === 'inventory') {
				fetchPlayerInventory(player.uuid);
			}
		} else if (!open) {
			detail = null;
			inventoryData = null;
			inventoryError = '';
			errorMessage = '';
			actionLoading = null;
		}
	});

	// Reactively fetch inventory data when user switches to inventory tab
	$effect(() => {
		if (open && (detail?.uuid || player?.uuid) && activeTab === 'inventory' && !inventoryData && !loadingInventory) {
			fetchPlayerInventory((detail || player).uuid);
		}
	});

	async function fetchPlayerInventory(uuid) {
		if (!uuid) return;
		loadingInventory = true;
		inventoryError = '';
		try {
			const res = await apiGet(`/api/players/${uuid}/inventory`);
			inventoryData = res;
		} catch (err) {
			console.error(`Failed to fetch inventory for player ${uuid}:`, err);
			inventoryError = err.message || 'Failed to load player inventory';
		} finally {
			loadingInventory = false;
		}
	}

	async function fetchPlayerDetail(uuid) {
		loadingDetail = true;
		errorMessage = '';
		try {
			const res = await apiGet(`/api/players/${uuid}`);
			detail = res;
			if (res) {
				// `|| 0` used to coerce a genuine position of 0 (and a genuine y=0) into
				// the placeholder, silently retargeting a teleport. These are form
				// defaults for an unknown position only.
				tpX = res.position_x == null ? 0 : Math.round(res.position_x);
				tpY = res.position_y == null ? 64 : Math.round(res.position_y);
				tpZ = res.position_z == null ? 0 : Math.round(res.position_z);
			}
		} catch (err) {
			console.error(`Failed to fetch detail for player ${uuid}:`, err);
			// Do NOT synthesise a detail record here. The old fallback invented a
			// complete, plausible player (20/20 HP, level 0, spawn coords, Overworld,
			// zero playtime) that was indistinguishable from real telemetry (B1).
			// `detail` stays null; the template falls back to the summary `player`,
			// and every field the summary lacks renders as "—".
			detail = null;
			errorMessage =
				err.message || 'Failed to load full player detail — showing summary data only.';
		} finally {
			loadingDetail = false;
		}
	}

	// Format helpers
	function formatPlaytime(seconds) {
		// An unreported playtime is unknown, not zero.
		if (seconds == null) return UNAVAILABLE;
		if (seconds <= 0) return '0 mins';
		const hours = Math.floor(seconds / 3600);
		const mins = Math.floor((seconds % 3600) / 60);

		if (hours >= 24) {
			const days = Math.floor(hours / 24);
			const remHours = hours % 24;
			return `${days}d ${remHours}h ${mins}m`;
		}
		if (hours > 0) {
			return `${hours}h ${mins}m`;
		}
		return `${mins} mins`;
	}

	function formatDate(timestamp) {
		if (!timestamp || timestamp === 0) return 'Unknown';
		const date = new Date(timestamp * 1000);
		return date.toLocaleString(undefined, {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// An unreported dimension is unknown, NOT the Overworld (B1). An unrecognised
	// dimension string is shown verbatim rather than coerced.
	function formatDimension(dimStr) {
		if (!dimStr) return UNAVAILABLE;
		const d = dimStr.toLowerCase();
		if (d.includes('nether')) return 'Nether';
		if (d.includes('end')) return 'The End';
		if (d.includes('overworld')) return 'Overworld';
		// Modded dimension: show it rather than guessing, but drop the namespace so a
		// long id cannot stretch this nowrap badge past the stat box.
		return dimStr.split(':').pop();
	}

	function getDimensionBadgeClass(dimStr) {
		const d = (dimStr || '').toLowerCase();
		if (d.includes('nether')) return 'badge-danger';
		if (d.includes('end')) return 'badge-purple';
		if (d.includes('overworld')) return 'badge-success';
		return 'badge-secondary';
	}

	/** Bar width for a 0..max vital, or null when there is nothing to draw. */
	function vitalPercent(value, max) {
		if (value == null || max == null || !max) return null;
		return Math.min(100, Math.max(0, (value / max) * 100));
	}

	/** @param {number | null | undefined} v */
	function fmtCoord(v) {
		return v == null ? UNAVAILABLE : v.toFixed(1);
	}

	// Action Handler: Execute moderation commands via /api/players/action
	async function handleExecuteAction(actionType) {
		// `detail` is null whenever the detail fetch failed — the same summary the
		// template renders is still a valid action target, so key off that rather
		// than requiring a detail record we may never get.
		const target = detail || player;
		if (!target?.uuid || actionLoading) return;
		actionLoading = actionType;
		errorMessage = '';

		let body = {
			uuid: target.uuid,
			action: actionType
		};

		if (actionType === 'kick') {
			body.reason = kickReason.trim() || 'Kicked by admin via ChiPanel';
		} else if (actionType === 'ban') {
			body.reason = banReason.trim() || 'Banned by admin via ChiPanel';
		} else if (actionType === 'teleport') {
			body.target_coords = `${tpX} ${tpY} ${tpZ}`;
		}

		try {
			const res = await apiPost('/api/players/action', body);

			// Locally update state if relevant. Applied to whichever record the
			// template is actually rendering (`detail || player`).
			if (actionType === 'ban') {
				target.is_banned = true;
				target.ban_reason = body.reason;
			} else if (actionType === 'pardon') {
				target.is_banned = false;
				target.ban_reason = null;
			} else if (actionType === 'op') {
				target.is_op = true;
			} else if (actionType === 'deop') {
				target.is_op = false;
			} else if (actionType === 'kick') {
				target.is_online = false;
			}

			const successMessage = res.message || res.output || `Action '${actionType}' executed successfully.`;
			onActionSuccess(actionType, successMessage);
		} catch (err) {
			console.error(`Failed to execute player action ${actionType}:`, err);
			errorMessage = err.message || `Failed to execute action ${actionType}`;
		} finally {
			actionLoading = null;
		}
	}
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape' && open) onClose(); }} />

{#if open && (detail || player)}
	{@const p = detail || player}
		{@const healthPct = vitalPercent(p.health, p.max_health)}
		{@const foodPct = vitalPercent(p.food, 20)}
		{@const expPct = vitalPercent(p.exp_progress, 1)}
	<div class="modal-backdrop" onclick={onClose}>
		<div
			class="modal profile-modal-drawer"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => {
				e.stopPropagation();
				if (e.key === 'Escape') onClose();
			}}
			role="dialog"
			aria-modal="true"
			aria-labelledby="profile-modal-title"
		>
			<!-- Modal Header -->
			<div class="modal-header">
				<div class="player-header-info">
					<img
						src="https://mc-heads.net/avatar/{encodeURIComponent(p.uuid || p.username)}/32"
						alt="{p.username} avatar"
						class="header-avatar"
						width="32"
						height="32"
					/>
					<div>
						<div class="title-row">
							<h2 id="profile-modal-title" class="modal-title">{p.username}</h2>
							{#if p.is_op}
								<span class="badge badge-op">
									<Crown size={12} />
									OP
								</span>
							{/if}
						</div>
						<span class="header-uuid">{p.uuid}</span>
					</div>
				</div>

				<div class="header-right-tools">
					{#if p.is_online}
						<span class="badge badge-success">
							<span class="status-dot status-dot-success status-dot-pulse"></span>
							ONLINE
						</span>
					{:else if p.is_banned}
						<span class="badge badge-danger">BANNED</span>
					{:else}
						<span class="badge badge-secondary">OFFLINE</span>
					{/if}

					<button class="btn btn-ghost btn-icon btn-sm close-modal-btn" onclick={onClose} aria-label="Close modal">
						<X size={18} />
					</button>
				</div>
			</div>

			<!-- Sub-Navigation Tabs -->
			<div class="modal-nav-tabs" role="tablist">
				<button
					id="tab-overview"
					class="nav-tab-btn {activeTab === 'overview' ? 'active' : ''}"
					onclick={() => (activeTab = 'overview')}
					role="tab"
					aria-selected={activeTab === 'overview'}
					aria-controls="tabpanel-overview"
				>
					<Sparkles size={15} />
					<span>Overview</span>
				</button>

				<button
					id="tab-inventory"
					class="nav-tab-btn {activeTab === 'inventory' ? 'active' : ''}"
					onclick={() => (activeTab = 'inventory')}
					role="tab"
					aria-selected={activeTab === 'inventory'}
					aria-controls="tabpanel-inventory"
				>
					<Package size={15} />
					<span>Inventory</span>
				</button>

				<button
					id="tab-effects"
					class="nav-tab-btn {activeTab === 'effects' ? 'active' : ''}"
					onclick={() => (activeTab = 'effects')}
					role="tab"
					aria-selected={activeTab === 'effects'}
					aria-controls="tabpanel-effects"
				>
					<Activity size={15} />
					<span>Status Effects</span>
				</button>

				<button
					id="tab-permissions"
					class="nav-tab-btn {activeTab === 'permissions' ? 'active' : ''}"
					onclick={() => (activeTab = 'permissions')}
					role="tab"
					aria-selected={activeTab === 'permissions'}
					aria-controls="tabpanel-permissions"
				>
					<Key size={15} />
					<span>Permissions</span>
				</button>

				<button
					id="tab-moderation"
					class="nav-tab-btn {activeTab === 'moderation' ? 'active' : ''}"
					onclick={() => (activeTab = 'moderation')}
					role="tab"
					aria-selected={activeTab === 'moderation'}
					aria-controls="tabpanel-moderation"
				>
					<ShieldAlert size={15} />
					<span>Moderation</span>
				</button>
			</div>

			<!-- Error Alert Display -->
			{#if errorMessage}
				<div class="error-banner">
					<AlertCircle size={18} />
					<span>{errorMessage}</span>
				</div>
			{/if}

			<!-- Modal Scrollable Body -->
			<div class="modal-body profile-body">
				{#if loadingDetail}
					<div class="loading-overlay">
						<Loader2 size={32} class="spinner" />
						<span>Fetching full player telemetry data...</span>
					</div>
				{/if}

				{#if activeTab === 'overview'}
					<!-- Overview Tab: 3D Body Viewer + Stats Grid -->
					<div role="tabpanel" id="tabpanel-overview" aria-labelledby="tab-overview">
						<div class="overview-layout">
							<!-- Left Box: 3D Full Body Skin Viewer -->
							<div class="skin-viewer-card">
								<div class="skin-viewer-container">
									<img
										src="https://mc-heads.net/body/{encodeURIComponent(p.uuid || p.username)}/150"
										alt="{p.username}'s 3D full body skin viewer"
										class="skin-body-image"
										width="150"
										height="220"
										loading="eager"
									/>
								</div>
								<span class="skin-caption">3D Skin View</span>
							</div>

							<!-- Right Box: Detailed Stats Grid -->
							<div class="stats-grid">
								<!-- Health -->
								<div class="stat-box">
									<div class="stat-header">
										<Heart size={16} class="icon-danger" />
										<span class="stat-title">Health</span>
									</div>
									<div class="stat-value-container">
										<span class="stat-main-value">
											{p.health == null ? UNAVAILABLE : p.health.toFixed(1)} / {p.max_health ??
												UNAVAILABLE} HP
										</span>
										{#if healthPct == null}
											<div class="gauge-bar gauge-unknown" title="Health not reported for this player"></div>
										{:else}
											<div class="gauge-bar">
												<div class="gauge-fill gauge-fill-success" style="width: {healthPct}%;"></div>
											</div>
										{/if}
									</div>
								</div>

								<!-- Food -->
								<div class="stat-box">
									<div class="stat-header">
										<Utensils size={16} class="icon-warning" />
										<span class="stat-title">Food Level</span>
									</div>
									<div class="stat-value-container">
										<span class="stat-main-value">{p.food ?? UNAVAILABLE} / 20</span>
										{#if foodPct == null}
											<div
												class="gauge-bar gauge-unknown"
												title="Food level not reported for this player"
											></div>
										{:else}
											<div class="gauge-bar">
												<div class="gauge-fill gauge-fill-warning" style="width: {foodPct}%;"></div>
											</div>
										{/if}
									</div>
								</div>

								<!-- EXP Level & Progress -->
								<div class="stat-box">
									<div class="stat-header">
										<Zap size={16} class="icon-green" />
										<span class="stat-title">EXP Level</span>
									</div>
									<div class="stat-value-container">
										<span class="stat-main-value">
											{p.exp_level == null ? UNAVAILABLE : `Level ${p.exp_level}`}
										</span>
										{#if expPct == null}
											<div class="gauge-bar gauge-unknown" title="EXP progress not reported"></div>
										{:else}
											<div class="gauge-bar">
												<div class="gauge-fill gauge-fill-success" style="width: {expPct}%;"></div>
											</div>
										{/if}
									</div>
								</div>

								<!-- Playtime -->
								<div class="stat-box">
									<div class="stat-header">
										<Clock size={16} class="icon-blue" />
										<span class="stat-title">Total Playtime</span>
									</div>
									<span class="stat-large-text">{formatPlaytime(p.playtime_seconds)}</span>
								</div>

								<!-- Position Coords -->
								<div class="stat-box stat-box-wide">
									<div class="stat-header">
										<MapPin size={16} class="icon-purple" />
										<span class="stat-title">World Position Coords</span>
									</div>
									<div class="coords-display">
										<div class="coord-pill">
											<span class="coord-axis">X:</span>
											<span class="coord-val">{fmtCoord(p.position_x)}</span>
										</div>
										<div class="coord-pill">
											<span class="coord-axis">Y:</span>
											<span class="coord-val">{fmtCoord(p.position_y)}</span>
										</div>
										<div class="coord-pill">
											<span class="coord-axis">Z:</span>
											<span class="coord-val">{fmtCoord(p.position_z)}</span>
										</div>
									</div>
								</div>

								<!-- Dimension -->
								<div class="stat-box">
									<div class="stat-header">
										<Globe size={16} class="icon-blue" />
										<span class="stat-title">Dimension</span>
									</div>
									<span class="badge {getDimensionBadgeClass(p.dimension)} stat-badge">
										{formatDimension(p.dimension)}
									</span>
								</div>

								<!-- First Joined -->
								<div class="stat-box">
									<div class="stat-header">
										<Calendar size={16} class="icon-muted" />
										<span class="stat-title">First Joined</span>
									</div>
									<span class="stat-small-text">{formatDate(p.first_joined_timestamp)}</span>
								</div>

								<!-- Last Joined / Seen -->
								<div class="stat-box">
									<div class="stat-header">
										<Calendar size={16} class="icon-muted" />
										<span class="stat-title">Last Seen</span>
									</div>
									<span class="stat-small-text">{formatDate(p.last_joined_timestamp)}</span>
								</div>
							</div>
						</div>
					</div>
				{:else if activeTab === 'inventory'}
					<!-- Inventory & Ender Chest Visualizer Tab -->
					<div role="tabpanel" id="tabpanel-inventory" aria-labelledby="tab-inventory">
						<InventoryVisualizer
							inventory={inventoryData}
							loading={loadingInventory}
							error={inventoryError}
							onRefresh={() => fetchPlayerInventory(p.uuid)}
						/>
					</div>
				{:else if activeTab === 'effects'}
					<!-- Status Effects Tab -->
					<div role="tabpanel" id="tabpanel-effects" aria-labelledby="tab-effects">
						<StatusEffectsPanel uuid={p.uuid} isOnline={p.is_online} />
					</div>
				{:else if activeTab === 'permissions'}
					<!-- Basic Permissions Tab -->
					<div role="tabpanel" id="tabpanel-permissions" aria-labelledby="tab-permissions">
						<BasicPermissionsPanel uuid={p.uuid} />
					</div>
				{:else}
					<!-- Moderation Action Panel Tab -->
					<div role="tabpanel" id="tabpanel-moderation" aria-labelledby="tab-moderation">
						<div class="moderation-panel">
							<!-- Kick Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<UserX size={18} class="icon-warning" />
										<div>
											<h4 class="mod-title">Kick Player from Server</h4>
											<p class="mod-desc">Disconnects active player immediately with an optional reason.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<div class="form-group mb-0">
										<label for="kick-reason" class="label">Reason for Kick</label>
										<input
											id="kick-reason"
											type="text"
											class="input"
											placeholder="Reason displayed on disconnect screen..."
											bind:value={kickReason}
										/>
									</div>
								</div>
								<div class="mod-card-footer">
									<button
										class="btn btn-secondary {actionLoading === 'kick' ? 'btn-loading' : ''}"
										onclick={() => handleExecuteAction('kick')}
										disabled={!p.is_online || actionLoading === 'kick'}
									>
										{#if actionLoading !== 'kick'}
											<UserX size={16} />
										{/if}
										<span>{p.is_online ? 'Kick Player' : 'Player Offline'}</span>
									</button>
								</div>
							</div>

							<!-- Ban / Pardon Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<ShieldAlert size={18} class="icon-danger" />
										<div>
											<h4 class="mod-title">Ban & Blacklist Management</h4>
											<p class="mod-desc">Prevent player from joining the Minecraft server.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									{#if p.is_banned}
										<div class="banned-status-box">
											<AlertCircle size={18} class="icon-danger" />
											<div>
												<strong>Player is currently Banned</strong>
												<p class="ban-reason-text">Reason: {p.ban_reason || 'No reason specified'}</p>
											</div>
										</div>
									{:else}
										<div class="form-group mb-0">
											<label for="ban-reason" class="label">Ban Reason</label>
											<input
												id="ban-reason"
												type="text"
												class="input"
												placeholder="Reason recorded in ban list..."
												bind:value={banReason}
											/>
										</div>
									{/if}
								</div>
								<div class="mod-card-footer">
									{#if p.is_banned}
										<button
											class="btn btn-primary {actionLoading === 'pardon' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('pardon')}
											disabled={actionLoading === 'pardon'}
										>
											{#if actionLoading !== 'pardon'}
												<ShieldCheck size={16} />
											{/if}
											<span>Pardon Player (Unban)</span>
										</button>
									{:else}
										<button
											class="btn btn-danger {actionLoading === 'ban' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('ban')}
											disabled={actionLoading === 'ban'}
										>
											{#if actionLoading !== 'ban'}
												<ShieldAlert size={16} />
											{/if}
											<span>Ban Player</span>
										</button>
									{/if}
								</div>
							</div>

							<!-- Teleport Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<Navigation size={18} class="icon-blue" />
										<div>
											<h4 class="mod-title">Teleport Coordinates</h4>
											<p class="mod-desc">Teleports online player to specified X, Y, Z target location.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<div class="tp-coords-inputs">
										<div class="form-group">
											<label for="tp-x" class="label">X Coord</label>
											<input id="tp-x" type="number" class="input" bind:value={tpX} />
										</div>
										<div class="form-group">
											<label for="tp-y" class="label">Y Coord</label>
											<input id="tp-y" type="number" class="input" bind:value={tpY} />
										</div>
										<div class="form-group">
											<label for="tp-z" class="label">Z Coord</label>
											<input id="tp-z" type="number" class="input" bind:value={tpZ} />
										</div>
									</div>
								</div>
								<div class="mod-card-footer">
									<button
										class="btn btn-primary {actionLoading === 'teleport' ? 'btn-loading' : ''}"
										onclick={() => handleExecuteAction('teleport')}
										disabled={!p.is_online || actionLoading === 'teleport'}
									>
										{#if actionLoading !== 'teleport'}
											<Navigation size={16} />
										{/if}
										<span>{p.is_online ? 'Teleport Player' : 'Player Offline'}</span>
									</button>
								</div>
							</div>

							<!-- OP / DeOP Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<Crown size={18} class="icon-purple" />
										<div>
											<h4 class="mod-title">Operator Permissions (OP)</h4>
											<p class="mod-desc">Grant or revoke operator privileges on the Minecraft server.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<p class="op-desc-text">
										Operators have full administrator permissions to execute all server commands in-game.
									</p>
								</div>
								<div class="mod-card-footer">
									{#if p.is_op}
										<button
											class="btn btn-secondary {actionLoading === 'deop' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('deop')}
											disabled={actionLoading === 'deop'}
										>
											{#if actionLoading !== 'deop'}
												<Crown size={16} />
											{/if}
											<span>Revoke Operator (De-OP)</span>
										</button>
									{:else}
										<button
											class="btn btn-primary btn-purple {actionLoading === 'op' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('op')}
											disabled={actionLoading === 'op'}
										>
											{#if actionLoading !== 'op'}
												<Crown size={16} />
											{/if}
											<span>Grant Operator (OP)</span>
										</button>
									{/if}
								</div>
							</div>
						</div>
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={onClose}>
					Close Drawer
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.profile-modal-drawer {
		max-width: 720px;
		width: 100%;
		max-height: 90vh;
		display: flex;
		flex-direction: column;
	}

	.player-header-info {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.header-avatar {
		width: 32px;
		height: 32px;
		border-radius: 4px;
		border: 1px solid var(--border-subtle);
		image-rendering: pixelated;
		background-color: var(--bg-elevated);
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.modal-title {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		line-height: 1.2;
	}

	.header-uuid {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.header-right-tools {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.close-modal-btn {
		color: var(--text-muted);
	}

	/* Modal Sub Nav Tabs */
	.modal-nav-tabs {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background-color: var(--bg-base);
		padding: var(--space-2) var(--space-4);
		border-bottom: 1px solid var(--border);
	}

	.nav-tab-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--text-muted);
		background: transparent;
		border: none;
		border-radius: var(--radius-btn);
		cursor: pointer;
		transition: color var(--transition-fast), background-color var(--transition-fast);
	}

	.nav-tab-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
	}

	.nav-tab-btn.active {
		color: var(--accent-blue-text);
		background-color: var(--accent-blue-bg);
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--danger-bg);
		border-bottom: 1px solid var(--danger-border);
		color: var(--danger-text);
		padding: var(--space-3) var(--space-6);
		font-size: var(--font-size-sm);
	}

	.profile-body {
		padding: var(--space-6);
		position: relative;
		overflow-y: auto;
		flex: 1;
	}

	.loading-overlay {
		position: absolute;
		inset: 0;
		background-color: rgba(17, 17, 24, 0.75);
		backdrop-filter: blur(2px);
		z-index: 10;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		color: var(--text-muted);
		font-size: var(--font-size-sm);
	}

	/* Overview Tab Layout */
	.overview-layout {
		display: grid;
		grid-template-columns: 180px 1fr;
		gap: var(--space-6);
	}

	@media (max-width: 640px) {
		.overview-layout {
			grid-template-columns: 1fr;
		}
	}

	.skin-viewer-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		padding: var(--space-4);
		gap: var(--space-3);
	}

	.skin-viewer-container {
		width: 150px;
		height: 220px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.skin-body-image {
		width: 150px;
		height: 220px;
		image-rendering: pixelated;
		object-fit: contain;
		filter: drop-shadow(0 8px 16px rgba(0, 0, 0, 0.5));
	}

	.skin-caption {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-3);
	}

	.stat-box {
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-input);
		padding: var(--space-3);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.stat-box-wide {
		grid-column: span 2;
	}

	.stat-header {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.stat-title {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.stat-value-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}

	.stat-main-value {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		font-family: var(--font-mono);
	}

	.stat-large-text {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}

	.stat-small-text {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.stat-badge {
		align-self: flex-start;
		margin-top: 2px;
	}

	.coords-display {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.coord-pill {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		padding: 2px 8px;
		border-radius: var(--radius-sm);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
	}

	.coord-axis {
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-bold);
	}

	.coord-val {
		color: var(--text-primary);
	}

	/* Moderation Panel */
	.moderation-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.mod-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		overflow: hidden;
	}

	.mod-card-header {
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
		background-color: rgba(255, 255, 255, 0.01);
	}

	.mod-title-group {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
	}

	.mod-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.mod-desc {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.mod-card-body {
		padding: var(--space-4);
	}

	.mod-card-footer {
		padding: var(--space-3) var(--space-4);
		background-color: rgba(0, 0, 0, 0.2);
		border-top: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: flex-end;
	}

	.tp-coords-inputs {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-3);
	}

	.mb-0 {
		margin-bottom: 0;
	}

	.banned-status-box {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-input);
		padding: var(--space-3);
		color: var(--danger-text);
	}

	.ban-reason-text {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin-top: 2px;
	}

	.op-desc-text {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}

	/* White label, so it needs the solid (white-safe) fill, not the palette hue. */
	.btn-purple {
		background-color: var(--accent-purple-solid);
		border-color: var(--accent-purple-solid);
		color: #ffffff;
	}

	.btn-purple:hover:not(:disabled) {
		background-color: var(--accent-purple-solid-hover);
		border-color: var(--accent-purple-solid-hover);
		box-shadow: 0 0 16px rgba(139, 92, 246, 0.3);
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

	/* Icon Accents */
	.icon-danger { color: var(--danger-text); }
	.icon-warning { color: var(--warning); }
	.icon-green { color: var(--accent-green); }
	.icon-blue { color: var(--accent-blue-text); }
	.icon-purple { color: var(--accent-purple-text); }
	.icon-muted { color: var(--text-muted); }
</style>
