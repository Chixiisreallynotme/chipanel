<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import {
		HardDrive,
		RefreshCw,
		CheckCircle2,
		AlertCircle,
		Trash2,
		Play,
		ArrowRightLeft,
		Calendar,
		Boxes,
		Layers,
		Loader2,
		X,
		ShieldAlert,
		Check
	} from '$lib/icons.js';

	/**
	 * @typedef {Object} ServerProfile
	 * @property {string} id
	 * @property {string} name
	 * @property {string} [modpack_title]
	 * @property {string} created_at
	 * @property {boolean} is_active
	 * @property {string} loader
	 * @property {number} mod_count
	 * @property {string} [game_version]
	 */

	let {
		onSwitchProfile = async () => {},
		onDeleteProfile = async () => {}
	} = $props();

	// Component State
	let profiles = $state([]);
	let isLoading = $state(false);
	let loadError = $state(null);

	// Switch Modal State
	let switchModalOpen = $state(false);
	let targetSwitchProfile = $state(null);
	let isSwitching = $state(false);
	let switchError = $state(null);

	// Delete Modal State
	let deleteModalOpen = $state(false);
	let targetDeleteProfile = $state(null);
	let isDeleting = $state(false);
	let deleteError = $state(null);

	onMount(() => {
		loadProfiles();
	});

	export async function loadProfiles() {
		isLoading = true;
		loadError = null;

		try {
			const res = await apiGet('/api/modpacks/profiles');
			if (Array.isArray(res)) {
				profiles = res;
			} else {
				profiles = [];
			}
		} catch (err) {
			console.log('Erreur chargement profils:', err.message);
			profiles = [];
		} finally {
			isLoading = false;
		}
	}

	function formatDate(dateStr) {
		if (!dateStr) return 'N/A';
		try {
			return new Date(dateStr).toLocaleDateString('fr-FR', {
				year: 'numeric',
				month: 'short',
				day: 'numeric',
				hour: '2-digit',
				minute: '2-digit'
			});
		} catch {
			return dateStr;
		}
	}

	// Switch Modal Actions
	function openSwitchModal(profile) {
		if (profile.is_active) return;
		targetSwitchProfile = profile;
		switchModalOpen = true;
		switchError = null;
	}

	function closeSwitchModal() {
		if (isSwitching) return;
		switchModalOpen = false;
		targetSwitchProfile = null;
		switchError = null;
	}

	async function confirmSwitch() {
		if (!targetSwitchProfile || isSwitching) return;
		isSwitching = true;
		switchError = null;

		try {
			await apiPost('/api/modpacks/profiles/switch', { profile_name: targetSwitchProfile.name });
			
			profiles = profiles.map((p) => ({
				...p,
				is_active: p.name === targetSwitchProfile.name
			}));

			await onSwitchProfile(targetSwitchProfile);
			closeSwitchModal();
		} catch (err) {
			console.error('Failed to switch profile:', err);
			switchError = err.message || 'Impossible de basculer vers ce profil.';
		} finally {
			isSwitching = false;
		}
	}

	// Delete Modal Actions
	function openDeleteModal(profile) {
		if (profile.is_active) return;
		targetDeleteProfile = profile;
		deleteModalOpen = true;
		deleteError = null;
	}

	function closeDeleteModal() {
		if (isDeleting) return;
		deleteModalOpen = false;
		targetDeleteProfile = null;
		deleteError = null;
	}

	async function confirmDelete() {
		if (!targetDeleteProfile || isDeleting) return;
		isDeleting = true;
		deleteError = null;

		try {
			await apiPost('/api/modpacks/profiles/delete', { profile_name: targetDeleteProfile.name });

			profiles = profiles.filter((p) => p.name !== targetDeleteProfile.name);
			await onDeleteProfile(targetDeleteProfile);
			closeDeleteModal();
		} catch (err) {
			console.error('Failed to delete profile:', err);
			deleteError = err.message || 'Échec de la suppression du profil.';
		} finally {
			isDeleting = false;
		}
	}

	function handleWindowKeydown(e) {
		if (e.key === 'Escape') {
			if (switchModalOpen) closeSwitchModal();
			if (deleteModalOpen) closeDeleteModal();
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="profile-manager-container">
	<!-- Top Overview Card -->
	<div class="profiles-header-card card">
		<div class="header-body">
			<div class="header-text">
				<div class="header-icon-box">
					<HardDrive size={22} />
				</div>
				<div>
					<h3 class="header-title">Profils & Instantanés Serveur</h3>
					<p class="header-desc">
						Basculez instantanément entre différentes configurations de mods, plugins et mondes sans perdre vos fichiers.
					</p>
				</div>
			</div>

			<button
				type="button"
				class="btn btn-secondary btn-sm refresh-btn {isLoading ? 'btn-loading' : ''}"
				disabled={isLoading}
				onclick={loadProfiles}
				title="Actualiser la liste des profils"
			>
				{#if !isLoading}
					<RefreshCw size={14} />
				{/if}
				<span>Actualiser</span>
			</button>
		</div>
	</div>

	<!-- Profiles Grid -->
	{#if isLoading && profiles.length === 0}
		<div class="loading-state card">
			<Loader2 size={36} class="spinner" />
			<p>Chargement des profils du serveur…</p>
		</div>
	{:else if profiles.length === 0}
		<EmptyState card title="Aucun profil sauvegardé" description="Installez un modpack depuis l'onglet Modpacks pour créer automatiquement votre premier profil isolé.">
			{#snippet icon()}
				<HardDrive size={44} class="empty-icon" />
			{/snippet}
		</EmptyState>
	{:else}
		<div class="profiles-grid">
			{#each profiles as profile (profile.name)}
				<div class="card profile-card {profile.is_active ? 'card-active' : ''}">
					<!-- Card Header with status -->
					<div class="profile-card-header">
						<div class="profile-title-box">
							<h3 class="profile-name" title={profile.name}>{profile.name}</h3>
							{#if profile.modpack_title}
								<span class="modpack-parent-badge">
									<Boxes size={12} />
									{profile.modpack_title}
								</span>
							{/if}
						</div>

						{#if profile.is_active}
							<span class="badge badge-success active-badge">
								<span class="status-dot status-dot-success status-dot-pulse"></span>
								Profil Actif
							</span>
						{:else}
							<span class="badge badge-secondary inactive-badge">
								Inactif
							</span>
						{/if}
					</div>

					<!-- Card Body Details -->
					<div class="profile-card-body">
						<div class="profile-details-grid">
							<div class="detail-item">
								<span class="detail-label">Chargeur :</span>
								<span class="detail-value font-semibold text-primary">{profile.loader || 'Fabric'}</span>
							</div>

							<div class="detail-item">
								<span class="detail-label">Version MC :</span>
								<span class="detail-value font-mono">{profile.game_version || '1.20.4'}</span>
							</div>

							<div class="detail-item">
								<span class="detail-label">Mods installés :</span>
								<span class="detail-value">{profile.mod_count || 0} mods</span>
							</div>

							<div class="detail-item">
								<span class="detail-label">Créé le :</span>
								<span class="detail-value">{formatDate(profile.created_at)}</span>
							</div>
						</div>
					</div>

					<!-- Card Footer Actions -->
					<div class="profile-card-footer">
						{#if profile.is_active}
							<div class="active-indicator">
								<Check size={16} class="text-green" />
								<span>Actuellement utilisé</span>
							</div>
						{:else}
							<button
								type="button"
								class="btn btn-primary btn-sm switch-btn"
								onclick={() => openSwitchModal(profile)}
							>
								<ArrowRightLeft size={14} />
								<span>Activer ce profil</span>
							</button>

							<button
								type="button"
								class="btn btn-ghost btn-icon btn-sm delete-btn"
								onclick={() => openDeleteModal(profile)}
								title="Supprimer ce profil"
							>
								<Trash2 size={16} />
							</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Switch Profile Confirmation Modal -->
{#if switchModalOpen && targetSwitchProfile}
	<div class="modal-backdrop" onclick={closeSwitchModal} role="presentation">
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="modal-card card shadow-xl" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" aria-modal="true">
			<div class="modal-header">
				<div class="modal-title-with-icon">
					<div class="modal-icon-box icon-blue">
						<ArrowRightLeft size={20} />
					</div>
					<h3>Activer le profil</h3>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeSwitchModal} disabled={isSwitching}>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				<p>
					Voulez-vous basculer le serveur vers le profil <strong>{targetSwitchProfile.name}</strong> ?
				</p>
				<p class="modal-warning">
					Les dossiers <code>/mods</code> et <code>/config</code> actuels seront archivés dans votre profil actuel avant d'appliquer ce profil.
				</p>

				{#if switchError}
					<div class="alert-banner alert-danger">
						<AlertCircle size={16} />
						<span>{switchError}</span>
					</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeSwitchModal} disabled={isSwitching}>
					Annuler
				</button>
				<button
					type="button"
					class="btn btn-primary {isSwitching ? 'btn-loading' : ''}"
					onclick={confirmSwitch}
					disabled={isSwitching}
				>
					{#if !isSwitching}
						<ArrowRightLeft size={16} />
					{/if}
					<span>Confirmer le basculement</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Profile Confirmation Modal -->
{#if deleteModalOpen && targetDeleteProfile}
	<div class="modal-backdrop" onclick={closeDeleteModal} role="presentation">
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="modal-card card shadow-xl" onclick={(e) => e.stopPropagation()} role="dialog" tabindex="-1" aria-modal="true">
			<div class="modal-header">
				<div class="modal-title-with-icon">
					<div class="modal-icon-box icon-danger">
						<Trash2 size={20} />
					</div>
					<h3>Supprimer le profil</h3>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeDeleteModal} disabled={isDeleting}>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				<p>
					Êtes-vous sûr de vouloir supprimer définitivement le profil <strong>{targetDeleteProfile.name}</strong> ?
				</p>
				<p class="modal-warning">
					Tous les mods et fichiers de configuration sauvegardés dans ce profil seront irréversiblement effacés.
				</p>

				{#if deleteError}
					<div class="alert-banner alert-danger">
						<AlertCircle size={16} />
						<span>{deleteError}</span>
					</div>
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeDeleteModal} disabled={isDeleting}>
					Annuler
				</button>
				<button
					type="button"
					class="btn btn-danger {isDeleting ? 'btn-loading' : ''}"
					onclick={confirmDelete}
					disabled={isDeleting}
				>
					{#if !isDeleting}
						<Trash2 size={16} />
					{/if}
					<span>Supprimer</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.profile-manager-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.profiles-header-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
	}
	.header-body {
		padding: var(--space-4);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
	}
	.header-text {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}
	.header-icon-box {
		width: 40px;
		height: 40px;
		border-radius: var(--radius-input);
		background-color: var(--accent-blue-bg, rgba(59, 130, 246, 0.12));
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text, #60a5fa);
		flex-shrink: 0;
	}
	.header-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		margin: 0;
	}
	.header-desc {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin: 0;
	}

	.profiles-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: var(--space-4);
	}
	.profile-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		transition: border-color var(--transition-fast), box-shadow var(--transition-fast), background-color var(--transition-fast);
	}
	.card-active {
		border-color: var(--accent-blue);
		box-shadow: 0 0 12px rgba(59, 130, 246, 0.15);
	}
	.profile-card-header {
		padding: var(--space-4);
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-3);
		border-bottom: 1px solid var(--border);
	}
	.profile-title-box {
		flex: 1;
		min-width: 0;
	}
	.profile-name {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		margin: 0 0 4px 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.modpack-parent-badge {
		font-size: 11px;
		color: var(--text-muted);
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}
	.active-badge {
		font-size: 11px;
	}

	.profile-card-body {
		padding: var(--space-4);
		flex: 1;
	}
	.profile-details-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-3);
	}
	.detail-item {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.detail-label {
		font-size: 11px;
		color: var(--text-secondary);
	}
	.detail-value {
		font-size: var(--font-size-xs);
		color: var(--text-primary);
	}

	.profile-card-footer {
		padding: var(--space-3) var(--space-4);
		border-top: 1px solid var(--border);
		background-color: var(--bg-subtle, rgba(255, 255, 255, 0.02));
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}
	.active-indicator {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		font-weight: var(--font-weight-medium);
	}
	.switch-btn {
		flex: 1;
	}

	.loading-state {
		padding: var(--space-10);
		text-align: center;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: var(--text-secondary);
	}

	/* Modal Backdrop */
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
	.modal-warning {
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
</style>
