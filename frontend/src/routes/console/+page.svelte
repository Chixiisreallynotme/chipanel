<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { serverState, UNAVAILABLE } from '$lib/components/dashboard/serverState.js';
	import LogViewer from '$lib/components/console/LogViewer.svelte';
	import CommandBar from '$lib/components/console/CommandBar.svelte';
	import QuickCommands from '$lib/components/console/QuickCommands.svelte';
	import { Terminal, RefreshCw, Activity, Users } from 'lucide-svelte';

	let activeCommand = $state('');

	let srv = $derived(serverState());

	function handleFillCommand(cmd) {
		activeCommand = cmd;
	}

	function handleExecuteCommand(cmd) {
		// Log or track command execution if needed
	}

	function reconnectWebSocket() {
		wsStore.connect();
	}
</script>

<svelte:head>
	<title>Server Console | ChiPanel</title>
</svelte:head>

<div class="console-page-layout">
	<!-- Page Header Toolbar -->
	<div class="console-header-card">
		<div class="header-main-info">
			<div class="icon-badge">
				<Terminal size={22} class="console-icon" />
			</div>
			<div class="header-titles">
				<div class="title-with-status">
					<h1 class="page-title">Server Console</h1>
					<div class="status-indicator-badge badge {wsStore.connected ? 'badge-success' : 'badge-danger'}">
						<span class="status-dot {wsStore.connected ? 'status-dot-success status-dot-pulse' : 'status-dot-danger'}"></span>
						<span>{wsStore.connected ? 'WebSocket Live' : 'Disconnected'}</span>
					</div>
				</div>
				<p class="page-subtitle">
					Real-time interactive terminal log stream and direct RCON server command controller
				</p>
			</div>
		</div>

		<div class="header-stats-group">
			<div class="stat-item" title="Live Server TPS">
				<Activity size={14} class="stat-icon" />
				<span class="stat-label">TPS:</span>
				<span class="stat-val">{srv.tps == null ? 'n/a' : srv.tps.toFixed(1)}</span>
			</div>
			<div class="stat-item-divider" aria-hidden="true">|</div>
			<div class="stat-item" title="Online Players">
				<Users size={14} class="stat-icon" />
				<span class="stat-label">Players:</span>
				<span class="stat-val">
					{srv.onlinePlayers ?? UNAVAILABLE}/{srv.maxPlayers ?? UNAVAILABLE}
				</span>
			</div>
			{#if !wsStore.connected}
				<button class="btn btn-secondary btn-sm reconnect-btn" onclick={reconnectWebSocket}>
					<RefreshCw size={14} />
					<span>Reconnect</span>
				</button>
			{/if}
		</div>
	</div>

	<!-- Main Console Body Section -->
	<div class="console-body">
		<!-- Live Log Output Viewer (Fills remaining height) -->
		<div class="log-viewer-container">
			<LogViewer />
		</div>

		<!-- Quick Action Chips Grid -->
		<div class="quick-commands-section">
			<QuickCommands
				onFillCommand={handleFillCommand}
				onExecuteCommand={handleExecuteCommand}
			/>
		</div>

		<!-- Interactive Command Input Bar (Pinned at bottom) -->
		<div class="command-bar-section">
			<CommandBar
				bind:commandValue={activeCommand}
				onCommandExecuted={handleExecuteCommand}
			/>
		</div>
	</div>
</div>

<style>
	.console-page-layout {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		height: calc(100vh - 60px - 48px);
		height: calc(100dvh - 60px - 48px);
		max-height: calc(100vh - 60px - 48px);
		max-height: calc(100dvh - 60px - 48px);
		min-height: 550px;
	}

	/* Header Card */
	.console-header-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: var(--card-shadow);
		flex-shrink: 0;
	}

	.header-main-info {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.icon-badge {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
	}

	.header-titles {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.title-with-status {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.page-title {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		line-height: 1.2;
	}

	.status-indicator-badge {
		font-size: var(--font-size-xs);
		padding: 2px 10px;
	}

	.page-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.header-stats-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: var(--space-2) var(--space-4);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
	}

	.stat-item {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}

	.stat-icon {
		color: var(--accent-blue-text);
	}

	.stat-label {
		color: var(--text-muted);
	}

	.stat-val {
		color: var(--text-primary);
		font-weight: var(--font-weight-semibold);
	}

	/* Was var(--border-focus) — 1.84:1 against --bg-base, i.e. invisible. Same
	   decorative "|" separator as .pill-divider in +layout.svelte, same fix:
	   --text-muted gives 5.12:1. */
	.stat-item-divider {
		color: var(--text-muted);
	}

	.reconnect-btn {
		height: 26px;
		padding: 0 var(--space-2);
		font-size: var(--font-size-xs);
	}

	/* Console Body Layout */
	.console-body {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		min-height: 0;
		overflow: hidden;
	}

	.log-viewer-container {
		flex: 1;
		min-height: 250px;
		overflow: hidden;
	}

	.quick-commands-section {
		flex-shrink: 0;
	}

	.command-bar-section {
		flex-shrink: 0;
	}

	@media (max-width: 768px) {
		.console-page-layout {
			height: auto;
			max-height: none;
		}

		.console-header-card {
			flex-direction: column;
			align-items: flex-start;
			gap: var(--space-3);
			padding: var(--space-4);
		}

		.header-stats-group {
			width: 100%;
			justify-content: center;
		}
	}
</style>
