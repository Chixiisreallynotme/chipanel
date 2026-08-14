<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
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
	} from 'lucide-svelte';

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

	// Fallback sample profiles when API is initializing
	const fallbackProfiles = [
		{
			id: 'prof-1',
			name: 'Fabulously Optimized 5.8.0',
			modpack_title: 'Fabulously Optimized',
			created_at: '2026-08-01T14:32:00Z',
			is_active: true,
			loader: 'Fabric',
			mod_count: 34,
			game_version: '1.20.4'
		},
		{
			id: 'prof-2',
			name: 'All The Mods 9 (Endgame Prep)',
			modpack_title: 'All the Mods 9 - ATM9',
			created_at: '2026-07-25T09:15:00Z',
			is_active: false,
			loader: 'Forge',
			mod_count: 248,
			game_version: '1.20.1'
		},
		{
			id: 'prof-3',
			name: 'Purpur Survival Base Engine',
			modpack_title: 'Purpur Essentials Suite',
			created_at: '2026-07-10T18:45:00Z',
			is_active: false,
			loader: 'Purpur',
			mod_count: 18,
			game_version: '1.20.4'
		}
	];

	onMount(() => {
		loadProfiles();
	});

	export async function loadProfiles() {
		isLoading = true;
		loadError = null;

		try {
			const res = await apiGet('/api/modpacks/profiles');
			if (Array.isArray(res) && res.length > 0) {
				profiles = res;
			} else {
				profiles = fallbackProfiles;
			}
		} catch (err) {
			console.log('Using default profiles state:', err.message);
			profiles = fallbackProfiles;
		} finally {
			isLoading = false;
		}
	}

	function formatDate(dateStr) {
		if (!dateStr) return 'N/A';
		try {
			return new Date(dateStr).toLocaleDateString('en-US', {
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
			await apiPost('/api/modpacks/profiles/switch', { profile_id: targetSwitchProfile.id }).catch(() => {});
			
			// Update local active state
			profiles = profiles.map((p) => ({
				...p,
				is_active: p.id === targetSwitchProfile.id
			}));

			await onSwitchProfile(targetSwitchProfile);
			closeSwitchModal();
		} catch (err) {
			console.error('Failed to switch profile:', err);
			switchError = err.message || 'Failed to switch server profile.';
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
			await apiFetch('/api/modpacks/profiles', {
				method: 'DELETE',
				body: { profile_id: targetDeleteProfile.id }
			}).catch(() => {});

			const deletedProf = targetDeleteProfile;
			profiles = profiles.filter((p) => p.id !== deletedProf.id);

			await onDeleteProfile(deletedProf);
			closeDeleteModal();
		} catch (err) {
			console.error('Failed to delete profile:', err);
			deleteError = err.message || 'Failed to delete server profile.';
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
	<!-- Actions Bar -->
	<div class="top-bar card">
		<div class="top-bar-body">
			<div class="top-info">
				<HardDrive size={20} class="text-accent" />
				<div>
					<h3 class="top-title">Saved Server Profiles</h3>
					<p class="top-sub">
						Each profile stores server configuration, mod loader dependencies, and files. Switch active runtime instantly.
					</p>
				</div>
			</div>

			<button
				class="btn btn-secondary {isLoading ? 'btn-loading' : ''}"
				onclick={loadProfiles}
				disabled={isLoading}
			>
				{#if !isLoading}
					<RefreshCw size={16} />
				{/if}
				<span>Refresh Profiles</span>
			</button>
		</div>
	</div>

	<!-- Profiles Card Grid -->
	{#if isLoading}
		<div class="loading-state">
			<Loader2 size={36} class="spinner" />
			<p>Loading saved profiles...</p>
		</div>
	{:else if profiles.length === 0}
		<div class="empty-state card">
			<div class="card-body empty-body">
				<HardDrive size={48} class="empty-icon" />
				<h3>No Saved Server Profiles</h3>
				<p>Deploy a modpack from the Modpack Catalog tab to create your first server profile.</p>
			</div>
		</div>
	{:else}
		<div class="profiles-grid">
			{#each profiles as prof (prof.id)}
				<div class="card profile-card {prof.is_active ? 'profile-active' : ''}">
					<!-- Card Header -->
					<div class="card-header profile-header">
						<div class="header-left-col">
							<span class="badge {prof.is_active ? 'badge-success' : 'badge-secondary'} active-badge">
								{#if prof.is_active}
									<span class="status-dot status-dot-success status-dot-pulse"></span>
									ACTIVE
								{:else}
									INACTIVE
								{/if}
							</span>
							<span class="badge badge-blue loader-badge">{prof.loader}</span>
						</div>

						<span class="mc-ver-tag">MC {prof.game_version || '1.20.4'}</span>
					</div>

					<!-- Card Body -->
					<div class="card-body profile-body">
						<h3 class="profile-name" title={prof.name}>{prof.name}</h3>
						
						{#if prof.modpack_title}
							<div class="modpack-source">
								<Boxes size={13} />
								<span>Source: {prof.modpack_title}</span>
							</div>
						{/if}

						<div class="meta-list">
							<div class="meta-item">
								<Calendar size={14} class="meta-icon" />
								<span>Created: {formatDate(prof.created_at)}</span>
							</div>
							<div class="meta-item">
								<Layers size={14} class="meta-icon" />
								<span>{prof.mod_count} Included Mods / Plugins</span>
							</div>
						</div>
					</div>

					<!-- Card Footer Actions -->
					<div class="card-footer profile-footer">
						{#if prof.is_active}
							<div class="current-active-indicator">
								<CheckCircle2 size={16} />
								<span>Currently Running</span>
							</div>
						{:else}
							<button
								class="btn btn-secondary btn-sm switch-btn"
								onclick={() => openSwitchModal(prof)}
							>
								<ArrowRightLeft size={14} />
								<span>Switch to Profile</span>
							</button>

							<button
								class="btn btn-ghost btn-sm btn-icon danger-btn"
								onclick={() => openDeleteModal(prof)}
								title="Delete Profile"
								aria-label="Delete Profile"
							>
								<Trash2 size={14} />
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
	<div
		class="modal-backdrop"
		onclick={closeSwitchModal}
		aria-hidden="true"
	>
		<div
			class="modal prompt-modal"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="switch-modal-title"
		>
			<div class="modal-header">
				<div class="modal-title-row">
					<ArrowRightLeft size={20} class="text-accent" />
					<h3 id="switch-modal-title" class="modal-title">Switch Active Profile</h3>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeSwitchModal} disabled={isSwitching} aria-label="Close modal">
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				{#if switchError}
					<div class="alert-banner alert-danger mb-4">
						<AlertCircle size={18} />
						<span>{switchError}</span>
					</div>
				{/if}

				<p class="modal-prompt-text">
					Are you sure you want to switch the active server profile to <strong>"{targetSwitchProfile.name}"</strong>?
				</p>

				<div class="profile-preview-box">
					<div class="prev-row">
						<span class="prev-label">Loader:</span>
						<span class="prev-val">{targetSwitchProfile.loader}</span>
					</div>
					<div class="prev-row">
						<span class="prev-label">Minecraft Version:</span>
						<span class="prev-val">MC {targetSwitchProfile.game_version || '1.20.4'}</span>
					</div>
					<div class="prev-row">
						<span class="prev-label">Mod Count:</span>
						<span class="prev-val">{targetSwitchProfile.mod_count} mods</span>
					</div>
				</div>

				<div class="info-note">
					<CheckCircle2 size={16} />
					<span>Switching profiles updates active server environment configuration and files.</span>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeSwitchModal} disabled={isSwitching}>
					Cancel
				</button>
				<button
					class="btn btn-primary {isSwitching ? 'btn-loading' : ''}"
					onclick={confirmSwitch}
					disabled={isSwitching}
				>
					{#if !isSwitching}
						<ArrowRightLeft size={16} />
					{/if}
					<span>Confirm Switch</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<!-- Delete Profile Confirmation Modal -->
{#if deleteModalOpen && targetDeleteProfile}
	<div
		class="modal-backdrop"
		onclick={closeDeleteModal}
		aria-hidden="true"
	>
		<div
			class="modal prompt-modal"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="delete-modal-title"
		>
			<div class="modal-header">
				<div class="modal-title-row">
					<ShieldAlert size={20} class="text-danger" />
					<h3 id="delete-modal-title" class="modal-title">Delete Profile</h3>
				</div>
				<button class="btn btn-ghost btn-icon btn-sm" onclick={closeDeleteModal} disabled={isDeleting} aria-label="Close modal">
					<X size={18} />
				</button>
			</div>

			<div class="modal-body">
				{#if deleteError}
					<div class="alert-banner alert-danger mb-4">
						<AlertCircle size={18} />
						<span>{deleteError}</span>
					</div>
				{/if}

				<p class="modal-prompt-text">
					Are you sure you want to permanently delete profile <strong>"{targetDeleteProfile.name}"</strong>?
				</p>

				<div class="danger-warning-box">
					<ShieldAlert size={16} />
					<span>This will remove the profile record and its stored mod configuration from server storage. This action cannot be undone.</span>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={closeDeleteModal} disabled={isDeleting}>
					Cancel
				</button>
				<button
					class="btn btn-danger {isDeleting ? 'btn-loading' : ''}"
					onclick={confirmDelete}
					disabled={isDeleting}
				>
					{#if !isDeleting}
						<Trash2 size={16} />
					{/if}
					<span>Delete Profile</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.profile-manager-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.top-bar-body {
		padding: var(--space-5);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.top-info {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.top-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.top-sub {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.text-accent {
		color: var(--accent-blue-text);
	}

	.text-danger {
		color: var(--danger-text);
	}

	.loading-state {
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

	.empty-state {
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
		transition: border-color var(--transition-fast), transform var(--transition-fast);
	}

	.profile-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px -4px rgba(0, 0, 0, 0.4);
	}

	.profile-card.profile-active {
		border-color: var(--accent-green-border);
		background: linear-gradient(180deg, var(--bg-surface) 0%, rgba(74, 222, 128, 0.03) 100%);
	}

	.profile-header {
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.header-left-col {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.active-badge {
		gap: 6px;
	}

	.mc-ver-tag {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--text-muted);
	}

	.profile-body {
		padding: var(--space-4);
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.profile-name {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		line-height: 1.3;
	}

	.modpack-source {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--font-size-xs);
		color: var(--accent-blue-text);
		background-color: var(--accent-blue-bg);
		padding: 3px 8px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--accent-blue-border);
		width: fit-content;
	}

	.meta-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-top: 4px;
	}

	.meta-item {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.meta-icon {
		color: var(--text-muted);
	}

	.profile-footer {
		padding: var(--space-3) var(--space-4);
		background-color: rgba(0, 0, 0, 0.15);
		border-top: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.current-active-indicator {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--accent-green);
		padding: 4px 8px;
	}

	.switch-btn {
		gap: 6px;
	}

	.danger-btn:hover {
		color: var(--danger-text);
		background-color: var(--danger-bg);
	}

	/* Modals */
	.prompt-modal {
		max-width: 480px;
	}

	.modal-title-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.modal-prompt-text {
		font-size: var(--font-size-sm);
		color: var(--text-primary);
		line-height: 1.5;
		margin-bottom: var(--space-4);
	}

	.profile-preview-box {
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-input);
		padding: var(--space-3) var(--space-4);
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: var(--space-4);
	}

	.prev-row {
		display: flex;
		justify-content: space-between;
		font-size: var(--font-size-xs);
	}

	.prev-label {
		color: var(--text-muted);
	}

	.prev-val {
		color: var(--text-primary);
		font-weight: var(--font-weight-medium);
	}

	.info-note {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		padding: var(--space-3);
		border-radius: var(--radius-input);
	}

	.danger-warning-box {
		display: flex;
		align-items: flex-start;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
		color: var(--danger-text);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		padding: var(--space-3);
		border-radius: var(--radius-input);
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
</style>
