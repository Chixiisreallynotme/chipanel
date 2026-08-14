<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { Terminal, Info, AlertTriangle, XCircle, Inbox } from 'lucide-svelte';

	// Extract the 5 most recent console log events from live wsStore
	let recentLogs = $derived.by(() => {
		if (!wsStore.consoleLogs || wsStore.consoleLogs.length === 0) return [];
		return wsStore.consoleLogs.slice(-5).reverse();
	});

	function formatTime(isoOrTimestamp) {
		if (!isoOrTimestamp) return '';
		try {
			const date = new Date(isoOrTimestamp);
			if (isNaN(date.getTime())) return '';
			return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
		} catch (e) {
			return '';
		}
	}
</script>

<div class="card activity-card">
	<div class="card-header">
		<h3 class="card-title">Événements en direct</h3>
		<span class="badge badge-secondary">Console RCON</span>
	</div>

	<div class="card-body activity-body">
		{#if recentLogs.length > 0}
			<div class="activity-feed">
				{#each recentLogs as log}
					<div class="activity-item">
						<div class="activity-icon-badge {log.level === 'warn' ? 'icon-orange' : log.level === 'error' ? 'icon-red' : 'icon-blue'}">
							{#if log.level === 'error'}
								<XCircle size={14} />
							{:else if log.level === 'warn'}
								<AlertTriangle size={14} />
							{:else}
								<Terminal size={14} />
							{/if}
						</div>
						<div class="activity-details">
							<span class="activity-title" title={log.message}>{log.message}</span>
							{#if formatTime(log.timestamp)}
								<span class="activity-time">{formatTime(log.timestamp)}</span>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<div class="empty-state">
				<Inbox size={28} class="empty-icon" />
				<span class="empty-text">Aucun événement récents</span>
				<span class="empty-sub">Les flux de la console RCON s'afficheront ici en direct.</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.activity-card {
		height: 100%;
	}

	.activity-body {
		padding: var(--space-4) var(--space-6);
	}

	.activity-feed {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.activity-item {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: 6px var(--space-2);
		border-radius: var(--radius-sm);
		transition: background-color var(--transition-fast);
	}

	.activity-item:hover {
		background-color: var(--bg-base);
	}

	.activity-icon-badge {
		width: 28px;
		height: 28px;
		border-radius: var(--radius-sm);
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

	.icon-orange {
		background-color: var(--accent-orange-bg);
		color: var(--accent-orange);
		border: 1px solid var(--accent-orange-border);
	}

	.icon-red {
		background-color: var(--danger-bg);
		color: var(--danger-text);
		border: 1px solid var(--danger-border);
	}

	.activity-details {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
	}

	.activity-title {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.activity-time {
		font-size: 0.6875rem;
		color: var(--text-muted);
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-8) var(--space-4);
		text-align: center;
		gap: var(--space-2);
		height: 100%;
	}

	.empty-icon {
		color: var(--text-muted);
		opacity: 0.6;
	}

	.empty-text {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--text-secondary);
	}

	.empty-sub {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}
</style>
