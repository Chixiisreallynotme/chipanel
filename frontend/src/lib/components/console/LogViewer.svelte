<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { apiFetch } from '$lib/api/client.js';
	import {
		Search,
		Download,
		Trash2,
		ArrowDown,
		Filter,
		Info,
		AlertTriangle,
		AlertCircle,
		Check,
		Terminal,
		Share2,
		ExternalLink
	} from 'lucide-svelte';

	let severityFilter = $state('ALL'); // 'ALL' | 'INFO' | 'WARN' | 'ERROR'
	let searchQuery = $state('');
	let autoScroll = $state(true);
	let logContainer = $state(null);
	let isUploadingMclogs = $state(false);
	let mclogsUrl = $state('');
	let mclogsError = $state('');

	/**
	 * Parse log entry into structured object { timestamp, level, message }
	 * Handles both string logs and object logs.
	 */
	const logCache = new WeakMap();
	const primitiveLogCache = new Map();

	function parseLog(log) {
		if (!log) {
			return { timestamp: new Date().toLocaleTimeString(), level: 'INFO', message: '' };
		}

		if (typeof log === 'string') {
			// Regex for standard Minecraft log format: [12:34:56 INFO]: Message
			const match = log.match(/^\[(.*?)\]\s*\[?(INFO|WARN|WARNING|ERROR|SEVERE|FATAL)\]?[:\s]\s*(.*)$/i);
			if (match) {
				let level = match[2].toUpperCase();
				if (level === 'WARNING') level = 'WARN';
				if (level === 'SEVERE' || level === 'FATAL') level = 'ERROR';
				return {
					timestamp: match[1],
					level,
					message: match[3]
				};
			}
			return {
				timestamp: new Date().toLocaleTimeString(),
				level: 'INFO',
				message: log
			};
		}

		let rawLevel = (log.level || 'INFO').toUpperCase();
		if (rawLevel === 'WARNING') rawLevel = 'WARN';
		if (rawLevel === 'SEVERE' || rawLevel === 'FATAL') rawLevel = 'ERROR';

		return {
			timestamp: log.timestamp ? formatTimestamp(log.timestamp) : new Date().toLocaleTimeString(),
			level: rawLevel,
			message: log.message || JSON.stringify(log)
		};
	}

	function getParsedLog(log) {
		if (log && typeof log === 'object') {
			if (!logCache.has(log)) {
				logCache.set(log, parseLog(log));
			}
			return logCache.get(log);
		}
		if (typeof log === 'string') {
			if (!primitiveLogCache.has(log)) {
				primitiveLogCache.set(log, parseLog(log));
			}
			return primitiveLogCache.get(log);
		}
		return parseLog(log);
	}

	function formatTimestamp(ts) {
		try {
			const d = new Date(ts);
			if (isNaN(d.getTime())) return String(ts);
			const h = String(d.getHours()).padStart(2, '0');
			const m = String(d.getMinutes()).padStart(2, '0');
			const s = String(d.getSeconds()).padStart(2, '0');
			return `${h}:${m}:${s}`;
		} catch {
			return String(ts);
		}
	}

	let parsedLogs = $derived.by(() => {
		return wsStore.consoleLogs.map((item, index) => ({
			id: index,
			raw: item,
			parsed: getParsedLog(item)
		}));
	});

	let filteredLogs = $derived.by(() => {
		const q = searchQuery.trim().toLowerCase();
		return parsedLogs.filter(({ parsed }) => {
			// Severity level filter
			if (severityFilter !== 'ALL') {
				if (severityFilter === 'INFO' && parsed.level !== 'INFO') return false;
				if (severityFilter === 'WARN' && parsed.level !== 'WARN') return false;
				if (severityFilter === 'ERROR' && parsed.level !== 'ERROR') return false;
			}
			// Search query text filter
			if (q !== '') {
				const msg = parsed.message.toLowerCase();
				const lvl = parsed.level.toLowerCase();
				const ts = parsed.timestamp.toLowerCase();
				return msg.includes(q) || lvl.includes(q) || ts.includes(q);
			}
			return true;
		});
	});

	let visibleLogs = $derived(filteredLogs.slice(-2000));

	// Calculated severity counts for filter badges
	let counts = $derived.by(() => {
		let info = 0,
			warn = 0,
			error = 0;
		for (const { parsed } of parsedLogs) {
			if (parsed.level === 'WARN') warn++;
			else if (parsed.level === 'ERROR') error++;
			else info++;
		}
		return { all: parsedLogs.length, info, warn, error };
	});

	// Auto-scroll effect on log stream updates
	$effect(() => {
		const count = filteredLogs.length;
		if (autoScroll && logContainer && count > 0) {
			requestAnimationFrame(() => {
				if (logContainer) {
					logContainer.scrollTop = logContainer.scrollHeight;
				}
			});
		}
	});

	function handleScroll() {
		if (!logContainer) return;
		// Detect if user scrolled up away from bottom
		const isAtBottom =
			logContainer.scrollHeight - logContainer.scrollTop - logContainer.clientHeight < 35;
		if (!isAtBottom && autoScroll) {
			autoScroll = false;
		}
	}

	function scrollToBottom() {
		autoScroll = true;
		if (logContainer) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	}

	async function shareToMclogs() {
		if (filteredLogs.length === 0 || isUploadingMclogs) return;
		isUploadingMclogs = true;
		mclogsUrl = '';
		mclogsError = '';

		try {
			const textContent = filteredLogs
				.map(({ parsed }) => `[${parsed.timestamp}] [${parsed.level}] ${parsed.message}`)
				.join('\n');

			const res = await apiFetch('/api/logs/mclogs', {
				method: 'POST',
				body: { content: textContent }
			});

			const data = await res.json();
			if (data.success && data.url) {
				mclogsUrl = data.url;
				window.open(data.url, '_blank');
			} else {
				mclogsError = data.error || 'Échec de la publication sur mclo.gs';
			}
		} catch (err) {
			mclogsError = err.message || 'Erreur réseau lors de la publication sur mclo.gs';
		} finally {
			isUploadingMclogs = false;
		}
	}

	function exportLogs() {
		if (filteredLogs.length === 0) return;
		const textContent = filteredLogs
			.map(({ parsed }) => `[${parsed.timestamp}] [${parsed.level}] ${parsed.message}`)
			.join('\n');

		const blob = new Blob([textContent], { type: 'text/plain;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'chipanel-console-logs.txt';
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		setTimeout(() => URL.revokeObjectURL(url), 1000);
	}

	function clearLogs() {
		wsStore.consoleLogs = [];
		primitiveLogCache.clear();
	}
</script>

<div class="log-viewer-card card">
	<!-- Log Viewer Header Controls Bar -->
	<div class="log-viewer-toolbar">
		<div class="toolbar-left">
			<div class="search-input-wrapper">
				<Search size={14} class="search-icon" />
				<input
					type="text"
					class="input input-sm search-input"
					placeholder="Search console logs..."
					bind:value={searchQuery}
					aria-label="Filter console logs"
				/>
				{#if searchQuery}
					<button
						class="btn-clear-search"
						onclick={() => (searchQuery = '')}
						title="Clear search filter"
						aria-label="Clear search filter"
					>
						&times;
					</button>
				{/if}
			</div>

			<!-- Severity Filter Chips -->
			<div class="severity-filters" role="group" aria-label="Severity log filters">
				<button
					class="filter-btn {severityFilter === 'ALL' ? 'active' : ''}"
					onclick={() => (severityFilter = 'ALL')}
					aria-pressed={severityFilter === 'ALL'}
				>
					<span>ALL</span>
					<span class="count-badge">{counts.all}</span>
				</button>
				<button
					class="filter-btn filter-info {severityFilter === 'INFO' ? 'active' : ''}"
					onclick={() => (severityFilter = 'INFO')}
					aria-pressed={severityFilter === 'INFO'}
				>
					<Info size={12} />
					<span>INFO</span>
					<span class="count-badge">{counts.info}</span>
				</button>
				<button
					class="filter-btn filter-warn {severityFilter === 'WARN' ? 'active' : ''}"
					onclick={() => (severityFilter = 'WARN')}
					aria-pressed={severityFilter === 'WARN'}
				>
					<AlertTriangle size={12} />
					<span>WARN</span>
					<span class="count-badge">{counts.warn}</span>
				</button>
				<button
					class="filter-btn filter-error {severityFilter === 'ERROR' ? 'active' : ''}"
					onclick={() => (severityFilter = 'ERROR')}
					aria-pressed={severityFilter === 'ERROR'}
				>
					<AlertCircle size={12} />
					<span>ERROR</span>
					<span class="count-badge">{counts.error}</span>
				</button>
			</div>
		</div>

		<div class="toolbar-right">
			<!-- Auto-scroll Pin Checkbox -->
			<label class="checkbox-group auto-scroll-label" title="Pin viewport to bottom on incoming logs">
				<input type="checkbox" class="checkbox" bind:checked={autoScroll} />
				<span class="checkbox-text">Auto-scroll</span>
			</label>

			<!-- Share to mclo.gs Button -->
			<button
				class="btn btn-secondary btn-sm"
				onclick={shareToMclogs}
				disabled={filteredLogs.length === 0 || isUploadingMclogs}
				title="Partager et anonymiser automatiquement les logs sur mclo.gs"
			>
				<Share2 size={14} />
				<span>{isUploadingMclogs ? 'Upload en cours...' : 'mclo.gs'}</span>
			</button>

			<!-- Log Export Button -->
			<button
				class="btn btn-secondary btn-sm"
				onclick={exportLogs}
				disabled={filteredLogs.length === 0}
				title="Download filtered console logs as .txt"
			>
				<Download size={14} />
				<span>Export .txt</span>
			</button>

			<!-- Clear Logs Button -->
			<button
				class="btn btn-ghost btn-sm btn-icon"
				onclick={clearLogs}
				disabled={wsStore.consoleLogs.length === 0}
				title="Clear output view"
				aria-label="Clear console log stream"
			>
				<Trash2 size={14} />
			</button>
		</div>
	</div>

	{#if mclogsUrl}
		<div class="mclogs-alert">
			<div class="mclogs-alert-content">
				<Check size={16} class="text-success" />
				<span>Logs publiés avec succès sur mclo.gs (IPs et tokens masqués) :</span>
				<a href={mclogsUrl} target="_blank" rel="noopener noreferrer" class="mclogs-link font-mono">
					{mclogsUrl}
					<ExternalLink size={12} />
				</a>
			</div>
			<button class="btn btn-ghost btn-sm" onclick={() => (mclogsUrl = '')}>Fermer</button>
		</div>
	{/if}
	{#if mclogsError}
		<div class="mclogs-alert alert-error">
			<AlertCircle size={16} class="text-danger" />
			<span>{mclogsError}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (mclogsError = '')}>Fermer</button>
		</div>
	{/if}

	<!-- Monospace Terminal Stream Output Box -->
	<div class="console-wrapper">
		<div
			class="console-output"
			bind:this={logContainer}
			onscroll={handleScroll}
			role="log"
			aria-live="polite"
			aria-label="Server console output log"
		>
			{#if filteredLogs.length === 0}
				<div class="empty-logs">
					{#if wsStore.consoleLogs.length === 0}
						<Terminal size={24} class="empty-icon" />
						<p>Waiting for server console logs...</p>
						<span class="empty-subtext">Logs streaming live via WebSocket connection</span>
					{:else}
						<Filter size={24} class="empty-icon" />
						<p>No log messages match active filters.</p>
						<span class="empty-subtext">Try clearing your search query or severity filter</span>
					{/if}
				</div>
			{:else}
				{#each visibleLogs as { id, parsed } (id)}
					<div class="log-line log-level-{parsed.level.toLowerCase()}">
						<span class="log-timestamp">[{parsed.timestamp}]</span>
						<span class="log-badge badge-level-{parsed.level.toLowerCase()}">{parsed.level}</span>
						<span class="log-message">{parsed.message}</span>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Scroll to Bottom Floating Button -->
		{#if !autoScroll && filteredLogs.length > 0}
			<button class="btn btn-primary btn-sm scroll-bottom-btn" onclick={scrollToBottom}>
				<ArrowDown size={14} />
				<span>Scroll to bottom</span>
			</button>
		{/if}
	</div>
</div>

<style>
	.log-viewer-card {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 380px;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		overflow: hidden;
	}

	.log-viewer-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-surface);
		border-bottom: 1px solid var(--border);
		flex-wrap: wrap;
	}

	.toolbar-left,
	.toolbar-right {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	/* Search input field */
	.search-input-wrapper {
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
		padding-left: 32px;
		padding-right: 28px;
		width: 220px;
		font-size: var(--font-size-xs);
		background-color: var(--bg-base);
	}

	.btn-clear-search {
		position: absolute;
		right: 8px;
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 14px;
		line-height: 1;
		padding: 2px 4px;
	}

	.btn-clear-search:hover {
		color: var(--text-primary);
	}

	/* Severity Filters */
	.severity-filters {
		display: flex;
		align-items: center;
		gap: 2px;
		background-color: var(--bg-base);
		padding: 2px;
		border-radius: var(--radius-btn);
		border: 1px solid var(--border-subtle);
	}

	.filter-btn {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		height: 26px;
		padding: 0 var(--space-2);
		border-radius: var(--radius-input);
		border: none;
		background: transparent;
		color: var(--text-muted);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast);
	}

	.filter-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.05);
	}

	.filter-btn.active {
		background-color: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	.filter-info.active {
		color: var(--accent-blue-text);
	}

	.filter-warn.active {
		color: var(--accent-orange-text);
	}

	.filter-error.active {
		color: var(--danger-text);
	}

	.count-badge {
		font-size: 10px;
		padding: 0 5px;
		border-radius: var(--radius-badge);
		background-color: rgba(255, 255, 255, 0.08);
		color: var(--text-secondary);
	}

	.auto-scroll-label {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		user-select: none;
	}

	.checkbox-text {
		font-size: var(--font-size-xs);
	}

	.action-divider {
		width: 1px;
		height: 18px;
		background-color: var(--border);
	}

	/* Terminal Output Area */
	.console-wrapper {
		position: relative;
		flex: 1;
		display: flex;
		flex-direction: column;
		min-height: 0;
		background-color: #0c0e14;
		box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.4);
	}

	.console-output {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-3) var(--space-4);
		font-family: var(--font-mono);
		font-size: 13px;
		line-height: 1.55;
		color: var(--text-primary);
		word-break: break-word;
	}

	.empty-logs {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		color: var(--text-muted);
		text-align: center;
		padding: var(--space-12) 0;
	}

	.empty-icon {
		color: var(--border-focus);
	}

	.empty-subtext {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.log-line {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		padding: 2px 0;
		border-radius: 2px;
	}

	.log-line:hover {
		background-color: rgba(255, 255, 255, 0.03);
	}

	.log-timestamp {
		color: var(--text-muted);
		font-size: var(--font-size-xs);
		user-select: none;
		flex-shrink: 0;
	}

	.log-badge {
		font-size: 10px;
		font-weight: var(--font-weight-bold);
		padding: 1px 5px;
		border-radius: 3px;
		text-transform: uppercase;
		flex-shrink: 0;
		line-height: 1.3;
		margin-top: 1px;
	}

	.badge-level-info {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border: 1px solid var(--accent-blue-border);
	}

	.badge-level-warn {
		background-color: var(--accent-orange-bg);
		color: var(--accent-orange-text);
		border: 1px solid var(--accent-orange-border);
	}

	.badge-level-error {
		background-color: var(--danger-bg);
		color: var(--danger-text);
		border: 1px solid var(--danger-border);
	}

	.log-level-warn .log-message {
		color: var(--accent-orange-text);
	}

	.log-level-error .log-message {
		color: var(--danger-text);
	}

	.log-message {
		flex: 1;
		white-space: pre-wrap;
	}

	/* Scroll to bottom button */
	.scroll-bottom-btn {
		position: absolute;
		bottom: 16px;
		right: 20px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
		z-index: 10;
		animation: fadeIn var(--transition-fast) ease-out;
	}

	.mclogs-alert {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-2) var(--space-4);
		background-color: var(--accent-green-bg);
		border-bottom: 1px solid var(--accent-green-border);
		color: var(--accent-green-text);
		font-size: var(--font-size-sm);
	}

	.mclogs-alert.alert-error {
		background-color: var(--danger-bg);
		border-bottom: 1px solid var(--danger-border);
		color: var(--danger-text);
	}

	.mclogs-alert-content {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-wrap: wrap;
	}

	.mclogs-link {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--accent-blue-text);
		text-decoration: underline;
		font-weight: var(--font-weight-medium);
	}
</style>
