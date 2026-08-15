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
		Sliders,
		Search,
		Users,
		ShieldCheck,
		HardDrive,
		AlertCircle,
		CheckCheck,
		Radio,
		ArrowRight,
		FolderArchive,
		Wrench
	} from 'lucide-svelte';

	const UNKNOWN = 'Inconnu';

	// Page State — `null` means "ChiPanel could not read the config"
	let currentType = $state(null);
	let currentVersion = $state(null);
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

	// Selection modal state
	let selectedEngine = $state(null);
	let selectedVersionMap = $state({}); // engine_id -> selected_version
	let showConfirmModal = $state(false);
	let autoBackupWorld = $state(true);
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

				// Initialize default version map
				const map = {};
				availableTypes.forEach((engine) => {
					map[engine.id] = engine.recommended_versions?.[0] ?? allVersions[0] ?? '';
				});
				selectedVersionMap = map;
			}
		} catch (err) {
			console.error('Échec du chargement de la configuration des moteurs:', err);
			addToast('error', 'Erreur de chargement', err.message || 'Impossible de récupérer la liste des moteurs.');
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadEngineData();
	});

	function openConfirmModal(engine) {
		selectedEngine = engine;
		autoBackupWorld = true;
		switchError = null;
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

			let successMsg = `Moteur basculé vers ${selectedEngine.name} (${targetVersion}).`;
			if (data?.backup_file) {
				successMsg += ` Sauvegarde créée : ${data.backup_file}`;
			}
			addToast('success', 'Moteur mis à jour !', successMsg);

			currentType = targetType;
			currentVersion = data?.resolved_version ?? targetVersion;
			if (data?.warning) {
				addToast('warning', 'Avertissement', data.warning);
			}
			closeConfirmModal();
			await loadEngineData();
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
	 * Date lisible pour une alerte de nouvelle version Mojang.
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
		let list = availableTypes;

		// Category filter
		if (selectedCategory === 'PERFORMANCE') {
			list = list.filter((e) => ['PURPUR', 'PAPER', 'FOLIA'].includes(e.id));
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
	<title>Moteurs & Versions | ChiPanel</title>
</svelte:head>

<div class="engine-page">
	<!-- Page Header -->
	<header class="page-header">
		<div class="header-content">
			<div class="header-title-group">
				<div class="header-icon-box">
					<Sliders size={26} />
				</div>
				<div>
					<h1>Moteurs & Versions du Serveur</h1>
					<p class="subtitle">Basculez dynamiquement entre Purpur, Paper, Fabric, Forge, NeoForge, Spigot, Vanilla, Quilt, Folia, Mohist et Arclight.</p>
				</div>
			</div>
			<button class="btn btn-secondary btn-sm" onclick={loadEngineData} disabled={isLoading}>
				<RefreshCw size={15} class={isLoading ? 'spin' : ''} />
				<span>Actualiser</span>
			</button>
		</div>
	</header>

	{#if configError}
		<div class="config-alert" role="alert">
			<AlertTriangle size={18} />
			<span>{configError}</span>
		</div>
	{/if}

	<!-- New version alerts (background Mojang watcher) -->
	{#if versionWatch?.new_release}
		{@const alert = versionWatch.new_release}
		<div class="new-release-banner" role="status">
			<Sparkles size={20} class="text-emerald" />
			<div class="new-version-text">
				<strong>Minecraft {alert.id} est sorti officiellement !</strong>
				<span>
					{alert.previous ? `Succède à ${alert.previous}` : 'Nouvelle release officielle Mojang'}
					{formatVersionDate(alert.published_at, alert.detected_at) ? ` · Publié le ${formatVersionDate(alert.published_at, alert.detected_at)}` : ''}
					{currentVersion && !['LATEST', alert.id].includes(currentVersion) ? ` — Votre serveur tourne actuellement en ${currentVersion}` : ''}
				</span>
			</div>
		</div>
	{:else if versionWatch?.new_snapshot}
		{@const alert = versionWatch.new_snapshot}
		<div class="new-snapshot-banner" role="status">
			<Sparkles size={18} />
			<div class="new-version-text">
				<strong>Nouveau snapshot disponible : {alert.id}</strong>
				<span>
					{alert.previous ? `Succède à ${alert.previous}` : 'Nouvelle pré-version de test'}
					{formatVersionDate(alert.published_at, alert.detected_at) ? ` · ${formatVersionDate(alert.published_at, alert.detected_at)}` : ''}
				</span>
			</div>
		</div>
	{/if}

	<!-- Current Engine Status Bento Card -->
	<section class="banner-card glow-card">
		<div class="banner-info">
			<div class="banner-badge">
				<Radio size={12} class="pulse-icon" />
				<span>MOTEUR DU SERVEUR ACTIF</span>
			</div>
			<div class="banner-title-row">
				<span class="active-engine-name {currentType ? '' : 'unknown-val'}">{currentType ?? UNKNOWN}</span>
				<span class="badge {currentVersion ? 'badge-version' : 'badge-muted'}">{currentVersion ?? UNKNOWN}</span>
			</div>
			{#if currentType && currentVersion}
				<p class="banner-desc">
					Le serveur est configuré pour exécuter <strong>{currentType}</strong> sur la version <strong>{currentVersion}</strong>.
					Le changement de moteur reconfigure automatiquement le conteneur et synchronise les outils.
				</p>
			{:else}
				<p class="banner-desc">
					La configuration actuelle n'a pas pu être lue depuis le fichier Quadlet. Un changement appliquera une nouvelle configuration propre.
				</p>
			{/if}
		</div>

		<div class="banner-stats">
			<div class="stat-chip">
				<span class="stat-label">Statut Serveur</span>
				<span class="stat-value flex-align">
					{#if serverState === 'running'}
						<span class="status-dot dot-green"></span>
						<span class="text-green">En ligne</span>
					{:else if serverState === 'hibernating'}
						<span class="status-dot dot-amber"></span>
						<span class="text-amber">En veille (lazymc)</span>
					{:else}
						<span class="status-dot dot-gray"></span>
						<span class="text-muted">Arrêté</span>
					{/if}
				</span>
			</div>

			<div class="stat-chip">
				<span class="stat-label">Joueurs en ligne</span>
				<span class="stat-value text-primary flex-align">
					<Users size={14} />
					<span>{onlinePlayersCount}</span>
				</span>
			</div>

			<div class="stat-chip">
				<span class="stat-label">Monde Actif</span>
				<span class="stat-value flex-align" title="Data Version: {worldDataVersion ?? 'N/A'}">
					<HardDrive size={14} class="text-muted" />
					<span>{activeWorld}</span>
				</span>
			</div>

			<div class="stat-chip">
				<span class="stat-label">Outils Auto-Gérés</span>
				<span class="stat-value text-emerald flex-align">
					<ShieldCheck size={14} />
					<span>Spark, Chunky, LuckPerms</span>
				</span>
			</div>
		</div>
	</section>

	<!-- Filters & Search Toolbar -->
	<div class="filters-toolbar">
		<div class="search-box">
			<Search size={16} class="search-icon" />
			<input
				type="text"
				class="search-input"
				placeholder="Rechercher un moteur (ex: Purpur, Forge, Folia, SMP, moddé)..."
				bind:value={searchQuery}
			/>
			{#if searchQuery}
				<button class="clear-btn" onclick={() => (searchQuery = '')} aria-label="Effacer">✕</button>
			{/if}
		</div>

		<div class="category-tabs">
			<button class="tab-btn {selectedCategory === 'ALL' ? 'active' : ''}" onclick={() => (selectedCategory = 'ALL')}>
				Tous ({availableTypes.length})
			</button>
			<button class="tab-btn {selectedCategory === 'PERFORMANCE' ? 'active' : ''}" onclick={() => (selectedCategory = 'PERFORMANCE')}>
				⚡ Performance & SMP
			</button>
			<button class="tab-btn {selectedCategory === 'MODS' ? 'active' : ''}" onclick={() => (selectedCategory = 'MODS')}>
				🧩 Moddé
			</button>
			<button class="tab-btn {selectedCategory === 'HYBRID' ? 'active' : ''}" onclick={() => (selectedCategory = 'HYBRID')}>
				✨ Hybride (Mods + Plugins)
			</button>
			<button class="tab-btn {selectedCategory === 'VANILLA' ? 'active' : ''}" onclick={() => (selectedCategory = 'VANILLA')}>
				📦 Officiel Mojang
			</button>
		</div>
	</div>

	<!-- Engine Cards Grid -->
	{#if isLoading}
		<div class="loading-state">
			<RefreshCw size={32} class="spin text-primary" />
			<p>Chargement des spécifications des moteurs...</p>
		</div>
	{:else if filteredEngines.length === 0}
		<div class="empty-state">
			<Info size={32} class="text-muted" />
			<p>Aucun moteur ne correspond à votre recherche "{searchQuery}".</p>
			<button class="btn btn-secondary btn-sm" onclick={() => { searchQuery = ''; selectedCategory = 'ALL'; }}>
				Réinitialiser les filtres
			</button>
		</div>
	{:else}
		<div class="engine-grid">
			{#each filteredEngines as engine (engine.id)}
				{@const IconComp = getEngineIcon(engine.icon)}
				{@const isActive = currentType === engine.id}
				<div class="engine-card {isActive ? 'active-card' : ''}">
					<!-- Card Header -->
					<div class="card-header">
						<div class="icon-wrapper {isActive ? 'active-icon' : ''}">
							<IconComp size={24} />
						</div>
						<div class="header-titles">
							<div class="title-row">
								<h3>{engine.name}</h3>
								{#if isActive}
									<span class="badge badge-active"><Check size={12} /> Actif</span>
								{/if}
							</div>
							<div class="tag-row">
								<span class="category-tag">{engine.category}</span>
								{#if engine.stability}
									<span class="stability-tag">{engine.stability}</span>
								{/if}
							</div>
						</div>
					</div>

					<!-- RAM and Specs Chip -->
					<div class="specs-bar">
						<div class="spec-item">
							<span class="spec-label">RAM conseillée :</span>
							<span class="spec-val">{engine.recommended_ram || '2 à 4 Go'}</span>
						</div>
						{#if engine.min_version}
							<div class="spec-item">
								<span class="spec-label">Support :</span>
								<span class="spec-val">
									{engine.min_version}+ {engine.max_version ? `(max ${engine.max_version})` : ''}
								</span>
							</div>
						{/if}
					</div>

					<!-- Description -->
					<p class="engine-description">{engine.description}</p>

					<!-- Pros & Cons list -->
					<div class="features-list">
						{#if engine.pros && engine.pros.length > 0}
							<div class="pros-section">
								{#each engine.pros.slice(0, 2) as pro}
									<div class="feature-item pro-item">
										<CheckCircle2 size={13} class="text-green flex-shrink-0" />
										<span>{pro}</span>
									</div>
								{/each}
							</div>
						{/if}
						{#if engine.cons && engine.cons.length > 0}
							<div class="cons-section">
								{#each engine.cons.slice(0, 1) as con}
									<div class="feature-item con-item">
										<AlertCircle size={13} class="text-amber flex-shrink-0" />
										<span>{con}</span>
									</div>
								{/each}
							</div>
						{/if}
					</div>

					<!-- Tools Sync Badge -->
					{#if engine.supports_tools}
						<div class="tools-sync-badge">
							<ShieldCheck size={13} class="text-emerald" />
							<span>Outils auto-gérés : Spark, Chunky, LuckPerms</span>
						</div>
					{/if}

					<!-- Card Footer: Version Picker and Switch Button -->
					<div class="card-footer">
						<VersionPickerButton
							selectedVersion={selectedVersionMap[engine.id]}
							{releaseVersions}
							{snapshotVersions}
							recommendedVersions={engine.recommended_versions}
							minVersion={engine.min_version}
							maxVersion={engine.max_version}
							allowAll={false}
							label="Version Cible :"
							disabled={isActive}
							onSelect={(newVer) => (selectedVersionMap[engine.id] = newVer)}
						/>

						{#if isActive}
							<button class="btn btn-secondary w-full" disabled>
								<Check size={16} /> Moteur Actif
							</button>
						{:else}
							<button class="btn btn-primary w-full" onclick={() => openConfirmModal(engine)}>
								<RotateCw size={16} /> Passer à {engine.name}
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Detailed Confirmation & Impact Modal ("Tout ce qui va être modifié") -->
{#if showConfirmModal && selectedEngine}
	{@const targetType = selectedEngine.id}
	{@const targetVersion = selectedVersionMap[targetType] || UNKNOWN}
	<div
		class="modal-backdrop"
		onclick={closeConfirmModal}
		onkeydown={(e) => e.key === 'Escape' && closeConfirmModal()}
		role="button"
		tabindex="0"
	>
		<div
			class="modal-card impact-modal"
			onclick={(e) => e.stopPropagation()}
			onkeydown={() => {}}
			role="dialog"
			tabindex="-1"
		>
			<div class="modal-header">
				<div class="modal-icon-box">
					<RotateCw size={24} />
				</div>
				<div>
					<h2>Changement de Moteur du Serveur</h2>
					<p class="modal-subtitle">Bilan d'impact et reconfiguration sécurisée</p>
				</div>
			</div>

			<div class="modal-body">
				<!-- Engine Change Diff Header -->
				<div class="switch-diff-card">
					<div class="diff-side">
						<span class="diff-label">Moteur Actuel</span>
						<span class="diff-title">{currentType ?? UNKNOWN}</span>
						<span class="diff-sub">{currentVersion ?? UNKNOWN}</span>
					</div>
					<div class="diff-arrow">
						<ArrowRight size={22} />
					</div>
					<div class="diff-side diff-target">
						<span class="diff-label">Nouveau Moteur</span>
						<span class="diff-title text-primary">{selectedEngine.name}</span>
						<span class="diff-sub text-emerald">{targetVersion}</span>
					</div>
				</div>

				<!-- Live Warnings -->
				{#if onlinePlayersCount > 0}
					<div class="alert-box alert-danger">
						<Users size={18} />
						<div>
							<strong>{onlinePlayersCount} joueur(s) connecté(s) en ce moment !</strong>
							<p>Le changement de moteur redémarrera le serveur et déconnectera immédiatement les joueurs.</p>
						</div>
					</div>
				{/if}

				<!-- Step-by-Step Impact Checklist -->
				<div class="impact-section">
					<h4 class="impact-heading">Ce qui va être modifié et exécuté :</h4>

					<div class="impact-list">
						<!-- 1. World Safety Backup -->
						<div class="impact-item {autoBackupWorld ? 'item-active' : ''}">
							<div class="impact-checkbox-row">
								<input
									type="checkbox"
									id="backup-world-cb"
									class="custom-cb"
									bind:checked={autoBackupWorld}
								/>
								<label for="backup-world-cb" class="impact-label">
									<FolderArchive size={16} class="text-primary" />
									<span>Créer une sauvegarde de sécurité du monde actif (<strong>{activeWorld}</strong>)</span>
								</label>
							</div>
							<p class="impact-desc">Une archive complète du monde sera générée dans ChiPanel avant toute modification pour garantir zéro perte de données.</p>
						</div>

						<!-- 2. LuckPerms Migration -->
						<div class="impact-item">
							<div class="impact-header-row">
								<ShieldCheck size={16} class="text-emerald" />
								<span class="impact-title">Migration automatique des permissions LuckPerms</span>
								<span class="badge badge-auto">Auto</span>
							</div>
							<p class="impact-desc">
								Vos groupes et permissions LuckPerms existants sont automatiquement sauvegardés puis restaurés dans le répertoire adapté au nouveau moteur ({selectedEngine.supports_plugins ? 'plugins/LuckPerms' : 'mods/luckperms ou config/luckperms'}).
							</p>
						</div>

						<!-- 3. Tools Re-sync (Spark & Chunky) -->
						<div class="impact-item">
							<div class="impact-header-row">
								<Wrench size={16} class="text-emerald" />
								<span class="impact-title">Synchronisation des outils d'administration (Spark, Chunky)</span>
								<span class="badge badge-auto">Auto</span>
							</div>
							<p class="impact-desc">
								ChiPanel télécharge et réinstalle automatiquement les binaires Spark et Chunky adaptés à {selectedEngine.name} pour que vos profils et pré-générations restent opérationnels.
							</p>
						</div>

						<!-- 4. Addons Impact -->
						{#if selectedEngine.supports_plugins && !selectedEngine.supports_mods && installedModsCount > 0}
							<div class="impact-item item-warning">
								<div class="impact-header-row">
									<AlertCircle size={16} class="text-amber" />
									<span class="impact-title">Impact sur les mods installés ({installedModsCount} mods)</span>
								</div>
								<p class="impact-desc">
									Les mods Fabric/Forge dans le dossier <code>mods/</code> seront conservés sur le disque mais ne seront pas chargés par le moteur {selectedEngine.name} (qui n'exécute que des plugins Bukkit).
								</p>
							</div>
						{:else if selectedEngine.supports_mods && !selectedEngine.supports_plugins && installedPluginsCount > 0}
							<div class="impact-item item-warning">
								<div class="impact-header-row">
									<AlertCircle size={16} class="text-amber" />
									<span class="impact-title">Impact sur les plugins installés ({installedPluginsCount} plugins)</span>
								</div>
								<p class="impact-desc">
									Les plugins dans le dossier <code>plugins/</code> seront conservés mais ne seront pas exécutés par le chargeur de mods {selectedEngine.name}.
								</p>
							</div>
						{/if}

						<!-- 5. Configuration & Restart -->
						<div class="impact-item">
							<div class="impact-header-row">
								<RotateCw size={16} class="text-primary" />
								<span class="impact-title">Mise à jour Quadlet & Redémarrage du proxy</span>
								<span class="badge badge-auto">Auto</span>
							</div>
							<p class="impact-desc">
								Mise à jour de <code>Environment=TYPE={targetType}</code> et <code>Environment=VERSION={targetVersion}</code> dans le conteneur, synchronisation du proxy lazymc et redémarrage automatique.
							</p>
						</div>
					</div>
				</div>

				{#if switchError}
					<div class="switch-error" role="alert">
						<AlertTriangle size={18} />
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
				<button class="btn btn-secondary" onclick={closeConfirmModal} disabled={isSubmitting}>
					Annuler
				</button>
				<button class="btn btn-danger" onclick={confirmEngineSwitch} disabled={isSubmitting}>
					{#if isSubmitting}
						<RefreshCw size={16} class="spin" />
						<span>Application & Redémarrage...</span>
					{:else}
						<RotateCw size={16} />
						<span>Confirmer le Changement</span>
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
				<CheckCircle2 size={18} class="text-green" />
			{:else if toast.type === 'warning'}
				<AlertTriangle size={18} class="text-amber" />
			{:else}
				<AlertCircle size={18} class="text-danger" />
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
		gap: var(--space-6);
		max-width: 1350px;
		margin: 0 auto;
		padding-bottom: var(--space-8);
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
		gap: var(--space-4);
	}

	.header-icon-box {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 50px;
		height: 50px;
		border-radius: var(--radius-lg);
		background: rgba(99, 102, 241, 0.15);
		color: var(--accent-primary, #818cf8);
		border: 1px solid rgba(99, 102, 241, 0.3);
	}

	h1 {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		margin: 0;
	}

	.subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		margin: 0.25rem 0 0 0;
	}

	/* Banner Bento Card */
	.banner-card {
		display: grid;
		grid-template-columns: 1.2fr 1fr;
		gap: var(--space-6);
		padding: var(--space-6);
		border-radius: var(--radius-xl);
		background: var(--bg-card);
		border: 1px solid var(--border-subtle);
		box-shadow: var(--shadow-md);
	}

	.banner-badge {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.7rem;
		font-weight: 800;
		letter-spacing: 0.06em;
		color: var(--accent-primary, #818cf8);
		margin-bottom: 0.5rem;
	}

	.banner-title-row {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 0.5rem;
	}

	.active-engine-name {
		font-size: 1.85rem;
		font-weight: 800;
		color: var(--text-primary);
		letter-spacing: -0.02em;
	}

	.badge-version {
		background: rgba(99, 102, 241, 0.2);
		color: #a5b4fc;
		border: 1px solid rgba(99, 102, 241, 0.4);
		font-family: var(--font-mono);
		padding: 0.25rem 0.6rem;
		font-size: 0.85rem;
		border-radius: var(--radius-md);
	}

	.banner-desc {
		font-size: 0.875rem;
		color: var(--text-secondary);
		line-height: 1.5;
		margin: 0;
	}

	.banner-stats {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 0.75rem;
	}

	.stat-chip {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.75rem 1rem;
		border-radius: var(--radius-lg);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
	}

	.stat-label {
		font-size: 0.75rem;
		color: var(--text-muted);
		font-weight: 500;
	}

	.stat-value {
		font-size: 0.95rem;
		font-weight: 700;
		color: var(--text-primary);
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.dot-green { background: var(--accent-green, #10b981); box-shadow: 0 0 8px #10b981; }
	.dot-amber { background: var(--accent-amber, #f59e0b); }
	.dot-gray { background: var(--text-muted, #64748b); }

	/* Banners */
	.new-release-banner, .new-snapshot-banner {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem 1.4rem;
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-sm);
	}

	.new-release-banner {
		background: rgba(16, 185, 129, 0.12);
		border: 1px solid rgba(16, 185, 129, 0.35);
		color: #34d399;
	}

	.new-snapshot-banner {
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

	.new-version-text span {
		color: var(--text-muted);
		font-size: 0.8rem;
	}

	.config-alert {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.85rem 1.25rem;
		border-radius: var(--radius-lg);
		background: rgba(245, 158, 11, 0.12);
		border: 1px solid rgba(245, 158, 11, 0.3);
		color: #fbbf24;
		font-size: 0.875rem;
	}

	/* Filters & Search Toolbar */
	.filters-toolbar {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.search-box {
		position: relative;
		display: flex;
		align-items: center;
		width: 100%;
	}

	:global(.search-icon) {
		position: absolute;
		left: 1rem;
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: 0.7rem 2.5rem 0.7rem 2.75rem;
		background: var(--bg-card);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-lg);
		color: var(--text-primary);
		font-size: 0.875rem;
		outline: none;
		transition: all 0.2s ease;
	}

	.search-input:focus {
		border-color: var(--accent-primary, #6366f1);
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
	}

	.clear-btn {
		position: absolute;
		right: 1rem;
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		font-size: 0.85rem;
	}

	.category-tabs {
		display: flex;
		gap: 0.5rem;
		background: var(--bg-card);
		padding: 0.35rem;
		border-radius: var(--radius-lg);
		border: 1px solid var(--border-subtle);
		overflow-x: auto;
	}

	.tab-btn {
		padding: 0.5rem 1rem;
		font-size: 0.825rem;
		font-weight: 600;
		border-radius: var(--radius-md);
		border: none;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.2s;
		white-space: nowrap;
	}

	.tab-btn:hover {
		color: var(--text-primary);
		background: var(--bg-hover);
	}

	.tab-btn.active {
		background: var(--accent-primary, #6366f1);
		color: #ffffff;
	}

	/* Engine Grid */
	.engine-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
		gap: var(--space-4);
	}

	.engine-card {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		padding: 1.5rem;
		border-radius: var(--radius-xl);
		background: var(--bg-card);
		border: 1px solid var(--border-subtle);
		transition: transform 0.2s, border-color 0.2s, box-shadow 0.2s;
		gap: 1rem;
	}

	.engine-card:hover {
		border-color: rgba(99, 102, 241, 0.4);
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.active-card {
		border-color: var(--accent-green, #10b981);
		background: linear-gradient(180deg, rgba(16, 185, 129, 0.06), var(--bg-card));
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
		width: 48px;
		height: 48px;
		border-radius: var(--radius-lg);
		background: var(--bg-base);
		color: var(--text-secondary);
		border: 1px solid var(--border-subtle);
		flex-shrink: 0;
	}

	.active-icon {
		background: rgba(16, 185, 129, 0.15);
		color: #10b981;
		border-color: rgba(16, 185, 129, 0.3);
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
		font-size: 1.2rem;
		font-weight: 700;
		color: var(--text-primary);
		margin: 0;
	}

	.tag-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.25rem;
		flex-wrap: wrap;
	}

	.category-tag {
		font-size: 0.75rem;
		color: var(--text-muted);
		font-weight: 500;
	}

	.stability-tag {
		font-size: 0.7rem;
		font-weight: 600;
		padding: 0.1rem 0.45rem;
		border-radius: 4px;
		background: rgba(99, 102, 241, 0.12);
		color: #a5b4fc;
		border: 1px solid rgba(99, 102, 241, 0.25);
	}

	.badge-active {
		background: rgba(16, 185, 129, 0.2);
		color: #34d399;
		border: 1px solid rgba(16, 185, 129, 0.35);
		padding: 0.2rem 0.5rem;
		border-radius: var(--radius-sm);
		font-size: 0.75rem;
		font-weight: 700;
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
	}

	/* Specs Bar */
	.specs-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 0.75rem;
		background: var(--bg-base);
		border-radius: var(--radius-md);
		border: 1px solid var(--border-subtle);
		font-size: 0.75rem;
	}

	.spec-item {
		display: flex;
		align-items: center;
		gap: 0.35rem;
	}

	.spec-label {
		color: var(--text-muted);
	}

	.spec-val {
		color: var(--text-primary);
		font-weight: 600;
	}

	.engine-description {
		font-size: 0.85rem;
		color: var(--text-secondary);
		line-height: 1.45;
		margin: 0;
		min-height: 2.5rem;
	}

	.features-list {
		display: flex;
		flex-direction: column;
		gap: 0.35rem;
	}

	.feature-item {
		display: flex;
		align-items: flex-start;
		gap: 0.4rem;
		font-size: 0.78rem;
		line-height: 1.35;
	}

	.pro-item span {
		color: var(--text-secondary);
	}

	.con-item span {
		color: #fbbf24;
	}

	.tools-sync-badge {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.4rem 0.6rem;
		border-radius: var(--radius-md);
		background: rgba(16, 185, 129, 0.08);
		border: 1px solid rgba(16, 185, 129, 0.2);
		color: #34d399;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.card-footer {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		margin-top: auto;
		padding-top: 0.5rem;
	}

	/* Modal */
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.75);
		backdrop-filter: blur(6px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal);
		padding: 1rem;
	}

	.modal-card {
		width: 100%;
		max-width: 620px;
		background: var(--bg-card);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		padding: 1.75rem;
		box-shadow: var(--shadow-lg);
		max-height: 90vh;
		overflow-y: auto;
	}

	.modal-header {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 1.25rem;
	}

	.modal-icon-box {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-lg);
		background: rgba(99, 102, 241, 0.15);
		color: #818cf8;
		border: 1px solid rgba(99, 102, 241, 0.3);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	h2 {
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--text-primary);
		margin: 0;
	}

	.modal-subtitle {
		font-size: 0.8rem;
		color: var(--text-muted);
		margin-top: 0.15rem;
	}

	.switch-diff-card {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
		border-radius: var(--radius-lg);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		margin-bottom: 1rem;
	}

	.diff-side {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}

	.diff-label {
		font-size: 0.7rem;
		font-weight: 700;
		text-transform: uppercase;
		color: var(--text-muted);
	}

	.diff-title {
		font-size: 1.1rem;
		font-weight: 800;
		color: var(--text-primary);
	}

	.diff-sub {
		font-size: 0.8rem;
		font-family: var(--font-mono);
		color: var(--text-muted);
	}

	.diff-arrow {
		color: var(--text-muted);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.alert-box {
		display: flex;
		align-items: flex-start;
		gap: 0.75rem;
		padding: 0.85rem 1rem;
		border-radius: var(--radius-md);
		font-size: 0.825rem;
		margin-bottom: 1rem;
	}

	.alert-danger {
		background: rgba(239, 68, 68, 0.12);
		border: 1px solid rgba(239, 68, 68, 0.35);
		color: #fca5a5;
	}

	.alert-danger strong {
		color: #ffffff;
	}

	.alert-danger p {
		margin: 0.2rem 0 0 0;
		color: #fca5a5;
	}

	.impact-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.impact-heading {
		font-size: 0.85rem;
		font-weight: 700;
		color: var(--text-secondary);
		margin: 0;
	}

	.impact-list {
		display: flex;
		flex-direction: column;
		gap: 0.65rem;
	}

	.impact-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0.75rem;
		border-radius: var(--radius-md);
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
	}

	.item-active {
		border-color: rgba(99, 102, 241, 0.35);
		background: rgba(99, 102, 241, 0.05);
	}

	.item-warning {
		border-color: rgba(245, 158, 11, 0.35);
		background: rgba(245, 158, 11, 0.05);
	}

	.impact-checkbox-row, .impact-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
	}

	.impact-checkbox-row {
		justify-content: flex-start;
	}

	.impact-label, .impact-title {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.85rem;
		font-weight: 600;
		color: var(--text-primary);
		cursor: pointer;
	}

	.custom-cb {
		width: 18px;
		height: 18px;
		accent-color: var(--accent-primary, #6366f1);
		cursor: pointer;
	}

	.impact-desc {
		font-size: 0.78rem;
		color: var(--text-muted);
		line-height: 1.4;
		margin: 0;
		padding-left: 1.5rem;
	}

	.badge-auto {
		font-size: 0.65rem;
		font-weight: 700;
		text-transform: uppercase;
		background: rgba(16, 185, 129, 0.15);
		color: #34d399;
		border: 1px solid rgba(16, 185, 129, 0.3);
		padding: 0.1rem 0.35rem;
		border-radius: 4px;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		margin-top: 1.5rem;
	}

	.switch-error {
		display: flex;
		align-items: flex-start;
		gap: 0.65rem;
		margin-top: 1rem;
		padding: 0.75rem;
		border-radius: var(--radius-md);
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
		font-family: var(--font-mono);
		color: #fecaca;
		word-break: break-word;
	}

	/* Buttons */
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		padding: 0.6rem 1.1rem;
		font-size: 0.875rem;
		font-weight: 600;
		border-radius: var(--radius-md);
		border: none;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.btn-sm {
		padding: 0.4rem 0.8rem;
		font-size: 0.8rem;
	}

	.btn-secondary {
		background: var(--bg-base);
		border: 1px solid var(--border-subtle);
		color: var(--text-secondary);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.btn-primary {
		background: var(--accent-primary, #6366f1);
		color: #ffffff;
	}

	.btn-primary:hover:not(:disabled) {
		opacity: 0.9;
		transform: translateY(-1px);
	}

	.btn-danger {
		background: var(--danger-bg, #ef4444);
		color: #ffffff;
	}

	.btn-danger:hover:not(:disabled) {
		opacity: 0.9;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.w-full {
		width: 100%;
	}

	.flex-align {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
	}

	.text-green, .text-emerald { color: var(--accent-green, #10b981) !important; }
	.text-primary { color: var(--accent-primary, #818cf8) !important; }
	.text-amber { color: var(--accent-amber, #fbbf24) !important; }
	.text-danger { color: var(--danger-text, #ef4444) !important; }
	.text-muted { color: var(--text-muted) !important; }

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	/* Toasts */
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
		border-radius: var(--radius-lg);
		background: var(--bg-card);
		border: 1px solid var(--border-subtle);
		box-shadow: var(--shadow-lg);
		color: var(--text-primary);
		min-width: 300px;
		animation: slideIn 0.2s ease-out;
	}

	.toast-success { border-left: 4px solid var(--accent-green, #10b981); }
	.toast-error { border-left: 4px solid var(--danger-bg, #ef4444); }
	.toast-warning { border-left: 4px solid var(--accent-amber, #f59e0b); }

	.toast-title {
		font-weight: 700;
		font-size: 0.85rem;
	}

	.toast-message {
		font-size: 0.75rem;
		color: var(--text-secondary);
	}

	.empty-state, .loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 4rem 1rem;
		gap: 1rem;
		color: var(--text-muted);
		text-align: center;
	}

	@keyframes slideIn {
		from { transform: translateX(100%); opacity: 0; }
		to { transform: translateX(0); opacity: 1; }
	}

	@media (max-width: 900px) {
		.banner-card {
			grid-template-columns: 1fr;
		}
		.banner-stats {
			grid-template-columns: 1fr;
		}
	}
</style>
