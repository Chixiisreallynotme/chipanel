<script>
	import { onMount } from 'svelte';
	import { apiGet, apiFetch } from '$lib/api/client.js';
	import VersionPickerButton from '$lib/components/common/VersionPickerButton.svelte';
	import {
		Cpu,
		Zap,
		FileText,
		Hammer,
		ShieldAlert,
		Layers,
		Feather,
		Box,
		Activity,
		Crosshair,
		Sparkles,
		CheckCircle2,
		AlertTriangle,
		RefreshCw,
		Check,
		Info,
		RotateCw,
		Sliders
	} from 'lucide-svelte';

	const UNKNOWN = 'inconnu';

	// Page State — `null` means "ChiPanel could not read the config", never a default.
	let currentType = $state(null);
	let currentVersion = $state(null);
	let configError = $state(null);
	let availableTypes = $state([]);
	let allVersions = $state([]);
	/** @type {string[]} */
	let releaseVersions = $state([]);
	/** @type {string[]} */
	let snapshotVersions = $state([]);
	/** @type {any} — background version-watch state (latest release/snapshot + alerts) */
	let versionWatch = $state(null);
	let isLoading = $state(true);
	let isSubmitting = $state(false);

	// Category filter state
	let selectedCategory = $state('ALL');

	// Selection modal state
	let selectedEngine = $state(null);
	let selectedVersionMap = $state({}); // engine_id -> selected_version
	let showConfirmModal = $state(false);
	/** @type {{ detail: string, serverMessage: string } | null} */
	let switchError = $state(null);

	// Toasts Notification Queue State
	let toasts = $state([]);

	function addToast(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		const toast = { id, type, title, message };
		toasts = [...toasts, toast];
		setTimeout(() => {
			removeToast(id);
		}, 5000);
	}

	function removeToast(id) {
		toasts = toasts.filter((t) => t.id !== id);
	}

	async function loadEngineData() {
		isLoading = true;
		try {
			const res = await apiGet('/api/server/engine');
			if (res) {
				currentType = res.current_type ?? null;
				currentVersion = res.current_version ?? null;
				configError = res.config_error ?? null;
				availableTypes = res.available_types || [];
				// No hardcoded fallback list: the backend allow-list is the only
				// source of accepted versions, and anything off it now 400s.
				allVersions = res.all_versions || [];
				releaseVersions = res.release_versions || [];
				snapshotVersions = res.snapshot_versions || [];
				versionWatch = res.version_watch ?? null;

				// Initialize default version map
				const map = {};
				availableTypes.forEach((engine) => {
					map[engine.id] = engine.recommended_versions?.[0] ?? allVersions[0] ?? '';
				});
				selectedVersionMap = map;
			}
		} catch (err) {
			console.error('Failed to load server engine config:', err);
			addToast('error', 'Error Loading Engine Config', err.message || 'Could not fetch engine types.');
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadEngineData();
	});

	function openConfirmModal(engine) {
		selectedEngine = engine;
		switchError = null;
		showConfirmModal = true;
	}

	function closeConfirmModal() {
		showConfirmModal = false;
		selectedEngine = null;
		switchError = null;
	}

	/** @type {Record<number, string>} */
	const SWITCH_ERRORS = {
		400: 'Version refusée : elle ne fait pas partie des versions supportées.',
		403: "Accès refusé : cette action est réservée à l'administrateur.",
		500: "Erreur interne du serveur : le moteur n'a pas été modifié."
	};

	async function confirmEngineSwitch() {
		if (!selectedEngine) return;
		const targetType = selectedEngine.id;
		const targetVersion = selectedVersionMap[targetType];
		if (!targetVersion) {
			switchError = { detail: 'Sélectionnez une version avant de confirmer.', serverMessage: '' };
			return;
		}

		isSubmitting = true;
		switchError = null;
		try {
			// apiFetch serializes plain objects; the cast only satisfies RequestInit.
			const res = await apiFetch('/api/server/engine', {
				method: 'POST',
				body: /** @type {any} */ ({ engine_type: targetType, version: targetVersion })
			});
			const data = await res.json().catch(() => ({}));

			if (!res.ok) {
				switchError = {
					detail: SWITCH_ERRORS[res.status] ?? `La requête a échoué (HTTP ${res.status}).`,
					serverMessage: data?.error || data?.message || ''
				};
				return;
			}

			addToast('success', 'Engine Switched!', data?.message || `Switched to ${targetType} (${targetVersion}). Server is restarting...`);
			currentType = targetType;
			currentVersion = data?.resolved_version ?? targetVersion;
			if (data?.warning) {
				addToast('warning', 'Attention', data.warning);
			}
			closeConfirmModal();
		} catch (err) {
			switchError = {
				detail: "Requête impossible — le panneau n'a pas pu joindre le backend.",
				serverMessage: err instanceof Error ? err.message : ''
			};
		} finally {
			isSubmitting = false;
		}
	}

	/**
	 * Human date for a watch alert: Mojang's ISO publication date when available,
	 * otherwise the unix detection timestamp.
	 * @param {string | undefined} iso
	 * @param {number | undefined} fallbackTs
	 */
	function formatVersionDate(iso, fallbackTs) {
		const d = iso ? new Date(iso) : fallbackTs ? new Date(fallbackTs * 1000) : null;
		if (!d || isNaN(d.getTime())) return '';
		return d.toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' });
	}

	// Filtered engines calculation
	let filteredEngines = $derived.by(() => {
		if (selectedCategory === 'ALL') return availableTypes;
		if (selectedCategory === 'PLUGINS') return availableTypes.filter((e) => e.supports_plugins && !e.supports_mods);
		if (selectedCategory === 'MODS') return availableTypes.filter((e) => e.supports_mods && !e.supports_plugins);
		if (selectedCategory === 'HYBRID') return availableTypes.filter((e) => e.supports_mods && e.supports_plugins);
		if (selectedCategory === 'VANILLA') return availableTypes.filter((e) => e.id === 'VANILLA');
		return availableTypes;
	});

	function getEngineIcon(iconName) {
		switch (iconName) {
			case 'zap': return Zap;
			case 'file-text': return FileText;
			case 'cpu': return Cpu;
			case 'hammer': return Hammer;
			case 'shield-alert': return ShieldAlert;
			case 'layers': return Layers;
			case 'feather': return Feather;
			case 'box': return Box;
			case 'activity': return Activity;
			case 'crosshair': return Crosshair;
			case 'sparkles': return Sparkles;
			default: return Cpu;
		}
	}
