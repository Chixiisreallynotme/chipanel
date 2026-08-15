<script>
	import { onMount } from 'svelte';
	import { apiGet } from '$lib/api/client.js';
	import VersionPickerButton from '$lib/components/common/VersionPickerButton.svelte';
	import {
		Search,
		Download,
		Package,
		Boxes,
		Filter,
		Loader2,
		X,
		User,
		ExternalLink,
		Sparkles,
		Check,
		AlertCircle,
		HardDrive,
		Calendar,
		Layers,
		Palette,
		Scroll,
		Cpu
	} from 'lucide-svelte';

	/**
	 * @typedef {Object} ModrinthProject
	 * @property {string} project_id
	 * @property {string} project_type
	 * @property {string} slug
	 * @property {string} author
	 * @property {string} title
	 * @property {string} description
	 * @property {string[]} categories
	 * @property {number} downloads
	 * @property {string|null} icon_url
	 * @property {string|null} latest_version
	 * @property {string|null} license
	 */

	/**
	 * @typedef {Object} ModrinthVersion
	 * @property {string} id
	 * @property {string} project_id
	 * @property {string} name
	 * @property {string} version_number
	 * @property {string|null} changelog
	 * @property {string} date_published
	 * @property {number} downloads
	 * @property {string} version_type
	 * @property {string[]} game_versions
	 * @property {string[]} loaders
	 */

	let {
		onInstall = async () => {},
		initialProjectType = 'all',
		lockProjectType = false,
		serverEngine = '',
		serverVersion = ''
	} = $props();

	// Search & Filter States
	let searchQuery = $state('');
	let selectedLoader = $state('all'); // 'all', 'paper', 'purpur', 'spigot', 'fabric', 'forge', 'neoforge', 'quilt'
	let selectedGameVersion = $state('all');
	let selectedProjectType = $state(initialProjectType); // 'all', 'plugin', 'mod', 'resourcepack', 'datapack'
	let selectedSort = $state('downloads'); // 'relevance', 'downloads', 'follows', 'newest', 'updated'
	let autoFilterServer = $state(true);

	let projects = $state([]);
	let isSearching = $state(false);
	let searchError = $state(null);

	let releaseVersions = $state([]);
	let snapshotVersions = $state([]);

	// Modrinth Install Modal States
	let installModalOpen = $state(false);
	let selectedProject = $state(null);
	let versions = $state([]);
	let isLoadingVersions = $state(false);
	let versionFetchError = $state(null);
	let selectedVersionId = $state('');
	let targetDir = $state('plugins'); // 'plugins' | 'mods' | 'resourcepacks' | 'datapacks'
	let isInstalling = $state(false);
	let installError = $state(null);

	// Standard Clean Minecraft Game Versions
	const standardGameVersions = [
		'1.21.4',
		'1.21.3',
		'1.21.1',
		'1.21',
		'1.20.6',
		'1.20.4',
		'1.20.2',
		'1.20.1',
		'1.20',
		'1.19.4',
		'1.19.3',
		'1.19.2',
		'1.19',
		'1.18.2',
		'1.18.1',
		'1.17.1',
		'1.16.5',
		'1.15.2',
		'1.14.4',
		'1.13.2',
		'1.12.2',
		'1.11.2',
		'1.10.2',
		'1.9.4',
		'1.8.9',
		'1.7.10'
	];

	// Initial Search on mount
	onMount(async () => {
		try {
			const res = await apiGet('/api/server/engine');
			if (res) {
				if (Array.isArray(res.release_versions) && res.release_versions.length > 0) {
					releaseVersions = res.release_versions;
				}
				if (Array.isArray(res.snapshot_versions) && res.snapshot_versions.length > 0) {
					snapshotVersions = res.snapshot_versions;
				}

				// Auto-align default filter with server engine if not explicitly overridden
				if (res.current_version && selectedGameVersion === 'all') {
					selectedGameVersion = res.current_version;
				}
				if (res.current_type && selectedLoader === 'all') {
					selectedLoader = engineToLoader(res.current_type);
				}
			}
		} catch (e) {
			console.log('Utilisation des versions par défaut:', e);
		}

		fetchModrinthCatalog();
	});

	function engineToLoader(engine) {
		const e = (engine || '').toUpperCase();
		if (e === 'PURPUR' || e === 'PAPER') return 'paper';
		if (e === 'SPIGOT' || e === 'BUKKIT') return 'spigot';
		if (e === 'FABRIC') return 'fabric';
		if (e === 'FORGE') return 'forge';
		if (e === 'NEOFORGE') return 'neoforge';
		if (e === 'QUILT') return 'quilt';
		return 'all';
	}

	async function fetchModrinthCatalog() {
		isSearching = true;
		searchError = null;

		try {
			const loaderParam = selectedLoader === 'all' ? '' : selectedLoader;
			const versionParam = selectedGameVersion === 'all' ? '' : selectedGameVersion;
			const typeParam = selectedProjectType === 'all' ? '' : selectedProjectType;
			const sortParam = selectedSort;

			const url = `/api/plugins/search?query=${encodeURIComponent(searchQuery)}&loader=${encodeURIComponent(loaderParam)}&game_version=${encodeURIComponent(versionParam)}&project_type=${encodeURIComponent(typeParam)}&sort=${encodeURIComponent(sortParam)}&auto_filter=false`;

			const results = await apiGet(url);
			projects = Array.isArray(results) ? results : [];
		} catch (err) {
			console.error('Échec de recherche Modrinth:', err);
			searchError = err.message || 'Impossible de joindre le catalogue Modrinth. Vérifiez la connexion Internet.';
			projects = [];
		} finally {
			isSearching = false;
		}
	}

	function handleSearchSubmit(e) {
		e?.preventDefault();
		fetchModrinthCatalog();
	}

	// Format Downloads Count
	function formatDownloads(count) {
		if (!count) return '0';
		if (count >= 1_000_000) return (count / 1_000_000).toFixed(1) + 'M';
		if (count >= 1_000) return (count / 1_000).toFixed(1) + 'k';
		return count.toLocaleString();
	}

	// Open Install Modal for a Project
	async function openInstallModal(project) {
		selectedProject = project;
		installModalOpen = true;
		versions = [];
		selectedVersionId = '';
		versionFetchError = null;
		installError = null;

		// Determine intelligent default target_dir
		const type = (project.project_type || '').toLowerCase();
		const categories = (project.categories || []).map((c) => c.toLowerCase());

		if (type === 'resourcepack' || categories.includes('resourcepack')) {
			targetDir = 'resourcepacks';
		} else if (type === 'datapack' || categories.includes('datapack')) {
			targetDir = 'datapacks';
		} else if (
			type === 'mod' ||
			categories.includes('fabric') ||
			categories.includes('forge') ||
			categories.includes('neoforge') ||
			categories.includes('quilt') ||
			selectedLoader === 'fabric' ||
			selectedLoader === 'forge'
		) {
			targetDir = 'mods';
		} else {
			targetDir = 'plugins';
		}

		// Fetch project versions from backend
		isLoadingVersions = true;
		try {
			const fetchedVersions = await apiGet(`/api/plugins/versions?project_id=${encodeURIComponent(project.project_id)}`);
			versions = Array.isArray(fetchedVersions) ? fetchedVersions : [];
			if (versions.length > 0) {
				// Pick the version matching server version or the latest
				const matchingVer = versions.find((v) =>
					selectedGameVersion !== 'all' && (v.game_versions || []).includes(selectedGameVersion)
				);
				selectedVersionId = matchingVer ? matchingVer.id : versions[0].id;
			}
		} catch (err) {
			console.error('Failed to load project versions:', err);
			versionFetchError = err.message || 'Impossible de récupérer les versions depuis Modrinth.';
		} finally {
			isLoadingVersions = false;
		}
	}

	function closeInstallModal() {
		if (isInstalling) return;
		installModalOpen = false;
		selectedProject = null;
		versions = [];
		selectedVersionId = '';
		installError = null;
	}

	async function confirmInstall() {
		if (!selectedProject || isInstalling) return;
		isInstalling = true;
		installError = null;

		try {
			await onInstall({
				projectId: selectedProject.project_id,
				versionId: selectedVersionId || undefined,
				targetDir: targetDir,
				projectTitle: selectedProject.title
			});
			closeInstallModal();
		} catch (err) {
			console.error('Install failed:', err);
			installError = err.message || 'Échec du téléchargement et de l\'installation.';
		} finally {
			isInstalling = false;
		}
	}

	function formatPublishDate(dateStr) {
		if (!dateStr) return '';
		try {
			return new Date(dateStr).toLocaleDateString('fr-FR', {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}

	function handleWindowKeydown(e) {
		if (e.key === 'Escape' && installModalOpen) {
			closeInstallModal();
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="modrinth-catalog-container">
	<!-- Catalog Search & Filter Form -->
	<form class="search-filter-card card" onsubmit={handleSearchSubmit}>
		<div class="search-filter-body">
			<!-- Search Bar -->
			<div class="search-main-row">
				<div class="search-input-wrapper">
					<Search size={18} class="search-icon" />
					<input
						type="text"
						class="input search-input"
						placeholder="Rechercher parmi des milliers de mods, plugins et packs Modrinth…"
						bind:value={searchQuery}
						aria-label="Rechercher dans le catalogue Modrinth"
					/>
					{#if searchQuery}
						<button
							type="button"
							class="btn btn-ghost btn-icon btn-sm clear-search-btn"
							onclick={() => {
								searchQuery = '';
								fetchModrinthCatalog();
							}}
							title="Effacer"
						>
							<X size={14} />
						</button>
					{/if}
				</div>

				<button type="submit" class="btn btn-primary search-submit-btn {isSearching ? 'btn-loading' : ''}" disabled={isSearching}>
					{#if !isSearching}
						<Search size={16} />
					{/if}
					<span>Rechercher</span>
				</button>
			</div>

			<!-- Filter Selectors Row -->
			<div class="filters-row">
				<!-- Loader Filter -->
				<div class="filter-item">
					<span class="filter-label">Chargeur :</span>
					<select
						class="select select-sm"
						bind:value={selectedLoader}
						onchange={fetchModrinthCatalog}
						aria-label="Filtrer par chargeur"
					>
						<option value="all">Tous les chargeurs</option>
						<option value="paper">Paper / Spigot / Purpur</option>
						<option value="fabric">Fabric</option>
						<option value="forge">Forge</option>
						<option value="neoforge">NeoForge</option>
						<option value="quilt">Quilt</option>
					</select>
				</div>

				<!-- Game Version Selector -->
				<div class="filter-item">
					<VersionPickerButton
						selectedVersion={selectedGameVersion}
						releaseVersions={releaseVersions.length > 0 ? releaseVersions : standardGameVersions}
						{snapshotVersions}
						recommendedVersions={['1.21.4', '1.20.4', '1.20.1', '1.16.5']}
						label="Version MC :"
						onSelect={(newVer) => {
							selectedGameVersion = newVer;
							fetchModrinthCatalog();
						}}
					/>
				</div>

				<!-- Sort By Selector -->
				<div class="filter-item">
					<span class="filter-label">Trier par :</span>
					<select
						class="select select-sm"
						bind:value={selectedSort}
						onchange={fetchModrinthCatalog}
						aria-label="Trier par"
					>
						<option value="downloads">Plus téléchargés</option>
						<option value="relevance">Pertinence</option>
						<option value="follows">Plus suivis</option>
						<option value="newest">Récents</option>
						<option value="updated">Dernière mise à jour</option>
					</select>
				</div>
			</div>
		</div>
	</form>

	<!-- Search Error Alert -->
	{#if searchError}
		<div class="alert-banner alert-danger">
			<AlertCircle size={18} />
			<span>{searchError}</span>
		</div>
	{/if}

	<!-- Catalog Grid Section -->
	{#if isSearching}
		<div class="catalog-loading-state">
			<Loader2 size={36} class="spinner" />
			<p>Recherche d'extensions sur Modrinth…</p>
		</div>
	{:else if projects.length === 0}
		<div class="catalog-empty-state card">
			<div class="card-body empty-body">
				<Sparkles size={48} class="empty-sparkle-icon" />
				<h3>Aucune extension Modrinth trouvée</h3>
				<p>Essayez un autre mot-clé ou élargissez vos filtres de chargeur et de version.</p>
			</div>
		</div>
	{:else}
		<div class="projects-grid">
			{#each projects as project (project.project_id)}
				<div class="card project-card">
					<div class="project-header">
						<div class="project-icon-box">
							{#if project.icon_url}
								<img src={project.icon_url} alt={project.title} class="project-icon-img" loading="lazy" decoding="async" />
							{:else}
								<Package size={24} class="project-icon-fallback" />
							{/if}
						</div>
						<div class="project-title-meta">
							<h3 class="project-title" title={project.title}>{project.title}</h3>
							<span class="project-author">
								<User size={12} />
								{project.author || 'Inconnu'}
							</span>
						</div>
						<span class="badge {project.project_type === 'mod' ? 'badge-blue' : project.project_type === 'resourcepack' ? 'badge-amber' : 'badge-success'} type-badge">
							{project.project_type ? project.project_type.toUpperCase() : 'PLUGIN'}
						</span>
					</div>

					<div class="project-body">
						<p class="project-description">
							{project.description || 'Aucune description disponible pour cette extension.'}
						</p>

						{#if project.categories && project.categories.length > 0}
							<div class="categories-tags">
								{#each project.categories.slice(0, 4) as cat}
									<span class="tag-pill">{cat}</span>
								{/each}
							</div>
						{/if}
					</div>

					<div class="project-footer">
						<div class="project-stats">
							<span class="downloads-stat" title="Nombre total de téléchargements">
								<Download size={13} />
								{formatDownloads(project.downloads)}
							</span>
						</div>

						<div class="project-actions">
							<a
								href="https://modrinth.com/{project.project_type || 'project'}/{project.slug || project.project_id}"
								target="_blank"
								rel="noreferrer noopener"
								class="btn btn-ghost btn-icon btn-sm"
								title="Voir la page Modrinth officielle"
							>
								<ExternalLink size={15} />
							</a>

							<button
								type="button"
								class="btn btn-primary btn-sm install-btn"
								onclick={() => openInstallModal(project)}
							>
								<Download size={14} />
								<span>Installer</span>
							</button>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Modrinth Install Modal -->
{#if installModalOpen && selectedProject}
	<div class="modal-backdrop" onclick={closeInstallModal} role="presentation">
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="modal-card card shadow-xl" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
			<div class="modal-header">
				<div class="modal-project-title-box">
					{#if selectedProject.icon_url}
						<img src={selectedProject.icon_url} alt="" class="modal-project-img" />
					{:else}
						<Package size={22} />
					{/if}
					<div>
						<h3>Installer {selectedProject.title}</h3>
						<span class="modal-author-sub">par {selectedProject.author}</span>
					</div>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeInstallModal} disabled={isInstalling}>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body install-modal-body">
				<!-- Target Directory Choice -->
				<div class="form-group">
					<!-- svelte-ignore a11y_label_has_associated_control -->
					<label class="form-label">Dossier de destination :</label>
					<div class="segmented-control">
						<button
							type="button"
							class="segment-btn {targetDir === 'plugins' ? 'active' : ''}"
							onclick={() => (targetDir = 'plugins')}
						>
							<Package size={14} />
							<span>Plugins (/plugins)</span>
						</button>
						<button
							type="button"
							class="segment-btn {targetDir === 'mods' ? 'active' : ''}"
							onclick={() => (targetDir = 'mods')}
						>
							<Cpu size={14} />
							<span>Mods (/mods)</span>
						</button>
						<button
							type="button"
							class="segment-btn {targetDir === 'resourcepacks' ? 'active' : ''}"
							onclick={() => (targetDir = 'resourcepacks')}
						>
							<Palette size={14} />
							<span>Textures (/resourcepacks)</span>
						</button>
						<button
							type="button"
							class="segment-btn {targetDir === 'datapacks' ? 'active' : ''}"
							onclick={() => (targetDir = 'datapacks')}
						>
							<Scroll size={14} />
							<span>Data (/datapacks)</span>
						</button>
					</div>
				</div>

				<!-- Version Selector -->
				<div class="form-group">
					<label class="form-label" for="modrinth-version-select">Version à installer :</label>
					{#if isLoadingVersions}
						<div class="loading-inline">
							<Loader2 size={16} class="spinner" />
							<span>Chargement des versions Modrinth…</span>
						</div>
					{:else if versionFetchError}
						<div class="alert-inline alert-danger">
							<AlertCircle size={15} />
							<span>{versionFetchError}</span>
						</div>
					{:else if versions.length === 0}
						<p class="text-muted">Aucune version publiée trouvée pour ce projet.</p>
					{:else}
						<select id="modrinth-version-select" class="select select-full" bind:value={selectedVersionId}>
							{#each versions as ver (ver.id)}
								<option value={ver.id}>
									{ver.name || ver.version_number} — (MC {ver.game_versions?.join(', ') || 'N/A'}) [{ver.loaders?.join(', ') || 'N/A'}]
								</option>
							{/each}
						</select>
					{/if}
				</div>

				{#if installError}
					<div class="alert-banner alert-danger">
						<AlertCircle size={16} />
						<span>{installError}</span>
					</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeInstallModal} disabled={isInstalling}>
					Annuler
				</button>
				<button
					type="button"
					class="btn btn-primary {isInstalling ? 'btn-loading' : ''}"
					onclick={confirmInstall}
					disabled={isInstalling || isLoadingVersions || !selectedVersionId}
				>
					{#if !isInstalling}
						<Download size={16} />
					{/if}
					<span>Confirmer l'installation</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.modrinth-catalog-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	/* Search & Filter Header Card */
	.search-filter-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
	}
	.search-filter-body {
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.search-main-row {
		display: flex;
		gap: var(--space-2);
	}
	.search-input-wrapper {
		position: relative;
		flex: 1;
	}
	.search-icon {
		position: absolute;
		left: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-muted);
	}
	.search-input {
		padding-left: 2.2rem;
		padding-right: 2rem;
		width: 100%;
	}
	.clear-search-btn {
		position: absolute;
		right: 4px;
		top: 50%;
		transform: translateY(-50%);
	}
	.filters-row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-3);
		align-items: center;
	}
	.filter-item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.filter-label {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	/* Projects Grid */
	.projects-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: var(--space-4);
	}
	.project-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		transition: transform var(--transition-fast), border-color var(--transition-fast);
	}
	.project-card:hover {
		border-color: var(--accent-blue-border, rgba(59, 130, 246, 0.4));
		transform: translateY(-2px);
	}
	.project-header {
		padding: var(--space-4);
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		border-bottom: 1px solid var(--border);
	}
	.project-icon-box {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-input);
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.04));
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		overflow: hidden;
	}
	.project-icon-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}
	.project-icon-fallback {
		color: var(--text-muted);
	}
	.project-title-meta {
		flex: 1;
		min-width: 0;
	}
	.project-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		margin: 0 0 2px 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.project-author {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.type-badge {
		font-size: 10px;
		font-family: var(--font-mono);
	}

	.project-body {
		padding: var(--space-4);
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.project-description {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		line-height: 1.45;
		margin: 0;
		display: -webkit-box;
		-webkit-line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	.categories-tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		margin-top: auto;
	}
	.tag-pill {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: 4px;
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.06));
		color: var(--text-secondary);
	}

	.project-footer {
		padding: var(--space-3) var(--space-4);
		border-top: 1px solid var(--border);
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.02));
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.project-stats {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}
	.downloads-stat {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.project-actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	/* Modal styling */
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.7);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 999;
		padding: var(--space-4);
	}
	.modal-card {
		width: 100%;
		max-width: 540px;
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
	.modal-project-title-box {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.modal-project-img {
		width: 36px;
		height: 36px;
		border-radius: 6px;
		object-fit: cover;
	}
	.modal-author-sub {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}
	.install-modal-body {
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.form-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.form-label {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}
	.segmented-control {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-2);
	}
	.segment-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.04));
		color: var(--text-secondary);
		cursor: pointer;
		font-size: var(--font-size-xs);
		transition: all var(--transition-fast);
	}
	.segment-btn:hover {
		background-color: var(--bg-subtle-hover, rgba(255, 255, 255, 0.08));
	}
	.segment-btn.active {
		border-color: var(--accent-blue);
		background-color: var(--accent-blue-bg, rgba(59, 130, 246, 0.12));
		color: var(--accent-blue-text, #60a5fa);
		font-weight: var(--font-weight-semibold);
	}
	.select-full {
		width: 100%;
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
	.catalog-loading-state, .catalog-empty-state {
		padding: var(--space-10);
		text-align: center;
		color: var(--text-secondary);
	}
	.empty-sparkle-icon {
		color: var(--accent-blue);
		margin-bottom: var(--space-3);
	}
</style>
