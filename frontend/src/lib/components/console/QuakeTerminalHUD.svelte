<script>
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { Terminal, Share2, X, Send, CornerDownLeft, Sparkles, Check, ArrowDown } from '$lib/icons.js';

	let { open = $bindable(false), onclose } = $props();

	let inputEl = $state(/** @type {HTMLInputElement | null} */ (null));
	let logsContainer = $state(/** @type {HTMLDivElement | null} */ (null));
	let commandInput = $state('');
	let activeFilter = $state('ALL'); // 'ALL' | 'INFO' | 'WARN' | 'ERROR' | 'RCON'
	let autoScroll = $state(true);
	let isExporting = $state(false);
	let exportSuccess = $state(false);
	let exportUrl = $state('');

	// Command history
	let commandHistory = $state(/** @type {string[]} */ ([]));
	let historyIndex = $state(-1);
	let draftInput = $state('');

	// Filtered logs slice (max 300 in DOM for 60fps GPU performance)
	let displayedLogs = $derived.by(() => {
		const raw = wsStore.consoleLogs.slice(-300);
		if (activeFilter === 'ALL') return raw;
		if (activeFilter === 'INFO') return raw.filter((l) => l.level === 'info');
		if (activeFilter === 'WARN') return raw.filter((l) => l.level === 'warn');
		if (activeFilter === 'ERROR') return raw.filter((l) => l.level === 'error');
		if (activeFilter === 'RCON') return raw.filter((l) => l.message.startsWith('>') || l.level === 'rcon');
		return raw;
	});

	let rconLive = $derived(wsStore.connected && wsStore.telemetry.rconAvailable);

	function toggleHud() {
		open = !open;
		if (open) {
			autoScroll = true;
			setTimeout(() => {
				inputEl?.focus();
				scrollToBottom();
			}, 50);
		} else {
			if (onclose) onclose();
		}
	}

	function closeHud() {
		open = false;
		if (onclose) onclose();
	}

	function scrollToBottom() {
		if (logsContainer && autoScroll) {
			logsContainer.scrollTop = logsContainer.scrollHeight;
		}
	}

	function handleScroll() {
		if (!logsContainer) return;
		const { scrollTop, scrollHeight, clientHeight } = logsContainer;
		// Auto-scroll if user is within 40px of bottom
		autoScroll = scrollHeight - (scrollTop + clientHeight) < 40;
	}

	function handleGlobalKeyDown(e) {
		const isInputActive = ['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName || '');

		// Toggle HUD on `~` (only if not inside an input) or `Ctrl+\`` or `F12`
		if ((e.key === '`' || e.key === '~') && !e.ctrlKey && !e.metaKey && !e.altKey) {
			if (!isInputActive) {
				e.preventDefault();
				toggleHud();
				return;
			}
		} else if ((e.key === '`' || e.key === '~') && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			toggleHud();
			return;
		} else if (e.key === 'F12') {
			e.preventDefault();
			toggleHud();
			return;
		}

		if (!open) return;

		// When HUD is open:
		if (e.key === 'Escape') {
			e.preventDefault();
			closeHud();
			return;
		}

		// Quick numeric filters 1-5 (only if input is not focused or empty)
		if (!isInputActive || commandInput === '') {
			if (e.key === '1') { activeFilter = 'ALL'; e.preventDefault(); }
			else if (e.key === '2') { activeFilter = 'INFO'; e.preventDefault(); }
			else if (e.key === '3') { activeFilter = 'WARN'; e.preventDefault(); }
			else if (e.key === '4') { activeFilter = 'ERROR'; e.preventDefault(); }
			else if (e.key === '5') { activeFilter = 'RCON'; e.preventDefault(); }
		}
	}

	function handleInputKeyDown(e) {
		if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (commandHistory.length === 0) return;
			if (historyIndex === -1) {
				draftInput = commandInput;
				historyIndex = commandHistory.length - 1;
			} else if (historyIndex > 0) {
				historyIndex--;
			}
			commandInput = commandHistory[historyIndex] || '';
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (historyIndex === -1) return;
			if (historyIndex < commandHistory.length - 1) {
				historyIndex++;
				commandInput = commandHistory[historyIndex];
			} else {
				historyIndex = -1;
				commandInput = draftInput;
			}
		} else if (e.key === 'Enter') {
			e.preventDefault();
			submitCommand();
		}
	}

	function submitCommand() {
		const cmd = commandInput.trim();
		if (!cmd) return;

		const cleanCmd = cmd.startsWith('/') ? cmd.slice(1) : cmd;
		wsStore.sendCommand(cleanCmd);

		if (commandHistory[commandHistory.length - 1] !== cmd) {
			commandHistory = [...commandHistory, cmd].slice(-50);
		}

		commandInput = '';
		historyIndex = -1;
		draftInput = '';
		autoScroll = true;
		setTimeout(scrollToBottom, 30);
	}

	async function exportMclogs() {
		if (isExporting || wsStore.consoleLogs.length === 0) return;
		isExporting = true;
		exportSuccess = false;

		try {
			// Gather last 500 lines and clean sensitive tokens
			const logText = wsStore.consoleLogs
				.slice(-500)
				.map((l) => `[${l.timestamp.slice(11, 19)}] [${l.level.toUpperCase()}] ${l.message}`)
				.join('\n');

			const res = await fetch('https://api.mclo.gs/1/log', {
				method: 'POST',
				headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
				body: new URLSearchParams({ content: logText })
			});

			if (res.ok) {
				const data = await res.json();
				if (data?.url) {
					exportUrl = data.url;
					exportSuccess = true;
					if (navigator.clipboard) {
						navigator.clipboard.writeText(data.url);
					}
					setTimeout(() => { exportSuccess = false; }, 4000);
				}
			}
		} catch (err) {
			console.error('Failed to export logs to mclo.gs:', err);
		} finally {
			isExporting = false;
		}
	}

	// Watch new logs and auto-scroll
	$effect(() => {
		if (open && displayedLogs.length > 0) {
			setTimeout(scrollToBottom, 20);
		}
	});

	onMount(() => {
		if (browser) {
			window.addEventListener('keydown', handleGlobalKeyDown);
			return () => window.removeEventListener('keydown', handleGlobalKeyDown);
		}
	});
