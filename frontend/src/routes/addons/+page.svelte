<script>
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
	import InstalledPluginsList from '$lib/components/plugins/InstalledPluginsList.svelte';
	import ModrinthCatalogBrowser from '$lib/components/plugins/ModrinthCatalogBrowser.svelte';
	import ToolsSyncPanel from '$lib/components/plugins/ToolsSyncPanel.svelte';
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
		AlertTriangle,
		Info,
		X,
		Layers,
		Cpu,
		Palette,
		Scroll,
		RefreshCw,
		DownloadCloud,
		Check,
		Wrench,
		ExternalLink,
		Loader2
	} from 'lucide-svelte';

	// Active tab state: 'plugins' | 'mods' | 'resourcepacks' | 'datapacks' | 'modpacks' | 'profiles'
	let activeTab = $state('plugins');
	// Sub-view segmented switcher: 'installed' | 'catalog'
	let subView = $state('installed');

	let installedPlugins = $state([]);
	let isLoadingInstalled = $state(false);
	let profileManagerRef = $state(null);

	// Server engine and resource pack states
	let serverEngine = $state('');
	let serverVersion = $state('');
	let serverResourcePack = $state(null);

	// Addon Updates State
	let isCheckingUpdates = $state(false);
	let updatesModalOpen = $state(false);
	let updatesReport = $state(null);
	let isApplyingUpdates = $state(false);
	let availableUpdatesCount = $state(0);

	// Sync activeTab with URL search params on mount / state change
	$effect(() => {
		const tabParam = page.url.searchParams.get('tab');
		if (tabParam && ['plugins', 'mods', 'resourcepacks', 'datapacks', 'modpacks', 'profiles'].includes(tabParam)) {
			activeTab = tabParam;
		}
		const subParam = page.url.searchParams.get('sub');
		if (subParam && ['installed', 'catalog'].includes(subParam)) {
			subView = subParam;
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

	// Fetch Server Context
	async function loadServerContext() {
		try {
			const [engineRes, rpRes] = await Promise.allSettled([
				apiGet('/api/server/engine'),
				apiGet('/api/server/resource-pack')
			]);

			if (engineRes.status === 'fulfilled' && engineRes.value) {
				serverEngine = engineRes.value.current_type || '';
				serverVersion = engineRes.value.current_version || '';
			}

			if (rpRes.status === 'fulfilled' && rpRes.value) {
				serverResourcePack = rpRes.value;
			}
		} catch (err) {
			console.error('Erreur chargement contexte serveur:', err);
		}
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

	// Check for Addon Updates
	async function handleCheckUpdates() {
		isCheckingUpdates = true;
		try {
			const res = await apiGet('/api/plugins/updates/check');
			updatesReport = res;
			availableUpdatesCount = res.updates_available_count || 0;
			updatesModalOpen = true;

			if (availableUpdatesCount > 0) {
				addToast(
					'info',
					'Mises à jour trouvées',
					`${availableUpdatesCount} extension(s) peuvent être mise(s) à jour.`
				);
			} else {
				addToast(
					'success',
					'Extensions à jour',
					'Toutes vos extensions installées sont compatibles et à jour.'
				);
			}
		} catch (err) {
			console.error('Erreur vérification mises à jour:', err);
			addToast('error', 'Erreur de vérification', err.message || 'Impossible de vérifier les mises à jour.');
		} finally {
			isCheckingUpdates = false;
		}
	}

	// Apply Single or All Updates
	async function handleApplyUpdates(itemsToUpdate) {
		if (!itemsToUpdate || itemsToUpdate.length === 0 || isApplyingUpdates) return;
		isApplyingUpdates = true;

		const payload = {
			updates: itemsToUpdate.map((item) => ({
				target_dir: item.target_dir,
				old_filename: item.filename,
				new_filename: item.latest_filename || item.filename,
				download_url: item.latest_download_url,
				version_number: item.latest_version_number || 'latest'
			}))
		};

		try {
			const res = await apiPost('/api/plugins/updates/apply', payload);
			if (res.success) {
				addToast(
					'success',
					'Mise à jour réussie',
					`${res.updated_count} extension(s) mise(s) à jour avec succès. Pensez à redémarrer le serveur.`
				);
			} else if (res.errors && res.errors.length > 0) {
				addToast(
					'error',
					'Mise à jour partielle',
					res.errors.join(' | ')
				);
			}

			await loadInstalledPlugins();
			// Recheck updates
			const refreshed = await apiGet('/api/plugins/updates/check');
			updatesReport = refreshed;
			availableUpdatesCount = refreshed.updates_available_count || 0;
		} catch (err) {
			console.error('Erreur application mises à jour:', err);
			addToast('error', 'Erreur de mise à jour', err.message || 'Échec de la mise à jour.');
		} finally {
			isApplyingUpdates = false;
		}
	}

	onMount(() => {
		loadServerContext();
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
				`"${res.name}" a été ${statusText}. Redémarrez le serveur pour appliquer.`
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
			subView = 'installed';
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
		await loadServerContext();
		await loadInstalledPlugins();
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
	<title>Centre des Addons & Extensions - ChiPanel</title>
</svelte:head>

<div class="addons-page">
	<!-- Hero Header & Engine Context Banner -->
	<div class="page-header">
		<div class="header-main font-ui">
			<div class="title-with-icon">
				<div class="page-icon-box">
					<Package size={26} />
				</div>
				<div>
					<h1 class="page-title">Centre des Addons & Extensions</h1>
					<p class="page-description">
						Gérez vos plugins, mods, packs de textures et modpacks avec synchronisation automatique selon votre moteur.
					</p>
				</div>
			</div>

			<!-- Live Engine Status & Update Checker Bar -->
			<div class="header-actions">
				{#if serverEngine}
					<div class="engine-badge-box">
						<span class="engine-indicator-dot"></span>
						<span class="engine-text">Moteur : <strong>{serverEngine}</strong> ({serverVersion || 'MC'})</span>
					</div>
				{/if}

				<button
					type="button"
					class="btn btn-secondary check-updates-btn {isCheckingUpdates ? 'btn-loading' : ''}"
					disabled={isCheckingUpdates}
					onclick={handleCheckUpdates}
				>
					{#if isCheckingUpdates}
						<Loader2 size={16} class="spin" />
					{:else}
						<RefreshCw size={16} />
					{/if}
					<span>Vérifier les mises à jour</span>
					{#if availableUpdatesCount > 0}
						<span class="badge badge-amber badge-pill">{availableUpdatesCount}</span>
					{/if}
				</button>
			</div>
		</div>

		<!-- Unified Navigation Tabs Bar -->
		<div class="tabs-nav-container" role="tablist">
			<button
				class="tab-btn {activeTab === 'plugins' ? 'active' : ''}"
				role="tab"
				aria-selected={activeTab === 'plugins'}
				onclick={() => { activeTab = 'plugins'; subView = 'installed'; }}
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
				onclick={() => { activeTab = 'mods'; subView = 'installed'; }}
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
				onclick={() => { activeTab = 'resourcepacks'; subView = 'installed'; }}
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
				onclick={() => { activeTab = 'datapacks'; subView = 'installed'; }}
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
		{#if activeTab === 'plugins' || activeTab === 'mods' || activeTab === 'resourcepacks' || activeTab === 'datapacks'}
			<!-- Secondary Sub-View Segmented Switcher (Installed vs Catalogue) -->
			<div class="subview-header">
				<div class="subview-segmented">
					<button
						type="button"
						class="subview-btn {subView === 'installed' ? 'active' : ''}"
						onclick={() => (subView = 'installed')}
					>
						<HardDrive size={15} />
						<span>
							{#if activeTab === 'plugins'}Plugins installés ({installedPlugins.filter(p => p.target_dir === 'plugins').length})
							{:else if activeTab === 'mods'}Mods installés ({installedPlugins.filter(p => p.target_dir === 'mods').length})
							{:else if activeTab === 'resourcepacks'}Packs installés ({installedPlugins.filter(p => p.target_dir === 'resourcepacks').length})
							{:else}Data Packs installés ({installedPlugins.filter(p => p.target_dir === 'datapacks').length})
							{/if}
						</span>
					</button>

					<button
						type="button"
						class="subview-btn {subView === 'catalog' ? 'active' : ''}"
						onclick={() => (subView = 'catalog')}
					>
						<Store size={15} />
						<span>Explorer le Catalogue Modrinth</span>
					</button>
				</div>
			</div>

			{#if activeTab === 'plugins' || activeTab === 'mods'}
				<ToolsSyncPanel onToast={addToast} />
			{/if}

			{#if subView === 'installed'}
				<InstalledPluginsList
					plugins={installedPlugins.filter(p => p.target_dir === activeTab)}
					loading={isLoadingInstalled}
					initialTargetDir={activeTab}
					lockFolderFilter={true}
					onToggle={handleToggle}
					onDelete={handleDelete}
					onRefresh={loadInstalledPlugins}
					onToast={addToast}
					{serverResourcePack}
					onResourcePackChanged={(newRp) => (serverResourcePack = newRp)}
				/>
			{:else}
				<ModrinthCatalogBrowser
					initialProjectType={activeTab === 'plugins' ? 'plugin' : activeTab === 'mods' ? 'mod' : activeTab === 'resourcepacks' ? 'resourcepack' : 'datapack'}
					lockProjectType={true}
					{serverEngine}
					{serverVersion}
					onInstall={handleInstall}
				/>
			{/if}

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

	<!-- Updates & Compatibility Modal -->
	{#if updatesModalOpen && updatesReport}
		<div class="modal-backdrop" onclick={() => (updatesModalOpen = false)} role="presentation">
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<div class="modal-card card shadow-xl updates-modal-card" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
				<div class="modal-header">
					<div class="title-with-icon">
						<div class="modal-icon-box icon-blue">
							<RefreshCw size={20} />
						</div>
						<div>
							<h3>Rapport de Compatibilité & Mises à Jour</h3>
							<span class="modal-sub">Moteur cible : {updatesReport.engine} ({updatesReport.game_version})</span>
						</div>
					</div>
					<button class="btn btn-ghost btn-icon btn-sm" onclick={() => (updatesModalOpen = false)} disabled={isApplyingUpdates}>
						<X size={18} />
					</button>
				</div>

				<div class="modal-body updates-modal-body">
					<!-- Overview Statistics -->
					<div class="updates-stats-row">
						<div class="update-stat-chip chip-green">
							<CheckCircle2 size={16} />
							<span>{updatesReport.up_to_date_count} à jour</span>
						</div>
						<div class="update-stat-chip chip-amber">
							<DownloadCloud size={16} />
							<span>{updatesReport.updates_available_count} mise(s) à jour</span>
						</div>
						{#if updatesReport.incompatible_count > 0}
							<div class="update-stat-chip chip-red">
								<AlertTriangle size={16} />
								<span>{updatesReport.incompatible_count} incompatible(s)</span>
							</div>
						{/if}
					</div>

					<!-- Incompatible Warning Notice -->
					{#if updatesReport.incompatible_count > 0}
						<div class="alert-banner alert-warning">
							<AlertTriangle size={18} />
							<div>
								<strong>Attention :</strong> {updatesReport.incompatible_count} extension(s) n'ont pas de version compatible avec <strong>{updatesReport.game_version}</strong> sur Modrinth. Elles peuvent causer des erreurs au démarrage si vous changez de version.
							</div>
						</div>
					{/if}

					<!-- Addons List -->
					<div class="updates-list-container">
						{#each updatesReport.items as item (item.filename)}
							<div class="update-list-item">
								<div class="update-item-info">
									<div class="item-header-row">
										<span class="item-title">{item.name}</span>
										<code class="item-file">{item.filename}</code>
										{#if item.status === 'up_to_date'}
											<span class="badge badge-success badge-sm">À jour</span>
										{:else if item.status === 'update_available'}
											<span class="badge badge-amber badge-sm">MAJ disponible</span>
										{:else if item.status === 'incompatible'}
											<span class="badge badge-danger badge-sm">Non disponible en {updatesReport.game_version}</span>
										{:else}
											<span class="badge badge-secondary badge-sm">Inconnu (Modrinth)</span>
										{/if}
									</div>

									<div class="item-version-diff">
										<span class="ver-curr">v{item.current_version}</span>
										{#if item.status === 'update_available' && item.latest_version_number}
											<span class="ver-arrow">→</span>
											<span class="ver-next font-bold text-green">v{item.latest_version_number}</span>
										{/if}
									</div>
								</div>

								{#if item.status === 'update_available'}
									<button
										type="button"
										class="btn btn-primary btn-sm"
										disabled={isApplyingUpdates}
										onclick={() => handleApplyUpdates([item])}
									>
										<DownloadCloud size={14} />
										<span>Mettre à jour</span>
									</button>
								{/if}
							</div>
						{/each}
					</div>
				</div>

				<div class="modal-footer">
					<button class="btn btn-secondary" onclick={() => (updatesModalOpen = false)} disabled={isApplyingUpdates}>
						Fermer
					</button>
					{#if updatesReport.updates_available_count > 0}
						<button
							type="button"
							class="btn btn-primary {isApplyingUpdates ? 'btn-loading' : ''}"
							disabled={isApplyingUpdates}
							onclick={() => handleApplyUpdates(updatesReport.items.filter(i => i.status === 'update_available'))}
						>
							{#if !isApplyingUpdates}
								<DownloadCloud size={16} />
							{/if}
							<span>Tout mettre à jour ({updatesReport.updates_available_count})</span>
						</button>
					{/if}
				</div>
			</div>
		</div>
	{/if}

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
						<div class="toast-msg">{toast.message}</div>
					</div>

					<button class="toast-close" onclick={() => removeToast(toast.id)}>
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
		gap: var(--space-6);
	}

	/* Header */
	.page-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.header-main {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
	}
	.title-with-icon {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.page-icon-box {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg, rgba(59, 130, 246, 0.12));
		border: 1px solid var(--accent-blue-border, rgba(59, 130, 246, 0.25));
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text, #60a5fa);
	}
	.page-title {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		margin: 0;
	}
	.page-description {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		margin: 0;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.engine-badge-box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-input);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}
	.engine-indicator-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: #22c55e;
		box-shadow: 0 0 6px rgba(34, 197, 94, 0.6);
	}
	.check-updates-btn {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	/* Tabs Bar */
	.tabs-nav-container {
		display: flex;
		overflow-x: auto;
		gap: var(--space-2);
		border-bottom: 1px solid var(--border);
		padding-bottom: var(--space-1);
	}
	.tab-btn {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-4);
		border: none;
		background: none;
		color: var(--text-secondary);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		border-bottom: 2px solid transparent;
		transition: color var(--transition-fast), border-color var(--transition-fast);
		white-space: nowrap;
	}
	.tab-btn:hover {
		color: var(--text-primary);
	}
	.tab-btn:active {
		transform: scale(0.97);
	}
	.tab-btn.active {
		color: var(--accent-blue-text);
		border-bottom-color: var(--accent-blue);
	}
	.tab-badge {
		font-size: 11px;
		font-family: var(--font-mono);
		padding: 1px 6px;
		border-radius: var(--radius-badge);
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.08));
	}
	.badge-pill {
		border-radius: var(--radius-badge);
		padding: 2px 7px;
	}

	/* SubView Segmented Switcher */
	.subview-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--space-4);
	}
	.subview-segmented {
		display: flex;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		padding: 3px;
		gap: 2px;
	}
	.subview-btn {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-4);
		border: none;
		background: none;
		border-radius: calc(var(--radius-input) - 2px);
		color: var(--text-secondary);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
	}
	.subview-btn:hover {
		color: var(--text-primary);
	}
	.subview-btn:active {
		transform: scale(0.97);
	}
	.subview-btn.active {
		background-color: var(--accent-blue-bg, rgba(59, 130, 246, 0.12));
		color: var(--accent-blue-text, #60a5fa);
		font-weight: var(--font-weight-semibold);
	}

	/* Updates Modal */
	.updates-modal-card {
		max-width: 680px;
	}
	.updates-modal-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		max-height: 60vh;
		overflow-y: auto;
	}
	.updates-stats-row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.update-stat-chip {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-input);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
	}
	.chip-green { background-color: rgba(34, 197, 94, 0.12); color: #4ade80; }
	.chip-amber { background-color: rgba(245, 158, 11, 0.12); color: #fbbf24; }
	.chip-red { background-color: rgba(239, 68, 68, 0.12); color: #f87171; }

	.updates-list-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.update-list-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3);
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.02));
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		gap: var(--space-3);
	}
	.update-item-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}
	.item-header-row {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.item-title {
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
	}
	.item-file {
		font-size: 11px;
		color: var(--text-muted);
	}
	.item-version-diff {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}
	.ver-arrow {
		color: var(--text-muted);
	}

	/* Toast Container */
	.toast-container {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		max-width: 380px;
	}
	.toast-item {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-card);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		box-shadow: var(--shadow-lg);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
	}
	.toast-success {
		border-color: var(--accent-green-border);
		background-color: var(--bg-surface);
	}
	.toast-success .toast-icon {
		color: var(--accent-green-text);
	}
	.toast-error {
		border-color: var(--danger-border);
		background-color: var(--bg-surface);
	}
	.toast-error .toast-icon {
		color: var(--danger-text);
	}
	.toast-info {
		border-color: var(--accent-blue-border);
		background-color: var(--bg-surface);
	}
	.toast-info .toast-icon {
		color: var(--accent-blue-text);
	}
	.toast-body { flex: 1; }
	.toast-title { font-weight: var(--font-weight-semibold); margin-bottom: 2px; }
	.toast-msg { font-size: var(--font-size-xs); color: var(--text-secondary); }
	.toast-close {
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		padding: 0;
	}

	/* Modal generic backdrop */
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.7);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal);
		padding: var(--space-4);
	}
	.modal-card {
		width: 100%;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-modal, 12px);
		overflow: hidden;
	}
	.modal-header {
		padding: var(--space-4);
		display: flex;
		align-items: center;
		justify-content: space-between;
		border-bottom: 1px solid var(--border);
	}
	.modal-sub {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}
	.modal-footer {
		padding: var(--space-4);
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-3);
		border-top: 1px solid var(--border);
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.02));
	}
</style>
