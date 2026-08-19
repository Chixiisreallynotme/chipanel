<script>
	import { onMount, onDestroy } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import PlayerList from '$lib/components/players/PlayerList.svelte';
	import PlayerProfileModal from '$lib/components/players/PlayerProfileModal.svelte';
	import {
		Users,
		UserCheck,
		ShieldAlert,
		Crown,
		RefreshCw,
		CheckCircle2,
		AlertCircle,
		Info,
		X,
		UserX,
		ShieldCheck,
		Radio
	} from 'lucide-svelte';

	// Page state
	let players = $state([]);
	let loading = $state(false);
	let isSilentRefreshing = $state(false);
	let autoRefreshEnabled = $state(true);
	let lastUpdatedTime = $state('');

	// Profile Modal state
	let selectedPlayer = $state(null);
	let profileModalOpen = $state(false);

	// Direct Kick/Ban inline action modal states
	let quickActionModalOpen = $state(false);
	let quickActionType = $state(null); // 'kick' | 'ban'
	let targetPlayerForQuickAction = $state(null);
	let quickActionReason = $state('');
	let isExecutingQuickAction = $state(false);

	// Toast Notification state
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

	function updateTimestamp() {
		const now = new Date();
		lastUpdatedTime = now.toLocaleTimeString(undefined, {
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});
	}

	// Fetch players from /api/players
	async function loadPlayers(showFullLoader = true) {
		if (showFullLoader) {
			loading = true;
		} else {
			isSilentRefreshing = true;
		}

		try {
			const data = await apiGet('/api/players');
			players = Array.isArray(data) ? data : [];
			updateTimestamp();
		} catch (err) {
			console.error('Failed to load players:', err);
			if (showFullLoader) {
				addToast(
					'error',
					'Failed to Load Players',
					err.message || 'Could not fetch player list from server.'
				);
			}
		} finally {
			if (showFullLoader) loading = false;
			isSilentRefreshing = false;
		}
	}

	// Auto-refresh interval (10s)
	let autoRefreshTimer = null;

	function setupAutoRefresh() {
		if (autoRefreshTimer) clearInterval(autoRefreshTimer);
		autoRefreshTimer = setInterval(() => {
			if (autoRefreshEnabled && !loading) {
				loadPlayers(false);
			}
		}, 10000);
	}

	onMount(() => {
		loadPlayers(true);
		setupAutoRefresh();
	});

	onDestroy(() => {
		if (autoRefreshTimer) clearInterval(autoRefreshTimer);
	});

	// Reactive synchronization with WebSocket telemetry
	let previousOnlineCount = $state(null);
	$effect(() => {
		const currentOnline = wsStore.telemetry.players?.online;
		if (
			currentOnline !== undefined &&
			previousOnlineCount !== null &&
			currentOnline !== previousOnlineCount
		) {
			// Trigger fast reactive update when player connects or disconnects
			loadPlayers(false);
		}
		previousOnlineCount = currentOnline;
	});

	// Reactive notification when pending commands are executed automatically upon player connect
	let previousExecutionStamp = $state(null);
	$effect(() => {
		const execEvent = wsStore.lastPendingExecution;
		if (execEvent && execEvent.timestamp !== previousExecutionStamp) {
			previousExecutionStamp = execEvent.timestamp;
			addToast(
				'success',
				'Commandes différées appliquées',
				`${execEvent.commandsCount} commande(s) exécutée(s) avec succès pour ${execEvent.playerName} !`
			);
			loadPlayers(false);
		}
	});

	// Derived Hero Header Stats
	let totalCount = $derived(players.length);
	let onlineCount = $derived.by(() => {
		const listOnline = players.filter((p) => p.is_online).length;
		return wsStore.telemetry.players?.online ?? listOnline;
	});
	let bannedCount = $derived(players.filter((p) => p.is_banned).length);
	let opCount = $derived(players.filter((p) => p.is_op).length);

	// Action Handlers
	function handleViewProfile(player) {
		selectedPlayer = player;
		profileModalOpen = true;
	}

	function closeProfileModal() {
		profileModalOpen = false;
		selectedPlayer = null;
	}

	function handleOpenQuickKick(player) {
		targetPlayerForQuickAction = player;
		quickActionType = 'kick';
		quickActionReason = 'Kicked by admin via ChiPanel';
		quickActionModalOpen = true;
	}

	function handleOpenQuickBan(player) {
		targetPlayerForQuickAction = player;
		quickActionType = 'ban';
		quickActionReason = 'Banned by admin via ChiPanel';
		quickActionModalOpen = true;
	}

	function closeQuickActionModal() {
		if (isExecutingQuickAction) return;
		quickActionModalOpen = false;
		targetPlayerForQuickAction = null;
		quickActionType = null;
		quickActionReason = '';
	}

	async function confirmQuickAction() {
		if (!targetPlayerForQuickAction || !quickActionType || isExecutingQuickAction) return;
		isExecutingQuickAction = true;

		const username = targetPlayerForQuickAction.username;
		const uuid = targetPlayerForQuickAction.uuid;
		const action = quickActionType;

		try {
			const res = await apiPost('/api/players/action', {
				uuid,
				action,
				reason: quickActionReason.trim()
			});

			const actionFormatted = action.toUpperCase();
			addToast(
				'success',
				`Player ${actionFormatted}`,
				res.message || `Successfully executed ${action} for ${username}`
			);

			closeQuickActionModal();
			await loadPlayers(false);
		} catch (err) {
			console.error(`Failed quick ${action} action:`, err);
			addToast('error', `Action Failed`, err.message || `Failed to ${action} ${username}`);
		} finally {
			isExecutingQuickAction = false;
		}
	}

	// Callback from Profile Modal when moderation action succeeds
	function handleProfileModalActionSuccess(actionType, message) {
		const actionTitleMap = {
			kick: 'Player Kicked',
			ban: 'Player Banned',
			pardon: 'Player Pardoned',
			unban: 'Player Pardoned',
			teleport: 'Teleport Executed',
			op: 'Operator Granted',
			deop: 'Operator Revoked',
			gamemode: 'Gamemode Changed',
			heal: 'Player Healed & Fed',
			feed: 'Player Fed',
			kill: 'Player Killed',
			clear: 'Inventory Cleared',
			give: 'Item Given',
			whitelist_add: 'Added to Whitelist',
			whitelist_remove: 'Removed from Whitelist',
			effects: 'Status Effect Updated',
			permissions: 'Permissions Updated'
		};

		addToast('success', actionTitleMap[actionType] || 'Action Executed', message);
		loadPlayers(false);
	}

	function handleKeydown(e) {
		if (e.key === 'Escape') {
			if (quickActionModalOpen) {
				closeQuickActionModal();
			} else if (profileModalOpen) {
				closeProfileModal();
			}
		}
	}
