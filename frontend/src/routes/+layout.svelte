<script>
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { browser } from '$app/environment';
	import { apiGet } from '$lib/api/client.js';
	import { auth } from '$lib/stores/auth.svelte.js';
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import {
		LayoutDashboard,
		Terminal,
		Package,
		Boxes,
		Users,
		ShieldCheck,
		Globe,
		FileCode,
		Activity,
		LogOut,
		Menu,
		X,
		Sliders,
		User as UserIcon,
		Settings,
		ChevronDown,
		HardDrive,
		Database,
		Network,
		Archive,
		ShieldAlert,
		Webhook,
		CalendarClock,
		KeyRound,
		Layers,
		Sparkles,
		Zap
	} from '$lib/icons.js';
	import ChiPanelLogo from '$lib/components/common/ChiPanelLogo.svelte';
	import ModeSwitch from '$lib/components/onboarding/ModeSwitch.svelte';
	import QuakeTerminalHUD from '$lib/components/console/QuakeTerminalHUD.svelte';

	let { children } = $props();
	let sidebarOpen = $state(false);
	let quakeOpen = $state(false);

	const navSections = [
		{
			title: 'GESTION',
			items: [
				{ label: "Vue d'ensemble", path: '/', icon: LayoutDashboard },
				{ label: 'Assistant 1-Click', path: '/setup', icon: Sparkles },
				{ label: 'Console', path: '/console', icon: Terminal },
				{ label: 'Joueurs', path: '/players', icon: Users },
				{ label: 'Fichiers', path: '/files', icon: FileCode },
				{ label: 'Addons', path: '/addons', icon: Package },
				{ label: 'Monde', path: '/worlds', icon: Globe },
				{ label: 'Sauvegardes & Snapshots', path: '/backups', icon: Archive },
				{ label: 'Version & Moteur', path: '/engine', icon: Sliders }
			]
		},
		{
			title: 'CONFIGURATION',
			items: [
				{ label: 'Diagnostics & Auto-Tuner', path: '/diagnostics', icon: Zap },
				{ label: 'Planificateur & Télémétrie', path: '/metrics', icon: CalendarClock },
				{ label: 'Base de Données & Purge', path: '/database', icon: Database },
				{ label: 'Cross-Play & Réseau', path: '/network', icon: Network },
				{ label: 'Registre d\'Audit', path: '/audit', icon: ShieldAlert },
				{ label: 'Permissions & Rôles', path: '/permissions', icon: ShieldCheck },
				{ label: 'Gestion des Comptes', path: '/accounts', icon: Users }
			]
		}
	];

	// Route the user asked for before being bounced to /login, so the session round-trip
	// comes back where it started instead of dumping everyone on '/'.
	/** @type {string | null} */
	let redirectAfterLogin = null;

	// Initialize auth state on mount
	onMount(() => {
		auth.init();
	});

	// Manage WebSocket lifecycle based on auth state
	$effect(() => {
		if (!browser || auth.isInitializing) return;

		if (auth.isAuthenticated) {
			wsStore.connect(auth.token);
		} else {
			wsStore.disconnect();
		}
		return () => {
			wsStore.disconnect();
		};
	});

	// Authentication route guard. Held until auth.init() has settled: running it against a
	// half-restored session caused a /login flash and a burst of 401s on every page load.
	$effect(() => {
		if (!browser || auth.isInitializing) return;
		const currentPath = page.url.pathname;

		if (!auth.isAuthenticated) {
			if (currentPath !== '/login') {
				redirectAfterLogin = currentPath + page.url.search;
				goto('/login');
			}
		} else if (currentPath === '/login') {
			const target = redirectAfterLogin ?? '/';
			redirectAfterLogin = null;
			goto(target);
		}
	});

	/** @param {string} targetPath */
	function isPathActive(targetPath) {
		const current = page.url.pathname;
		if (targetPath === '/' || targetPath === '/dashboard') {
			return current === '/' || current === '/dashboard';
		}
		return current.startsWith(targetPath);
	}

	function handleLogout() {
		auth.logout();
		wsStore.disconnect();
		goto('/login');
	}

	function toggleSidebar() {
		sidebarOpen = !sidebarOpen;
	}

	function closeSidebar() {
		sidebarOpen = false;
	}

	const UNAVAILABLE = '—';

	/**
	 * Telemetry values are `number | null`; `null` means the source did not answer and must
	 * read as unavailable. No numeric fallback — a fabricated value looks like a healthy server.
	 * @param {number | null} value
	 * @param {(n: number) => string} format
	 */
	function show(value, format) {
		return typeof value === 'number' ? format(value) : UNAVAILABLE;
	}

	// Server Power Mode Tracking (live from /api/server/status and WebSocket)
	let powerMode = $state(/** @type {'on' | 'hibernating' | 'off' | null} */ (null));

	async function fetchPowerMode() {
		if (!browser || !auth.isAuthenticated) return;
		try {
			const res = await apiGet('/api/server/status');
			if (res?.power_mode) {
				powerMode = res.power_mode === 'hibernate' ? 'hibernating' : res.power_mode;
			}
		} catch {
			// silently handled fallback
		}
	}

	$effect(() => {
		if (browser && auth.isAuthenticated) {
			fetchPowerMode();
		}
	});

	// Derive power_mode from state or live telemetry
	let effectivePowerMode = $derived.by(() => {
		if (powerMode) return powerMode;
		if (!wsStore.connected) return 'off';
		if (wsStore.telemetry.containerRunning === true) return 'on';
		if (wsStore.telemetry.podmanAvailable && wsStore.telemetry.containerRunning === false) return 'hibernating';
		return 'off';
	});

	let logoStatus = $derived.by(() => {
		if (effectivePowerMode === 'on') return 'active';
		if (effectivePowerMode === 'hibernating' || effectivePowerMode === 'hibernate') return 'hibernating';
		return 'stopped';
	});

	// Server Status Helper
	let serverStatus = $derived.by(() => {
		const danger = { dotClass: 'status-dot-danger', textClass: 'badge-danger' };
		const warning = { dotClass: 'status-dot-warning status-dot-pulse', textClass: 'badge-warning' };
		const success = { dotClass: 'status-dot-success status-dot-pulse', textClass: 'badge-success' };

		if (wsStore.status === 'connecting' || wsStore.status === 'reconnecting') {
			return { label: 'Connexion…', ...warning };
		}

		if (!wsStore.connected) {
			return { label: 'Hors ligne', ...danger };
		}

		if (effectivePowerMode === 'hibernating' || effectivePowerMode === 'hibernate') {
			return { label: 'En veille lazymc', ...warning };
		}

		const { containerRunning, rconAvailable, tps } = wsStore.telemetry;

		if (containerRunning === false) {
			return { label: 'Arrêté', ...danger };
		}
		if (!rconAvailable) {
			return { label: 'En veille / RCON indisponible', ...warning };
		}
		if (tps === null) {
			// RCON is up but the server has no `tps` command (vanilla) — not an error, not 20.0.
			return { label: 'En ligne', ...success };
		}
		if (tps >= 18) {
			return { label: 'En ligne', ...success };
		}
		if (tps > 0) {
			return { label: `Lenteur (${tps.toFixed(1)} TPS)`, ...warning };
		}
		return { label: 'Arrêté', ...danger };
	});

	let isLoginPage = $derived(page.url.pathname === '/login');
