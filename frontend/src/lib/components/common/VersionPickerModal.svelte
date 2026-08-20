<script>
	import {
		X,
		Search,
		CheckCircle2,
		Sparkles,
		Flame,
		Check,
		Layers,
		FlaskConical,
		HelpCircle
	} from '$lib/icons.js';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import Modal from '$lib/components/ui/Modal.svelte';

	let {
		open = false,
		selectedVersion = '',
		releaseVersions = [],
		snapshotVersions = [],
		recommendedVersions = [],
		minVersion = null,
		maxVersion = null,
		allowAll = true,
		title = 'Sélectionner une version de Minecraft',
		onSelect = () => {},
		onClose = () => {}
	} = $props();

	let searchQuery = $state('');
	let draftVersion = $state('');

	// Sync draft selection with incoming selectedVersion on open
	$effect(() => {
		if (open) {
			draftVersion = selectedVersion || (allowAll ? 'all' : (releaseVersions[0] || '1.20.4'));
			searchQuery = '';
		}
	});

	// Normalize arrays
	let safeReleases = $derived(Array.isArray(releaseVersions) ? releaseVersions : []);
	let safeSnapshots = $derived(Array.isArray(snapshotVersions) ? snapshotVersions : []);
	let safeRecommended = $derived(Array.isArray(recommendedVersions) ? recommendedVersions : []);

	function isVersionSupported(ver) {
		if (!ver) return true;
		if (ver === 'all' || ver === 'LATEST') {
			if (maxVersion && safeReleases.length > 0) {
				const maxIdx = safeReleases.findIndex((r) => r.toLowerCase() === maxVersion.toLowerCase());
				if (maxIdx > 0) return false;
			}
			return true;
		}
		if (ver === 'SNAPSHOT') {
			return !maxVersion;
		}
		const relIdx = safeReleases.findIndex((r) => r.toLowerCase() === ver.toLowerCase());
		if (relIdx !== -1) {
			if (minVersion) {
				const minIdx = safeReleases.findIndex((r) => r.toLowerCase() === minVersion.toLowerCase());
				if (minIdx !== -1 && relIdx > minIdx) return false;
			}
			if (maxVersion) {
				const maxIdx = safeReleases.findIndex((r) => r.toLowerCase() === maxVersion.toLowerCase());
				if (maxIdx !== -1 && relIdx < maxIdx) return false;
			}
			return true;
		}
		// Snapshots
		if (safeSnapshots.some((s) => s.toLowerCase() === ver.toLowerCase())) {
			return !maxVersion;
		}
		return true;
	}

	// Filtered releases
	let filteredReleases = $derived(
		safeReleases.filter((ver) =>
			ver.toLowerCase().includes(searchQuery.trim().toLowerCase())
		)
	);

	// Filtered snapshots
	let filteredSnapshots = $derived(
		safeSnapshots.filter((ver) =>
			ver.toLowerCase().includes(searchQuery.trim().toLowerCase())
		)
	);

	// Group release versions by major release prefix (e.g. 1.21, 1.20, 1.19...)
	let groupedReleases = $derived.by(() => {
		const groups = {};
		filteredReleases.forEach((ver) => {
			if (ver === 'LATEST') {
				if (!groups['Spécial']) groups['Spécial'] = [];
				groups['Spécial'].push(ver);
				return;
			}
			const parts = ver.split('.');
			const majorKey = parts.length >= 2 ? `${parts[0]}.${parts[1]}.x` : 'Autres';
			if (!groups[majorKey]) groups[majorKey] = [];
			groups[majorKey].push(ver);
		});
		return Object.entries(groups);
	});

	function handleConfirm() {
		if (draftVersion && isVersionSupported(draftVersion)) {
			onSelect(draftVersion);
		}
		onClose();
	}
</script>

