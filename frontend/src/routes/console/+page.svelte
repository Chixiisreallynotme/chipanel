<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { serverState, UNAVAILABLE } from '$lib/components/dashboard/serverState.js';
	import LogViewer from '$lib/components/console/LogViewer.svelte';
	import CommandBar from '$lib/components/console/CommandBar.svelte';
	import QuickCommands from '$lib/components/console/QuickCommands.svelte';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
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
	<!-- Page Header -->
	<PageHeader
		title="Console Serveur"
		subtitle="Flux interactif de logs en temps réel et contrôleur RCON direct"
	>
		{#snippet icon()}
			<Terminal size={20} />
		{/snippet}
		{#snippet badge()}
			<div class="status-indicator-badge badge {wsStore.connected ? 'badge-success' : 'badge-danger'}">
				<span class="status-dot {wsStore.connected ? 'status-dot-success status-dot-pulse' : 'status-dot-danger'}"></span>
				<span class="status-badge-text">{wsStore.connected ? 'WebSocket Direct' : 'Déconnecté'}</span>
			</div>
		{/snippet}
		<div class="header-stats-group">
			<div class="stat-item" title="TPS serveur en direct">
				<Activity size={14} class="stat-icon" />
				<span class="stat-label">TPS :</span>
				<span class="stat-val tabular-nums">{srv.tps == null ? 'n/a' : srv.tps.toFixed(1)}</span>
			</div>
			<div class="stat-item-divider" aria-hidden="true">|</div>
			<div class="stat-item" title="Joueurs en ligne">
				<Users size={14} class="stat-icon" />
				<span class="stat-label">Joueurs :</span>
				<span class="stat-val tabular-nums">
					{srv.onlinePlayers ?? UNAVAILABLE}/{srv.maxPlayers ?? UNAVAILABLE}
				</span>
			</div>
			{#if !wsStore.connected}
				<button class="btn btn-secondary btn-sm reconnect-btn" onclick={reconnectWebSocket}>
					<RefreshCw size={13} />
					<span>Reconnecter</span>
				</button>
			{/if}
		</div>
	</PageHeader>

	<!-- Main Console Body Section -->
	<div class="console-body">
		<!-- Live Log Output Viewer in Machined Obsidian Chassis (#080a0f) -->
		<div class="log-viewer-container obsidian-chassis hardware-shell">
			<div class="obsidian-core hardware-core">
				<LogViewer />
			</div>
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

	.status-indicator-badge {
		font-size: 11px;
		padding: 2px 9px;
		border-radius: var(--radius-badge);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.status-badge-text {
		font-weight: var(--font-weight-medium);
	}

	.header-stats-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-badge);
		padding: var(--space-2) var(--space-4);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
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
		font-variant-numeric: tabular-nums;
	}

	.stat-item-divider {
		color: var(--text-muted);
		opacity: 0.6;
	}

	.reconnect-btn {
		height: 24px;
		padding: 0 var(--space-2);
		font-size: 11px;
		border-radius: var(--radius-sm);
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

	/* Machined Obsidian Chassis (#080a0f) */
	.obsidian-chassis {
		flex: 1;
		min-height: 250px;
		overflow: hidden;
		background-color: rgba(255, 255, 255, 0.02);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		padding: 4px;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.35);
		display: flex;
		flex-direction: column;
	}

	.obsidian-core {
		flex: 1;
		background-color: #080a0f;
		border: 1px solid rgba(255, 255, 255, 0.04);
		border-radius: calc(var(--radius-card) - 4px);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.09), 0 2px 8px rgba(0, 0, 0, 0.25);
		padding: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		min-height: 0;
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

		.header-stats-group {
			width: 100%;
			justify-content: center;
		}
	}
</style>
