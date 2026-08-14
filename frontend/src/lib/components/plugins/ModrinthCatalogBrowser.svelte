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
		Layers
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
		lockProjectType = false
	} = $props();

	// Search & Filter States
	let searchQuery = $state('');
	let selectedLoader = $state('all'); // 'all', 'paper', 'purpur', 'fabric', 'forge', 'neoforge', 'quilt'
	let selectedGameVersion = $state('all');
	let selectedProjectType = $state(initialProjectType); // 'all', 'plugin', 'mod'
	let selectedSort = $state('downloads'); // 'relevance', 'downloads', 'follows', 'newest', 'updated'

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
	let targetDir = $state('plugins'); // 'plugins' | 'mods'
	let isInstalling = $state(false);
	let installError = $state(null);

	// Comprehensive Minecraft Game Versions up to latest 26.3 Snapshot 7 & 26.2 release
	const gameVersionOptions = [
		'26.3 (Snapshot 7)',
		'26.3',
		'26.2',
		'26.1',
		'26.0',
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
		'1.19.1',
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
		fetchModrinthCatalog();
		try {
			const res = await apiGet('/api/server/engine');
			if (res) {
				if (Array.isArray(res.release_versions) && res.release_versions.length > 0) {
					releaseVersions = res.release_versions;
				}
				if (Array.isArray(res.snapshot_versions) && res.snapshot_versions.length > 0) {
					snapshotVersions = res.snapshot_versions;
				}
			}
		} catch (e) {
			console.log('Using default version list:', e);
		}
	});

	async function fetchModrinthCatalog() {
		isSearching = true;
		searchError = null;

		try {
			const loaderParam = selectedLoader === 'all' ? '' : selectedLoader;
			const versionParam = selectedGameVersion === 'all' ? '' : selectedGameVersion;
			const typeParam = selectedProjectType === 'all' ? '' : selectedProjectType;
			const sortParam = selectedSort;

			const url = `/api/plugins/search?query=${encodeURIComponent(searchQuery)}&loader=${encodeURIComponent(loaderParam)}&game_version=${encodeURIComponent(versionParam)}&project_type=${encodeURIComponent(typeParam)}&sort=${encodeURIComponent(sortParam)}`;
			
			const results = await apiGet(url);
			projects = Array.isArray(results) ? results : [];
		} catch (err) {
			console.error('Failed to search Modrinth catalog:', err);
			searchError = err.message || 'Failed to search Modrinth catalog. Check internet connectivity.';
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
		if (count >= 1_000) return (count / 1_000).toFixed(1) + 'K';
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
				selectedVersionId = versions[0].id;
			}
		} catch (err) {
			console.error('Failed to load project versions:', err);
			versionFetchError = err.message || 'Could not load project versions from Modrinth.';
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
			installError = err.message || 'Failed to download and install extension.';
		} finally {
			isInstalling = false;
		}
	}

	function formatPublishDate(dateStr) {
		if (!dateStr) return '';
		try {
			return new Date(dateStr).toLocaleDateString('en-US', {
				year: 'numeric',
				month: 'short',
				day: 'numeric'
			});
		} catch {
			return dateStr;
		}
	}

	function handleTargetDirKeydown(e) {
		if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
			e.preventDefault();
			targetDir = targetDir === 'plugins' ? 'mods' : 'plugins';
			requestAnimationFrame(() => {
				e.currentTarget?.querySelector?.('.segment-btn.active')?.focus();
			});
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
						placeholder="Search thousands of plugins & mods on Modrinth (e.g. WorldEdit, EssentialsX, Lithium)..."
						bind:value={searchQuery}
						aria-label="Search Modrinth catalog"
					/>
					{#if searchQuery}
						<button
							type="button"
							class="btn btn-ghost btn-icon btn-sm clear-search-btn"
							onclick={() => {
								searchQuery = '';
								fetchModrinthCatalog();
							}}
							title="Clear search"
						>
							<X size={14} />
						</button>
					{/if}
				</div>

				<button type="submit" class="btn btn-primary search-submit-btn {isSearching ? 'btn-loading' : ''}" disabled={isSearching}>
					{#if !isSearching}
						<Search size={16} />
					{/if}
					<span>Search</span>
				</button>
			</div>

			<!-- Filter Selectors Row -->
			<div class="filters-row">
				<!-- Loader Filter -->
				<div class="filter-item">
					<span class="filter-label">Loader Type:</span>
					<select
						class="select select-sm"
						bind:value={selectedLoader}
						onchange={fetchModrinthCatalog}
						aria-label="Filter by loader type"
					>
						<option value="all">All Loaders</option>
						<option value="paper">Paper / Spigot</option>
						<option value="purpur">Purpur</option>
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
						releaseVersions={releaseVersions.length > 0 ? releaseVersions : gameVersionOptions}
						{snapshotVersions}
						recommendedVersions={['26.2', '1.21.4', '1.20.4', '1.20.1', '1.16.5']}
						label="MC Version :"
						onSelect={(newVer) => {
							selectedGameVersion = newVer;
							fetchModrinthCatalog();
						}}
					/>
				</div>

				<!-- Sort By Selector -->
				<div class="filter-item">
					<span class="filter-label">Sort By:</span>
					<select
						class="select select-sm"
						bind:value={selectedSort}
						onchange={fetchModrinthCatalog}
						aria-label="Sort by"
					>
						<option value="downloads">Most Downloaded</option>
						<option value="relevance">Relevance</option>
						<option value="follows">Most Followed</option>
						<option value="newest">Recently Added</option>
						<option value="updated">Recently Updated</option>
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
			<p>Searching Modrinth catalog for server extensions...</p>
		</div>
	{:else if projects.length === 0}
		<div class="catalog-empty-state card">
			<div class="card-body empty-body">
				<Sparkles size={48} class="empty-sparkle-icon" />
				<h3>No Modrinth Extensions Found</h3>
				<p>Try searching for a different keyword or relaxing your filter dropdowns (Loader / Version).</p>
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
								{project.author || 'Unknown'}
							</span>
						</div>
						<span class="badge {project.project_type === 'mod' ? 'badge-blue' : 'badge-success'} type-badge">
							{project.project_type ? project.project_type.toUpperCase() : 'PLUGIN'}
						</span>
					</div>

					<div class="project-body">
						<p class="project-description">
							{project.description || 'No description provided for this extension.'}
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
						<div class="downloads-info" title="Total Modrinth Downloads">
							<Download size={14} />
							<span>{formatDownloads(project.downloads)} downloads</span>
						</div>

						<button
							class="btn btn-primary btn-sm install-trigger-btn"
							onclick={() => openInstallModal(project)}
						>
							<Download size={14} />
							<span>1-Click Install</span>
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Install Modal -->
{#if installModalOpen && selectedProject}
	<div
		class="modal-backdrop"
		onclick={closeInstallModal}
		aria-hidden="true"
	>
		<div
			class="modal install-modal"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="install-modal-title"
		>
			<div class="modal-header">
				<div class="modal-title-row">
					{#if selectedProject.icon_url}
						<img src={selectedProject.icon_url} alt="" class="modal-project-icon" loading="lazy" decoding="async" />
					{:else}
						<Package size={22} class="text-accent" />
					{/if}
					<h3 id="install-modal-title" class="modal-title">Install {selectedProject.title}</h3>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeInstallModal} disabled={isInstalling} aria-label="Close modal">
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				{#if installError}
					<div class="alert-banner alert-danger mb-4">
						<AlertCircle size={18} />
						<span>{installError}</span>
					</div>
				{/if}

				<!-- Target Directory Choice Segmented Switch -->
				<div class="form-group">
					<label id="target-folder-label" class="label label-required">Destination Directory</label>
					<div
						id="target-folder-select"
						class="segmented-control"
						role="radiogroup"
						aria-labelledby="target-folder-label"
						onkeydown={handleTargetDirKeydown}
					>
						<button
							type="button"
							class="segment-btn {targetDir === 'plugins' ? 'active' : ''}"
							role="radio"
							aria-checked={targetDir === 'plugins'}
							tabindex={targetDir === 'plugins' ? 0 : -1}
							onclick={() => (targetDir = 'plugins')}
						>
							<HardDrive size={14} />
							<span>Server Plugins (/plugins)</span>
						</button>
						<button
							type="button"
							class="segment-btn {targetDir === 'mods' ? 'active' : ''}"
							role="radio"
							aria-checked={targetDir === 'mods'}
							tabindex={targetDir === 'mods' ? 0 : -1}
							onclick={() => (targetDir = 'mods')}
						>
							<Boxes size={14} />
							<span>Server Mods (/mods)</span>
						</button>
					</div>
				</div>

				<!-- Version Selection -->
				<div class="form-group">
					<label for="version-select" class="label label-required">Select Extension Version</label>
					{#if isLoadingVersions}
						<div class="versions-loading">
							<Loader2 size={18} class="spinner" />
							<span>Fetching compatible release versions...</span>
						</div>
					{:else if versionFetchError}
						<div class="version-error-box">
							<AlertCircle size={16} />
							<span>{versionFetchError}</span>
						</div>
					{:else if versions.length === 0}
						<p class="no-versions-text">No downloadable releases found for this project.</p>
					{:else}
						<select id="version-select" class="select" bind:value={selectedVersionId}>
							{#each versions as ver}
								<option value={ver.id}>
									v{ver.version_number || ver.name} ({ver.version_type || 'release'}) - {formatPublishDate(ver.date_published)}
								</option>
							{/each}
						</select>

						<!-- Selected Version Summary Card -->
						{#if selectedVersionId}
							{@const currentVer = versions.find((v) => v.id === selectedVersionId)}
							{#if currentVer}
								<div class="version-summary-card">
									<div class="ver-summary-row">
										<span class="ver-label"><Calendar size={12} /> Published:</span>
										<span class="ver-val">{formatPublishDate(currentVer.date_published)}</span>
									</div>
									<div class="ver-summary-row">
										<span class="ver-label"><Layers size={12} /> Loaders:</span>
										<span class="ver-val">{currentVer.loaders?.join(', ') || 'Paper/Spigot'}</span>
									</div>
									<div class="ver-summary-row">
										<span class="ver-label"><Check size={12} /> Game Versions:</span>
										<span class="ver-val">{currentVer.game_versions?.slice(0, 5).join(', ') || '1.20.x'}</span>
									</div>
								</div>
							{/if}
						{/if}
					{/if}
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeInstallModal} disabled={isInstalling}>
					Cancel
				</button>
				<button
					class="btn btn-primary {isInstalling ? 'btn-loading' : ''}"
					onclick={confirmInstall}
					disabled={isInstalling || !selectedVersionId}
				>
					{#if !isInstalling}
						<Download size={16} />
					{/if}
					<span>Install Now</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.modrinth-catalog-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	/* Search Filter Card */
	.search-filter-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
	}

	.search-filter-body {
		padding: var(--space-5);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.search-main-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.search-input-wrapper {
		position: relative;
		flex: 1;
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
		padding-left: var(--space-8);
		padding-right: var(--space-8);
		height: 42px;
	}

	.clear-search-btn {
		position: absolute;
		right: 4px;
		color: var(--text-muted);
	}

	.search-submit-btn {
		height: 42px;
		padding: 0 var(--space-6);
	}

	.filters-row {
		display: flex;
		align-items: center;
		gap: var(--space-6);
		flex-wrap: wrap;
	}

	.filter-item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.filter-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		white-space: nowrap;
	}

	/* Alert Banners */
	.alert-banner {
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-btn);
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--font-size-sm);
	}

	.alert-danger {
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
	}

	.mb-4 {
		margin-bottom: var(--space-4);
	}

	/* Catalog States */
	.catalog-loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-16) var(--space-6);
		gap: var(--space-4);
		color: var(--text-muted);
	}

	.catalog-empty-state {
		text-align: center;
	}

	.empty-body {
		padding: var(--space-12) var(--space-6);
	}

	.empty-sparkle-icon {
		color: var(--text-muted);
		opacity: 0.5;
		margin-bottom: var(--space-3);
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
		height: 100%;
		transition: border-color var(--transition-fast), transform var(--transition-fast);
	}

	.project-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px -4px rgba(0, 0, 0, 0.4);
	}

	.project-header {
		padding: var(--space-4) var(--space-4) var(--space-2) var(--space-4);
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
	}

	.project-icon-box {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-input);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		flex-shrink: 0;
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
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
	}

	.project-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.project-author {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.type-badge {
		flex-shrink: 0;
		font-size: 10px;
		padding: 1px 6px;
	}

	.project-body {
		padding: var(--space-2) var(--space-4) var(--space-4) var(--space-4);
		flex: 1;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		gap: var(--space-3);
	}

	.project-description {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		display: -webkit-box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
		line-height: 1.45;
	}

	.categories-tags {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-1);
	}

	.tag-pill {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: var(--radius-sm);
		background-color: var(--bg-base);
		color: var(--text-muted);
		border: 1px solid var(--border-subtle);
	}

	.project-footer {
		padding: var(--space-3) var(--space-4);
		background-color: rgba(0, 0, 0, 0.15);
		border-top: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.downloads-info {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.install-trigger-btn {
		gap: var(--space-1);
	}

	/* Modal Styling */
	.install-modal {
		max-width: 520px;
	}

	.modal-project-icon {
		width: 24px;
		height: 24px;
		border-radius: 4px;
		object-fit: cover;
	}

	.text-accent {
		color: var(--accent-blue-text);
	}

	.segmented-control {
		display: flex;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		padding: 2px;
		gap: 2px;
	}

	.segment-btn {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border: none;
		border-radius: var(--radius-sm);
		background: transparent;
		color: var(--text-muted);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.segment-btn:focus-visible {
		outline: 2px solid var(--accent-blue);
		outline-offset: -2px;
	}

	.segment-btn.active {
		background-color: var(--bg-elevated);
		color: var(--text-primary);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	.versions-loading {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		padding: var(--space-3);
	}

	.version-error-box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--danger-text);
		font-size: var(--font-size-xs);
	}

	.no-versions-text {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}

	.version-summary-card {
		margin-top: var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-input);
		padding: var(--space-3);
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.ver-summary-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
	}

	.ver-label {
		color: var(--text-muted);
		display: flex;
		align-items: center;
		gap: 4px;
		min-width: 110px;
	}

	.ver-val {
		color: var(--text-primary);
		font-weight: var(--font-weight-medium);
	}
</style>