<Modal open={open} onclose={onClose} class="modal-card version-picker-modal" backdropClass="modal-backdrop--dim" ariaLabelledBy="modal-title">
	{#snippet content()}
			<!-- Header -->
			<div class="modal-header">
				<div class="header-title-box">
					<div class="icon-badge">
						<Layers size={20} />
					</div>
					<div>
						<h2 id="modal-title" class="modal-title">{title}</h2>
						<p class="modal-subtitle">
							Choisissez parmi les versions officielles stables ou les snapshots de développement.
						</p>
					</div>
				</div>

				<button
					type="button"
					class="close-btn"
					onclick={onClose}
					title="Fermer"
					aria-label="Fermer la fenêtre"
				>
					<X size={18} />
				</button>
			</div>

			<!-- Search Bar -->
			<div class="modal-search-bar">
				<div class="search-input-wrapper">
					<Search size={16} class="search-icon" />
					<input
						type="text"
						class="search-input"
						placeholder="Rechercher une version (ex: 1.20.4, 24w14a)..."
						bind:value={searchQuery}
						aria-label="Rechercher une version"
					/>
					{#if searchQuery}
						<button
							type="button"
							class="clear-search-btn"
							onclick={() => (searchQuery = '')}
							title="Effacer la recherche"
						>
							<X size={14} />
						</button>
					{/if}
				</div>
			</div>

			<!-- Columns Body (Left: Releases Stables | Right: Snapshots) -->
			<div class="modal-body-columns">
				<!-- Left Column: Releases Stables -->
				<div class="version-column releases-column">
					<div class="column-header">
						<div class="column-title-box">
							<CheckCircle2 size={16} class="text-success" />
							<span>Releases Stables ({filteredReleases.length})</span>
						</div>
					</div>

					<div class="version-list-scroll">
						{#if allowAll && !searchQuery}
							<div class="version-group">
								<div class="group-header">Filtre Global</div>
								<button
									type="button"
									class="version-pill version-pill-all {draftVersion === 'all' ? 'selected' : ''}"
									onclick={() => (draftVersion = 'all')}
								>
									<span class="version-name">
										<Sparkles size={14} class="filter-all-icon" />
										<span>Toutes les versions (Sans filtre)</span>
									</span>
									{#if draftVersion === 'all'}
										<Check size={14} class="check-icon" />
									{/if}
								</button>
							</div>
						{/if}

						{#if filteredReleases.length === 0}
							<EmptyState description={`Aucune version stable trouvée pour "${searchQuery}".`} />
						{:else}
							{#each groupedReleases as [groupName, versions]}
								<div class="version-group">
									<div class="group-header">{groupName}</div>
									<div class="version-grid">
										{#each versions as ver}
											{@const isDraft = draftVersion === ver}
											{@const isRecommended = safeRecommended.includes(ver)}
											{@const isLatest = ver === safeReleases[0] || ver === 'LATEST'}
											{@const supported = isVersionSupported(ver)}

											<button
												type="button"
												class="version-pill {isDraft ? 'selected' : ''} {!supported ? 'unsupported' : ''}"
												disabled={!supported}
												onclick={() => {
													if (supported) draftVersion = ver;
												}}
												title={!supported ? `Version non supportée (${minVersion ? `Min: ${minVersion}` : ''}${maxVersion ? ` Max: ${maxVersion}` : ''})` : ''}
											>
												<span class="version-name">{ver}</span>
												{#if isDraft}
													<Check size={14} class="check-icon" />
												{/if}
												{#if !supported}
													<span class="badge badge-unsupported">Incompatible</span>
												{:else if isRecommended}
													<span class="badge badge-recommended" title="Version recommandée">
														<Sparkles size={10} />
														<span>Top</span>
													</span>
												{:else if isLatest}
													<span class="badge badge-latest" title="Dernière version stable">
														<Flame size={10} />
														<span>New</span>
													</span>
												{/if}
											</button>
										{/each}
									</div>
								</div>
							{/each}
						{/if}
					</div>
				</div>

				<!-- Right Column: Snapshots & Pre-Releases -->
				<div class="version-column snapshots-column">
					<div class="column-header">
						<div class="column-title-box">
							<FlaskConical size={16} class="text-purple" />
							<span>Snapshots & Pre-releases ({filteredSnapshots.length})</span>
						</div>
					</div>

					<div class="version-list-scroll">
						{#if filteredSnapshots.length === 0}
							<EmptyState
									description={safeSnapshots.length === 0 ? 'Aucune snapshot disponible pour ce moteur.' : `Aucune snapshot trouvée pour "${searchQuery}".`}
								/>
						{:else}
							<div class="version-grid">
								{#each filteredSnapshots as ver}
									{@const isDraft = draftVersion === ver}
									{@const supported = isVersionSupported(ver)}
									<button
										type="button"
										class="version-pill version-pill-snapshot {isDraft ? 'selected' : ''} {!supported ? 'unsupported' : ''}"
										disabled={!supported}
										onclick={() => {
											if (supported) draftVersion = ver;
										}}
										title={!supported ? 'Les snapshots ne sont pas supportées par ce moteur' : ''}
									>
										<span class="version-name">{ver}</span>
										{#if isDraft}
											<Check size={14} class="check-icon" />
										{/if}
										{#if !supported}
											<span class="badge badge-unsupported">Incompatible</span>
										{:else}
											<span class="badge badge-snapshot">Exp</span>
										{/if}
									</button>
								{/each}
							</div>
						{/if}
					</div>
				</div>
			</div>

			<!-- Footer Bar with Confirmation Button -->
			<div class="modal-footer">
				<div class="selected-summary">
					<span class="summary-label">Version sélectionnée :</span>
					<span class="summary-value">
						{draftVersion === 'all' ? 'Toutes les versions' : (draftVersion || 'Aucune')}
					</span>
				</div>

				<div class="footer-actions">
					<button
						type="button"
						class="btn btn-secondary"
						onclick={onClose}
					>
						Annuler
					</button>

					<button
						type="button"
						class="btn btn-primary"
						disabled={!draftVersion}
						onclick={handleConfirm}
					>
						<Check size={16} />
						<span>Confirmer la version</span>
					</button>
				</div>
			</div>
		{/snippet}
	</Modal>

<style>
	.version-picker-modal {
		display: flex;
		flex-direction: column;
		width: 100%;
		max-width: 900px;
		max-height: 85vh;
		background: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: var(--modal-shadow);
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.25rem 1.5rem;
		border-bottom: 1px solid var(--border-subtle);
		background: rgba(255, 255, 255, 0.02);
	}

	.header-title-box {
		display: flex;
		align-items: center;
		gap: 0.875rem;
	}

	.icon-badge {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border-radius: var(--radius-btn);
		background: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		color: var(--accent-blue-text);
	}

	.modal-title {
		font-size: 1.125rem;
		font-weight: 700;
		color: var(--text-primary);
		margin: 0;
	}

	.modal-subtitle {
		font-size: 0.8125rem;
		color: var(--text-secondary);
		margin-top: 0.125rem;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: var(--radius-btn);
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast), transform var(--transition-fast);
	}

	.close-btn:hover {
		background: rgba(255, 255, 255, 0.08);
		color: var(--text-primary);
	}

	.close-btn:active {
		transform: scale(0.96);
	}

	/* Search Bar */
	.modal-search-bar {
		padding: 1rem 1.5rem;
		border-bottom: 1px solid var(--border-subtle);
		background: var(--bg-base);
	}

	.search-input-wrapper {
		position: relative;
		display: flex;
		align-items: center;
		width: 100%;
	}

	:global(.search-icon) {
		position: absolute;
		left: 0.875rem;
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: 0.625rem 2.5rem 0.625rem 2.5rem;
		background: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
		outline: none;
		transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
	}

	.search-input:focus {
		border-color: var(--accent-blue);
		box-shadow: 0 0 0 2px var(--accent-blue-bg);
	}

	.clear-search-btn {
		position: absolute;
		right: 0.75rem;
		background: transparent;
		border: none;
		color: #64748b;
		cursor: pointer;
		padding: 0.25rem;
	}

	.clear-search-btn:hover {
		color: #ffffff;
	}

	/* Columns Body */
	.modal-body-columns {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1px;
		background: rgba(255, 255, 255, 0.06);
		flex: 1;
		overflow: hidden;
		min-height: 380px;
	}

	.version-column {
		display: flex;
		flex-direction: column;
		background: #161922;
		overflow: hidden;
	}

	.column-header {
		padding: 0.875rem 1.25rem;
		background: #11141c;
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
	}

	.column-title-box {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		font-weight: 600;
		color: #ffffff;
	}

	.text-success {
		color: #10b981;
	}

	.text-purple {
		color: #c084fc;
	}

	.version-list-scroll {
		flex: 1;
		overflow-y: auto;
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.version-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.group-header {
		font-size: 0.75rem;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: #64748b;
		padding-left: 0.25rem;
	}

	.version-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
		gap: 0.5rem;
	}

	.version-pill {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 0.75rem;
		background: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		color: var(--text-secondary);
		font-size: var(--font-size-xs);
		font-weight: 500;
		cursor: pointer;
		transition: background-color var(--transition-fast),
					border-color var(--transition-fast),
					color var(--transition-fast),
					transform var(--transition-fast);
		text-align: left;
	}

	.version-pill:hover {
		background: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
		color: var(--text-primary);
	}

	.version-pill:active {
		transform: scale(0.96);
	}

	.version-pill.selected {
		background: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
		color: var(--accent-blue-text);
		font-weight: 700;
	}

	.version-pill-snapshot {
		border-style: dashed;
	}

	.version-name {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	:global(.check-icon) {
		color: var(--accent-green-text);
		flex-shrink: 0;
	}

	.badge {
		display: inline-flex;
		align-items: center;
		gap: 0.125rem;
		padding: 0.125rem 0.375rem;
		font-size: 0.6875rem;
		font-weight: 700;
		border-radius: var(--radius-sm);
		text-transform: uppercase;
		flex-shrink: 0;
	}

	.badge-recommended {
		background: var(--accent-green-bg);
		color: var(--accent-green-text);
		border: 1px solid var(--accent-green-border);
	}

	.badge-latest {
		background: var(--warning-bg);
		color: var(--warning-text);
		border: 1px solid var(--warning-border);
	}

	.version-pill.unsupported {
		opacity: 0.4;
		cursor: not-allowed;
		border-color: var(--danger-border);
		background: var(--danger-bg);
	}

	.version-pill.unsupported:hover {
		background: var(--danger-bg);
		border-color: var(--danger-border);
		color: var(--text-muted);
	}

	.badge-unsupported {
		background: var(--danger-bg);
		color: var(--danger-text);
		border: 1px solid var(--danger-border);
	}

	.badge-snapshot {
		background: var(--accent-purple-bg);
		color: var(--accent-purple-text);
		border: 1px solid var(--accent-purple-border);
	}

	

	/* Footer Bar */
	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem 1.5rem;
		background: var(--bg-base);
		border-top: 1px solid var(--border-subtle);
	}

	.selected-summary {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: var(--font-size-sm);
	}

	.summary-label {
		color: var(--text-secondary);
	}

	.summary-value {
		font-weight: 700;
		color: var(--accent-blue-text);
		background: var(--accent-blue-bg);
		padding: 0.25rem 0.625rem;
		border-radius: var(--radius-sm);
		border: 1px solid var(--accent-blue-border);
	}

	.footer-actions {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}

	.btn {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.625rem 1.25rem;
		font-size: var(--font-size-sm);
		font-weight: 600;
		border-radius: var(--radius-btn);
		cursor: pointer;
		transition: background-color var(--transition-fast),
					border-color var(--transition-fast),
					color var(--transition-fast),
					transform var(--transition-fast);
	}

	.btn:active:not(:disabled) {
		transform: scale(0.97);
	}

	.btn-secondary {
		background: transparent;
		border: 1px solid var(--border);
		color: var(--text-secondary);
	}

	.btn-secondary:hover {
		background: rgba(255, 255, 255, 0.08);
		color: var(--text-primary);
	}

	.btn-primary {
		background: var(--accent-blue-solid);
		border: 1px solid transparent;
		color: #ffffff;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15);
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--accent-blue-solid-hover);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	@media (max-width: 640px) {
		.modal-body-columns {
			grid-template-columns: 1fr;
		}
	}
</style>
