<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { apiPost } from '$lib/api/client.js';
	import { Send, Terminal, CheckCircle2, AlertTriangle, Trash2 } from 'lucide-svelte';

	let commandInput = $state('');
	let isSending = $state(false);
	let commandHistory = $state([
		{
			id: 1,
			command: 'list',
			response: 'There are 3 of a max of 20 players online: Steve, Alex, Notch',
			status: 'success',
			time: '15:00:00'
		}
	]);

	function formatTime(date = new Date()) {
		const h = String(date.getHours()).padStart(2, '0');
		const m = String(date.getMinutes()).padStart(2, '0');
		const s = String(date.getSeconds()).padStart(2, '0');
		return `${h}:${m}:${s}`;
	}

	async function executeCommand() {
		const cmd = commandInput.trim();
		if (!cmd || isSending) return;

		isSending = true;
		const currentTime = formatTime();

		try {
			// Try sending via WebSocket first for real-time streaming
			const sentViaWs = wsStore.sendCommand(cmd);
			let responseMsg = '';

			if (sentViaWs) {
				responseMsg = `Command sent via WebSocket connection.`;
			} else {
				// Fallback to API REST execution
				const res = await apiPost('/api/rcon/execute', { command: cmd });
				responseMsg = res.output || res.message || 'Command executed successfully.';
			}

			commandHistory = [
				...commandHistory,
				{
					id: Date.now(),
					command: cmd,
					response: responseMsg,
					status: 'success',
					time: currentTime
				}
			].slice(-50);

			commandInput = '';
		} catch (err) {
			console.error('RCON Command execution error:', err);
			commandHistory = [
				...commandHistory,
				{
					id: Date.now(),
					command: cmd,
					response: err.message || 'Failed to execute RCON command.',
					status: 'error',
					time: currentTime
				}
			].slice(-50);
		} finally {
			isSending = false;
		}
	}

	function handleKeyDown(event) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			executeCommand();
		}
	}

	function clearHistory() {
		commandHistory = [];
	}
</script>

<div class="card quick-rcon-card">
	<div class="card-header quick-rcon-header">
		<div class="header-title-group">
			<Terminal size={18} class="terminal-icon" />
			<h3 class="card-title">Quick RCON Console Bar</h3>
		</div>
		{#if commandHistory.length > 0}
			<button class="btn btn-ghost btn-sm" onclick={clearHistory} title="Clear RCON Output History">
				<Trash2 size={14} />
				<span>Clear Log</span>
			</button>
		{/if}
	</div>

	<div class="card-body quick-rcon-body">
		<!-- Output Feedback Container -->
		<div
			class="rcon-output-container"
			role="log"
			aria-live="polite"
			aria-label="RCON console log"
		>
			{#if commandHistory.length === 0}
				<div class="rcon-empty">
					<span class="prompt-symbol">&gt;</span>
					<span>RCON console ready. Enter a Minecraft server command below...</span>
				</div>
			{:else}
				{#each commandHistory as item (item.id)}
					<div class="rcon-log-item {item.status === 'error' ? 'log-error' : 'log-success'}">
						<div class="command-line">
							<span class="log-time">[{item.time}]</span>
							<span class="prompt-symbol">&gt;</span>
							<code class="cmd-text">{item.command}</code>
						</div>
						<div class="response-line">
							{#if item.status === 'error'}
								<AlertTriangle size={14} class="status-icon error-icon" />
							{:else}
								<CheckCircle2 size={14} class="status-icon success-icon" />
							{/if}
							<pre class="response-text">{item.response}</pre>
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Input Action Bar -->
		<div class="rcon-input-group">
			<div class="input-prefix">
				<span class="prompt-symbol">&gt;</span>
			</div>
			<input
				type="text"
				class="input rcon-input"
				placeholder="Enter RCON command... e.g. say Hello, time set day"
				aria-label="RCON command input"
				bind:value={commandInput}
				onkeydown={handleKeyDown}
				disabled={isSending}
			/>
			<button
				class="btn btn-primary send-btn {isSending ? 'btn-loading' : ''}"
				onclick={executeCommand}
				disabled={!commandInput.trim() || isSending}
			>
				{#if !isSending}
					<Send size={16} />
				{/if}
				<span>Send</span>
			</button>
		</div>
	</div>
</div>

<style>
	.quick-rcon-card {
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		background-color: var(--bg-surface);
	}

	.quick-rcon-header {
		padding: var(--space-3) var(--space-6);
	}

	.header-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.terminal-icon {
		color: var(--accent-blue-text);
	}

	.quick-rcon-body {
		padding: var(--space-4) var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.rcon-output-container {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		padding: var(--space-3) var(--space-4);
		max-height: 180px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
	}

	.rcon-empty {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-muted);
		font-style: italic;
	}

	.prompt-symbol {
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-semibold);
		user-select: none;
	}

	.rcon-log-item {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		padding-bottom: var(--space-2);
		border-bottom: 1px solid var(--border-subtle);
	}

	.rcon-log-item:last-child {
		border-bottom: none;
		padding-bottom: 0;
	}

	.command-line {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.log-time {
		color: var(--text-muted);
		font-size: var(--font-size-xs);
		user-select: none;
	}

	.cmd-text {
		color: var(--text-primary);
		font-weight: var(--font-weight-semibold);
	}

	.response-line {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		padding-left: var(--space-4);
	}

	.status-icon {
		flex-shrink: 0;
		margin-top: 2px;
	}

	.success-icon {
		color: var(--accent-green);
	}

	.error-icon {
		color: var(--danger-text);
	}

	.response-text {
		color: var(--text-secondary);
		white-space: pre-wrap;
		word-break: break-all;
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		line-height: 1.4;
	}

	.rcon-input-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.input-prefix {
		height: 38px;
		padding: 0 var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-right: none;
		border-radius: var(--radius-input) 0 0 var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.rcon-input {
		flex: 1;
		font-family: var(--font-mono);
		border-radius: 0 var(--radius-input) var(--radius-input) 0;
	}

	.rcon-input:focus {
		z-index: 1;
	}

	.send-btn {
		flex-shrink: 0;
	}
</style>