</script>

{#if auth.isInitializing}
	<!-- Nothing route-specific renders until the stored session has been validated: mounting
	     the real pages first fired their data loads with a not-yet-restored token (3 × 401)
	     and flashed /login on top of a perfectly valid session. -->
	<div class="boot-screen" role="status" aria-live="polite">
		<span class="boot-label">Restauration de la session…</span>
	</div>
{:else if isLoginPage}
	{@render children()}
{:else}
	<!-- Quake Terminal HUD Global Component -->
	<QuakeTerminalHUD bind:open={quakeOpen} />

	<div class="app-layout">
		<!-- Mobile Sidebar Backdrop Overlay -->
		{#if sidebarOpen}
			<div
				class="sidebar-backdrop"
				onclick={closeSidebar}
				onkeydown={(e) => e.key === 'Escape' && closeSidebar()}
				role="button"
				tabindex="0"
				aria-label="Close Sidebar"
			></div>
		{/if}

		<!-- Sidebar Navigation Shell -->
		<aside class="sidebar {sidebarOpen ? 'open' : ''}">
			<div class="sidebar-header">
				<div class="brand-logo-container">
					<ChiPanelLogo status={logoStatus} size={32} />
					<div class="brand-title-group">
						<span class="brand-title">ChiPanel</span>
						<span class="online-status-badge tabular-nums">
							<span class="status-dot {serverStatus.dotClass}"></span>
							<span>{serverStatus.label}</span>
						</span>
					</div>
				</div>
				<button class="btn btn-ghost btn-icon mobile-close-btn" onclick={closeSidebar} aria-label="Close sidebar">
					<X size={18} />
				</button>
			</div>

			<div class="sidebar-scrollable">
				<nav class="sidebar-nav">
					{#each navSections as section}
						<div class="nav-section">
							<div class="nav-section-title">{section.title}</div>
							{#each section.items as item}
								{@const IconComponent = item.icon}
								<a
									href={item.path}
									class="nav-item {isPathActive(item.path) ? 'active' : ''}"
									onclick={closeSidebar}
								>
									<IconComponent size={18} class="nav-icon" />
									<span>{item.label}</span>
								</a>
							{/each}
						</div>
					{/each}
				</nav>
			</div>

			<div class="sidebar-footer">
				<div class="user-profile-box">
					<div class="user-profile-top">
						<div class="user-avatar-badge">
							<UserIcon size={18} />
						</div>
						<div class="user-meta">
							<span class="user-name">{auth.user?.username || UNAVAILABLE}</span>
							<span class="user-role">{auth.user?.role || UNAVAILABLE}</span>
						</div>
						<ChevronDown size={16} class="user-dropdown-icon" />
					</div>
					<div class="user-actions-row">
						<button class="action-btn" title="Paramètres">
							<Settings size={16} />
						</button>
						<!-- The "Mode sombre (Actif)" toggle used to sit here. It changed nothing: there is
						     no light palette in app.css and no [data-theme] hook anywhere. Removed rather
						     than faked. -->
						<button class="action-btn action-logout" onclick={handleLogout} title="Déconnexion">
							<LogOut size={16} />
						</button>
					</div>
				</div>
			</div>
		</aside>

		<!-- Main Content Wrapper -->
		<div class="content-area">
			<!-- Header Bar -->
			<header class="header-bar">
				<div class="header-left">
					<button class="btn btn-ghost btn-icon mobile-toggle-btn" onclick={toggleSidebar} aria-label="Open navigation menu">
						<Menu size={20} />
					</button>

					<!-- Server Status Indicator -->
					<div class="status-badge badge {serverStatus.textClass} tabular-nums">
						<span class="status-dot {serverStatus.dotClass}"></span>
						<span class="status-label">{serverStatus.label}</span>
					</div>
				</div>

				<div class="header-right">
					<!-- Quake HUD Trigger Button -->
					<button
						type="button"
						class="btn btn-ghost btn-sm quake-trigger-btn font-mono"
						onclick={() => (quakeOpen = !quakeOpen)}
						title="Ouvrir la console Quake HUD (Raccourci : ~ ou F12)"
					>
						<Terminal size={14} class="text-blue" />
						<span class="quake-btn-text">Console</span>
						<kbd class="quake-kbd">~</kbd>
					</button>

					<ModeSwitch compact={true} />

					<!-- Quick Telemetry Status Pill -->
					<div class="quick-status-pill tabular-nums">
						<div class="pill-item" title="Utilisation CPU">
							<span class="pill-label">CPU:</span>
							<span class="pill-value">{show(wsStore.telemetry.cpu, (v) => `${v.toFixed(0)}%`)}</span>
						</div>
						<span class="pill-divider" aria-hidden="true">|</span>
						<div class="pill-item" title="RAM Utilisée">
							<span class="pill-label">RAM:</span>
							<span class="pill-value">
								{show(wsStore.telemetry.ram, (v) => `${(v / 1024).toFixed(1)} Go`)}
							</span>
						</div>
						<span class="pill-divider" aria-hidden="true">|</span>
						<div class="pill-item" title="Joueurs connectés">
							<span class="pill-label">Joueurs:</span>
							<span class="pill-value">
								{show(wsStore.telemetry.players.online, (v) => String(v))}/{show(
									wsStore.telemetry.players.max,
									(v) => String(v)
								)}
							</span>
						</div>
						{#if wsStore.status === 'connecting' || wsStore.status === 'reconnecting'}
							<span class="pill-divider" aria-hidden="true">|</span>
							<span class="pill-hint" title="Connexion au flux télémétrique">
								Connexion…
							</span>
						{:else if wsStore.connected && !wsStore.telemetry.rconAvailable}
							<span class="pill-divider" aria-hidden="true">|</span>
							<span class="pill-hint" title="Les données de jeu proviennent de RCON">
								RCON indisponible
							</span>
						{/if}
					</div>

					<!-- Quick User Action -->
					<div class="header-user-profile">
						<div class="header-user-avatar" title={`Connecté en tant que ${auth.user?.username || 'Utilisateur'}`}>
							<UserIcon size={15} />
						</div>
						<button class="btn btn-ghost btn-icon btn-sm" onclick={handleLogout} title="Déconnexion">
							<LogOut size={15} />
						</button>
					</div>
				</div>
			</header>

			<!-- Page Body Render Slot -->
			<main class="main-body">
				{@render children()}
			</main>
		</div>
	</div>
{/if}

<style>
	.app-layout {
		display: flex;
		min-height: 100vh;
		min-height: 100dvh;
		background-color: var(--bg-base);
		color: var(--text-primary);
	}

	/* Sidebar Structure */
	.sidebar {
		width: 250px;
		background-color: var(--bg-surface);
		border-right: 1px solid var(--border);
		display: flex;
		flex-direction: column;
		flex-shrink: 0;
		height: 100vh;
		height: 100dvh;
		position: sticky;
		top: 0;
		z-index: var(--z-dropdown);
	}

	.boot-screen {
		min-height: 100vh;
		min-height: 100dvh;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--bg-base);
	}

	.boot-label {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
	}

	.sidebar-scrollable {
		flex: 1;
		/* Flex items default to min-height:auto — without this the nav refuses to shrink and
		   its tail slides under the sticky footer instead of scrolling. */
		min-height: 0;
		overflow-y: auto;
		padding: var(--space-4) var(--space-3);
	}

	/* app.css used to style .sidebar-nav with its own padding + flex:1 + overflow-y:auto,
	   which double-padded the list inside .sidebar-scrollable and pushed the last item
	   (/accounts) 16px past the footer at 1280x800 (B8). That duplicate is now deleted from
	   app.css; these resets stay as a guard so a reintroduced rule cannot silently regress
	   the layout. The scroll container is the wrapper; the nav itself is just a list. */
	.sidebar-nav {
		padding: 0;
		gap: 0;
		overflow: visible;
	}

	/* Brand Header */
	.sidebar-header {
		padding: var(--space-4) var(--space-5);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.brand-logo-container {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.user-avatar-badge {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-btn);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		color: var(--accent-blue-text);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
	}

	.brand-title-group {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.brand-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		letter-spacing: -0.01em;
	}

	.online-status-badge {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: var(--font-size-xs);
		color: var(--accent-green);
		font-weight: var(--font-weight-medium);
	}

	.mobile-close-btn {
		display: none;
	}

	/* Categorized Nav */
	.nav-section {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-bottom: var(--space-4);
	}

	.nav-section:last-child {
		margin-bottom: 0;
	}

	.nav-section-title {
		font-size: 0.6875rem;
		font-weight: var(--font-weight-bold);
		color: var(--text-muted);
		letter-spacing: 0.08em;
		text-transform: uppercase;
		padding: 0 var(--space-3) var(--space-2);
	}

	.nav-item {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: 8px 12px;
		border-radius: var(--radius-btn);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--text-secondary);
		border: 1px solid transparent;
		transition: color 150ms var(--ease-out),
					background-color 150ms var(--ease-out),
					border-color 150ms var(--ease-out),
					transform 160ms var(--ease-out);
		text-decoration: none;
	}

	.nav-item:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
	}

	.nav-item:active {
		transform: scale(0.97);
	}

	.nav-item.active {
		color: var(--accent-green-text);
		background-color: var(--accent-green-bg);
		border: 1px solid var(--accent-green-border);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
		font-weight: var(--font-weight-semibold);
	}

	.nav-item.active :global(.nav-icon) {
		color: var(--accent-green-text);
	}

	/* Footer User Profile Box */
	.sidebar-footer {
		padding: var(--space-3);
		border-top: 1px solid var(--border-subtle);
		background-color: var(--bg-surface);
	}

	.user-profile-box {
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-3);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05), 0 2px 8px rgba(0, 0, 0, 0.2);
	}

	.user-profile-top {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.steve-head-avatar {
		width: 32px;
		height: 32px;
		border-radius: 6px;
		background: #5C4033;
		border: 1px solid rgba(255, 255, 255, 0.15);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.1rem;
		flex-shrink: 0;
	}

	.user-meta {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
	}

	.user-name {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.user-role {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.user-dropdown-icon {
		color: var(--text-muted);
	}

	.user-actions-row {
		display: flex;
		align-items: center;
		justify-content: space-around;
		padding-top: var(--space-2);
		border-top: 1px solid var(--border-subtle);
	}

	.action-btn {
		background: transparent;
		border: 1px solid transparent;
		color: var(--text-muted);
		cursor: pointer;
		padding: 4px;
		border-radius: var(--radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		transition: color 150ms var(--ease-out),
					background-color 150ms var(--ease-out),
					border-color 150ms var(--ease-out),
					transform 160ms var(--ease-out);
	}

	.action-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
		border-color: var(--border-subtle);
	}

	.action-btn:active {
		transform: scale(0.97);
	}

	.action-btn.action-logout:hover {
		color: var(--danger-text);
		background-color: var(--danger-bg);
		border-color: var(--danger-border);
	}

	/* Content Area */
	.content-area {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	/* Header Bar Styling */
	.header-bar {
		height: 60px;
		background-color: var(--bg-surface);
		border-bottom: 1px solid var(--border);
		box-shadow: 0 1px 0 rgba(255, 255, 255, 0.03);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 var(--space-6);
		position: sticky;
		top: 0;
		z-index: var(--z-sticky);
	}

	.header-left,
	.header-right {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		min-width: 0;
	}

	.header-left {
		flex: 1 1 auto;
	}

	.header-right {
		flex: 0 0 auto;
	}

	.mobile-toggle-btn {
		display: none;
	}

	.status-badge {
		padding: var(--space-1) var(--space-3);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		max-width: 100%;
		overflow: hidden;
	}

	.status-label {
		line-height: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Quake HUD Trigger Button */
	.quake-trigger-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		height: 30px;
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-btn);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.quake-trigger-btn:hover {
		color: var(--text-primary);
		border-color: var(--border-focus);
	}

	.quake-kbd {
		background-color: rgba(255, 255, 255, 0.06);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: var(--radius-sm);
		padding: 1px 4px;
		font-size: 10px;
		line-height: 1;
		color: var(--accent-blue-text);
	}

	/* Quick Telemetry Status Pill */
	.quick-status-pill {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-badge);
		padding: 4px var(--space-4);
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);
	}

	.pill-item {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}

	.pill-label {
		color: var(--text-muted);
	}

	.pill-value {
		color: var(--text-primary);
		font-weight: var(--font-weight-semibold);
	}

	.pill-divider {
		color: var(--text-muted);
	}

	.pill-hint {
		color: var(--text-secondary);
		font-style: italic;
	}

	/* Header User Profile */
	.header-user-profile {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding-left: var(--space-2);
		border-left: 1px solid var(--border-subtle);
	}

	.header-user-avatar {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-focus);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-secondary);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
	}

	.header-user-name {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--text-primary);
	}

	.main-body {
		flex: 1;
		padding: var(--space-6);
		overflow-y: auto;
	}

	/* Responsive Media Queries */
	@media (max-width: 1200px) {
		.quick-status-pill {
			display: none;
		}
	}

	@media (max-width: 768px) {
		.sidebar-backdrop {
			display: block;
			position: fixed;
			inset: 0;
			background: rgba(5, 5, 8, 0.6);
			z-index: calc(var(--z-modal) - 1);
		}

		.nav-item {
			min-height: 44px;
			padding: var(--space-3);
		}

		.action-btn {
			min-width: 44px;
			min-height: 44px;
			padding: var(--space-3);
		}

		.sidebar {
			position: fixed;
			left: 0;
			top: 0;
			bottom: 0;
			z-index: var(--z-modal);
			transform: translateX(-100%);
			transition: transform var(--transition-normal);
			box-shadow: var(--elevation-shadow);
		}

		.sidebar.open {
			transform: translateX(0);
		}

		.mobile-close-btn {
			display: flex;
		}

		.mobile-toggle-btn {
			display: flex;
		}

		.btn-text {
			display: none;
		}

		.header-user-name {
			display: none;
		}

		.header-bar {
			padding: 0 var(--space-4);
		}

		.main-body {
			padding: var(--space-4);
		}
	}

	@media (max-width: 640px) {
		.header-bar {
			padding: 0 var(--space-3);
		}

		.header-left,
		.header-right {
			gap: var(--space-2);
		}

		.quake-btn-text,
		.quake-kbd {
			display: none;
		}

		.quake-trigger-btn {
			padding: 4px 6px;
		}

		.header-user-profile {
			padding-left: var(--space-1);
		}
	}
</style>
