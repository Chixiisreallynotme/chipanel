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
	} from 'lucide-svelte';

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

	function handleKeyDown(e) {
		if (e.key === 'Escape' && open) {
			onClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if open}
	<div
		class="modal-backdrop"
		role="presentation"
		onclick={onClose}
	>
		<div
			class="modal-card version-picker-modal"
			role="dialog"
			aria-modal="true"
			aria-labelledby="modal-title"
			tabindex="-1"
			onclick={(e) => e.stopPropagation()}
		>
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
									<span class="version-name">✨ Toutes les versions (Sans filtre)</span>
									{#if draftVersion === 'all'}
										<Check size={14} class="check-icon" />
									{/if}
								</button>
							</div>
						{/if}

						{#if filteredReleases.length === 0}
							<div class="empty-state">
								<p>Aucune version stable trouvée pour "{searchQuery}".</p>
							</div>
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
							<div class="empty-state">
								<p>
									{safeSnapshots.length === 0
										? 'Aucune snapshot disponible pour ce moteur.'
										: `Aucune snapshot trouvée pour "${searchQuery}".`}
								</p>
							</div>
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
		</div>
	</div>
{/if}

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		z-index: 9999;
		background: rgba(8, 10, 15, 0.75);
		backdrop-filter: blur(6px);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 1.5rem;
		animation: fadeIn 0.2s ease-out;
	}

	.version-picker-modal {
		display: flex;
		flex-direction: column;
		width: 100%;
		max-width: 900px;
		max-height: 85vh;
		background: #161922;
		border: 1px solid rgba(255, 255, 255, 0.12);
		border-radius: 16px;
		box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.7);
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1.25rem 1.5rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.08);
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
		border-radius: 10px;
		background: linear-gradient(135deg, rgba(99, 102, 241, 0.2), rgba(168, 85, 247, 0.2));
		border: 1px solid rgba(99, 102, 241, 0.3);
		color: #a855f7;
	}

	.modal-title {
		font-size: 1.125rem;
		font-weight: 700;
		color: #ffffff;
		margin: 0;
	}

	.modal-subtitle {
		font-size: 0.8125rem;
		color: #94a3b8;
		margin-top: 0.125rem;
	}

	.close-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 8px;
		background: transparent;
		border: none;
		color: #94a3b8;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.close-btn:hover {
		background: rgba(255, 255, 255, 0.1);
		color: #ffffff;
	}

	/* Search Bar */
	.modal-search-bar {
		padding: 1rem 1.5rem;
		border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		background: #11141c;
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
		color: #64748b;
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: 0.625rem 2.5rem 0.625rem 2.5rem;
		background: #1a1e2b;
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 8px;
		color: #ffffff;
		font-size: 0.875rem;
		outline: none;
		transition: all 0.2s ease;
	}

	.search-input:focus {
		border-color: #6366f1;
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.2);
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
		background: #1e2330;
		border: 1px solid rgba(255, 255, 255, 0.08);
		border-radius: 8px;
		color: #cbd5e1;
		font-size: 0.8125rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s ease;
		text-align: left;
	}

	.version-pill:hover {
		background: rgba(99, 102, 241, 0.15);
		border-color: rgba(99, 102, 241, 0.3);
		color: #ffffff;
	}

	.version-pill.selected {
		background: linear-gradient(135deg, rgba(99, 102, 241, 0.3), rgba(168, 85, 247, 0.3));
		border-color: #818cf8;
		color: #ffffff;
		font-weight: 700;
		box-shadow: 0 0 12px rgba(99, 102, 241, 0.25);
	}

	.version-pill-snapshot {
		border-style: dashed;
	}

	.version-name {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	:global(.check-icon) {
		color: #34d399;
		flex-shrink: 0;
	}

	.badge {
		display: inline-flex;
		align-items: center;
		gap: 0.125rem;
		padding: 0.125rem 0.375rem;
		font-size: 0.6875rem;
		font-weight: 700;
		border-radius: 4px;
		text-transform: uppercase;
		flex-shrink: 0;
	}

	.badge-recommended {
		background: rgba(16, 185, 129, 0.2);
		color: #34d399;
		border: 1px solid rgba(16, 185, 129, 0.3);
	}

	.badge-latest {
		background: rgba(245, 158, 11, 0.2);
		color: #fbbf24;
		border: 1px solid rgba(245, 158, 11, 0.3);
	}

	.version-pill.unsupported {
		opacity: 0.4;
		cursor: not-allowed;
		border-color: rgba(239, 68, 68, 0.2);
		background: rgba(239, 68, 68, 0.04);
	}

	.version-pill.unsupported:hover {
		background: rgba(239, 68, 68, 0.06);
		border-color: rgba(239, 68, 68, 0.25);
		color: #94a3b8;
	}

	.badge-unsupported {
		background: rgba(239, 68, 68, 0.15);
		color: #fca5a5;
		border: 1px solid rgba(239, 68, 68, 0.3);
	}

	.badge-snapshot {
		background: rgba(168, 85, 247, 0.15);
		color: #c084fc;
		border: 1px solid rgba(168, 85, 247, 0.3);
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 3rem 1rem;
		color: #64748b;
		font-size: 0.8125rem;
		text-align: center;
	}

	/* Footer Bar */
	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 1rem 1.5rem;
		background: #11141c;
		border-top: 1px solid rgba(255, 255, 255, 0.08);
	}

	.selected-summary {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
	}

	.summary-label {
		color: #94a3b8;
	}

	.summary-value {
		font-weight: 700;
		color: #a855f7;
		background: rgba(168, 85, 247, 0.15);
		padding: 0.25rem 0.625rem;
		border-radius: 6px;
		border: 1px solid rgba(168, 85, 247, 0.3);
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
		font-size: 0.875rem;
		font-weight: 600;
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.btn-secondary {
		background: transparent;
		border: 1px solid rgba(255, 255, 255, 0.15);
		color: #cbd5e1;
	}

	.btn-secondary:hover {
		background: rgba(255, 255, 255, 0.08);
		color: #ffffff;
	}

	.btn-primary {
		background: linear-gradient(135deg, #6366f1, #a855f7);
		border: none;
		color: #ffffff;
		box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
	}

	.btn-primary:hover:not(:disabled) {
		opacity: 0.9;
		transform: translateY(-1px);
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
