<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import VersionPickerButton from '$lib/components/common/VersionPickerButton.svelte';
	import {
		Search,
		Download,
		Boxes,
		Filter,
		Loader2,
		X,
		User,
		Sparkles,
		Check,
		AlertCircle,
		Calendar,
		Layers,
		CheckCircle2,
		HardDrive,
		FileText,
		Tag
	} from 'lucide-svelte';

	/**
	 * @typedef {Object} ModpackVersion
	 * @property {string} id
	 * @property {string} version_number
	 * @property {string} game_version
	 * @property {string} [release_date]
	 */

	/**
	 * @typedef {Object} ModpackItem
	 * @property {string} id
	 * @property {string} title
	 * @property {string} author
	 * @property {'modrinth' | 'curseforge'} provider
	 * @property {string} loader
	 * @property {number} downloads
	 * @property {string} [thumbnail]
	 * @property {string} [icon_url]
	 * @property {string} summary
	 * @property {ModpackVersion[]} [versions]
	 * @property {string[]} [mods]
	 */

	let { onDeploy = async () => {} } = $props();

	// Controls state
	const provider = 'modrinth';
	let searchQuery = $state('');
	let selectedLoader = $state('all'); // 'all' | 'Fabric' | 'Forge' | 'NeoForge' | 'Quilt' | 'Purpur'
	let selectedGameVersion = $state('all');

	const gameVersionOptions = [
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

	// Modpacks list state
	let modpacks = $state([]);
	let isSearching = $state(false);
	let searchError = $state(null);

	let releaseVersions = $state([]);
	let snapshotVersions = $state([]);

	// Detail & Deploy Modal State
	let isModalOpen = $state(false);
	let selectedModpack = $state(null);
	let versions = $state([]);
	let isLoadingVersions = $state(false);
	let versionFetchError = $state(null);
	let selectedVersionId = $state('');
	let profileNameInput = $state('');
	let isDeploying = $state(false);
	let deployError = $state(null);

	// Rich fallback modpacks database (Modrinth only)
	const fallbackModpacks = [
		{
			id: 'fabulously-optimized',
			title: 'Fabulously Optimized',
			author: 'Fabulously Optimized Team',
			provider: 'modrinth',
			loader: 'Fabric',
			downloads: 4800000,
			thumbnail: '',
			summary: 'A simple modpack focused on performance improvements and bug fixes, making Minecraft run faster.',
			versions: [
				{ id: 'fo-5.8.0', version_number: '5.8.0', game_version: '1.20.4', release_date: '2026-06-10' }
			],
			mods: ['Sodium', 'Lithium', 'FerriteCore', 'Indium', 'Iris Shaders', 'Entity Culling', 'Krypton']
		},
		{
			id: 'cobblemon-official',
			title: 'Cobblemon Official Modpack',
			author: 'Cobblemon Team',
			provider: 'modrinth',
			loader: 'Fabric',
			downloads: 2900000,
			thumbnail: '',
			summary: 'The official modpack for Cobblemon, adding open-world Pokémon capturing and battling to Minecraft.',
			versions: [
				{ id: 'cobblemon-1.5', version_number: '1.5.0', game_version: '1.20.1', release_date: '2026-05-20' }
			],
			mods: ['Cobblemon', 'Architectury', 'Cloth Config', 'JEI / REI', 'Xaero Minimap']
		},
		{
			id: 'purpur-essentials-suite',
			title: 'Purpur Essentials & Performance Suite',
			author: 'ChiPanel Team',
			provider: 'modrinth',
			loader: 'Purpur',
			downloads: 940000,
			thumbnail: '',
			summary: 'Optimized Purpur paper-fork server configuration pack bundled with essential administrative plugins and performance tweaks.',
			versions: [
				{ id: 'purpur-1.20.4-r2', version_number: '1.20.4-R2', game_version: '1.20.4', release_date: '2026-07-12' }
			],
			mods: ['Purpur Core', 'EssentialsX', 'LuckPerms', 'Vault', 'WorldEdit', 'Chunky', 'Spark Profiler', 'ProtocolLib']
		}
	];

	onMount(async () => {
		fetchModpacks();
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

	async function fetchModpacks() {
		isSearching = true;
		searchError = null;

		try {
			const loaderParam = selectedLoader === 'all' ? '' : selectedLoader;
			const versionParam = selectedGameVersion === 'all' ? '' : selectedGameVersion;
			const queryParam = searchQuery.trim();
			const url = `/api/modpacks/search?provider=modrinth&query=${encodeURIComponent(queryParam)}&loader=${encodeURIComponent(loaderParam)}&game_version=${encodeURIComponent(versionParam)}`;

			const res = await apiGet(url);
			if (Array.isArray(res) && res.length > 0) {
				modpacks = res;
			} else {
				modpacks = filterFallbackModpacks('modrinth', queryParam, selectedLoader, selectedGameVersion);
			}
		} catch (err) {
			console.log('Using local catalog index:', err.message);
			modpacks = filterFallbackModpacks('modrinth', searchQuery.trim(), selectedLoader, selectedGameVersion);
		} finally {
			isSearching = false;
		}
	}

	function filterFallbackModpacks(prov, qParam, ldr, verParam) {
		const q = searchQuery.trim().toLowerCase();
		const ldrLower = ldr && ldr !== 'all' ? ldr.toLowerCase() : '';
		const verFilter = verParam && verParam !== 'all' ? verParam : '';

		return fallbackModpacks.filter((item) => {
			if (prov && item.provider !== prov) return false;
			if (ldrLower && item.loader.toLowerCase() !== ldrLower) return false;
			if (verFilter) {
				const hasVer = (item.versions || []).some((v) => v.game_version === verFilter);
				if (!hasVer) return false;
			}
			if (q) {
				const titleMatch = item.title.toLowerCase().includes(q);
				const summaryMatch = item.summary.toLowerCase().includes(q);
				const authorMatch = item.author.toLowerCase().includes(q);
				if (!titleMatch && !summaryMatch && !authorMatch) return false;
			}
			return true;
		});
	}

	function handleSearchSubmit(e) {
		e?.preventDefault();
		fetchModpacks();
	}

	function formatDownloads(count) {
		if (!count) return '0';
		if (count >= 1_000_000) return (count / 1_000_000).toFixed(1) + 'M';
		if (count >= 1_000) return (count / 1_000).toFixed(1) + 'K';
		return count.toLocaleString();
	}

	// Open Detail & Deploy Modal
	async function openDetailModal(modpack) {
		selectedModpack = modpack;
		isModalOpen = true;
		versions = modpack.versions || [];
		selectedVersionId = '';
		profileNameInput = `${modpack.title} Profile`;
		versionFetchError = null;
		deployError = null;

		// Fetch detailed versions from backend if needed
		isLoadingVersions = true;
		try {
			const fetchedVer = await apiGet(`/api/modpacks/versions?provider=${encodeURIComponent(modpack.provider)}&project_id=${encodeURIComponent(modpack.id)}`);
			if (Array.isArray(fetchedVer) && fetchedVer.length > 0) {
				versions = fetchedVer;
			}
		} catch {
			// keep embedded versions
		} finally {
			if (versions.length > 0) {
				selectedVersionId = versions[0].id;
			}
			isLoadingVersions = false;
		}
	}

	function closeModal() {
		if (isDeploying) return;
		isModalOpen = false;
		selectedModpack = null;
		versions = [];
		selectedVersionId = '';
		deployError = null;
	}

	async function confirmDeploy() {
		if (!selectedModpack || isDeploying) return;
		isDeploying = true;
		deployError = null;

		const currentVer = versions.find((v) => v.id === selectedVersionId) || versions[0] || { id: 'v1.0.0', version_number: '1.0.0', game_version: '1.20.4' };
		const finalProfileName = profileNameInput.trim() || `${selectedModpack.title} Profile`;

		try {
			await onDeploy({
				provider: selectedModpack.provider,
				modpackId: selectedModpack.id,
				modpackTitle: selectedModpack.title,
				versionId: currentVer.id,
				versionNumber: currentVer.version_number,
				gameVersion: currentVer.game_version,
				loader: selectedModpack.loader,
				profileName: finalProfileName,
				mods: selectedModpack.mods || []
			});
			closeModal();
		} catch (err) {
			console.error('Failed to deploy modpack:', err);
			deployError = err.message || 'Failed to deploy modpack to server.';
		} finally {
			isDeploying = false;
		}
	}

	function handleWindowKeydown(e) {
		if (e.key === 'Escape' && isModalOpen) {
			closeModal();
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="modpack-catalog-browser">
	<!-- Top Controls Card -->
	<form class="search-filter-card card" onsubmit={handleSearchSubmit}>
		<div class="search-filter-body">
			<!-- Row 1: Search input and submit -->
			<div class="search-main-row">
				<div class="search-input-wrapper">
					<Search size={18} class="search-icon" />
					<input
						type="text"
						class="input search-input"
						placeholder="Rechercher des modpacks sur Modrinth (ex: Fabulously Optimized, Cobblemon)..."
						bind:value={searchQuery}
						aria-label="Search modpacks"
					/>
					{#if searchQuery}
						<button
							type="button"
							class="btn btn-ghost btn-icon btn-sm clear-search-btn"
							onclick={() => {
								searchQuery = '';
								fetchModpacks();
							}}
							title="Clear search"
						>
							<X size={14} />
						</button>
					{/if}
				</div>

				<button
					type="submit"
					class="btn btn-primary search-submit-btn {isSearching ? 'btn-loading' : ''}"
					disabled={isSearching}
				>
					{#if !isSearching}
						<Search size={16} />
					{/if}
					<span>Search</span>
				</button>
			</div>

			<!-- Row 2: Loader & Version Filter -->
			<div class="filters-row">

				<!-- Loader Filter Dropdown ("Fabric", "Forge", "NeoForge", "Quilt", "Purpur") -->
				<div class="filter-item loader-filter">
					<span class="filter-label">Mod Loader:</span>
					<select
						class="select select-sm"
						bind:value={selectedLoader}
						onchange={fetchModpacks}
						aria-label="Filter by loader"
					>
						<option value="all">All Loaders</option>
						<option value="Fabric">Fabric</option>
						<option value="Forge">Forge</option>
						<option value="NeoForge">NeoForge</option>
						<option value="Quilt">Quilt</option>
						<option value="Purpur">Purpur</option>
					</select>
				</div>

				<!-- Game Version Selector -->
				<div class="filter-item version-filter">
					<VersionPickerButton
						selectedVersion={selectedGameVersion}
						releaseVersions={releaseVersions.length > 0 ? releaseVersions : gameVersionOptions}
						{snapshotVersions}
						recommendedVersions={['26.2', '1.21.4', '1.20.4', '1.20.1', '1.16.5']}
						label="MC Version :"
						onSelect={(newVer) => {
							selectedGameVersion = newVer;
							fetchModpacks();
						}}
					/>
				</div>
			</div>
		</div>
	</form>

	<!-- Search Error -->
	{#if searchError}
		<div class="alert-banner alert-danger">
			<AlertCircle size={18} />
			<span>{searchError}</span>
		</div>
	{/if}

	<!-- Modpack Grid -->
	{#if isSearching}
		<div class="catalog-loading-state">
			<Loader2 size={36} class="spinner" />
			<p>Searching {provider === 'modrinth' ? 'Modrinth' : 'CurseForge'} catalog for modpacks...</p>
		</div>
	{:else if modpacks.length === 0}
		<div class="catalog-empty-state card">
			<div class="card-body empty-body">
				<Boxes size={48} class="empty-icon" />
				<h3>No Modpacks Found</h3>
				<p>Try searching for a different keyword or switching between Modrinth & CurseForge providers.</p>
			</div>
		</div>
	{:else}
		<div class="modpacks-grid">
			{#each modpacks as pack (pack.id)}
				<div class="card modpack-card">
					<div class="modpack-header">
						<div class="modpack-thumb-box">
							{#if pack.thumbnail || pack.icon_url}
								<img
									src={pack.thumbnail || pack.icon_url}
									alt={pack.title}
									class="modpack-thumb-img"
									loading="lazy"
									decoding="async"
								/>
							{:else}
								<Boxes size={26} class="modpack-thumb-fallback" />
							{/if}
						</div>

						<div class="modpack-title-meta">
							<div class="badges-row">
								<span class="badge {pack.provider === 'modrinth' ? 'badge-blue' : 'badge-warning'} provider-badge">
									{pack.provider === 'modrinth' ? 'Modrinth' : 'CurseForge'}
								</span>
								<span class="badge badge-success loader-badge">
									{pack.loader}
								</span>
							</div>

							<h3 class="modpack-title" title={pack.title}>{pack.title}</h3>
							<span class="modpack-author">
								<User size={12} />
								by {pack.author}
							</span>
						</div>
					</div>

					<div class="modpack-body">
						<p class="modpack-summary">
							{pack.summary}
						</p>

						{#if pack.mods && pack.mods.length > 0}
							<div class="mods-preview-bar">
								<Tag size={12} class="tag-icon" />
								<span>{pack.mods.length} Mods: {pack.mods.slice(0, 3).join(', ')}...</span>
							</div>
						{/if}
					</div>

					<div class="modpack-footer">
						<div class="downloads-info" title="Total Downloads">
							<Download size={14} />
							<span>{formatDownloads(pack.downloads)} downloads</span>
						</div>

						<button
							class="btn btn-primary btn-sm deploy-btn"
							onclick={() => openDetailModal(pack)}
						>
							<Sparkles size={14} />
							<span>View Details & Deploy</span>
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Detail & Deploy Modal -->
{#if isModalOpen && selectedModpack}
	<div
		class="modal-backdrop"
		onclick={closeModal}
		aria-hidden="true"
	>
		<div
			class="modal detail-modal"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="modal-modpack-title"
		>
			<div class="modal-header">
				<div class="modal-header-main">
					{#if selectedModpack.thumbnail || selectedModpack.icon_url}
						<img
							src={selectedModpack.thumbnail || selectedModpack.icon_url}
							alt=""
							class="modal-thumb"
							loading="lazy"
							decoding="async"
						/>
					{:else}
						<div class="modal-thumb-fallback">
							<Boxes size={20} />
						</div>
					{/if}
					<div>
						<h3 id="modal-modpack-title" class="modal-title">{selectedModpack.title}</h3>
						<div class="modal-meta-row">
							<span class="modal-author">by {selectedModpack.author}</span>
							<span class="badge {selectedModpack.provider === 'modrinth' ? 'badge-blue' : 'badge-warning'}">
								{selectedModpack.provider === 'modrinth' ? 'Modrinth' : 'CurseForge'}
							</span>
							<span class="badge badge-success">{selectedModpack.loader}</span>
						</div>
					</div>
				</div>

				<button
					class="btn btn-ghost btn-icon btn-sm"
					onclick={closeModal}
					disabled={isDeploying}
					aria-label="Close modal"
				>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				{#if deployError}
					<div class="alert-banner alert-danger mb-4">
						<AlertCircle size={18} />
						<span>{deployError}</span>
					</div>
				{/if}

				<!-- Summary Section -->
				<div class="section-box">
					<h4 class="section-title">
						<FileText size={14} />
						<span>Modpack Summary</span>
					</h4>
					<p class="summary-text">{selectedModpack.summary}</p>
				</div>

				<!-- Version Selector -->
				<div class="form-group">
					<label for="modpack-version-select" class="label label-required">Select Modpack Version</label>
					{#if isLoadingVersions}
						<div class="versions-loading">
							<Loader2 size={16} class="spinner" />
							<span>Fetching release versions...</span>
						</div>
					{:else if versions.length === 0}
						<p class="no-versions-text">No downloadable releases available.</p>
					{:else}
						<select id="modpack-version-select" class="select" bind:value={selectedVersionId}>
							{#each versions as ver}
								<option value={ver.id}>
									v{ver.version_number} (MC {ver.game_version}){ver.release_date ? ` - ${ver.release_date}` : ''}
								</option>
							{/each}
						</select>
					{/if}
				</div>

				<!-- Profile Name Input -->
				<div class="form-group">
					<label for="profile-name-input" class="label label-required">Target Server Profile Name</label>
					<div class="input-with-icon">
						<HardDrive size={16} class="input-icon" />
						<input
							id="profile-name-input"
							type="text"
							class="input"
							placeholder="e.g. Fabulously Optimized Profile"
							bind:value={profileNameInput}
						/>
					</div>
					<span class="field-hint">A new profile will be created with this name when deployed.</span>
				</div>

				<!-- Mods Included List -->
				{#if selectedModpack.mods && selectedModpack.mods.length > 0}
					<div class="section-box">
						<div class="section-header-row">
							<h4 class="section-title">
								<Layers size={14} />
								<span>Mods Included ({selectedModpack.mods.length})</span>
							</h4>
						</div>
						<div class="mods-chips-container">
							{#each selectedModpack.mods as modItem}
								<span class="mod-chip">
									<Check size={12} class="check-icon" />
									<span>{modItem}</span>
								</span>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeModal} disabled={isDeploying}>
					Cancel
				</button>
				<button
					class="btn btn-primary {isDeploying ? 'btn-loading' : ''}"
					onclick={confirmDeploy}
					disabled={isDeploying || !profileNameInput.trim()}
				>
					{#if !isDeploying}
						<Sparkles size={16} />
					{/if}
					<span>Deploy Modpack</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.modpack-catalog-browser {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

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
		gap: var(--space-3);
	}

	.filter-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		white-space: nowrap;
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
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 12px;
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--text-muted);
		background: transparent;
		border: none;
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast);
	}

	.segment-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
	}

	.segment-btn:focus-visible {
		outline: 2px solid var(--accent-blue);
		outline-offset: -2px;
	}

	.segment-btn.active {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border: 1px solid var(--accent-blue-border);
		font-weight: var(--font-weight-semibold);
	}

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

	.catalog-loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-16) var(--space-6);
		gap: var(--space-4);
		color: var(--text-muted);
	}

	.spinner {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.catalog-empty-state {
		text-align: center;
	}

	.empty-body {
		padding: var(--space-12) var(--space-6);
	}

	.empty-icon {
		color: var(--text-muted);
		opacity: 0.4;
		margin-bottom: var(--space-3);
	}

	.modpacks-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: var(--space-4);
	}

	.modpack-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		display: flex;
		flex-direction: column;
		height: 100%;
		transition: border-color var(--transition-fast), transform var(--transition-fast);
	}

	.modpack-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px -4px rgba(0, 0, 0, 0.4);
	}

	.modpack-header {
		padding: var(--space-4);
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
	}

	.modpack-thumb-box {
		width: 48px;
		height: 48px;
		border-radius: var(--radius-input);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		flex-shrink: 0;
	}

	.modpack-thumb-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.modpack-thumb-fallback {
		color: var(--accent-blue-text);
	}

	.modpack-title-meta {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
	}

	.badges-row {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 4px;
	}

	.provider-badge,
	.loader-badge {
		font-size: 10px;
		padding: 1px 6px;
	}

	.modpack-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.modpack-author {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.modpack-body {
		padding: 0 var(--space-4) var(--space-4) var(--space-4);
		flex: 1;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		gap: var(--space-3);
	}

	.modpack-summary {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		display: -webkit-box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
		overflow: hidden;
		line-height: 1.45;
	}

	.mods-preview-bar {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		background-color: var(--bg-base);
		padding: 4px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.tag-icon {
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	.modpack-footer {
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

	.deploy-btn {
		gap: var(--space-1);
	}

	/* Detail Modal Styling */
	.detail-modal {
		max-width: 580px;
	}

	.modal-header-main {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.modal-thumb {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-input);
		object-fit: cover;
		border: 1px solid var(--border-subtle);
	}

	.modal-thumb-fallback {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-input);
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.modal-meta-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		margin-top: 2px;
	}

	.modal-author {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.section-box {
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-input);
		padding: var(--space-3) var(--space-4);
		margin-bottom: var(--space-4);
	}

	.section-title {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--text-secondary);
		display: flex;
		align-items: center;
		gap: 6px;
		margin-bottom: 6px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.summary-text {
		font-size: var(--font-size-sm);
		color: var(--text-primary);
		line-height: 1.5;
	}

	.input-with-icon {
		position: relative;
		display: flex;
		align-items: center;
	}

	.input-icon {
		position: absolute;
		left: var(--space-3);
		color: var(--text-muted);
		pointer-events: none;
	}

	.input-with-icon .input {
		padding-left: var(--space-8);
	}

	.field-hint {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 4px;
	}

	.mods-chips-container {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		max-height: 140px;
		overflow-y: auto;
		padding-top: 4px;
	}

	.mod-chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--font-size-xs);
		background-color: var(--bg-surface);
		color: var(--text-primary);
		border: 1px solid var(--border-focus);
		padding: 2px 8px;
		border-radius: var(--radius-badge);
	}

	.check-icon {
		color: var(--accent-green);
	}

	.versions-loading {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}

	.no-versions-text {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}
</style>
