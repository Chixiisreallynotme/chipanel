<script>
	import {
		Search,
		Filter,
		RefreshCw,
		Trash2,
		Power,
		Package,
		Boxes,
		CheckCircle2,
		XCircle,
		AlertTriangle,
		X,
		FileText,
		HardDrive,
		Loader2,
		UploadCloud,
		Palette,
		ShieldCheck,
		Radio
	} from 'lucide-svelte';
	import { apiPost, apiFetch } from '$lib/api/client.js';

	/**
	 * @typedef {Object} InstalledPlugin
	 * @property {string} name
	 * @property {string} version
	 * @property {boolean} enabled
	 * @property {string} filename
	 * @property {number} file_size_bytes
	 * @property {string} loader_type
	 * @property {string} target_dir
	 * @property {string} [description]
	 * @property {number} [pack_format]
	 * @property {string} [sha1]
	 */

	let {
		plugins = [],
		loading = false,
		onToggle = async () => {},
		onDelete = async () => {},
		onRefresh = () => {},
		initialTargetDir = 'all',
		lockFolderFilter = false,
		onToast = () => {},
		serverResourcePack = null,
		onResourcePackChanged = () => {}
	} = $props();

	// Local filtering and modal states
	let searchQuery = $state('');
	let filterTargetDir = $state(initialTargetDir); // 'all', 'plugins', 'mods', 'resourcepacks', 'datapacks'
	let filterStatus = $state('all'); // 'all', 'enabled', 'disabled'
	let filterLoader = $state('all'); // 'all', 'paper', 'spigot', 'purpur', 'fabric', 'forge', 'neoforge', 'quilt'

	// Action tracking states
	let togglingFilename = $state(null);
	let deleteModalOpen = $state(false);
	let pluginToDelete = $state(null);
	let isDeleting = $state(false);

	// Batch selection state
	let selectedFilenames = $state([]);
	let isBatchProcessing = $state(false);

	// Drag and Drop Upload state
	let isDragging = $state(false);
	let isUploading = $state(false);
	let fileInputRef = $state(null);

	// Resource Pack server action states
	let settingServerPackFilename = $state(null);

	// Computed statistics
	let stats = $derived.by(() => {
		const total = plugins.length;
		const enabledCount = plugins.filter((p) => p.enabled).length;
		const disabledCount = total - enabledCount;
		const pluginsCount = plugins.filter((p) => p.target_dir === 'plugins').length;
		const modsCount = plugins.filter((p) => p.target_dir === 'mods').length;
		const rpCount = plugins.filter((p) => p.target_dir === 'resourcepacks').length;
		const dpCount = plugins.filter((p) => p.target_dir === 'datapacks').length;
		return { total, enabledCount, disabledCount, pluginsCount, modsCount, rpCount, dpCount };
	});

	// Filtered plugins array
	let filteredPlugins = $derived.by(() => {
		return plugins.filter((p) => {
			if (searchQuery.trim()) {
				const q = searchQuery.toLowerCase().trim();
				const nameMatch = (p.name || '').toLowerCase().includes(q);
				const fileMatch = (p.filename || '').toLowerCase().includes(q);
				const descMatch = (p.description || '').toLowerCase().includes(q);
				if (!nameMatch && !fileMatch && !descMatch) return false;
			}

			if (filterTargetDir !== 'all' && p.target_dir !== filterTargetDir) {
				return false;
			}

			if (filterStatus === 'enabled' && !p.enabled) return false;
			if (filterStatus === 'disabled' && p.enabled) return false;

			if (filterLoader !== 'all') {
				const loader = (p.loader_type || '').toLowerCase();
				if (filterLoader === 'spigot') {
					if (loader !== 'spigot' && loader !== 'paper' && loader !== 'purpur') return false;
				} else if (loader !== filterLoader) {
					return false;
				}
			}

			return true;
		});
	});

	// Formatting Helpers
	function formatFileSize(bytes) {
		if (!bytes || bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'Ko', 'Mo', 'Go'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatLoaderBadge(loader) {
		const l = (loader || '').toLowerCase();
		if (l === 'paper' || l === 'spigot' || l === 'purpur') return 'Paper/Spigot';
		if (l === 'fabric') return 'Fabric';
		if (l === 'forge') return 'Forge';
		if (l === 'neoforge') return 'NeoForge';
		if (l === 'quilt') return 'Quilt';
		if (l === 'resourcepack') return 'Pack Texture';
		if (l === 'datapack') return 'Data Pack';
		return loader ? loader.charAt(0).toUpperCase() + loader.slice(1) : 'Spigot';
	}

	function getLoaderBadgeClass(loader) {
		const l = (loader || '').toLowerCase();
		if (l === 'fabric') return 'badge-blue';
		if (l === 'forge') return 'badge-warning';
		if (l === 'neoforge') return 'badge-purple';
		if (l === 'paper' || l === 'spigot' || l === 'purpur') return 'badge-success';
		if (l === 'resourcepack') return 'badge-amber';
		if (l === 'datapack') return 'badge-teal';
		return 'badge-secondary';
	}

	// Selection Handlers
	function toggleSelectAll() {
		if (selectedFilenames.length === filteredPlugins.length) {
			selectedFilenames = [];
		} else {
			selectedFilenames = filteredPlugins.map((p) => p.filename);
		}
	}

	function toggleSelectItem(filename) {
		if (selectedFilenames.includes(filename)) {
			selectedFilenames = selectedFilenames.filter((f) => f !== filename);
		} else {
			selectedFilenames = [...selectedFilenames, filename];
		}
	}

	// Batch Action Handlers
	async function handleBatchToggle(enable) {
		if (isBatchProcessing || selectedFilenames.length === 0) return;
		isBatchProcessing = true;
		try {
			const targetItems = plugins.filter(
				(p) => selectedFilenames.includes(p.filename) && p.enabled !== enable
			);
			for (const item of targetItems) {
				await onToggle(item);
			}
			onToast?.('success', 'Action groupée', `${targetItems.length} extension(s) mise(s) à jour.`);
			selectedFilenames = [];
		} catch (err) {
			onToast?.('error', 'Erreur action groupée', err.message || 'Une erreur est survenue.');
		} finally {
			isBatchProcessing = false;
		}
	}

	async function handleBatchDelete() {
		if (isBatchProcessing || selectedFilenames.length === 0) return;
		if (!confirm(`Supprimer définitivement les ${selectedFilenames.length} extensions sélectionnées ?`)) {
			return;
		}
		isBatchProcessing = true;
		try {
			const targetItems = plugins.filter((p) => selectedFilenames.includes(p.filename));
			for (const item of targetItems) {
				await onDelete(item);
			}
			onToast?.('success', 'Suppression groupée', `${targetItems.length} extension(s) supprimée(s).`);
			selectedFilenames = [];
		} catch (err) {
			onToast?.('error', 'Erreur suppression groupée', err.message || 'Échec de la suppression.');
		} finally {
			isBatchProcessing = false;
		}
	}

	// Drag & Drop Upload
	async function handleFilesUpload(fileList) {
		if (!fileList || fileList.length === 0 || isUploading) return;
		isUploading = true;
		const targetDir = initialTargetDir !== 'all' ? initialTargetDir : 'plugins';

		let successCount = 0;
		for (const file of Array.from(fileList)) {
			if (!file.name.endsWith('.jar') && !file.name.endsWith('.zip') && !file.name.endsWith('.plugin')) {
				onToast?.('error', 'Format invalide', `Le fichier "${file.name}" n'est pas un .jar ou .zip valide.`);
				continue;
			}

			try {
				const formData = new FormData();
				formData.append('file', file);
				formData.append('target_dir', targetDir);

				const token = sessionStorage.getItem('chipanel_jwt');
				const res = await fetch('/api/plugins/upload', {
					method: 'POST',
					headers: token ? { Authorization: `Bearer ${token}` } : {},
					body: formData
				});

				if (!res.ok) {
					const errJson = await res.json().catch(() => ({}));
					throw new Error(errJson.message || `Erreur HTTP ${res.status}`);
				}

				successCount++;
			} catch (err) {
				onToast?.('error', 'Échec upload', `Impossible d'importer "${file.name}": ${err.message}`);
			}
		}

		if (successCount > 0) {
			onToast?.('success', 'Import réussi', `${successCount} extension(s) importée(s) dans /${targetDir}/.`);
			onRefresh?.();
		}
		isUploading = false;
	}

	function handleDrop(e) {
		e.preventDefault();
		isDragging = false;
		if (e.dataTransfer?.files) {
			handleFilesUpload(e.dataTransfer.files);
		}
	}

	// Server Resource Pack Activation / Disabling
	async function handleSetServerResourcePack(plugin) {
		settingServerPackFilename = plugin.filename;
		try {
			const res = await apiPost('/api/server/resource-pack/activate', {
				filename: plugin.filename,
				prompt: 'Pack de textures obligatoire pour rejoindre ce serveur.'
			});
			onToast?.(
				'success',
				'Pack serveur activé',
				`"${plugin.name}" est maintenant le pack officiel obligatoire pour tous les joueurs.`
			);
			onResourcePackChanged?.(res);
		} catch (err) {
			onToast?.('error', 'Échec configuration pack', err.message || 'Impossible d\'activer le pack serveur.');
		} finally {
			settingServerPackFilename = null;
		}
	}

	async function handleDisableServerResourcePack() {
		try {
			await apiPost('/api/server/resource-pack/disable', {});
			onToast?.('info', 'Pack serveur retiré', 'Le pack de texture obligatoire a été désactivé.');
			onResourcePackChanged?.(null);
		} catch (err) {
			onToast?.('error', 'Erreur désactivation', err.message || 'Impossible de désactiver le pack.');
		}
	}

	// Individual Action Handlers
	async function handleToggleClick(plugin) {
		if (togglingFilename) return;
		togglingFilename = plugin.filename;
		try {
			await onToggle(plugin);
		} finally {
			togglingFilename = null;
		}
	}

	function openDeleteModal(plugin) {
		pluginToDelete = plugin;
		deleteModalOpen = true;
	}

	function closeDeleteModal() {
		if (isDeleting) return;
		deleteModalOpen = false;
		pluginToDelete = null;
	}

	async function confirmDelete() {
		if (!pluginToDelete || isDeleting) return;
		isDeleting = true;
		try {
			await onDelete(pluginToDelete);
			closeDeleteModal();
		} catch (err) {
			console.error('Delete error:', err);
		} finally {
			isDeleting = false;
		}
	}

	function handleWindowKeydown(e) {
		if (e.key === 'Escape' && deleteModalOpen) {
			closeDeleteModal();
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="installed-plugins-container">
	<!-- Drag & Drop Upload Zone -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="dropzone-banner {isDragging ? 'dropzone-active' : ''} {isUploading ? 'dropzone-uploading' : ''}"
		ondragover={(e) => { e.preventDefault(); isDragging = true; }}
		ondragleave={() => (isDragging = false)}
		ondrop={handleDrop}
	>
		<div class="dropzone-content">
			{#if isUploading}
				<Loader2 size={24} class="spin text-blue" />
				<span>Téléversement et inspection de l'extension en cours…</span>
			{:else}
				<UploadCloud size={24} class="dropzone-icon" />
				<div>
					<span class="dropzone-title">Glissez-déposez vos fichiers <code>.jar</code> ou <code>.zip</code> ici</span>
					<span class="dropzone-desc">ou <button type="button" class="link-btn" onclick={() => fileInputRef?.click()}>parcourez vos dossiers</button> pour installer directement.</span>
				</div>
			{/if}
		</div>
		<input
			type="file"
			bind:this={fileInputRef}
			multiple
			accept=".jar,.zip,.plugin"
			style="display: none;"
			onchange={(e) => handleFilesUpload(e.target?.files)}
		/>
	</div>

	<!-- Summary Stats Bar -->
	<div class="stats-row">
		<div class="stat-card">
			<div class="stat-icon icon-blue">
				<Package size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.total}</span>
				<span class="stat-label">Total installés</span>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon icon-green">
				<CheckCircle2 size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.enabledCount}</span>
				<span class="stat-label">Actifs</span>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon icon-danger">
				<XCircle size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.disabledCount}</span>
				<span class="stat-label">Désactivés</span>
			</div>
		</div>

		<div class="stat-card">
			<div class="stat-icon icon-purple">
				<Boxes size={20} />
			</div>
			<div class="stat-content">
				<span class="stat-value">{stats.pluginsCount} P / {stats.modsCount} M / {stats.rpCount} T</span>
				<span class="stat-label">Répartition</span>
			</div>
		</div>
	</div>

	<!-- Server Resource Pack Active Notice (if in resourcepacks mode) -->
	{#if initialTargetDir === 'resourcepacks' && serverResourcePack?.active_filename}
		<div class="server-rp-banner">
			<div class="server-rp-left">
				<div class="server-rp-icon">
					<Radio size={20} />
				</div>
				<div>
					<div class="server-rp-title">
						Pack de textures serveur actif : <strong>{serverResourcePack.active_filename}</strong>
					</div>
					<div class="server-rp-sub">
						Téléchargement automatique et obligatoire pour tout joueur se connectant au serveur (SHA-1: <code>{serverResourcePack.sha1?.slice(0, 10)}…</code>).
					</div>
				</div>
			</div>
			<button type="button" class="btn btn-secondary btn-sm" onclick={handleDisableServerResourcePack}>
				Désactiver du serveur
			</button>
		</div>
	{/if}

	<!-- Batch Operations Bar (when items selected) -->
	{#if selectedFilenames.length > 0}
		<div class="batch-bar">
			<span class="batch-count">
				<strong>{selectedFilenames.length}</strong> extension(s) sélectionnée(s)
			</span>
			<div class="batch-actions">
				<button
					type="button"
					class="btn btn-secondary btn-sm"
					disabled={isBatchProcessing}
					onclick={() => handleBatchToggle(true)}
				>
					<Power size={14} />
					<span>Activer</span>
				</button>
				<button
					type="button"
					class="btn btn-secondary btn-sm"
					disabled={isBatchProcessing}
					onclick={() => handleBatchToggle(false)}
				>
					<Power size={14} />
					<span>Désactiver</span>
				</button>
				<button
					type="button"
					class="btn btn-danger btn-sm"
					disabled={isBatchProcessing}
					onclick={handleBatchDelete}
				>
					<Trash2 size={14} />
					<span>Supprimer</span>
				</button>
			</div>
		</div>
	{/if}

	<!-- Filter & Search Controls Header -->
	<div class="controls-card card">
		<div class="controls-body">
			<div class="search-input-wrapper">
				<Search size={18} class="search-icon" />
				<input
					type="text"
					class="input search-input"
					placeholder="Rechercher par nom, fichier ou description…"
					bind:value={searchQuery}
					aria-label="Rechercher des extensions"
				/>
				{#if searchQuery}
					<button
						class="btn btn-ghost btn-icon btn-sm clear-search-btn"
						onclick={() => (searchQuery = '')}
						title="Effacer la recherche"
					>
						<X size={14} />
					</button>
				{/if}
			</div>

			<div class="filter-group">
				{#if !lockFolderFilter}
					<div class="filter-select-wrapper">
						<span class="filter-label">Dossier :</span>
						<select class="select select-sm" bind:value={filterTargetDir} aria-label="Filtrer par dossier">
							<option value="all">Tous les dossiers</option>
							<option value="plugins">plugins/</option>
							<option value="mods">mods/</option>
							<option value="resourcepacks">resourcepacks/</option>
							<option value="datapacks">datapacks/</option>
						</select>
					</div>
				{/if}

				<!-- Status Filter -->
				<div class="filter-select-wrapper">
					<span class="filter-label">Statut :</span>
					<select class="select select-sm" bind:value={filterStatus} aria-label="Filtrer par statut">
						<option value="all">Tous</option>
						<option value="enabled">Actifs uniquement</option>
						<option value="disabled">Désactivés uniquement</option>
					</select>
				</div>

				<!-- Loader Filter -->
				<div class="filter-select-wrapper">
					<span class="filter-label">Chargeur :</span>
					<select class="select select-sm" bind:value={filterLoader} aria-label="Filtrer par chargeur">
						<option value="all">Tous</option>
						<option value="spigot">Paper / Spigot / Purpur</option>
						<option value="fabric">Fabric</option>
						<option value="forge">Forge</option>
						<option value="neoforge">NeoForge</option>
						<option value="quilt">Quilt</option>
					</select>
				</div>

				<button
					class="btn btn-secondary btn-sm refresh-btn {loading ? 'btn-loading' : ''}"
					onclick={onRefresh}
					disabled={loading}
					title="Actualiser la liste"
				>
					{#if !loading}
						<RefreshCw size={14} />
					{/if}
					<span>Actualiser</span>
				</button>
			</div>
		</div>
	</div>

	<!-- Plugins Data Table -->
	<div class="table-container shadow-card">
		{#if loading && plugins.length === 0}
			<div class="loading-state">
				<Loader2 size={32} class="spinner" />
				<p>Analyse des dossiers d'extensions du serveur…</p>
			</div>
		{:else if filteredPlugins.length === 0}
			<div class="table-empty">
				<Package size={40} class="empty-icon" />
				<h3>Aucune extension trouvée</h3>
				<p>
					{#if searchQuery || filterTargetDir !== 'all' || filterStatus !== 'all' || filterLoader !== 'all'}
						Aucune extension installée ne correspond à vos filtres actuels.
					{:else}
						Vous n'avez pas encore d'extension installée dans ce dossier. Glissez un fichier ou utilisez l'onglet Catalogue pour en installer !
					{/if}
				</p>
			</div>
		{:else}
			<table class="table">
				<thead>
					<tr>
						<th style="width: 40px;">
							<input
								type="checkbox"
								checked={selectedFilenames.length > 0 && selectedFilenames.length === filteredPlugins.length}
								onchange={toggleSelectAll}
								aria-label="Tout sélectionner"
							/>
						</th>
						<th>Nom & Fichier</th>
						<th>Version</th>
						<th>Chargeur</th>
						<th>Dossier</th>
						<th>Taille</th>
						<th>Statut</th>
						<th class="text-right">Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each filteredPlugins as plugin (plugin.filename)}
						<tr class={!plugin.enabled ? 'row-disabled' : ''}>
							<td>
								<input
									type="checkbox"
									checked={selectedFilenames.includes(plugin.filename)}
									onchange={() => toggleSelectItem(plugin.filename)}
									aria-label="Sélectionner {plugin.name}"
								/>
							</td>

							<!-- Name & Filename -->
							<td>
								<div class="plugin-name-cell">
									<div class="file-icon-box {plugin.enabled ? 'icon-active' : 'icon-inactive'}">
										{#if plugin.target_dir === 'resourcepacks'}
											<Palette size={18} />
										{:else}
											<FileText size={18} />
										{/if}
									</div>
									<div class="name-info">
										<div class="title-with-badge">
											<span class="plugin-title">{plugin.name}</span>
											{#if plugin.target_dir === 'resourcepacks' && serverResourcePack?.active_filename === plugin.filename}
												<span class="badge badge-amber badge-sm">Pack Serveur</span>
											{/if}
										</div>
										<code class="plugin-filename">{plugin.filename}</code>
										{#if plugin.description}
											<p class="plugin-desc-snippet">{plugin.description}</p>
										{/if}
									</div>
								</div>
							</td>

							<!-- Version -->
							<td>
								<span class="version-tag">v{plugin.version}</span>
							</td>

							<!-- Loader Badge -->
							<td>
								<span class="badge {getLoaderBadgeClass(plugin.loader_type)}">
									{formatLoaderBadge(plugin.loader_type)}
								</span>
							</td>

							<!-- Target Directory -->
							<td>
								<span class="badge badge-secondary dir-badge">
									<HardDrive size={12} />
									{plugin.target_dir}/
								</span>
							</td>

							<!-- File Size -->
							<td>
								<span class="file-size">{formatFileSize(plugin.file_size_bytes)}</span>
							</td>

							<!-- Status Badge -->
							<td>
								{#if plugin.enabled}
									<span class="badge badge-success">
										<span class="status-dot status-dot-success status-dot-pulse"></span>
										Actif
									</span>
								{:else}
									<span class="badge badge-danger">
										<span class="status-dot status-dot-danger"></span>
										Désactivé
									</span>
								{/if}
							</td>

							<!-- Actions -->
							<td class="text-right">
								<div class="action-buttons">
									<!-- Set as Server Resource Pack if in resourcepacks -->
									{#if plugin.target_dir === 'resourcepacks'}
										{#if serverResourcePack?.active_filename === plugin.filename}
											<button
												type="button"
												class="btn btn-sm btn-ghost"
												disabled
												title="Déjà configuré comme pack serveur obligatoire"
											>
												<ShieldCheck size={14} class="text-green" />
												<span>Actif</span>
											</button>
										{:else}
											<button
												type="button"
												class="btn btn-sm btn-secondary {settingServerPackFilename === plugin.filename ? 'btn-loading' : ''}"
												disabled={settingServerPackFilename === plugin.filename}
												onclick={() => handleSetServerResourcePack(plugin)}
												title="Définir comme pack de textures obligatoire pour le serveur"
											>
												<Palette size={14} />
												<span>Pack Serveur</span>
											</button>
										{/if}
									{/if}

									<!-- Toggle Enable / Disable -->
									<button
										class="btn btn-sm {plugin.enabled ? 'btn-secondary' : 'btn-primary'} {togglingFilename === plugin.filename ? 'btn-loading' : ''}"
										disabled={togglingFilename === plugin.filename}
										onclick={() => handleToggleClick(plugin)}
										title={plugin.enabled ? 'Désactiver l\'extension' : 'Activer l\'extension'}
									>
										{#if togglingFilename !== plugin.filename}
											<Power size={14} />
										{/if}
										<span>{plugin.enabled ? 'Désactiver' : 'Activer'}</span>
									</button>

									<!-- Delete Button -->
									<button
										class="btn btn-ghost btn-icon btn-sm delete-action-btn"
										onclick={() => openDeleteModal(plugin)}
										title="Supprimer le fichier"
									>
										<Trash2 size={16} />
									</button>
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>

<!-- Delete Confirmation Modal -->
{#if deleteModalOpen && pluginToDelete}
	<div class="modal-backdrop" onclick={closeDeleteModal} role="presentation">
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="modal-card card shadow-lg" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
			<div class="modal-header">
				<div class="modal-title-with-icon">
					<div class="modal-icon-box icon-danger">
						<AlertTriangle size={20} />
					</div>
					<h3>Confirmer la suppression</h3>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeDeleteModal} disabled={isDeleting}>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				<p>
					Êtes-vous sûr de vouloir supprimer définitivement l'extension <strong>{pluginToDelete.name}</strong> (<code>{pluginToDelete.filename}</code>) ?
				</p>
				<p class="modal-muted-warning">
					Cette action effacera le fichier du dossier <code>/{pluginToDelete.target_dir}/</code>.
				</p>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeDeleteModal} disabled={isDeleting}>
					Annuler
				</button>
				<button class="btn btn-danger {isDeleting ? 'btn-loading' : ''}" onclick={confirmDelete} disabled={isDeleting}>
					{#if !isDeleting}
						<Trash2 size={16} />
					{/if}
					<span>Supprimer définitivement</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.installed-plugins-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	/* Dropzone Banner */
	.dropzone-banner {
		background-color: var(--bg-surface);
		border: 2px dashed var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-4) var(--space-5);
		transition: all var(--transition-fast);
		cursor: pointer;
	}
	.dropzone-banner:hover, .dropzone-active {
		border-color: var(--accent-blue);
		background-color: var(--accent-blue-bg, rgba(59, 130, 246, 0.06));
	}
	.dropzone-uploading {
		border-color: var(--accent-blue);
	}
	.dropzone-content {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}
	.dropzone-icon {
		color: var(--accent-blue);
		flex-shrink: 0;
	}
	.dropzone-title {
		display: block;
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
	}
	.dropzone-desc {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}
	.link-btn {
		background: none;
		border: none;
		padding: 0;
		color: var(--accent-blue);
		cursor: pointer;
		text-decoration: underline;
		font-size: inherit;
	}

	/* Stats Row */
	.stats-row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: var(--space-3);
	}
	.stat-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.stat-icon {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
	.icon-blue { background-color: rgba(59, 130, 246, 0.12); color: #60a5fa; }
	.icon-green { background-color: rgba(34, 197, 94, 0.12); color: #4ade80; }
	.icon-danger { background-color: rgba(239, 68, 68, 0.12); color: #f87171; }
	.icon-purple { background-color: rgba(168, 85, 247, 0.12); color: #c084fc; }

	.stat-content {
		display: flex;
		flex-direction: column;
	}
	.stat-value {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}
	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	/* Server Resource Pack Banner */
	.server-rp-banner {
		background-color: var(--accent-amber-bg, rgba(245, 158, 11, 0.1));
		border: 1px solid var(--accent-amber-border, rgba(245, 158, 11, 0.3));
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}
	.server-rp-left {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.server-rp-icon {
		color: #f59e0b;
	}
	.server-rp-title {
		font-size: var(--font-size-sm);
		color: var(--text-primary);
	}
	.server-rp-sub {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	/* Batch Bar */
	.batch-bar {
		background-color: var(--accent-blue-bg, rgba(59, 130, 246, 0.12));
		border: 1px solid var(--accent-blue-border, rgba(59, 130, 246, 0.3));
		border-radius: var(--radius-card);
		padding: var(--space-2) var(--space-4);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.batch-count {
		font-size: var(--font-size-sm);
		color: var(--text-primary);
	}
	.batch-actions {
		display: flex;
		gap: var(--space-2);
	}

	/* Controls Card */
	.controls-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
	}
	.controls-body {
		padding: var(--space-3) var(--space-4);
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-3);
		align-items: center;
		justify-content: space-between;
	}
	.search-input-wrapper {
		position: relative;
		flex: 1;
		min-width: 260px;
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
	.filter-group {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-3);
		align-items: center;
	}
	.filter-select-wrapper {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.filter-label {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	/* Table Layout */
	.table-container {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		overflow-x: auto;
	}
	.table {
		width: 100%;
		border-collapse: collapse;
		font-size: var(--font-size-sm);
	}
	.table th, .table td {
		padding: var(--space-3) var(--space-4);
		text-align: left;
		border-bottom: 1px solid var(--border);
	}
	.table th {
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.02));
		color: var(--text-secondary);
		font-weight: var(--font-weight-medium);
		font-size: var(--font-size-xs);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
	.row-disabled {
		opacity: 0.65;
	}
	.plugin-name-cell {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.file-icon-box {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}
	.icon-active { background-color: rgba(59, 130, 246, 0.1); color: #60a5fa; }
	.icon-inactive { background-color: rgba(148, 163, 184, 0.1); color: #94a3b8; }
	.name-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.title-with-badge {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}
	.plugin-title {
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}
	.plugin-filename {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}
	.plugin-desc-snippet {
		font-size: 11px;
		color: var(--text-secondary);
		margin: 0;
		max-width: 400px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.version-tag {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}
	.dir-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-family: var(--font-mono);
		font-size: 11px;
	}
	.file-size {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}
	.status-dot {
		display: inline-block;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		margin-right: 4px;
	}
	.status-dot-success { background-color: #22c55e; }
	.status-dot-danger { background-color: #ef4444; }
	.status-dot-pulse {
		box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.4);
		animation: pulse 2s infinite;
	}
	.action-buttons {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	.delete-action-btn:hover {
		color: var(--danger);
	}

	.loading-state, .table-empty {
		padding: var(--space-8);
		text-align: center;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: var(--text-secondary);
	}
	.empty-icon {
		color: var(--text-muted);
		margin-bottom: var(--space-2);
	}

	/* Modal styles */
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
		max-width: 480px;
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
	.modal-title-with-icon {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.modal-icon-box {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.modal-body {
		padding: var(--space-4);
		color: var(--text-secondary);
		font-size: var(--font-size-sm);
	}
	.modal-muted-warning {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: var(--space-2);
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

	@keyframes pulse {
		0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7); }
		70% { transform: scale(1); box-shadow: 0 0 0 6px rgba(34, 197, 94, 0); }
		100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(34, 197, 94, 0); }
	}
</style>