</script>

<svelte:head>
	<title>Player Directory & Moderation - ChiPanel</title>
</svelte:head>
<svelte:window onkeydown={handleKeydown} />

<div class="players-page">
	<!-- Top Hero Banner -->
	<div class="hero-header">
		<div class="hero-header-top">
			<div class="hero-title-section">
				<div class="hero-icon-box">
					<Users size={26} />
				</div>
				<div>
					<h1 class="page-title">Player Directory & Moderation</h1>
					<p class="page-subtitle">
						Monitor active players, inspect inventories and effects, and execute admin moderation
						commands in real-time.
					</p>
				</div>
			</div>

			<!-- Real-time Live Status Controls -->
			<div class="hero-live-controls">
				<button
					class="live-status-pill {autoRefreshEnabled ? 'active' : ''}"
					onclick={() => {
						autoRefreshEnabled = !autoRefreshEnabled;
						if (autoRefreshEnabled) setupAutoRefresh();
					}}
					title="Toggle 10s auto-refresh"
				>
					<span class="live-dot {autoRefreshEnabled ? 'pulse' : ''}"></span>
					<span class="live-text"
						>{autoRefreshEnabled ? 'Live Sync (10s)' : 'Auto-refresh Paused'}</span
					>
				</button>

				{#if lastUpdatedTime}
					<span class="last-sync-time" title="Last synced with Minecraft server">
						Sync: {lastUpdatedTime}
					</span>
				{/if}

				<button
					class="btn btn-secondary btn-sm refresh-btn-hero {isSilentRefreshing
						? 'btn-loading'
						: ''}"
					onclick={() => loadPlayers(false)}
					disabled={isSilentRefreshing || loading}
					title="Refresh now"
				>
					<RefreshCw size={14} class={isSilentRefreshing ? 'spinning' : ''} />
				</button>
			</div>
		</div>

		<!-- Hero Stats Cards Grid -->
		<div class="hero-stats-row">
			<div class="stat-card">
				<div class="stat-icon icon-blue">
					<Users size={20} />
				</div>
				<div class="stat-info">
					<span class="stat-val">{totalCount}</span>
					<span class="stat-lbl">Total Players</span>
				</div>
			</div>

			<div class="stat-card">
				<div class="stat-icon icon-green">
					<UserCheck size={20} />
				</div>
				<div class="stat-info">
					<span class="stat-val">{onlineCount}</span>
					<span class="stat-lbl">Online Players</span>
				</div>
			</div>

			<div class="stat-card">
				<div class="stat-icon icon-danger">
					<ShieldAlert size={20} />
				</div>
				<div class="stat-info">
					<span class="stat-val">{bannedCount}</span>
					<span class="stat-lbl">Banned Players</span>
				</div>
			</div>

			<div class="stat-card">
				<div class="stat-icon icon-purple">
					<Crown size={20} />
				</div>
				<div class="stat-info">
					<span class="stat-val">{opCount}</span>
					<span class="stat-lbl">Operators</span>
				</div>
			</div>
		</div>
	</div>

	<!-- Main Player List Component -->
	<div class="page-body">
		<PlayerList
			{players}
			{loading}
			onViewProfile={handleViewProfile}
			onKickPlayer={handleOpenQuickKick}
			onBanPlayer={handleOpenQuickBan}
			onRefresh={() => loadPlayers(false)}
		/>
	</div>

	<!-- Profile Modal Drawer Component -->
	<PlayerProfileModal
		open={profileModalOpen}
		player={selectedPlayer}
		onClose={closeProfileModal}
		onActionSuccess={handleProfileModalActionSuccess}
	/>

	<!-- Quick Kick / Ban Dialog Modal -->
	{#if quickActionModalOpen && targetPlayerForQuickAction}
		<div class="modal-backdrop" onclick={closeQuickActionModal} aria-hidden="true">
			<div
				class="modal"
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => {
					e.stopPropagation();
					if (e.key === 'Escape') closeQuickActionModal();
				}}
				role="dialog"
				aria-modal="true"
				aria-labelledby="quick-action-title"
			>
				<div class="modal-header">
					<div class="quick-title-row">
						{#if quickActionType === 'kick'}
							<UserX size={20} class="icon-warning" />
							<h3 id="quick-action-title" class="modal-title">
								Kick {targetPlayerForQuickAction.username}
							</h3>
						{:else}
							<ShieldAlert size={20} class="icon-danger" />
							<h3 id="quick-action-title" class="modal-title">
								Ban {targetPlayerForQuickAction.username}
							</h3>
						{/if}
					</div>
					<button
						class="btn btn-ghost btn-icon btn-sm"
						onclick={closeQuickActionModal}
						aria-label="Close modal"
					>
						<X size={18} />
					</button>
				</div>

				<div class="modal-body">
					<p class="quick-action-text">
						Are you sure you want to {quickActionType}
						<strong>{targetPlayerForQuickAction.username}</strong> from the server?
					</p>

					<div class="form-group mt-4 mb-0">
						<label for="quick-reason-input" class="label">Reason</label>
						<input
							id="quick-reason-input"
							type="text"
							class="input"
							placeholder="Enter reason..."
							bind:value={quickActionReason}
						/>
					</div>
				</div>

				<div class="modal-footer">
					<button
						class="btn btn-secondary"
						onclick={closeQuickActionModal}
						disabled={isExecutingQuickAction}
					>
						Cancel
					</button>
					<button
						class="btn {quickActionType === 'kick'
							? 'btn-secondary'
							: 'btn-danger'} {isExecutingQuickAction ? 'btn-loading' : ''}"
						onclick={confirmQuickAction}
						disabled={isExecutingQuickAction}
					>
						{#if !isExecutingQuickAction}
							{#if quickActionType === 'kick'}
								<UserX size={16} />
							{:else}
								<ShieldAlert size={16} />
							{/if}
						{/if}
						<span>Confirm {quickActionType === 'kick' ? 'Kick' : 'Ban'}</span>
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Toast Notifications Container -->
	{#if toasts.length > 0}
		<div class="toast-container">
			{#each toasts as toast (toast.id)}
				<div class="toast-item toast-{toast.type}">
					<div class="toast-icon">
						{#if toast.type === 'success'}
							<CheckCircle2 size={18} />
						{:else if toast.type === 'error'}
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
						class="btn btn-ghost btn-icon btn-sm toast-close-btn"
						onclick={() => removeToast(toast.id)}
						aria-label="Close notification"
					>
						<X size={14} />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.players-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		position: relative;
	}

	/* Hero Header */
	.hero-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		border-bottom: 1px solid var(--border);
		padding-bottom: var(--space-6);
	}

	.hero-header-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.hero-title-section {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.hero-icon-box {
		width: 52px;
		height: 52px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	.page-title {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		letter-spacing: -0.015em;
		line-height: 1.2;
	}

	.page-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		margin-top: 2px;
	}

	/* Hero Live Controls */
	.hero-live-controls {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.live-status-pill {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-full);
		padding: 4px 12px;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--text-secondary);
		cursor: pointer;
		transition: border-color var(--transition-fast), background-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
	}

	.live-status-pill:hover {
		border-color: var(--border-hover);
		color: var(--text-primary);
	}

	.live-status-pill:active {
		transform: scale(0.97);
	}

	.live-status-pill.active {
		border-color: var(--accent-green-border);
		background-color: rgba(34, 197, 94, 0.08);
		color: var(--accent-green);
	}

	.live-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--text-muted);
	}

	.live-status-pill.active .live-dot {
		background-color: var(--accent-green);
	}

	.live-dot.pulse {
		box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7);
		animation: livePulse 1.6s infinite;
	}

	@keyframes livePulse {
		0% {
			transform: scale(0.95);
			box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7);
		}
		70% {
			transform: scale(1);
			box-shadow: 0 0 0 6px rgba(34, 197, 94, 0);
		}
		100% {
			transform: scale(0.95);
			box-shadow: 0 0 0 0 rgba(34, 197, 94, 0);
		}
	}

	.last-sync-time {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--text-muted);
	}

	.refresh-btn-hero {
		padding: 6px;
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		100% {
			transform: rotate(360deg);
		}
	}

	/* Stats Row */
	.hero-stats-row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
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

	.icon-warning {
		color: var(--warning);
	}

	.stat-info {
		display: flex;
		flex-direction: column;
	}

	.stat-val {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		line-height: 1.2;
	}

	.stat-lbl {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.page-body {
		min-height: 400px;
	}

	.quick-title-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.quick-action-text {
		font-size: var(--font-size-base);
		color: var(--text-primary);
	}

	.mt-4 {
		margin-top: var(--space-4);
	}
	.mb-0 {
		margin-bottom: 0;
	}

	/* Toast Notification Container */
	.toast-container {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		max-width: 380px;
		width: 100%;
		pointer-events: none;
	}

	.toast-item {
		pointer-events: auto;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-focus);
		border-radius: var(--radius-card);
		padding: var(--space-4);
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		box-shadow: var(--elevation-shadow);
		animation: toastSlideIn var(--transition-fast) ease-out;
	}

	@keyframes toastSlideIn {
		from {
			opacity: 0;
			transform: translateY(16px) scale(0.96);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.toast-success {
		border-color: var(--accent-green-border);
		background: linear-gradient(
			135deg,
			var(--bg-surface) 0%,
			rgba(74, 222, 128, 0.05) 100%
		);
	}

	.toast-success .toast-icon {
		color: var(--accent-green);
	}

	.toast-error {
		border-color: var(--danger-border);
		background: linear-gradient(135deg, var(--bg-surface) 0%, rgba(244, 63, 94, 0.05) 100%);
	}

	.toast-error .toast-icon {
		color: var(--danger-text);
	}

	.toast-info {
		border-color: var(--accent-blue-border);
		background: linear-gradient(135deg, var(--bg-surface) 0%, rgba(99, 102, 241, 0.05) 100%);
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
		word-break: break-word;
	}

	.toast-close-btn {
		color: var(--text-muted);
		padding: 2px;
		margin-top: -2px;
		margin-right: -4px;
	}

	.toast-close-btn:hover {
		color: var(--text-primary);
	}

	@media (max-width: 640px) {
		.toast-container {
			left: var(--space-4);
			right: var(--space-4);
			bottom: var(--space-4);
			max-width: none;
		}
	}
</style>
