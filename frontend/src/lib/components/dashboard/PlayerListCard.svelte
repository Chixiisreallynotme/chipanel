<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { apiPost } from '$lib/api/client.js';
	import { serverState, UNAVAILABLE } from './serverState.js';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import { Users, UserX, ShieldAlert, Search } from '$lib/icons.js';

	const USERNAME_REGEX = /^[a-zA-Z0-9_]{1,16}$/;

	let {
		/**
		 * Roster to display. The `/ws` telemetry frame carries counts only, no
		 * names — the caller must supply the list from whichever endpoint has it.
		 * Empty means "nobody known to be online", never a sample roster.
		 * @type {Array<{ username: string, ping: number, uuid?: string }>}
		 */
		roster = []
	} = $props();

	/** @type {string[]} */
	let kickedBannedUsernames = $state([]);
	let searchQuery = $state('');
	/** @type {Record<string, 'kick' | 'ban'>} */
	let actionLoading = $state({});

	let srv = $derived(serverState());

	let players = $derived(roster.filter((p) => !kickedBannedUsernames.includes(p.username)));

	let onlineCount = $derived(srv.onlinePlayers ?? UNAVAILABLE);
	let maxCount = $derived(srv.maxPlayers ?? UNAVAILABLE);

	let filteredPlayers = $derived.by(() => {
		if (!searchQuery.trim()) return players;
		return players.filter((p) =>
			p.username.toLowerCase().includes(searchQuery.toLowerCase().trim())
		);
	});

	function getPingStatus(ping) {
		if (ping < 50) return { dotClass: 'status-dot-success', labelClass: 'badge-success' };
		if (ping <= 120) return { dotClass: 'status-dot-warning', labelClass: 'badge-warning' };
		return { dotClass: 'status-dot-danger', labelClass: 'badge-danger' };
	}

	async function handleKick(username) {
		if (!USERNAME_REGEX.test(username)) {
			console.error(`Invalid username for kick: ${username}`);
			return;
		}
		if (actionLoading[username]) return;
		actionLoading[username] = 'kick';
		try {
			const command = `kick ${username} Kicked by admin via ChiPanel`;
			const sent = wsStore.sendCommand(command);
			if (!sent) {
				await apiPost('/api/rcon/execute', { command });
			}
			if (!kickedBannedUsernames.includes(username)) {
				kickedBannedUsernames = [...kickedBannedUsernames, username];
			}
		} catch (err) {
			console.error(`Failed to kick player ${username}:`, err);
		} finally {
			delete actionLoading[username];
		}
	}

	/** Username awaiting ban confirmation; null when the dialog is closed. */
	let banCandidate = $state(/** @type {string | null} */ (null));

	/** @param {string} username */
	function requestBan(username) {
		if (!USERNAME_REGEX.test(username)) {
			console.error(`Invalid username for ban: ${username}`);
			return;
		}
		if (actionLoading[username]) return;
		banCandidate = username;
	}

	function confirmBan() {
		const username = banCandidate;
		banCandidate = null;
		if (username) handleBan(username);
	}

	async function handleBan(username) {
		if (!USERNAME_REGEX.test(username)) {
			console.error(`Invalid username for ban: ${username}`);
			return;
		}
		if (actionLoading[username]) return;
		actionLoading[username] = 'ban';
		try {
			const command = `ban ${username} Banned by admin via ChiPanel`;
			const sent = wsStore.sendCommand(command);
			if (!sent) {
				await apiPost('/api/rcon/execute', { command });
			}
			if (!kickedBannedUsernames.includes(username)) {
				kickedBannedUsernames = [...kickedBannedUsernames, username];
			}
		} catch (err) {
			console.error(`Failed to ban player ${username}:`, err);
		} finally {
			delete actionLoading[username];
		}
	}
</script>

<ConfirmDialog
	open={banCandidate !== null}
	title="Ban player"
	message={`Are you sure you want to ban ${banCandidate ?? ''}?`}
	confirmLabel="Ban"
	cancelLabel="Cancel"
	onconfirm={confirmBan}
	oncancel={() => (banCandidate = null)}
/>

<div class="card player-list-card">
	<div class="card-header">
		<div class="header-left">
			<Users size={20} class="header-icon" />
			<h3 class="card-title">Online Players ({onlineCount}/{maxCount})</h3>
		</div>
		{#if players.length > 0}
			<div class="search-box">
				<Search size={14} class="search-icon" />
				<input
					type="text"
					class="input input-sm search-input"
					placeholder="Search online players..."
					aria-label="Search online players"
					bind:value={searchQuery}
				/>
			</div>
		{/if}
	</div>

	<div class="card-body player-card-body">
		{#if filteredPlayers.length === 0}
			<EmptyState
					compact
					title={searchQuery.trim() ? `No players matching "${searchQuery}"` : 'No players currently online'}
					description={!searchQuery.trim() ? 'Connect to the Minecraft server to see active players here.' : ''}
				>
					{#snippet icon()}
						<Users size={36} class="empty-icon" />
					{/snippet}
				</EmptyState>
		{:else}
			<div class="player-list">
				{#each filteredPlayers as player (player.username)}
					{@const pingInfo = getPingStatus(player.ping)}
					<div class="player-row">
						<div class="player-info">
							<img
								src="https://mc-heads.net/avatar/{encodeURIComponent(player.username)}/32"
								alt="{player.username}'s skin"
								class="player-avatar"
								width="32"
								height="32"
								loading="lazy"
							/>
							<div class="player-details">
								<span class="player-name">{player.username}</span>
								<div class="ping-badge">
									<span class="status-dot {pingInfo.dotClass}"></span>
									<span class="ping-text">{player.ping} ms</span>
								</div>
							</div>
						</div>

						<div class="player-actions">
							<button
								class="btn btn-secondary btn-sm {actionLoading[player.username] === 'kick' ? 'btn-loading' : ''}"
								disabled={!!actionLoading[player.username]}
								onclick={() => handleKick(player.username)}
								title="Kick {player.username} from server"
								aria-label="Kick {player.username}"
							>
								{#if actionLoading[player.username] !== 'kick'}
									<UserX size={14} />
								{/if}
								<span>Kick</span>
							</button>

							<button
								class="btn btn-danger btn-sm {actionLoading[player.username] === 'ban' ? 'btn-loading' : ''}"
								disabled={!!actionLoading[player.username]}
								onclick={() => requestBan(player.username)}
								title="Ban {player.username} from server"
								aria-label="Ban {player.username}"
							>
								{#if actionLoading[player.username] !== 'ban'}
									<ShieldAlert size={14} />
								{/if}
								<span>Ban</span>
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.player-list-card {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.header-icon {
		color: var(--accent-blue-text);
	}

	.search-box {
		position: relative;
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
		width: 180px;
	}

	.player-card-body {
		padding: var(--space-4) var(--space-6);
		overflow-y: auto;
		max-height: 380px;
	}

	.player-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.player-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-btn);
		transition: border-color var(--transition-fast), background-color var(--transition-fast);
	}

	.player-row:hover {
		border-color: var(--border-focus);
		background-color: var(--bg-elevated);
	}

	.player-info {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		min-width: 0;
	}

	.player-avatar {
		width: 32px;
		height: 32px;
		border-radius: 4px;
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		flex-shrink: 0;
		image-rendering: pixelated;
	}

	.player-details {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		min-width: 0;
	}

	.player-name {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		line-height: 1.2;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.ping-badge {
		display: inline-flex;
		align-items: center;
		gap: var(--space-1);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.ping-text {
		line-height: 1;
	}

	.player-actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
</style>