</script>

<!-- Backdrop overlay when open -->
{#if open}
	<div
		class="quake-backdrop"
		onclick={closeHud}
		onkeydown={(e) => e.key === 'Escape' && closeHud()}
		role="button"
		tabindex="0"
		aria-label="Fermer le Quake HUD"
	></div>
{/if}

<!-- Quake Terminal HUD Hardware Drawer -->
<aside
	class="quake-hud-drawer {open ? 'open' : ''}"
	aria-label="Console Quake Terminal HUD"
	aria-hidden={!open}
>
	<div class="quake-hardware-shell">
		<!-- Top Telemetry & Control Bar -->
		<header class="quake-top-bar">
			<div class="quake-stamps-group">
				<span class="micro-stamp {rconLive ? 'micro-stamp-active' : 'micro-stamp-warning'} tabular-nums">
					[0x7F::RCON {rconLive ? 'LIVE' : 'IDLE'}]
				</span>
				<span class="micro-stamp tabular-nums">[PORT:25565]</span>
				<span class="micro-stamp tabular-nums">[BUF:{displayedLogs.length}L]</span>
			</div>

			<!-- 1-Touch Filter Keys -->
			<nav class="quake-filters-row" aria-label="Filtres de logs rapides">
				{#each ['ALL', 'INFO', 'WARN', 'ERROR', 'RCON'] as filter, idx}
					<button
						type="button"
						class="filter-key-badge {activeFilter === filter ? 'active' : ''} tabular-nums"
						onclick={() => (activeFilter = filter)}
						title="Touche {idx + 1} pour filtrer"
					>
						<span class="key-num">{idx + 1}:</span>
						<span>{filter}</span>
					</button>
				{/each}
			</nav>

			<!-- Actions: mclo.gs & Close -->
			<div class="quake-actions-group">
				<button
					type="button"
					class="btn btn-ghost btn-sm quake-export-btn {exportSuccess ? 'export-success' : ''}"
					onclick={exportMclogs}
					disabled={isExporting || wsStore.consoleLogs.length === 0}
					title="Exporter et anonymiser les logs sur mclo.gs"
				>
					{#if exportSuccess}
						<Check size={13} class="text-green" />
						<span class="btn-label">Copié !</span>
					{:else}
						<Share2 size={13} />
						<span class="btn-label">{isExporting ? 'Envoi…' : 'mclo.gs'}</span>
					{/if}
				</button>

				<button
					type="button"
					class="btn btn-ghost btn-icon btn-sm close-hud-btn"
					onclick={closeHud}
					title="Fermer le Quake HUD (Échap ou ~)"
					aria-label="Fermer la console"
				>
					<X size={15} />
				</button>
			</div>
		</header>

		<!-- Log Output Window -->
		<div
			class="quake-logs-stream"
			bind:this={logsContainer}
			onscroll={handleScroll}
			role="log"
			aria-live="polite"
		>
			{#if displayedLogs.length === 0}
				<div class="empty-logs-hint">
					<Terminal size={18} class="empty-icon" />
					<span>En attente de flux de logs ou RCON en veille… (~ pour masquer)</span>
				</div>
			{:else}
				{#each displayedLogs as log (log.timestamp + log.message.slice(0, 20))}
					<div class="quake-log-line log-{log.level}">
						<span class="log-ts tabular-nums">[{log.timestamp.slice(11, 19)}]</span>
						<span class="log-level-tag">[{log.level.toUpperCase()}]</span>
						<span class="log-msg">{log.message}</span>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Bottom Command Input Prompt -->
		<form class="quake-cli-prompt" onsubmit={(e) => { e.preventDefault(); submitCommand(); }}>
			<span class="cli-prompt-label tabular-nums">chiserv &gt;</span>
			<input
				bind:this={inputEl}
				type="text"
				class="cli-input"
				placeholder="Commande RCON (/help, /tps, /say, /whitelist)... [↑↓ historique]"
				bind:value={commandInput}
				onkeydown={handleInputKeyDown}
				autocomplete="off"
				spellcheck="false"
				aria-label="Commande console RCON"
			/>
			<button
				type="submit"
				class="btn btn-primary btn-sm cli-submit-btn"
				disabled={!commandInput.trim()}
				title="Exécuter la commande (Entrée)"
			>
				<CornerDownLeft size={13} />
				<span>Exécuter</span>
			</button>
		</form>
	</div>
</aside>

<style>
	.quake-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(5, 5, 8, 0.65);
		backdrop-filter: blur(4px);
		-webkit-backdrop-filter: blur(4px);
		z-index: calc(var(--z-quake-hud) - 1);
		transition: opacity var(--duration-fast) var(--ease-out);
	}

	.quake-hud-drawer {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		z-index: var(--z-quake-hud);
		transform: translateY(-100%);
		transition: transform var(--duration-snappy) var(--ease-hypr-snap);
		pointer-events: none;
		will-change: transform;
	}

	.quake-hud-drawer.open {
		transform: translateY(0);
		pointer-events: auto;
	}

	.quake-hardware-shell {
		background-color: var(--bg-base);
		border-bottom: 1px solid var(--border);
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.8),
					inset 0 -1px 0 rgba(15, 169, 104, 0.35),
					0 2px 14px rgba(15, 169, 104, 0.12);
		display: flex;
		flex-direction: column;
		height: 44vh;
		min-height: 320px;
		max-height: 560px;
	}

	/* Top Bar */
	.quake-top-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-2) var(--space-4);
		background-color: var(--bg-surface);
		border-bottom: 1px solid var(--border-subtle);
		gap: var(--space-3);
		flex-shrink: 0;
	}

	.quake-stamps-group,
	.quake-filters-row,
	.quake-actions-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.filter-key-badge {
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		color: var(--text-muted);
		border-radius: var(--radius-sm);
		padding: 2px 6px;
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		cursor: pointer;
		display: inline-flex;
		align-items: center;
		gap: 3px;
		transition: color var(--transition-fast), background-color var(--transition-fast), border-color var(--transition-fast);
	}

	.filter-key-badge .key-num {
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-semibold);
	}

	.filter-key-badge:hover {
		color: var(--text-primary);
		border-color: var(--border-focus);
	}

	.filter-key-badge.active {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border-color: var(--accent-blue-border);
		font-weight: var(--font-weight-semibold);
	}

	.quake-export-btn {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		gap: 4px;
	}

	.export-success {
		color: var(--accent-green-text) !important;
		background-color: var(--accent-green-bg) !important;
	}

	/* Logs Stream Area */
	.quake-logs-stream {
		flex: 1;
		padding: var(--space-3) var(--space-4);
		background-color: #07080c;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 2px;
		font-family: var(--font-mono);
		font-size: 0.8125rem; /* 13px */
		line-height: 1.45;
	}

	.empty-logs-hint {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		height: 100%;
		color: var(--text-muted);
		font-size: var(--font-size-sm);
	}

	.empty-icon {
		color: var(--accent-blue-text);
		opacity: 0.6;
	}

	.quake-log-line {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		word-break: break-word;
	}

	.log-ts {
		color: var(--text-muted);
		flex-shrink: 0;
		user-select: none;
	}

	.log-level-tag {
		font-size: 0.6875rem;
		font-weight: var(--font-weight-semibold);
		flex-shrink: 0;
		user-select: none;
	}

	.log-info { color: var(--text-primary); }
	.log-info .log-level-tag { color: var(--accent-blue-text); }

	.log-warn { color: var(--warning-text); }
	.log-warn .log-level-tag { color: var(--warning-text); }

	.log-error { color: var(--danger-text); }
	.log-error .log-level-tag { color: var(--danger-text); }

	.log-rcon { color: var(--accent-purple-text); }
	.log-rcon .log-level-tag { color: var(--accent-purple-text); }

	.log-msg {
		flex: 1;
	}

	/* CLI Prompt Form */
	.quake-cli-prompt {
		display: flex;
		align-items: center;
		background-color: var(--bg-surface);
		border-top: 1px solid var(--border);
		padding: var(--space-2) var(--space-4);
		gap: var(--space-2);
		flex-shrink: 0;
	}

	.cli-prompt-label {
		color: var(--accent-green-text);
		font-family: var(--font-mono);
		font-weight: var(--font-weight-bold);
		font-size: var(--font-size-sm);
		user-select: none;
	}

	.cli-input {
		flex: 1;
		background: transparent;
		border: none;
		outline: none;
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
		height: 32px;
	}

	.cli-input::placeholder {
		color: var(--text-muted);
	}

	.cli-submit-btn {
		height: 30px;
		padding: 0 var(--space-3);
		font-size: var(--font-size-xs);
		gap: 4px;
	}

	.text-green { color: var(--accent-green-text); }

	@media (max-width: 640px) {
		.quake-top-bar {
			flex-wrap: wrap;
			padding: var(--space-2);
		}
		.filter-key-badge .key-num {
			display: none;
		}
		.quake-stamps-group {
			display: none;
		}
	}
</style>
