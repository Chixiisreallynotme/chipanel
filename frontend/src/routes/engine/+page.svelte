<script>
	import { onMount } from 'svelte';
	import { apiGet, apiFetch } from '$lib/api/client.js';
	import VersionPickerButton from '$lib/components/common/VersionPickerButton.svelte';
	import EngineLogo from '$lib/components/common/EngineLogo.svelte';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import HeroBanner from '$lib/components/ui/HeroBanner.svelte';
	import KpiCard from '$lib/components/ui/KpiCard.svelte';
	import KpiGrid from '$lib/components/ui/KpiGrid.svelte';
	import SearchBar from '$lib/components/ui/SearchBar.svelte';
	import CategoryTabs from '$lib/components/ui/CategoryTabs.svelte';
	import ChecklistCard from '$lib/components/ui/ChecklistCard.svelte';
	import { toast } from '$lib/stores/toast.svelte.js';
	import {
		RefreshCw,
		Check,
		RotateCw,
		Users,
		ShieldCheck,
		HardDrive,
		AlertCircle,
		CheckCircle2,
		Radio,
		ArrowRight,
		FolderArchive,
		Wrench,
		Settings2,
		ChevronDown,
		AlertTriangle,
		Sparkles,
		Info,
		Activity
	} from '$lib/icons.js';

	const UNKNOWN = 'Inconnu';

	// Engines that support distinct loader or build versions
	const LOADER_SUPPORTED_ENGINES = ['FABRIC', 'QUILT', 'FORGE', 'NEOFORGE', 'PAPER', 'PURPUR', 'FOLIA'];

	// Page State
	let currentType = $state(null);
	let currentVersion = $state(null);
	let currentLoaderVersion = $state(null);
	let configSource = $state('unavailable');
	let configError = $state(null);
	let availableTypes = $state([]);
	let allVersions = $state([]);
	/** @type {string[]} */
	let releaseVersions = $state([]);
	/** @type {string[]} */
	let snapshotVersions = $state([]);
	/** @type {any} */
	let versionWatch = $state(null);
	let activeWorld = $state('world');
	let worldDataVersion = $state(null);
	let installedPluginsCount = $state(0);
	let installedModsCount = $state(0);
	let onlinePlayersCount = $state(0);
	let serverState = $state('stopped');

	let isLoading = $state(true);
	let isSubmitting = $state(false);

	// Filters & Search
	let searchQuery = $state('');
	let selectedCategory = $state('ALL');

	// Version and Loader selection maps (engine_id -> version/loader)
	let selectedVersionMap = $state({});
	let selectedLoaderMap = $state({});
	let loaderOptionsMap = $state({});

	// Selection modal state
	let selectedEngine = $state(null);
	let showConfirmModal = $state(false);
	let autoBackupWorld = $state(true);
	/** @type {{ detail: string, serverMessage: string } | null} */
	let switchError = $state(null);

	async function fetchLoaderVersions(engineId, gameVersion) {
		if (!LOADER_SUPPORTED_ENGINES.includes(engineId)) return;

		loaderOptionsMap[engineId] = {
			...(loaderOptionsMap[engineId] || {}),
			loading: true
		};

		try {
			const res = await apiGet(`/api/server/engine/loader-versions?engine_type=${engineId}&game_version=${gameVersion || ''}`);
			if (res) {
				loaderOptionsMap[engineId] = {
					loading: false,
					versions: res.versions || [],
					default_version: res.default_version || 'LATEST',
					env_variable: res.env_variable || ''
				};

				if (!selectedLoaderMap[engineId]) {
					selectedLoaderMap[engineId] = (currentType === engineId && currentLoaderVersion)
						? currentLoaderVersion
						: (res.default_version || 'LATEST');
				}
			}
		} catch (err) {
			console.warn(`Échec de récupération des versions de loader pour ${engineId}:`, err);
			loaderOptionsMap[engineId] = {
				loading: false,
				versions: [
					{ version: 'LATEST', label: 'Dernière version recommandée (LATEST)', is_stable: true, is_recommended: true }
				],
				default_version: 'LATEST',
				env_variable: ''
			};
		}
	}

	async function loadEngineData() {
		isLoading = true;
		try {
			const res = await apiGet('/api/server/engine');
			if (res) {
				currentType = res.current_type ?? null;
				currentVersion = res.current_version ?? null;
				currentLoaderVersion = res.current_loader_version ?? null;
				configSource = res.config_source ?? 'unavailable';
				configError = res.config_error ?? null;
				availableTypes = res.available_types || [];
				allVersions = res.all_versions || [];
				releaseVersions = res.release_versions || [];
				snapshotVersions = res.snapshot_versions || [];
				versionWatch = res.version_watch ?? null;
				activeWorld = res.active_world || 'world';
				worldDataVersion = res.world_data_version ?? null;
				installedPluginsCount = res.installed_plugins_count ?? 0;
				installedModsCount = res.installed_mods_count ?? 0;
				onlinePlayersCount = res.online_players_count ?? 0;
				serverState = res.server_state ?? 'stopped';

				const vMap = {};
				const lMap = {};
				availableTypes.forEach((engine) => {
					if (currentType === engine.id && currentVersion) {
						vMap[engine.id] = currentVersion;
					} else {
						vMap[engine.id] = engine.recommended_versions?.[0] ?? allVersions[0] ?? '';
					}
					if (currentType === engine.id && currentLoaderVersion) {
						lMap[engine.id] = currentLoaderVersion;
					} else {
						lMap[engine.id] = 'LATEST';
					}

					if (LOADER_SUPPORTED_ENGINES.includes(engine.id)) {
						fetchLoaderVersions(engine.id, vMap[engine.id]);
					}
				});
				selectedVersionMap = vMap;
				selectedLoaderMap = lMap;
			}
		} catch (err) {
			console.error('Échec du chargement des moteurs:', err);
			toast.error('Erreur de chargement', err.message || 'Impossible de récupérer la liste des moteurs.');
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadEngineData();
	});

	function handleVersionChange(engineId, newVer) {
		selectedVersionMap[engineId] = newVer;
		if (LOADER_SUPPORTED_ENGINES.includes(engineId)) {
			fetchLoaderVersions(engineId, newVer);
		}
	}

	function openConfirmModal(engine) {
		selectedEngine = engine;
		autoBackupWorld = true;
		switchError = null;

		const targetType = engine.id;
		const targetVersion = selectedVersionMap[targetType];
		if (LOADER_SUPPORTED_ENGINES.includes(targetType)) {
			fetchLoaderVersions(targetType, targetVersion);
		}

		showConfirmModal = true;
	}

	function closeConfirmModal() {
		if (isSubmitting) return;
		showConfirmModal = false;
		selectedEngine = null;
		switchError = null;
	}

	/** @type {Record<number, string>} */
	const SWITCH_ERRORS = {
		400: 'Version ou moteur refusé : vérifiez la compatibilité.',
		403: "Accès refusé : cette action est réservée à l'administrateur.",
		500: "Erreur interne du serveur : le moteur n'a pas été modifié."
	};

	async function confirmEngineSwitch() {
		if (!selectedEngine) return;
		const targetType = selectedEngine.id;
		const targetVersion = selectedVersionMap[targetType];
		const targetLoader = selectedLoaderMap[targetType] || null;

		if (!targetVersion) {
			switchError = { detail: 'Veuillez sélectionner une version avant de confirmer.', serverMessage: '' };
			return;
		}

		isSubmitting = true;
		switchError = null;
		try {
			const res = await apiFetch('/api/server/engine', {
				method: 'POST',
				body: /** @type {any} */ ({
					engine_type: targetType,
					version: targetVersion,
					loader_version: targetLoader,
					backup_world: autoBackupWorld
				})
			});
			const data = await res.json().catch(() => ({}));

			if (!res.ok) {
				switchError = {
					detail: SWITCH_ERRORS[res.status] ?? `La requête a échoué (HTTP ${res.status}).`,
					serverMessage: data?.error || data?.message || ''
				};
				return;
			}

			let successMsg = `Moteur appliqué : ${selectedEngine.name} (${targetVersion})`;
			if (data?.resolved_loader_version && data.resolved_loader_version !== 'LATEST') {
				successMsg += ` · Loader ${data.resolved_loader_version}`;
			}
			if (data?.backup_file) {
				successMsg += ` · Sauvegarde : ${data.backup_file}`;
			}
			toast.success('Configuration mise à jour', successMsg);

			currentType = targetType;
			currentVersion = data?.resolved_version ?? targetVersion;
			currentLoaderVersion = data?.resolved_loader_version ?? targetLoader;
			if (data?.warning) {
				toast.warning('Avertissement', data.warning);
			}
			closeConfirmModal();
			await loadEngineData();
		} catch (err) {
			switchError = {
				detail: "Connexion au backend impossible.",
				serverMessage: err instanceof Error ? err.message : ''
			};
		} finally {
			isSubmitting = false;
		}
	}

	function formatVersionDate(iso, fallbackTs) {
		const d = iso ? new Date(iso) : fallbackTs ? new Date(fallbackTs * 1000) : null;
		if (!d || isNaN(d.getTime())) return '';
		return d.toLocaleDateString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' });
	}

	// Filtered engines calculation
	let filteredEngines = $derived.by(() => {
		let list = availableTypes;

		// Category filter
		if (selectedCategory === 'PERFORMANCE') {
			list = list.filter((e) => ['PURPUR', 'PAPER', 'FOLIA', 'SPIGOT'].includes(e.id));
		} else if (selectedCategory === 'MODS') {
			list = list.filter((e) => ['FABRIC', 'FORGE', 'NEOFORGE', 'QUILT'].includes(e.id));
		} else if (selectedCategory === 'HYBRID') {
			list = list.filter((e) => ['MOHIST', 'ARCLIGHT'].includes(e.id));
		} else if (selectedCategory === 'VANILLA') {
			list = list.filter((e) => e.id === 'VANILLA');
		}

		// Search query filter
		const q = searchQuery.trim().toLowerCase();
		if (q) {
			list = list.filter(
				(e) =>
					e.name.toLowerCase().includes(q) ||
					e.id.toLowerCase().includes(q) ||
					e.description.toLowerCase().includes(q) ||
					e.category.toLowerCase().includes(q) ||
					(e.pros && e.pros.some((p) => p.toLowerCase().includes(q)))
			);
		}

		return list;
	});