</script>

<svelte:head>
	<title>Server Engine Switcher | ChiPanel</title>
</svelte:head>

<div class="engine-page">
	<!-- Page Header -->
	<header class="page-header">
		<div class="header-content">
			<div class="header-title-group">
				<div class="header-icon-box">
					<Sliders size={24} />
				</div>
				<div>
					<h1>Server Engine & Version Switcher</h1>
					<p class="subtitle">Switch dynamically between Purpur, Paper, Fabric, Forge, NeoForge, Spigot, Vanilla, Quilt, and Folia.</p>
				</div>
			</div>
			<button class="btn btn-secondary btn-sm" onclick={loadEngineData} disabled={isLoading}>
				<RefreshCw size={15} class={isLoading ? 'spin' : ''} />
				<span>Refresh</span>
			</button>
		</div>
	</header>

	{#if configError}
		<div class="config-alert" role="alert">
			<AlertTriangle size={18} />
			<span>{configError}</span>
		</div>
	{/if}

	<!-- New version alerts (background watcher) -->
	{#if versionWatch?.new_release}
		{@const alert = versionWatch.new_release}
		<div class="new-release-banner" role="status">
			<Sparkles size={20} />
			<div class="new-version-text">
				<strong>Minecraft {alert.id} est sorti !</strong>
				<span>
					{alert.previous ? `Succède à ${alert.previous}` : 'Nouvelle release officielle'}
					{formatVersionDate(alert.published_at, alert.detected_at) ? ` · sortie le ${formatVersionDate(alert.published_at, alert.detected_at)}` : ''}
					{currentVersion && !['LATEST', alert.id].includes(currentVersion) ? ` — ce serveur est en ${currentVersion}` : ''}
				</span>
			</div>
		</div>
	{:else if versionWatch?.new_snapshot}
		{@const alert = versionWatch.new_snapshot}
		<div class="new-snapshot-banner" role="status">
			<Sparkles size={18} />
			<div class="new-version-text">
				<strong>Nouveau snapshot : {alert.id}</strong>
				<span>
					{alert.previous ? `succède à ${alert.previous}` : ''}
					{formatVersionDate(alert.published_at, alert.detected_at) ? ` · ${formatVersionDate(alert.published_at, alert.detected_at)}` : ''}
				</span>
			</div>
		</div>
	{/if}

	<!-- Current Engine Status Banner -->
	<section class="banner-card glow-card">
		<div class="banner-info">
			<div class="banner-badge">ACTIVE ENGINE</div>
			<div class="banner-title-row">
				<span class="active-engine-name {currentType ? '' : 'unknown-val'}">{currentType ?? UNKNOWN}</span>
				<span class="badge {currentVersion ? 'badge-success' : 'badge-muted'}">{currentVersion ?? UNKNOWN}</span>
			</div>
			{#if currentType && currentVersion}
				<p class="banner-desc">The server is configured to run <strong>{currentType}</strong> on version <strong>{currentVersion}</strong>. Switching engines triggers an automatic download and container restart.</p>
			{:else}
				<p class="banner-desc">ChiPanel cannot read the current engine configuration, so the active engine and version are unknown. Switching still works and will rewrite the configuration.</p>
			{/if}
		</div>
		<div class="banner-stats">
			<div class="stat-chip">
				<span class="stat-label">Latest Release</span>
				<span class="stat-value text-primary">{versionWatch?.latest_release?.id ?? '—'}</span>
			</div>
			<div class="stat-chip">
				<span class="stat-label">Auto-Download</span>
				<span class="stat-value text-emerald">Enabled</span>
			</div>
			<div class="stat-chip">
				<span class="stat-label">Container</span>
				<span class="stat-value">itzg/minecraft-server</span>
			</div>
		</div>
	</section>

	<!-- Category Filters & Search -->
	<div class="filters-bar">
		<div class="category-tabs">
			<button class="tab-btn {selectedCategory === 'ALL' ? 'active' : ''}" onclick={() => (selectedCategory = 'ALL')}>
				All Engines ({availableTypes.length})
			</button>
			<button class="tab-btn {selectedCategory === 'PLUGINS' ? 'active' : ''}" onclick={() => (selectedCategory = 'PLUGINS')}>
				Bukkit/Paper (Plugins)
			</button>
			<button class="tab-btn {selectedCategory === 'MODS' ? 'active' : ''}" onclick={() => (selectedCategory = 'MODS')}>
				Modded (Forge/Fabric)
			</button>
			<button class="tab-btn {selectedCategory === 'HYBRID' ? 'active' : ''}" onclick={() => (selectedCategory = 'HYBRID')}>
				Hybrid (Mods + Plugins)
			</button>
		</div>
	</div>

	<!-- Engine Cards Grid -->
	{#if isLoading}
		<div class="loading-state">
			<RefreshCw size={32} class="spin text-primary" />
			<p>Fetching available server engines...</p>
		</div>
	{:else}
		<div class="engine-grid">
			{#each filteredEngines as engine (engine.id)}
				{@const IconComp = getEngineIcon(engine.icon)}
				{@const isActive = currentType === engine.id}
				<div class="engine-card {isActive ? 'active-card' : ''}">
					<div class="card-header">
						<div class="icon-wrapper {isActive ? 'active-icon' : ''}">
							<IconComp size={24} />
						</div>
						<div class="header-titles">
							<div class="title-row">
								<h3>{engine.name}</h3>
								{#if isActive}
									<span class="badge badge-emerald"><Check size={12} /> Active</span>
								{/if}
							</div>
							<span class="category-tag">{engine.category}</span>
						</div>
					</div>

					<p class="engine-description">{engine.description}</p>

					<div class="features-row">
						{#if engine.supports_plugins}
							<span class="feature-badge plugins-badge">
								<CheckCircle2 size={13} /> Plugins Supported
							</span>
						{/if}
						{#if engine.supports_mods}
							<span class="feature-badge mods-badge">
								<Cpu size={13} /> Mods Supported
							</span>
						{/if}
						{#if !engine.supports_plugins && !engine.supports_mods}
							<span class="feature-badge vanilla-badge">
								<Info size={13} /> Vanilla Gameplay
							</span>
						{/if}
					</div>

					<div class="card-footer">
						<VersionPickerButton
							selectedVersion={selectedVersionMap[engine.id]}
							{releaseVersions}
							{snapshotVersions}
							recommendedVersions={engine.recommended_versions}
							allowAll={false}
							label="Version Cible :"
							disabled={isActive}
							onSelect={(newVer) => (selectedVersionMap[engine.id] = newVer)}
						/>

						{#if isActive}
							<button class="btn btn-secondary w-full" disabled>
								<Check size={16} /> Currently Active
							</button>
						{:else}
							<button class="btn btn-primary w-full" onclick={() => openConfirmModal(engine)}>
								<RotateCw size={16} /> Switch to {engine.name}
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Confirmation Modal -->
{#if showConfirmModal && selectedEngine}
	<div class="modal-backdrop" onclick={closeConfirmModal} onkeydown={(e) => e.key === 'Escape' && closeConfirmModal()} role="button" tabindex="0">
		<div class="modal-card" onclick={(e) => e.stopPropagation()} onkeydown={() => {}} role="dialog" tabindex="-1">
			<div class="modal-header">
				<div class="modal-icon-box danger-icon">
					<AlertTriangle size={24} />
				</div>
				<div>
					<h2>Confirm Engine Switch</h2>
					<p class="modal-subtitle">Reconfiguring Minecraft Server Core</p>
				</div>
			</div>

			<div class="modal-body">
				<p>You are about to switch the server engine to:</p>
				<div class="confirm-summary-box">
					<div class="summary-item">
						<span class="summary-label">Target Engine:</span>
						<span class="summary-val text-primary">{selectedEngine.name} ({selectedEngine.id})</span>
					</div>
					<div class="summary-item">
						<span class="summary-label">Target Version:</span>
						<span class="summary-val text-emerald">{selectedVersionMap[selectedEngine.id] || UNKNOWN}</span>
					</div>
				</div>
				<p class="modal-warning">
					⚠️ <strong>Important Note:</strong> Switching engines will restart the server. The container will automatically download and install the specified server software version.
				</p>

				{#if switchError}
					<div class="switch-error" role="alert">
						<AlertTriangle size={16} />
						<div>
							<div class="switch-error-detail">{switchError.detail}</div>
							{#if switchError.serverMessage}
								<div class="switch-error-msg">{switchError.serverMessage}</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn btn-ghost" onclick={closeConfirmModal} disabled={isSubmitting}>Cancel</button>
				<button class="btn btn-danger" onclick={confirmEngineSwitch} disabled={isSubmitting}>
					{#if isSubmitting}
						<RefreshCw size={16} class="spin" />
						<span>Applying & Restarting...</span>
					{:else}
						<RotateCw size={16} />
						<span>Confirm Switch & Restart</span>
					{/if}
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Toast Notification Area -->
<div class="toast-area">
	{#each toasts as toast (toast.id)}
		<div class="toast toast-{toast.type}">
			{#if toast.type === 'success'}
				<CheckCircle2 size={18} />
			{:else}
				<AlertTriangle size={18} />
			{/if}
			<div>
				<div class="toast-title">{toast.title}</div>
				<div class="toast-message">{toast.message}</div>
			</div>
		</div>
	{/each}
</div>

<style>
	.engine-page {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		max-width: 1300px;
		margin: 0 auto;
		padding-bottom: 3rem;
	}

	.page-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.header-content {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
	}

	.header-title-group {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.header-icon-box {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: 12px;
		background: rgba(99, 102, 241, 0.15);
		color: #818cf8;
		border: 1px solid rgba(99, 102, 241, 0.3);
	}

	h1 {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--color-heading, #f8fafc);
		margin: 0;
	}

	.subtitle {
		font-size: 0.875rem;
		color: var(--color-text-muted, #94a3b8);
		margin: 0.25rem 0 0 0;
	}

	.banner-card {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.5rem 2rem;
		border-radius: 16px;
		background: linear-gradient(135deg, rgba(30, 41, 59, 0.9), rgba(15, 23, 42, 0.95));
		border: 1px solid rgba(99, 102, 241, 0.3);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
	}

	.banner-badge {
		font-size: 0.7rem;
		font-weight: 800;
		letter-spacing: 0.05em;
		color: #818cf8;
		margin-bottom: 0.5rem;
	}

	.banner-title-row {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 0.5rem;
	}

	.active-engine-name {
		font-size: 1.75rem;
		font-weight: 800;
		color: #ffffff;
	}

	.banner-desc {
		font-size: 0.875rem;
		color: #cbd5e1;
		margin: 0;
		max-width: 700px;
	}

	.banner-stats {
		display: flex;
		gap: 1.25rem;
	}

	.stat-chip {
		display: flex;
		flex-direction: column;
		padding: 0.75rem 1.25rem;
		border-radius: 10px;
		background: rgba(15, 23, 42, 0.6);
		border: 1px solid rgba(255, 255, 255, 0.08);
	}

	.stat-label {
		font-size: 0.75rem;
		color: #94a3b8;
	}

	.stat-value {
		font-size: 1rem;
		font-weight: 700;
		color: #f1f5f9;
	}

	.text-emerald {
		color: #10b981 !important;
	}

	.text-primary {
		color: #818cf8 !important;
	}

	.filters-bar {
		display: flex;
		gap: 0.5rem;
		overflow-x: auto;
	}

	.category-tabs {
		display: flex;
		gap: 0.5rem;
		background: rgba(15, 23, 42, 0.6);
		padding: 0.35rem;
		border-radius: 12px;
		border: 1px solid rgba(255, 255, 255, 0.08);
	}

	.tab-btn {
		padding: 0.5rem 1rem;
		font-size: 0.85rem;
		font-weight: 600;
		border-radius: 8px;
		border: none;
		background: transparent;
		color: #94a3b8;
		cursor: pointer;
		transition: all 0.2s;
	}

	.tab-btn.active {
		background: #6366f1;
		color: #ffffff;
	}

	.engine-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
		gap: 1.25rem;
	}

	.engine-card {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		padding: 1.5rem;
		border-radius: 14px;
		background: rgba(30, 41, 59, 0.6);
		border: 1px solid rgba(255, 255, 255, 0.08);
		transition: transform 0.2s, border-color 0.2s, box-shadow 0.2s;
	}

	.engine-card:hover {
		border-color: rgba(99, 102, 241, 0.4);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
	}

	.active-card {
		border-color: #10b981;
		background: linear-gradient(180deg, rgba(16, 185, 129, 0.08), rgba(30, 41, 59, 0.7));
	}

	.card-header {
		display: flex;
		gap: 1rem;
		align-items: flex-start;
	}

	.icon-wrapper {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border-radius: 10px;
		background: rgba(255, 255, 255, 0.06);
		color: #cbd5e1;
		flex-shrink: 0;
	}

	.active-icon {
		background: rgba(16, 185, 129, 0.2);
		color: #10b981;
	}

	.header-titles {
		flex: 1;
	}

	.title-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	h3 {
		font-size: 1.15rem;
		font-weight: 700;
		color: #ffffff;
		margin: 0;
	}

	.category-tag {
		font-size: 0.75rem;
		color: #94a3b8;
	}

	.engine-description {
		font-size: 0.85rem;
		color: #cbd5e1;
		line-height: 1.5;
		margin: 1rem 0;
		min-height: 2.5rem;
	}

	.features-row {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin-bottom: 1.25rem;
	}

	.feature-badge {
		display: inline-flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.25rem 0.6rem;
		border-radius: 6px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.plugins-badge {
		background: rgba(59, 130, 246, 0.15);
		color: #60a5fa;
		border: 1px solid rgba(59, 130, 246, 0.3);
	}

	.mods-badge {
		background: rgba(168, 85, 247, 0.15);
		color: #c084fc;
		border: 1px solid rgba(168, 85, 247, 0.3);
	}

	.vanilla-badge {
		background: rgba(148, 163, 184, 0.15);
		color: #cbd5e1;
		border: 1px solid rgba(148, 163, 184, 0.3);
	}

	.card-footer {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.version-select-box {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}

	.select-label {
		font-size: 0.8rem;
		color: #94a3b8;
	}

	.select-input {
		padding: 0.4rem 0.75rem;
		border-radius: 8px;
		background: rgba(15, 23, 42, 0.8);
		border: 1px solid rgba(255, 255, 255, 0.15);
		color: #ffffff;
		font-size: 0.85rem;
		font-weight: 600;
	}

	.w-full {
		width: 100%;
	}

	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.75);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal);
	}

	.modal-card {
		width: 100%;
		max-width: 480px;
		background: #1e293b;
		border: 1px solid rgba(255, 255, 255, 0.15);
		border-radius: 16px;
		padding: 1.75rem;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
	}

	.modal-header {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.danger-icon {
		background: rgba(239, 68, 68, 0.2);
		color: #ef4444;
		width: 44px;
		height: 44px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.confirm-summary-box {
		background: rgba(15, 23, 42, 0.7);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 10px;
		padding: 1rem;
		margin: 1rem 0;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.summary-item {
		display: flex;
		justify-content: space-between;
		font-size: 0.9rem;
	}

	.summary-label {
		color: #94a3b8;
	}

	.summary-val {
		font-weight: 700;
	}

	.modal-warning {
		font-size: 0.85rem;
		color: #f1f5f9;
		background: rgba(245, 158, 11, 0.15);
		border: 1px solid rgba(245, 158, 11, 0.3);
		padding: 0.75rem;
		border-radius: 8px;
		margin: 0;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		margin-top: 1.5rem;
	}

	.toast-area {
		position: fixed;
		bottom: 1.5rem;
		right: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		z-index: var(--z-toast);
	}

	.toast {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.85rem 1.25rem;
		border-radius: 10px;
		background: #1e293b;
		border: 1px solid rgba(255, 255, 255, 0.1);
		box-shadow: 0 10px 25px rgba(0, 0, 0, 0.3);
		color: #ffffff;
		min-width: 280px;
	}

	.toast-success {
		border-left: 4px solid #10b981;
	}

	.toast-error {
		border-left: 4px solid #ef4444;
	}

	.toast-warning {
		border-left: 4px solid #f59e0b;
	}

	.toast-title {
		font-weight: 700;
		font-size: 0.85rem;
	}

	.toast-message {
		font-size: 0.75rem;
		color: #cbd5e1;
	}

	.config-alert {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.85rem 1.25rem;
		border-radius: 10px;
		background: rgba(245, 158, 11, 0.15);
		border: 1px solid rgba(245, 158, 11, 0.35);
		color: #fbbf24;
		font-size: 0.875rem;
	}

	.new-release-banner {
		display: flex;
		align-items: center;
		gap: 0.9rem;
		padding: 1rem 1.4rem;
		border-radius: 12px;
		background: linear-gradient(135deg, rgba(16, 185, 129, 0.18), rgba(16, 185, 129, 0.06));
		border: 1px solid rgba(16, 185, 129, 0.45);
		color: #34d399;
		box-shadow: 0 4px 20px rgba(16, 185, 129, 0.12);
	}

	.new-snapshot-banner {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 1.25rem;
		border-radius: 10px;
		background: rgba(99, 102, 241, 0.12);
		border: 1px solid rgba(99, 102, 241, 0.35);
		color: #a5b4fc;
	}

	.new-version-text {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
		font-size: 0.875rem;
	}

	.new-version-text strong {
		font-size: 0.95rem;
	}

	.new-version-text span {
		color: var(--color-text-muted, #94a3b8);
		font-size: 0.8rem;
	}

	.unknown-val {
		color: #94a3b8;
	}

	.badge-muted {
		background: rgba(148, 163, 184, 0.15);
		color: #cbd5e1;
		border: 1px solid rgba(148, 163, 184, 0.3);
	}

	.switch-error {
		display: flex;
		align-items: flex-start;
		gap: 0.65rem;
		margin-top: 1rem;
		padding: 0.75rem;
		border-radius: 8px;
		background: rgba(239, 68, 68, 0.15);
		border: 1px solid rgba(239, 68, 68, 0.35);
		color: #fca5a5;
	}

	.switch-error-detail {
		font-size: 0.85rem;
		font-weight: 600;
	}

	.switch-error-msg {
		font-size: 0.75rem;
		font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
		color: #fecaca;
		word-break: break-word;
	}

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
