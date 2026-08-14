<script>
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
	import InstalledPluginsList from '$lib/components/plugins/InstalledPluginsList.svelte';
	import ModrinthCatalogBrowser from '$lib/components/plugins/ModrinthCatalogBrowser.svelte';
	import ModpackCatalogBrowser from '$lib/components/modpacks/ModpackCatalogBrowser.svelte';
	import ProfileManager from '$lib/components/modpacks/ProfileManager.svelte';
	import {
		Package,
		Boxes,
		Store,
		HardDrive,
		Sparkles,
		CheckCircle2,
		AlertCircle,
		Info,
		X,
		Layers,
		Cpu,
		Palette,
		Scroll
	} from 'lucide-svelte';

	// Active tab state: 'plugins' | 'mods' | 'resourcepacks' | 'datapacks' | 'modpacks' | 'profiles'
	let activeTab = $state('plugins');
	let installedPlugins = $state([]);
	let isLoadingInstalled = $state(false);
	let profileManagerRef = $state(null);

	// Sync activeTab with URL search params on mount / state change
	$effect(() => {
		const tabParam = page.url.searchParams.get('tab');
		if (tabParam && ['plugins', 'mods', 'resourcepacks', 'datapacks', 'modpacks', 'profiles'].includes(tabParam)) {
			activeTab = tabParam;
		}
	});

	// Floating Toasts Notification System State
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

	// Fetch Installed Addons (Plugins, Mods, ResourcePacks, DataPacks)
	async function loadInstalledPlugins() {
		isLoadingInstalled = true;
		try {
			const data = await apiGet('/api/plugins/installed');
			installedPlugins = Array.isArray(data) ? data : [];
		} catch (err) {
			console.error('Error fetching installed addons:', err);
			addToast('error', 'Échec du chargement', err.message || 'Impossible de charger les extensions installées.');
		} finally {
			isLoadingInstalled = false;
		}
	}

	onMount(() => {
		loadInstalledPlugins();
	});

	// Action Handler: Toggle Enable/Disable Plugin/Mod
	async function handleToggle(plugin) {
		try {
			const res = await apiPost('/api/plugins/toggle', {
				filename: plugin.filename,
				target_dir: plugin.target_dir
			});

			installedPlugins = installedPlugins.map((p) =>
				p.filename === plugin.filename && p.target_dir === plugin.target_dir ? res : p
			);

			const statusText = res.enabled ? 'activée' : 'désactivée';
			addToast(
				'success',
				'Extension mise à jour',
				`"${res.name}" a été ${statusText} avec succès.`
			);
		} catch (err) {
			console.error('Failed to toggle extension:', err);
			addToast(
				'error',
				'Erreur de modification',
				err.message || `Échec du changement d'état pour ${plugin.name}`
			);
			throw err;
		}
	}

	// Action Handler: Delete Plugin/Mod
	async function handleDelete(plugin) {
		try {
			await apiFetch('/api/plugins/delete', {
				method: 'DELETE',
				body: {
					filename: plugin.filename,
					target_dir: plugin.target_dir
				}
			});

			installedPlugins = installedPlugins.filter(
				(p) => !(p.filename === plugin.filename && p.target_dir === plugin.target_dir)
			);

			addToast(
				'success',
				'Extension supprimée',
				`"${plugin.name}" a été supprimée du serveur.`
			);
		} catch (err) {
			console.error('Failed to delete extension:', err);
			addToast(
				'error',
				'Erreur de suppression',
				err.message || `Échec de la suppression de ${plugin.name}`
			);
			throw err;
		}
	}

	// Action Handler: Install from Modrinth Store
	async function handleInstall({ projectId, versionId, targetDir, projectTitle }) {
		try {
			const res = await apiPost('/api/plugins/install', {
				project_id: projectId,
				version_id: versionId,
				target_dir: targetDir
			});

			addToast(
				'success',
				'Installation réussie !',
				`"${projectTitle || res.filename}" a été installé dans /${res.target_dir}/`
			);

			await loadInstalledPlugins();
			activeTab = targetDir;
		} catch (err) {
			console.error('Failed to install addon:', err);
			addToast(
				'error',
				"Échec de l'installation",
				err.message || `Impossible d'installer ${projectTitle || 'l\'extension'}.`
			);
		}
	}

	// Modpack Deploy handler
	async function handleDeployModpack({ modpackTitle, profileName, loader, gameVersion }) {
		addToast(
			'success',
			'Modpack déployé !',
			`"${modpackTitle}" déployé dans le profil "${profileName}" (${loader} MC ${gameVersion || '1.20.4'}).`
		);

		activeTab = 'profiles';
		if (profileManagerRef && typeof profileManagerRef.loadProfiles === 'function') {
			profileManagerRef.loadProfiles();
		}
	}

	// Profile Switch Handler
	async function handleProfileSwitched(profile) {
		addToast(
			'success',
			'Profil actif mis à jour',
			`Le profil du serveur a été basculé vers "${profile.name}".`
		);
	}

	// Profile Delete Handler
	async function handleProfileDeleted(profile) {
		addToast(
			'info',
			'Profil supprimé',
			`Le profil "${profile.name}" a été supprimé.`
		);
	}