</script>

<svelte:head>
	<title>Moteurs & Versions | ChiPanel</title>
</svelte:head>

<div class="engine-page">
	<!-- Page Header -->
	<PageHeader
		title="Moteurs & Versions du Serveur"
		subtitle="Gérez et basculez l'environnement d'exécution Minecraft en toute sécurité."
	>
		<button class="btn btn-secondary btn-sm" onclick={loadEngineData} disabled={isLoading}>
			<RefreshCw size={14} class={isLoading ? 'spin' : ''} />
			<span>Actualiser</span>
		</button>
	</PageHeader>

	{#if configError}
		<div class="config-alert" role="alert">
			<AlertTriangle size={16} />
			<span>{configError}</span>
		</div>
	{/if}

	<!-- New version alerts (background Mojang watcher) -->
	{#if versionWatch?.new_release}
		{@const alert = versionWatch.new_release}
		<div class="version-banner release-banner" role="status">
			<Sparkles size={18} class="text-emerald" />
			<div class="version-banner-text">
				<strong>Minecraft {alert.id} est disponible officiellement</strong>
				<span class="banner-sub">
					{alert.previous ? `Succède à ${alert.previous}` : 'Nouvelle release officielle Mojang'}
					{formatVersionDate(alert.published_at, alert.detected_at) ? ` · Publié le ${formatVersionDate(alert.published_at, alert.detected_at)}` : ''}
					{currentVersion && !['LATEST', alert.id].includes(currentVersion) ? ` — Serveur actuel en ${currentVersion}` : ''}
				</span>
			</div>
		</div>
	{:else if versionWatch?.new_snapshot}
		{@const alert = versionWatch.new_snapshot}
		<div class="version-banner snapshot-banner" role="status">
			<Activity size={18} />
			<div class="version-banner-text">
				<strong>Nouveau snapshot disponible : {alert.id}</strong>
				<span class="banner-sub">
					{alert.previous ? `Succède à ${alert.previous}` : 'Pré-version de test Mojang'}
					{formatVersionDate(alert.published_at, alert.detected_at) ? ` · ${formatVersionDate(alert.published_at, alert.detected_at)}` : ''}
				</span>
			</div>
		</div>
	{/if}

	<!-- Current Engine Status Bento Card -->
	<HeroBanner title={currentType ?? UNKNOWN}>
		{#snippet badge()}
			<Radio size={12} class="pulse-dot" />
			<span>MOTEUR EN COURS D'EXÉCUTION</span>
		{/snippet}
		{#snippet visual()}
			{#if currentType}
				<EngineLogo engine={currentType} size={52} class="banner-logo" />
			{/if}
		{/snippet}
		{#snippet meta()}
			<div class="identity-header">
				<span class="tag tag-version">{currentVersion ?? UNKNOWN}</span>
				{#if currentLoaderVersion && currentLoaderVersion !== 'LATEST'}
					<span class="tag tag-loader">Loader {currentLoaderVersion}</span>
				{/if}
			</div>
			<p class="identity-desc">
				{#if currentType && currentVersion}
					Le conteneur exécute <strong>{currentType}</strong> sur Minecraft <strong>{currentVersion}</strong>.
					Les modifications de version ou de loader s'appliquent avec migration automatique des permissions et des outils.
				{:else}
					La configuration actuelle n'a pas pu être extraite. Sélectionner un moteur réinitialisera proprement l'environnement.
				{/if}
			</p>
		{/snippet}
		{#snippet kpis()}
			<KpiGrid>
				<KpiCard label="État du Serveur">
					{#if serverState === 'running'}
						<span class="status-indicator status-online"></span>
						<span class="text-emerald">En ligne</span>
					{:else if serverState === 'hibernating'}
						<span class="status-indicator status-hibernating"></span>
						<span class="text-amber">En veille</span>
					{:else}
						<span class="status-indicator status-stopped"></span>
						<span class="text-muted">Arrêté</span>
					{/if}
				</KpiCard>

				<KpiCard label="Joueurs en ligne">
					<span class="kpi-inline text-primary">
						<Users size={14} />
						<span>{onlinePlayersCount}</span>
					</span>
				</KpiCard>

				<KpiCard label="Monde Actif">
					<span class="kpi-inline" title="Data Version: {worldDataVersion ?? 'N/A'}">
						<HardDrive size={14} class="text-muted" />
						<span class="truncate">{activeWorld}</span>
					</span>
				</KpiCard>

				<KpiCard label="Outils Synchronisés">
					<span class="kpi-inline text-emerald">
						<ShieldCheck size={14} />
						<span>Spark, Chunky, LP</span>
					</span>
				</KpiCard>
			</KpiGrid>
		{/snippet}
	</HeroBanner>

	<!-- Filters & Search Toolbar -->
	<nav class="toolbar" aria-label="Filtres des moteurs">
		<SearchBar
			bind:value={searchQuery}
			placeholder="Rechercher un moteur (ex: Fabric, Purpur, Forge, Folia, SMP)..."
		/>

		<CategoryTabs
			bind:active={selectedCategory}
			options={[
				{ id: 'ALL', label: 'Tous', count: availableTypes.length },
				{ id: 'MODS', label: 'Moddé (Fabric, Forge, NeoForge, Quilt)' },
				{ id: 'PERFORMANCE', label: 'Performance & SMP (Purpur, Paper, Folia)' },
				{ id: 'HYBRID', label: 'Hybride (Mohist, Arclight)' },
				{ id: 'VANILLA', label: 'Vanilla (Officiel)' }
			]}
		/>
	</nav>

	<!-- Engine Cards Grid -->
	{#if isLoading}
		<div class="loading-state">
			<RefreshCw size={28} class="spin text-primary" />
			<p>Chargement des moteurs et versions...</p>
		</div>
	{:else if filteredEngines.length === 0}
		<EmptyState description={`Aucun moteur ne correspond à "${searchQuery}".`}>
			{#snippet icon()}
				<Info size={28} class="text-muted" />
			{/snippet}
			{#snippet action()}
				<button class="btn btn-secondary btn-sm" onclick={() => { searchQuery = ''; selectedCategory = 'ALL'; }}>
					Réinitialiser les filtres
				</button>
			{/snippet}
		</EmptyState>
	{:else}
		<div class="engine-grid">
			{#each filteredEngines as engine (engine.id)}
				{@const isActive = currentType === engine.id}
				{@const targetVer = selectedVersionMap[engine.id]}
				{@const targetLoader = selectedLoaderMap[engine.id]}
				{@const isVersionChanged = isActive && (targetVer !== currentVersion || (currentLoaderVersion && targetLoader !== currentLoaderVersion))}
				{@const loaderData = loaderOptionsMap[engine.id]}
				{@const hasLoaderSupport = LOADER_SUPPORTED_ENGINES.includes(engine.id)}

				<article class="engine-card {isActive ? 'card-active' : ''}">
					<!-- Card Header -->
					<header class="card-header">
						<EngineLogo engine={engine.id} size={44} class="engine-logo-frame" />
						<div class="header-details">
							<div class="title-row">
								<h3>{engine.name}</h3>
								{#if isActive}
									<span class="badge-status-active">
										<Check size={11} />
										<span>Actif</span>
									</span>
								{/if}
							</div>
							<div class="tags-row">
								<span class="tag-meta">{engine.category}</span>
								{#if engine.stability}
									<span class="tag-stability">{engine.stability}</span>
								{/if}
							</div>
						</div>
					</header>

					<!-- Minimal Specs Metadata -->
					<div class="specs-row">
						<span class="spec-meta">RAM : <strong>{engine.recommended_ram || '2 à 4 Go'}</strong></span>
						<span class="spec-dot">·</span>
						{#if engine.min_version}
							<span class="spec-meta">Support : <strong>{engine.min_version}+</strong> {engine.max_version ? `(max ${engine.max_version})` : ''}</span>
						{/if}
					</div>

					<!-- Narrative Description -->
					<p class="card-description">{engine.description}</p>

					<!-- Highlights List -->
					<div class="highlights-list">
						{#if engine.pros && engine.pros.length > 0}
							{#each engine.pros.slice(0, 2) as pro}
								<div class="highlight-item highlight-pro">
									<CheckCircle2 size={13} class="text-emerald flex-shrink-0" />
									<span>{pro}</span>
								</div>
							{/each}
						{/if}
						{#if engine.cons && engine.cons.length > 0}
							{#each engine.cons.slice(0, 1) as con}
								<div class="highlight-item highlight-con">
									<AlertCircle size={13} class="text-amber flex-shrink-0" />
									<span>{con}</span>
								</div>
							{/each}
						{/if}
					</div>

					<!-- Tools Sync Badge -->
					{#if engine.supports_tools}
						<div class="tools-pill">
							<ShieldCheck size={13} class="text-emerald" />
							<span>Spark, Chunky et LuckPerms synchronisés</span>
						</div>
					{/if}

					<!-- Controls & Action Button -->
					<footer class="card-footer">
						<div class="pickers-cluster">
							<!-- Minecraft Game Version Selector -->
							<VersionPickerButton
								selectedVersion={targetVer}
								{releaseVersions}
								{snapshotVersions}
								recommendedVersions={engine.recommended_versions}
								minVersion={engine.min_version}
								maxVersion={engine.max_version}
								allowAll={false}
								label="Version Minecraft"
								disabled={isSubmitting}
								onSelect={(newVer) => handleVersionChange(engine.id, newVer)}
							/>

							<!-- Loader / Build Selector -->
							{#if hasLoaderSupport}
								<div class="loader-cluster">
									<div class="loader-label-row">
										<span class="input-label">
											{engine.id === 'FABRIC' ? 'Fabric Loader' :
											 engine.id === 'QUILT' ? 'Quilt Loader' :
											 engine.id === 'FORGE' ? 'Version Forge' :
											 engine.id === 'NEOFORGE' ? 'Version NeoForge' :
											 'Build du Serveur'}
										</span>
										{#if loaderData?.loading}
											<RefreshCw size={10} class="spin text-muted" />
										{/if}
									</div>

									<div class="select-wrapper">
										<select
											class="select-control"
											bind:value={selectedLoaderMap[engine.id]}
											disabled={isSubmitting}
										>
											<option value="LATEST">Dernière version recommandée (LATEST)</option>
											{#if loaderData?.versions}
												{#each loaderData.versions as opt}
													{#if opt.version !== 'LATEST'}
														<option value={opt.version}>{opt.label || opt.version}</option>
													{/if}
												{/each}
											{/if}
										</select>
										<ChevronDown size={14} class="select-arrow" />
									</div>
								</div>
							{/if}
						</div>

						<!-- Action Button -->
						{#if isActive}
							{#if isVersionChanged}
								<button class="btn btn-primary btn-action" onclick={() => openConfirmModal(engine)}>
									<RotateCw size={15} />
									<span>Changer la version ({targetVer})</span>
								</button>
							{:else}
								<button class="btn btn-secondary btn-action" onclick={() => openConfirmModal(engine)}>
									<Check size={15} />
									<span>Version Active (Reconfigurer)</span>
								</button>
							{/if}
						{:else}
							<button class="btn btn-primary btn-action" onclick={() => openConfirmModal(engine)}>
								<RotateCw size={15} />
								<span>Passer à {engine.name}</span>
							</button>
						{/if}
					</footer>
				</article>
			{/each}
		</div>
	{/if}
</div>

<!-- Detailed Confirmation Modal -->
{#if showConfirmModal && selectedEngine}
	{@const targetType = selectedEngine.id}
	{@const targetVersion = selectedVersionMap[targetType] || UNKNOWN}
	{@const targetLoader = selectedLoaderMap[targetType] || 'LATEST'}
	{@const isSameEngine = currentType === targetType}
	{@const hasLoader = LOADER_SUPPORTED_ENGINES.includes(targetType)}
	{@const modalLoaderData = loaderOptionsMap[targetType]}

	<div
		class="modal-backdrop"
		onclick={closeConfirmModal}
		onkeydown={(e) => e.key === 'Escape' && closeConfirmModal()}
		role="button"
		tabindex="0"
	>
		<div
			class="modal-dialog"
			onclick={(e) => e.stopPropagation()}
			onkeydown={() => {}}
			role="dialog"
			tabindex="-1"
		>
			<header class="modal-header">
				<EngineLogo engine={selectedEngine.id} size={36} class="modal-logo" />
				<div>
					<h2>{isSameEngine ? `Mise à jour Minecraft (${selectedEngine.name})` : `Passage à ${selectedEngine.name}`}</h2>
					<p class="modal-subtitle">Bilan d'impact et reconfiguration automatique</p>
				</div>
			</header>

			<div class="modal-content">
				<!-- Diff View -->
				<div class="diff-card">
					<div class="diff-pane">
						<div class="diff-pane-header">
							<EngineLogo engine={currentType} size={28} />
							<div>
								<span class="diff-eyebrow">Actuel</span>
								<span class="diff-name">{currentType ?? UNKNOWN}</span>
							</div>
						</div>
						<div class="diff-ver-line">
							<span class="text-muted">{currentVersion ?? UNKNOWN}</span>
							{#if currentLoaderVersion && currentLoaderVersion !== 'LATEST'}
								<span class="diff-pill">Loader: {currentLoaderVersion}</span>
							{/if}
						</div>
					</div>

					<div class="diff-separator">
						<ArrowRight size={18} />
					</div>

					<div class="diff-pane diff-pane-target">
						<div class="diff-pane-header">
							<EngineLogo engine={selectedEngine.id} size={28} />
							<div>
								<span class="diff-eyebrow">{isSameEngine ? 'Nouvelle Version' : 'Nouveau Moteur'}</span>
								<span class="diff-name text-primary">{selectedEngine.name}</span>
							</div>
						</div>
						<div class="diff-ver-line">
							<span class="text-emerald font-semibold">{targetVersion}</span>
							{#if hasLoader && targetLoader && targetLoader !== 'LATEST'}
								<span class="diff-pill text-primary">Loader: {targetLoader}</span>
							{/if}
						</div>
					</div>
				</div>

				<!-- Loader Selector in Modal -->
				{#if hasLoader}
					<div class="modal-loader-row">
						<span class="modal-field-label">
							<Settings2 size={14} class="text-primary" />
							<span>Version du Loader / Build</span>
						</span>
						<div class="select-wrapper modal-select-wrap">
							<select
								class="select-control"
								bind:value={selectedLoaderMap[targetType]}
								disabled={isSubmitting}
							>
								<option value="LATEST">Dernière version recommandée (LATEST)</option>
								{#if modalLoaderData?.versions}
									{#each modalLoaderData.versions as opt}
										{#if opt.version !== 'LATEST'}
											<option value={opt.version}>{opt.label || opt.version}</option>
										{/if}
									{/each}
								{/if}
							</select>
							<ChevronDown size={14} class="select-arrow" />
						</div>
					</div>
				{/if}

				<!-- Active Players Warning -->
				{#if onlinePlayersCount > 0}
					<div class="callout callout-danger">
						<Users size={16} class="flex-shrink-0" />
						<div>
							<strong>{onlinePlayersCount} joueur(s) connecté(s)</strong>
							<p>L'application redémarrera immédiatement le conteneur Minecraft et déconnectera la session en cours.</p>
						</div>
					</div>
				{/if}

				<!-- Execution Checklist -->
				<div class="checklist-section">
					<h4 class="checklist-heading">Actions exécutées automatiquement :</h4>

				<div class="checklist-items">
					<ChecklistCard
						title={`Sauvegarde de sécurité du monde actif (${activeWorld})`}
						detail="Archive compressée créée avant modification pour prévenir toute corruption."
						bind:checked={autoBackupWorld}
					>
						{#snippet icon()}
							<FolderArchive size={15} class="text-primary" />
						{/snippet}
					</ChecklistCard>

					<ChecklistCard
						title="Permissions LuckPerms"
						badge="Auto"
						detail="Préservation et réinjection automatique des groupes et permissions."
					>
						{#snippet icon()}
							<ShieldCheck size={15} class="text-emerald" />
						{/snippet}
					</ChecklistCard>

					<ChecklistCard
						title="Outils Spark & Chunky"
						badge="Auto"
						detail={`Téléchargement et configuration des binaires compatibles ${selectedEngine.name} ${targetVersion}.`}
					>
						{#snippet icon()}
							<Wrench size={15} class="text-emerald" />
						{/snippet}
					</ChecklistCard>

					<ChecklistCard
						title="Conteneur & Proxy lazymc"
						badge="Auto"
						detail="Écriture des variables système et redémarrage propre."
					>
						{#snippet icon()}
							<RotateCw size={15} class="text-primary" />
						{/snippet}
					</ChecklistCard>
				</div>
				</div>

				{#if switchError}
					<div class="callout callout-danger" role="alert">
						<AlertTriangle size={16} />
						<div>
							<div class="font-semibold">{switchError.detail}</div>
							{#if switchError.serverMessage}
								<div class="text-xs text-muted font-mono mt-1">{switchError.serverMessage}</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			<footer class="modal-footer">
				<button class="btn btn-secondary" onclick={closeConfirmModal} disabled={isSubmitting}>
					Annuler
				</button>
				<button class="btn btn-primary" onclick={confirmEngineSwitch} disabled={isSubmitting}>
					{#if isSubmitting}
						<RefreshCw size={15} class="spin" />
						<span>Application en cours...</span>
					{:else}
						<RotateCw size={15} />
						<span>{isSameEngine ? 'Appliquer la Version' : 'Confirmer le Changement'}</span>
					{/if}
				</button>
			</footer>
		</div>
	</div>
{/if}

<style>
	.engine-page {
		display: flex;
		flex-direction: column;
		gap: 2rem;
		max-width: 1380px;
		margin: 0 auto;
		padding: 0 0 4rem 0;
	}

	/* Version Alerts */
	.version-banner {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.875rem 1.25rem;
		border-radius: 12px;
		font-size: 0.875rem;
	}

	.release-banner {
		background: rgba(16, 185, 129, 0.08);
		border: 1px solid var(--engine-emerald-border);
		color: var(--accent-emerald-text);
	}

	.snapshot-banner {
		background: rgba(99, 102, 241, 0.08);
		border: 1px solid rgba(99, 102, 241, 0.25);
		color: var(--text-code);
	}

	.version-banner-text {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}

	.banner-sub {
		color: var(--engine-slate-400);
		font-size: 0.8125rem;
	}

	.config-alert {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 1.25rem;
		border-radius: 10px;
		background: rgba(245, 158, 11, 0.08);
		border: 1px solid rgba(245, 158, 11, 0.25);
		color: var(--warning-text);
		font-size: 0.875rem;
	}

	/* Reliure HeroBanner : badge + meta restes locaux (valeurs exactes engine) */
	.pulse-dot {
		color: var(--accent-indigo-text);
	}

	.tag {
		padding: 0.2rem 0.6rem;
		border-radius: 6px;
		font-size: 0.8125rem;
		font-family: var(--font-mono, monospace);
		font-weight: 600;
	}

	.identity-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		flex-wrap: wrap;
	}

	.tag-version {
		background: var(--accent-indigo-bg);
		color: var(--text-code);
		border: 1px solid var(--accent-indigo-border);
	}

	.tag-loader {
		background: var(--accent-emerald-bg);
		color: var(--accent-emerald-text);
		border: 1px solid var(--accent-emerald-border);
	}

	.identity-desc {
		font-size: 0.875rem;
		color: var(--engine-slate-400);
		line-height: 1.5;
		margin: 0;
		max-width: 60ch;
	}

	/* Contenu KpiCard : pastille statut + aligneur icone/texte (sans couleur) */
	.kpi-inline {
		display: inline-flex;
		align-items: center;
		gap: 0.45rem;
	}

	.status-indicator {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.status-online { background: var(--engine-emerald); box-shadow: 0 0 8px var(--engine-emerald); }
	.status-hibernating { background: var(--engine-amber); }
	.status-stopped { background: var(--engine-slate-500); }

	/* Toolbar */
	.toolbar {
		display: flex;
		flex-direction: column;
		gap: 0.875rem;
	}

	/* Engine Grid */
	.engine-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
		gap: 1.5rem;
	}

	.engine-card {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		padding: 1.75rem;
		border-radius: 16px;
		background: var(--bg-surface-2);
		border: 1px solid var(--border-subtle);
		transition: transform 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
		gap: 1.125rem;
	}

	.engine-card:hover {
		border-color: rgba(99, 102, 241, 0.35);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
	}

	.card-active {
		border-color: rgba(16, 185, 129, 0.4);
		background: linear-gradient(180deg, rgba(16, 185, 129, 0.04), var(--bg-surface-2) 40%);
	}

	.card-header {
		display: flex;
		gap: 1rem;
		align-items: flex-start;
	}

	.header-details {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.title-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.title-row h3 {
		font-size: 1.15rem;
		font-weight: 700;
		color: var(--engine-slate-50);
		letter-spacing: -0.01em;
		margin: 0;
	}

	.tags-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	.tag-meta {
		font-size: 0.75rem;
		color: var(--engine-slate-400);
		font-weight: 500;
	}

	.tag-stability {
		font-size: 0.6875rem;
		font-weight: 600;
		padding: 0.1rem 0.45rem;
		border-radius: 4px;
		background: rgba(99, 102, 241, 0.1);
		color: var(--text-code);
		border: 1px solid rgba(99, 102, 241, 0.2);
	}

	.badge-status-active {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.15rem 0.5rem;
		border-radius: 6px;
		font-size: 0.6875rem;
		font-weight: 700;
		background: var(--accent-emerald-bg);
		color: var(--accent-emerald-text);
		border: 1px solid var(--accent-emerald-border);
	}

	/* Minimal Specs */
	.specs-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.8125rem;
		color: var(--engine-slate-500);
	}

	.spec-meta strong {
		color: var(--engine-slate-300);
	}

	.spec-dot {
		color: var(--engine-slate-600);
	}

	.card-description {
		font-size: 0.875rem;
		color: var(--engine-slate-400);
		line-height: 1.5;
		margin: 0;
		min-height: 2.75rem;
	}

	.highlights-list {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.highlight-item {
		display: flex;
		align-items: flex-start;
		gap: 0.5rem;
		font-size: 0.8125rem;
		line-height: 1.4;
	}

	.highlight-pro span {
		color: var(--engine-slate-300);
	}

	.highlight-con span {
		color: var(--warning-text);
	}

	.tools-pill {
		display: flex;
		align-items: center;
		gap: 0.45rem;
		padding: 0.4rem 0.75rem;
		border-radius: 8px;
		background: rgba(16, 185, 129, 0.06);
		border: 1px solid rgba(16, 185, 129, 0.15);
		color: var(--accent-emerald-text);
		font-size: 0.75rem;
		font-weight: 500;
	}

	/* Card Footer Controls */
	.card-footer {
		display: flex;
		flex-direction: column;
		gap: 0.875rem;
		margin-top: auto;
		padding-top: 0.5rem;
	}

	.pickers-cluster {
		display: flex;
		flex-direction: column;
		gap: 0.65rem;
	}

	.loader-cluster {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.loader-label-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.input-label {
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--engine-slate-400);
	}

	.select-wrapper {
		position: relative;
		display: flex;
		align-items: center;
		width: 100%;
	}

	.select-control {
		width: 100%;
		padding: 0.55rem 2rem 0.55rem 0.875rem;
		background: var(--bg-inset);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 8px;
		color: var(--engine-slate-100);
		font-size: 0.8125rem;
		font-weight: 500;
		outline: none;
		appearance: none;
		cursor: pointer;
		transition: border-color 0.2s;
	}

	.select-control:focus {
		border-color: var(--engine-indigo);
	}

	.select-arrow {
		position: absolute;
		right: 0.75rem;
		color: var(--engine-slate-500);
		pointer-events: none;
	}

	.btn-action {
		width: 100%;
		padding: 0.65rem 1rem;
		font-size: 0.875rem;
	}

	/* Buttons */
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.6rem 1.1rem;
		font-size: var(--font-size-sm);
		font-weight: 600;
		border-radius: var(--radius-btn);
		border: none;
		cursor: pointer;
		transition: background-color var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
	}

	.btn:active:not(:disabled) {
		transform: scale(0.97);
	}

	.btn-sm {
		padding: 0.4rem 0.85rem;
		font-size: 0.8125rem;
	}

	/* Boutons secondaires : gris d'origine sans token exact (rgb a valeur strictement identique, zero approximation) */
	.btn-secondary {
		background: rgb(31, 41, 55);
		border: 1px solid var(--border-engine-08);
		color: var(--engine-slate-300);
	}

	.btn-secondary:hover:not(:disabled) {
		background: rgb(55, 65, 81);
		color: var(--engine-white);
	}

	.btn-primary {
		background: var(--accent-indigo);
		color: var(--engine-white);
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--accent-indigo-hover);
		transform: translateY(-1px);
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* Modal */
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.8);
		backdrop-filter: blur(8px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal);
		padding: 1rem;
	}

	.modal-dialog {
		width: 100%;
		max-width: 600px;
		background: var(--bg-surface-2);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: var(--radius-lg);
		padding: 1.75rem;
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.5);
		max-height: 90vh;
		overflow-y: auto;
	}

	.modal-header {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 1.25rem;
	}

	.modal-header h2 {
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--engine-slate-50);
		margin: 0;
	}

	.modal-subtitle {
		font-size: 0.8125rem;
		color: var(--engine-slate-400);
		margin: 0.2rem 0 0 0;
	}

	.modal-content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.diff-card {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		gap: 1rem;
		padding: 1rem 1.25rem;
		border-radius: 12px;
		background: var(--bg-inset);
		border: 1px solid var(--border-engine-06);
	}

	.diff-pane {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.diff-pane-header {
		display: flex;
		align-items: center;
		gap: 0.6rem;
	}

	.diff-eyebrow {
		font-size: 0.6875rem;
		font-weight: 700;
		text-transform: uppercase;
		color: var(--engine-slate-500);
		display: block;
	}

	.diff-name {
		font-size: 1.05rem;
		font-weight: 700;
		color: var(--engine-slate-100);
	}

	.diff-ver-line {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.8125rem;
		font-family: var(--font-mono, monospace);
	}

	.diff-pill {
		font-size: 0.75rem;
		padding: 0.1rem 0.4rem;
		border-radius: 4px;
		background: rgba(255, 255, 255, 0.05);
	}

	.diff-separator {
		color: var(--engine-slate-500);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.modal-loader-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		padding: 0.75rem 1rem;
		border-radius: 10px;
		background: var(--bg-inset);
		border: 1px solid var(--border-engine-06);
	}

	.modal-field-label {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--engine-slate-200);
		white-space: nowrap;
	}

	.modal-select-wrap {
		max-width: 300px;
	}

	.callout {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 0.875rem 1rem;
		border-radius: 10px;
		font-size: 0.8125rem;
	}

	.callout-danger {
		background: rgba(239, 68, 68, 0.1);
		border: 1px solid rgba(239, 68, 68, 0.25);
		color: var(--engine-red-300);
	}

	.callout-danger strong {
		color: var(--engine-white);
	}

	.callout-danger p {
		margin: 0.2rem 0 0 0;
	}

	.checklist-section {
		display: flex;
		flex-direction: column;
		gap: 0.65rem;
	}

	.checklist-heading {
		font-size: 0.8125rem;
		font-weight: 700;
		color: var(--engine-slate-300);
		margin: 0;
	}

	.checklist-items {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		margin-top: 1.25rem;
		padding-top: 1rem;
		border-top: 1px solid var(--border-engine-06);
	}

	/* Helpers */
	.text-emerald { color: var(--accent-emerald-text) !important; }
	.text-primary { color: var(--accent-indigo-text) !important; }
	.text-amber { color: var(--warning-text) !important; }
	.text-muted { color: var(--engine-slate-500) !important; }
	.truncate { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 4rem 1rem;
		gap: 0.875rem;
		color: var(--engine-slate-500);
		text-align: center;
	}
</style>