</script>

<svelte:head>
	<title>Centre des Addons & Modpacks - ChiPanel</title>
</svelte:head>

<div class="addons-page">
	<!-- Hero Header -->
	<div class="page-header">
		<div class="header-main font-ui">
			<div class="title-with-icon">
				<div class="page-icon-box">
					<Package size={26} />
				</div>
				<div>
					<h1 class="page-title">Centre des Addons & Contenus</h1>
					<p class="page-description">
						Gérez vos plugins, mods, packs de textures, data packs et modpacks au même endroit via Modrinth.
					</p>
				</div>
			</div>
		</div>

		<!-- Unified Navigation Tabs Bar (Plugins, Mods, ResourcePacks, DataPacks, Modpacks, Profils) -->
		<div class="tabs-nav-container" role="tablist">
			<button
				class="tab-btn {activeTab === 'plugins' ? 'active' : ''}"
				role="tab"
				aria-selected={activeTab === 'plugins'}
				onclick={() => (activeTab = 'plugins')}
			>
				<Package size={16} />
				<span>Plugins</span>
				{#if installedPlugins.filter(p => p.target_dir === 'plugins').length > 0}
					<span class="tab-badge">{installedPlugins.filter(p => p.target_dir === 'plugins').length}</span>
				{/if}
			</button>

			<button
				class="tab-btn {activeTab === 'mods' ? 'active' : ''}"
				role="tab"
				aria-selected={activeTab === 'mods'}
				onclick={() => (activeTab = 'mods')}
			>
				<Cpu size={16} />
				<span>Mods</span>
				{#if installedPlugins.filter(p => p.target_dir === 'mods').length > 0}
					<span class="tab-badge badge-purple">{installedPlugins.filter(p => p.target_dir === 'mods').length}</span>
				{/if}
			</button>

			<button
				class="tab-btn {activeTab === 'resourcepacks' ? 'active' : ''}"
				role="tab"
				aria-selected={activeTab === 'resourcepacks'}
				onclick={() => (activeTab = 'resourcepacks')}
			>
				<Palette size={16} />
				<span>Packs de Textures</span>
				{#if installedPlugins.filter(p => p.target_dir === 'resourcepacks').length > 0}
					<span class="tab-badge badge-amber">{installedPlugins.filter(p => p.target_dir === 'resourcepacks').length}</span>
				{/if}
			</button>

			<button
				class="tab-btn {activeTab === 'datapacks' ? 'active' : ''}"
				role="tab"
				aria-selected={activeTab === 'datapacks'}
				onclick={() => (activeTab = 'datapacks')}
			>
				<Scroll size={16} />
				<span>Data Packs</span>
				{#if installedPlugins.filter(p => p.target_dir === 'datapacks').length > 0}
					<span class="tab-badge badge-teal">{installedPlugins.filter(p => p.target_dir === 'datapacks').length}</span>
				{/if}
			</button>

			<button
				class="tab-btn {activeTab === 'modpacks' ? 'active' : ''}"
				role="tab"
				aria-selected={activeTab === 'modpacks'}
				onclick={() => (activeTab = 'modpacks')}
			>
				<Boxes size={16} />
				<span>Modpacks</span>
			</button>

			<button
				class="tab-btn {activeTab === 'profiles' ? 'active' : ''}"
				role="tab"
				aria-selected={activeTab === 'profiles'}
				onclick={() => (activeTab = 'profiles')}
			>
				<HardDrive size={16} />
				<span>Profils & Presets</span>
			</button>
		</div>
	</div>

	<!-- Tab Body Content -->
	<div class="tab-content">
		{#if activeTab === 'plugins'}
			<div class="tab-section-container">
				<div class="section-block">
					<h2 class="section-title">
						<Package size={18} />
						<span>Plugins Installés (/plugins)</span>
					</h2>
					<InstalledPluginsList
						plugins={installedPlugins.filter(p => p.target_dir === 'plugins')}
						loading={isLoadingInstalled}
						initialTargetDir="plugins"
						lockFolderFilter={true}
						onToggle={handleToggle}
						onDelete={handleDelete}
						onRefresh={loadInstalledPlugins}
					/>
				</div>

				<div class="section-block">
					<h2 class="section-title">
						<Store size={18} />
						<span>Catalogue Modrinth (Plugins Spigot / Paper / Purpur)</span>
					</h2>
					<ModrinthCatalogBrowser
						initialProjectType="plugin"
						lockProjectType={true}
						onInstall={handleInstall}
					/>
				</div>
			</div>
		{:else if activeTab === 'mods'}
			<div class="tab-section-container">
				<div class="section-block">
					<h2 class="section-title">
						<Cpu size={18} />
						<span>Mods Installés (/mods)</span>
					</h2>
					<InstalledPluginsList
						plugins={installedPlugins.filter(p => p.target_dir === 'mods')}
						loading={isLoadingInstalled}
						initialTargetDir="mods"
						lockFolderFilter={true}
						onToggle={handleToggle}
						onDelete={handleDelete}
						onRefresh={loadInstalledPlugins}
					/>
				</div>

				<div class="section-block">
					<h2 class="section-title">
						<Store size={18} />
						<span>Catalogue Modrinth (Mods Fabric / Forge / NeoForge / Quilt)</span>
					</h2>
					<ModrinthCatalogBrowser
						initialProjectType="mod"
						lockProjectType={true}
						onInstall={handleInstall}
					/>
				</div>
			</div>
		{:else if activeTab === 'resourcepacks'}
			<div class="tab-section-container">
				<div class="section-block">
					<h2 class="section-title">
						<Palette size={18} />
						<span>Packs de Textures Installés (/resourcepacks)</span>
					</h2>
					<InstalledPluginsList
						plugins={installedPlugins.filter(p => p.target_dir === 'resourcepacks')}
						loading={isLoadingInstalled}
						initialTargetDir="resourcepacks"
						lockFolderFilter={true}
						onToggle={handleToggle}
						onDelete={handleDelete}
						onRefresh={loadInstalledPlugins}
					/>
				</div>

				<div class="section-block">
					<h2 class="section-title">
						<Store size={18} />
						<span>Catalogue Modrinth (Packs de Textures / Resource Packs)</span>
					</h2>
					<ModrinthCatalogBrowser
						initialProjectType="resourcepack"
						lockProjectType={true}
						onInstall={handleInstall}
					/>
				</div>
			</div>
		{:else if activeTab === 'datapacks'}
			<div class="tab-section-container">
				<div class="section-block">
					<h2 class="section-title">
						<Scroll size={18} />
						<span>Data Packs Installés (/datapacks)</span>
					</h2>
					<InstalledPluginsList
						plugins={installedPlugins.filter(p => p.target_dir === 'datapacks')}
						loading={isLoadingInstalled}
						initialTargetDir="datapacks"
						lockFolderFilter={true}
						onToggle={handleToggle}
						onDelete={handleDelete}
						onRefresh={loadInstalledPlugins}
					/>
				</div>

				<div class="section-block">
					<h2 class="section-title">
						<Store size={18} />
						<span>Catalogue Modrinth (Data Packs)</span>
					</h2>
					<ModrinthCatalogBrowser
						initialProjectType="datapack"
						lockProjectType={true}
						onInstall={handleInstall}
					/>
				</div>
			</div>
		{:else if activeTab === 'modpacks'}
			<ModpackCatalogBrowser onDeploy={handleDeployModpack} />
		{:else if activeTab === 'profiles'}
			<ProfileManager
				bind:this={profileManagerRef}
				onSwitchProfile={handleProfileSwitched}
				onDeleteProfile={handleProfileDeleted}
			/>
		{/if}
	</div>

	<!-- Floating Toast Notifications System -->
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
						<div class="toast-title">{toast.title}</div>
						<div class="toast-message">{toast.message}</div>
					</div>

					<button class="toast-close-btn" aria-label="Close notification" onclick={() => removeToast(toast.id)}>
						<X size={14} />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.addons-page {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		padding: 1.5rem;
		max-width: 1600px;
		margin: 0 auto;
		width: 100%;
	}

	.page-header {
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
		background: var(--surface-card, #161922);
		border: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
		border-radius: 12px;
		padding: 1.5rem;
	}

	.header-main {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.title-with-icon {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.page-icon-box {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: 10px;
		background: linear-gradient(135deg, rgba(99, 102, 241, 0.15), rgba(168, 85, 247, 0.15));
		border: 1px solid rgba(99, 102, 241, 0.3);
		color: #a855f7;
	}

	.page-title {
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--text-heading, #ffffff);
		letter-spacing: -0.02em;
		margin: 0;
	}

	.page-description {
		font-size: 0.875rem;
		color: var(--text-muted, #94a3b8);
		margin-top: 0.25rem;
	}

	/* Tabs Bar Styling */
	.tabs-nav-container {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		border-top: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
		padding-top: 1.25rem;
		overflow-x: auto;
	}

	.tab-btn {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.625rem 1rem;
		border-radius: 8px;
		background: transparent;
		border: 1px solid transparent;
		color: var(--text-muted, #94a3b8);
		font-weight: 500;
		font-size: 0.875rem;
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.tab-btn:hover {
		background: rgba(255, 255, 255, 0.05);
		color: var(--text-heading, #ffffff);
	}

	.tab-btn.active {
		background: var(--surface-hover, rgba(255, 255, 255, 0.08));
		border-color: rgba(99, 102, 241, 0.4);
		color: #a855f7;
		font-weight: 600;
	}

	.tab-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0.125rem 0.5rem;
		font-size: 0.75rem;
		font-weight: 600;
		border-radius: 999px;
		background: rgba(16, 185, 129, 0.2);
		color: #34d399;
	}

	.tab-badge.badge-purple {
		background: rgba(168, 85, 247, 0.2);
		color: #c084fc;
	}

	.tab-badge.badge-amber {
		background: rgba(245, 158, 11, 0.2);
		color: #fbbf24;
	}

	.tab-badge.badge-teal {
		background: rgba(20, 184, 166, 0.2);
		color: #2dd4bf;
	}

	.tab-sparkle-pill {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.125rem 0.5rem;
		font-size: 0.75rem;
		border-radius: 999px;
		background: rgba(99, 102, 241, 0.15);
		color: #818cf8;
		border: 1px solid rgba(99, 102, 241, 0.3);
	}

	.tab-content {
		width: 100%;
	}

	.tab-section-container {
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.section-block {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.section-title {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 1.125rem;
		font-weight: 600;
		color: var(--text-heading, #ffffff);
		margin: 0;
	}

	/* Toast Notification System Styles */
	.toast-container {
		position: fixed;
		bottom: 1.5rem;
		right: 1.5rem;
		z-index: 9999;
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		max-width: 400px;
	}

	.toast-item {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 0.875rem 1rem;
		border-radius: 10px;
		background: #1e2330;
		border: 1px solid rgba(255, 255, 255, 0.1);
		box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
		color: #ffffff;
		animation: slideIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.toast-success {
		border-left: 4px solid #10b981;
	}
	.toast-success .toast-icon {
		color: #10b981;
	}

	.toast-error {
		border-left: 4px solid #ef4444;
	}
	.toast-error .toast-icon {
		color: #ef4444;
	}

	.toast-info {
		border-left: 4px solid #3b82f6;
	}
	.toast-info .toast-icon {
		color: #3b82f6;
	}

	.toast-body {
		flex: 1;
	}

	.toast-title {
		font-weight: 600;
		font-size: 0.875rem;
		margin-bottom: 0.125rem;
	}

	.toast-message {
		font-size: 0.8125rem;
		color: #94a3b8;
		line-height: 1.4;
	}

	.toast-close-btn {
		background: transparent;
		border: none;
		color: #64748b;
		cursor: pointer;
		padding: 0.125rem;
		border-radius: 4px;

		&:hover {
			color: #ffffff;
		}
	}

	@keyframes slideIn {
		from {
			opacity: 0;
			transform: translateY(12px) scale(0.95);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}
</style>
