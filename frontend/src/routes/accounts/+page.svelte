<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost, apiFetch } from '$lib/api/client.js';
	import { auth } from '$lib/stores/auth.svelte.js';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import {
		Users,
		UserPlus,
		ShieldCheck,
		Trash2,
		Loader2,
		CheckCircle2,
		AlertCircle,
		Crown,
		UserCheck,
		Eye,
		Lock,
		Sliders,
		Check,
		Shield,
		Terminal,
		HardDrive,
		Package,
		Boxes,
		Globe,
		Activity,
		KeyRound
	} from '$lib/icons.js';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import FormModal from '$lib/components/ui/FormModal.svelte';
	import { toast } from '$lib/stores/toast.svelte.js';

	/**
	 * `apiGet` parses the body as JSON and rethrows whatever `res.json()` threw. When
	 * `/api/auth/users` was unmounted the SPA fallback answered with `index.html`, so the
	 * page printed `Unexpected token '<', "<!doctype "... is not valid JSON` straight at the
	 * operator (B3). The backend now returns JSON for both success and 404, but a parser
	 * exception is never a user-facing string — anything that still smells like one is
	 * replaced here, and 403 gets its own wording because it is an authorisation outcome,
	 * not a failure.
	 */
	const PARSER_NOISE = /unexpected token|not valid json|json\.parse|<!doctype|<html/i;

	/**
	 * @param {unknown} err
	 * @returns {{ kind: 'forbidden' | 'error', message: string }}
	 */
	function toDisplayError(err) {
		const raw = err instanceof Error ? err.message : String(err ?? '');
		if (/access denied|accès refusé/i.test(raw)) {
			return {
				kind: 'forbidden',
				message:
					"Accès refusé : la gestion des comptes est réservée à l'administrateur. Votre session n'a pas ce droit."
			};
		}
		if (!raw.trim() || PARSER_NOISE.test(raw)) {
			return {
				kind: 'error',
				message:
					"Réponse inattendue du serveur : la liste des comptes n'a pas pu être lue. Vérifiez que le backend ChiPanel répond sur /api/auth/users."
			};
		}
		return { kind: 'error', message: raw };
	}

	// Available Permission Definitions grouped by Category
	const PERMISSION_CATEGORIES = [
		{
			id: 'server',
			title: 'Contrôle serveur',
			icon: Terminal,
			permissions: [
				{ id: 'server.start', label: 'Démarrer le serveur', desc: 'Autorise l\'allumage de l\'instance' },
				{ id: 'server.stop', label: 'Arrêter / Redémarrer', desc: 'Autorise l\'extinction et les redémarrages' },
				{ id: 'server.console', label: 'Console RCON', desc: 'Accès et exécution des commandes en console' }
			]
		},
		{
			id: 'files',
			title: 'Fichiers & extension',
			icon: HardDrive,
			permissions: [
				{ id: 'files.read', label: 'Lecture fichiers', desc: 'Consulter l\'arborescence et le contenu' },
				{ id: 'files.write', label: 'Édition fichiers', desc: 'Modifier et créer des fichiers de config' },
				{ id: 'files.delete', label: 'Suppression fichiers', desc: 'Supprimer des fichiers et dossiers' },
				{ id: 'plugins.manage', label: 'Gestion plugins', desc: 'Installer et supprimer des plugins Paper/Spigot' },
				{ id: 'modpacks.manage', label: 'Gestion modpacks', desc: 'Installer et configurer des mods Modrinth/CurseForge' }
			]
		},
		{
			id: 'gameplay',
			title: 'Joueurs & mondes',
			icon: Globe,
			permissions: [
				{ id: 'players.manage', label: 'Gestion joueurs', desc: 'Bannir, kick, modifier les inventaires' },
				{ id: 'worlds.manage', label: 'Gestion mondes', desc: 'Gérer les sauvegardes et mondes du serveur' }
			]
		},
		{
			id: 'admin',
			title: 'Administration & suivi',
			icon: Activity,
			permissions: [
				{ id: 'metrics.view', label: 'Planificateur & métriques', desc: 'Consulter les métriques CPU/RAM et tâches' },
				{ id: 'users.manage', label: 'Gestion des comptes', desc: 'Créer et gérer d\'autres comptes utilisateurs' }
			]
		}
	];

	// Pre-configured Profiles (Profils Types)
	const PRESET_PROFILES = [
		{
			id: 'admin',
			title: 'Administrateur total',
			desc: 'Accès complet et illimité à l\'ensemble du panneau',
			badgeClass: 'badge-danger',
			icon: Crown,
			permissions: [
				'server.start', 'server.stop', 'server.console',
				'files.read', 'files.write', 'files.delete',
				'plugins.manage', 'modpacks.manage',
				'players.manage', 'worlds.manage',
				'metrics.view', 'users.manage'
			]
		},
		{
			id: 'operator',
			title: 'Opérateur serveur',
			desc: 'Gestion quotidienne du serveur, console, fichiers & mods',
			badgeClass: 'badge-blue',
			icon: UserCheck,
			permissions: [
				'server.start', 'server.stop', 'server.console',
				'files.read', 'files.write',
				'plugins.manage', 'modpacks.manage',
				'players.manage', 'worlds.manage',
				'metrics.view'
			]
		},
		{
			id: 'moderator',
			title: 'Modérateur joueurs',
			desc: 'Modération du serveur, commandes console & suivi des joueurs',
			badgeClass: 'badge-warning',
			icon: ShieldCheck,
			permissions: [
				'server.console',
				'players.manage',
				'metrics.view'
			]
		},
		{
			id: 'viewer',
			title: 'Observateur audit',
			desc: 'Consultation en lecture seule des fichiers et métriques',
			badgeClass: 'badge-blue',
			icon: Eye,
			permissions: [
				'files.read',
				'metrics.view'
			]
		},
		{
			id: 'custom',
			title: 'Profil personnalisé',
			desc: 'Sélectionnez manuellement chaque autorisation individuelle',
			badgeClass: '',
			icon: Sliders,
			permissions: []
		}
	];

	// Users List State
	let users = $state([]);
	let isLoading = $state(false);
	let error = $state(/** @type {string | null} */ (null));

	// Create User Modal State
	let createModalOpen = $state(false);
	let newUsername = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let selectedPreset = $state('operator'); // 'admin', 'operator', 'moderator', 'viewer', 'custom'
	let selectedPermissions = $state([
		'server.start', 'server.stop', 'server.console',
		'files.read', 'files.write',
		'plugins.manage', 'modpacks.manage',
		'players.manage', 'worlds.manage',
		'metrics.view'
	]);
	let isCreating = $state(false);
	let createError = $state(/** @type {string | null} */ (null));
	/** @type {'forbidden' | 'error'} */
	let errorKind = $state('error');

	// Toasts via le store global (ToastHost monte dans le layout).
	async function loadUsers() {
		isLoading = true;
		error = null;
		try {
			const res = await apiGet('/api/auth/users');
			users = Array.isArray(res) ? res : [];
		} catch (err) {
			console.error('Failed to load user accounts:', err);
			const shown = toDisplayError(err);
			errorKind = shown.kind;
			error = shown.message;
		} finally {
			isLoading = false;
		}
	}

	onMount(() => {
		loadUsers();
	});

	function selectPresetProfile(profileId) {
		selectedPreset = profileId;
		const found = PRESET_PROFILES.find((p) => p.id === profileId);
		if (found && profileId !== 'custom') {
			selectedPermissions = [...found.permissions];
		}
	}

	function togglePermission(permId) {
		if (selectedPermissions.includes(permId)) {
			selectedPermissions = selectedPermissions.filter((p) => p !== permId);
		} else {
			selectedPermissions = [...selectedPermissions, permId];
		}
		selectedPreset = 'custom';
	}

	function selectAllPermissions() {
		const allPerms = [];
		PERMISSION_CATEGORIES.forEach((cat) => {
			cat.permissions.forEach((p) => allPerms.push(p.id));
		});
		selectedPermissions = allPerms;
		selectedPreset = 'admin';
	}

	function deselectAllPermissions() {
		selectedPermissions = [];
		selectedPreset = 'custom';
	}

	function openCreateModal() {
		createModalOpen = true;
		newUsername = '';
		newPassword = '';
		confirmPassword = '';
		selectPresetProfile('operator');
		createError = null;
	}

	function closeCreateModal() {
		if (isCreating) return;
		createModalOpen = false;
		createError = null;
	}

	async function confirmCreateUser() {
		if (!newUsername.trim() || isCreating) return;
		
		if (newUsername.trim().length < 3) {
			createError = 'Le nom d\'utilisateur doit contenir au moins 3 caractères.';
			return;
		}

		if (newPassword.length < 4) {
			createError = 'Le mot de passe doit contenir au moins 4 caractères.';
			return;
		}

		if (newPassword !== confirmPassword) {
			createError = 'Les mots de passe ne correspondent pas.';
			return;
		}

		isCreating = true;
		createError = null;

		try {
			const res = await apiPost('/api/auth/users', {
				username: newUsername.trim(),
				password: newPassword,
				role: selectedPreset,
				permissions: selectedPermissions
			});

			closeCreateModal();
			await loadUsers();
			toast.success('Compte créé', `Le compte utilisateur "${res.username}" a été créé avec succès.`);
		} catch (err) {
			console.error('Failed to create user account:', err);
			createError = toDisplayError(err).message;
		} finally {
			isCreating = false;
		}
	}

	/** Account awaiting delete confirmation; null when the dialog is closed. */
	let userPendingDelete = $state(/** @type {any} */ (null));

	/** @param {any} userItem */
	function requestDeleteUser(userItem) {
		if (userItem.username.toLowerCase() === auth.user?.username?.toLowerCase()) {
			toast.error('Action interdite', 'Vous ne pouvez pas supprimer votre propre compte actuellement actif.');
			return;
		}
		userPendingDelete = userItem;
	}

	function confirmDeleteUser() {
		const userItem = userPendingDelete;
		userPendingDelete = null;
		if (userItem) deleteUser(userItem);
	}

	async function deleteUser(userItem) {
		try {
			await apiFetch(`/api/auth/users/${userItem.username}`, {
				method: 'DELETE'
			});

			users = users.filter((u) => u.username !== userItem.username);
			toast.info('Compte supprimé', `Le compte "${userItem.username}" a été révoqué.`);
		} catch (err) {
			console.error('Failed to delete user:', err);
			const shown = toDisplayError(err);
			toast.error(
				shown.kind === 'forbidden' ? 'Accès refusé' : 'Erreur de suppression',
				shown.message
			);
		}
	}

	function getRoleBadge(role) {
		switch (role.toLowerCase()) {
			case 'admin':
				return { label: 'Administrateur', badgeClass: 'badge-danger', icon: Crown };
			case 'operator':
			case 'op':
				return { label: 'Opérateur', badgeClass: 'badge-blue', icon: UserCheck };
			case 'moderator':
				return { label: 'Modérateur', badgeClass: 'badge-warning', icon: ShieldCheck };
			case 'custom':
				return { label: 'Sur mesure', badgeClass: '', icon: Sliders };
			default:
				return { label: 'Lecteur', badgeClass: 'badge-blue', icon: Eye };
		}
	}

	function handleWindowKeydown(e) {
		if (e.key === 'Escape' && createModalOpen) {
			closeCreateModal();
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<svelte:head>
	<title>Comptes & autorisations panneau - ChiServ</title>
</svelte:head>

<ConfirmDialog
	open={userPendingDelete !== null}
	title="Supprimer le compte"
	message={`Voulez-vous vraiment supprimer le compte "${userPendingDelete?.username ?? ''}" ? Il ne pourra plus accéder au panneau ChiPanel.`}
	confirmLabel="Supprimer"
	onconfirm={confirmDeleteUser}
	oncancel={() => (userPendingDelete = null)}
/>

<div class="accounts-page">
	<!-- Hero Header -->
	<PageHeader
		title="Comptes & autorisations sur-mesure"
		subtitle="Gérez les accès au ChiPanel avec des profils préconfigurés ou une personnalisation complète des droits."
	>
		{#snippet icon()}
			<Users size={26} />
		{/snippet}
		<button class="btn btn-primary" onclick={openCreateModal}>
			<UserPlus size={16} />
			<span>Nouveau compte sur-mesure</span>
		</button>
	</PageHeader>

	<!-- Semantic Tokenized Info Alert -->
	<div class="semantic-info-alert" role="status">
		<div class="alert-icon-box">
			<ShieldCheck size={28} />
		</div>
		<div class="alert-content font-ui">
			<h3 class="alert-title">Niveaux d'accès sécurisés & profils types</h3>
			<p class="alert-desc">
				Configurez des rôles précis pour chaque utilisateur (administrateurs, opérateurs, modérateurs ou profils personnalisés). Chaque session bénéficie d'un token valide 24 heures.
			</p>
		</div>
	</div>

	<!-- User Accounts Table with Double-Bezel -->
	<div class="hardware-shell accounts-table-shell">
		<div class="hardware-core accounts-table-card">
			<div class="table-card-header">
				<div class="table-header-title">
					<Users size={18} />
					<h3>Comptes d'accès enregistrés ({users.length})</h3>
				</div>
			</div>

			<div class="table-body">
				{#if isLoading}
					<div class="loading-state">
						<Loader2 size={32} class="spinner" />
						<p>Chargement des comptes utilisateurs...</p>
					</div>
				{:else if error}
					<!-- role="alert" so the failure is announced, and a separate visual treatment for
					     403: "you may not do this" is a different outcome from "this broke". -->
					<div
						class="alert-banner {errorKind === 'forbidden' ? 'alert-forbidden' : 'alert-danger'}"
						role="alert"
					>
						{#if errorKind === 'forbidden'}
							<Lock size={18} />
						{:else}
							<AlertCircle size={18} />
						{/if}
						<span>{error}</span>
					</div>
				{:else if users.length === 0}
					<EmptyState title="Aucun compte secondaire" description="Cliquez sur « Nouveau compte sur-mesure » pour créer un profil utilisateur.">
						{#snippet icon()}
							<Lock size={40} class="empty-icon" />
						{/snippet}
					</EmptyState>
				{:else}
					<div class="table-wrapper">
						<table class="table">
							<thead>
								<tr>
									<th>NOM D'UTILISATEUR</th>
									<th>PROFIL D'ACCÈS</th>
									<th>AUTORISATIONS ACCORDÉES</th>
									<th>DATE DE CRÉATION</th>
									<th class="text-right">ACTIONS</th>
								</tr>
							</thead>
							<tbody>
								{#each users as user (user.id || user.username)}
									{@const roleInfo = getRoleBadge(user.role)}
									{@const RoleIcon = roleInfo.icon}
									{@const permCount = Array.isArray(user.permissions) ? user.permissions.length : 0}
									<tr>
										<td class="font-medium text-primary">
											<div class="user-name-cell">
												<div class="user-avatar-sm">
													{user.username.charAt(0).toUpperCase()}
												</div>
												<span>{user.username}</span>
												{#if user.username.toLowerCase() === auth.user?.username?.toLowerCase()}
													<span class="current-user-tag">(Vous)</span>
												{/if}
											</div>
										</td>
										<td>
											<span class="badge {roleInfo.badgeClass} role-badge">
												<RoleIcon size={13} />
												<span>{roleInfo.label}</span>
											</span>
										</td>
										<td>
											{#if user.role.toLowerCase() === 'admin' || user.permissions?.includes('*')}
												<span class="badge badge-success">Accès Total (Tous droits)</span>
											{:else if permCount > 0}
												<span class="badge badge-blue">{permCount} Autorisations activées</span>
											{:else}
												<span class="badge badge-blue">Lecture seule</span>
											{/if}
										</td>
										<td class="text-muted text-sm tabular-nums">
											{user.created_at || 'Compte système'}
										</td>
										<td class="text-right">
											{#if user.username.toLowerCase() !== auth.user?.username?.toLowerCase()}
												<button
													class="btn btn-ghost btn-sm text-danger"
													onclick={() => requestDeleteUser(user)}
													title="Supprimer ce compte"
												>
													<Trash2 size={14} />
													<span>Supprimer</span>
												</button>
											{:else}
												<span class="text-muted text-xs italic">Compte Actif</span>
											{/if}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<!-- Modal: Create User Account with Custom Permissions -->
	<FormModal
		open={createModalOpen}
		title="Créer un compte & personnaliser les droits"
		onClose={closeCreateModal}
		style="max-width:720px"
	>
						{#if createError}
							<div class="alert-banner alert-danger mb-4">
								<AlertCircle size={18} />
								<span>{createError}</span>
							</div>
						{/if}

						<!-- User Info Fields -->
						<div class="form-grid-2 mb-4">
							<div class="form-group">
								<label for="new-username" class="label label-required">Nom d'Utilisateur</label>
								<input
									id="new-username"
									type="text"
									class="input"
									placeholder="ex. alex, op_server, mod_alex"
									bind:value={newUsername}
									disabled={isCreating}
								/>
							</div>

							<div class="form-group">
								<label for="new-password" class="label label-required">Mot de passe</label>
								<input
									id="new-password"
									type="password"
									class="input"
									placeholder="Définir un mot de passe sécurisé"
									bind:value={newPassword}
									disabled={isCreating}
								/>
							</div>
						</div>

						<div class="form-group mb-4">
							<label for="confirm-password" class="label label-required">Confirmer le mot de passe</label>
							<input
								id="confirm-password"
								type="password"
								class="input"
								placeholder="Confirmer le mot de passe"
								bind:value={confirmPassword}
								disabled={isCreating}
							/>
						</div>

						<!-- Section 1: Pre-configured Profiles (Profils Types) -->
						<div class="section-divider">
							<h4>1. Sélectionner un profil type préconfiguré</h4>
							<p class="section-subtitle">Choisissez un profil modèle pour appliquer automatiquement les autorisations correspondantes.</p>
						</div>

						<div class="preset-profiles-grid mb-6">
							{#each PRESET_PROFILES as profile (profile.id)}
								{@const ProfileIcon = profile.icon}
								{@const isSelected = selectedPreset === profile.id}
								<button
									type="button"
									class="preset-card {isSelected ? 'selected' : ''}"
									onclick={() => selectPresetProfile(profile.id)}
								>
									<div class="preset-header">
										<div class="preset-icon-box">
											<ProfileIcon size={18} />
										</div>
										<span class="badge {profile.badgeClass}">{profile.title}</span>
									</div>
									<p class="preset-desc">{profile.desc}</p>
									<div class="preset-check-mark">
										{#if isSelected}
											<CheckCircle2 size={16} class="text-accent" />
										{/if}
									</div>
								</button>
							{/each}
						</div>

						<!-- Section 2: Fine-Grained Custom Permissions Checklist -->
						<div class="section-divider flex-between">
							<div>
								<h4>2. Personnalisation détaillée des autorisations</h4>
								<p class="section-subtitle">Cochez ou décochez les droits individuels autorisés pour ce compte.</p>
							</div>
							<div class="perm-quick-actions">
								<button type="button" class="btn btn-ghost btn-xs" onclick={selectAllPermissions}>
									<span>Tout Cocher</span>
								</button>
								<button type="button" class="btn btn-ghost btn-xs" onclick={deselectAllPermissions}>
									<span>Tout Décocher</span>
								</button>
							</div>
						</div>

						<div class="permissions-categories-container">
							{#each PERMISSION_CATEGORIES as category (category.id)}
								{@const CatIcon = category.icon}
								<div class="category-block">
									<div class="category-header">
										<CatIcon size={16} class="text-accent" />
										<h5>{category.title}</h5>
									</div>

									<div class="permissions-items-grid">
										{#each category.permissions as perm (perm.id)}
											{@const isChecked = selectedPermissions.includes(perm.id)}
											<label class="permission-item-label {isChecked ? 'checked' : ''}">
												<input
													type="checkbox"
													checked={isChecked}
													onchange={() => togglePermission(perm.id)}
													disabled={isCreating}
												/>
												<div class="perm-info">
													<span class="perm-name">{perm.label}</span>
													<span class="perm-desc">{perm.desc}</span>
												</div>
											</label>
										{/each}
									</div>
								</div>
							{/each}
						</div>
		{#snippet footer()}
			<button class="btn btn-secondary" onclick={closeCreateModal} disabled={isCreating}>
				Annuler
			</button>
			<button
				class="btn btn-primary {isCreating ? 'btn-loading' : ''}"
				onclick={confirmCreateUser}
				disabled={isCreating || !newUsername.trim() || !newPassword}
			>
				{#if !isCreating}
					<UserPlus size={16} />
				{/if}
				<span>Créer le Compte ({selectedPermissions.length} droits)</span>
			</button>
		{/snippet}
	</FormModal>

</div>

<style>
	/* `.alert-banner` was only ever defined inside other components' scoped styles, so the
	   error state on this page rendered unstyled. Defined locally alongside the two
	   variants the page can show. */
	.alert-banner {
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-btn);
		display: flex;
		align-items: center;
		gap: var(--space-3);
		font-size: var(--font-size-sm);
		line-height: var(--line-height-normal);
	}

	.alert-danger {
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
	}

	/* Authorisation outcome, not a fault: amber, padlock, no red alarm. */
	.alert-forbidden {
		background-color: var(--warning-bg);
		border: 1px solid var(--warning-border);
		color: var(--warning);
	}

	.mb-4 {
		margin-bottom: var(--space-4);
	}

	.accounts-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.semantic-info-alert {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		border-radius: var(--radius-card);
		padding: var(--space-4) var(--space-5);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
	}

	.alert-icon-box {
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	.alert-content .alert-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		margin: 0;
	}

	.alert-content .alert-desc {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin: 2px 0 0 0;
	}

	.accounts-table-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
	}

	.table-header-title {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		color: var(--text-primary);
		font-weight: var(--font-weight-bold);
	}

	.table-body {
		padding: 0;
	}

	.table-wrapper {
		width: 100%;
		overflow-x: auto;
	}

	.table {
		width: 100%;
		border-collapse: collapse;
		text-align: left;
	}

	.table th {
		padding: var(--space-3) var(--space-4);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--text-secondary);
		border-bottom: 1px solid var(--border);
		background-color: var(--bg-base);
		letter-spacing: 0.04em;
		text-transform: uppercase;
	}

	.table td {
		padding: var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
		vertical-align: middle;
	}

	.table tr:last-child td {
		border-bottom: none;
	}

	.user-name-cell {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.user-avatar-sm {
		width: 32px;
		height: 32px;
		border-radius: 50%;
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		color: var(--accent-blue-text);
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: var(--font-weight-bold);
		font-size: var(--font-size-sm);
	}

	.current-user-tag {
		font-size: var(--font-size-xs);
		color: var(--accent-purple-text);
		font-style: italic;
	}

	.role-badge {
		display: inline-flex;
		align-items: center;
		gap: 5px;
	}

	.text-right {
		text-align: right;
	}

	.form-grid-2 {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-4);
	}

	.section-divider {
		border-top: 1px solid var(--border);
		padding-top: var(--space-4);
		margin-top: var(--space-4);
	}

	.section-divider h4 {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.section-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.flex-between {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.perm-quick-actions {
		display: flex;
		gap: var(--space-2);
	}

	/* Preset Profiles Cards */
	.preset-profiles-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-3);
		margin-top: var(--space-3);
	}

	.preset-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
		text-align: left;
		cursor: pointer;
		transition: border-color var(--transition-fast), background-color var(--transition-fast), transform var(--transition-fast);
		display: flex;
		flex-direction: column;
		gap: 6px;
		position: relative;
	}

	.preset-card:hover {
		border-color: var(--accent-purple-border);
		background-color: var(--bg-elevated);
	}

	.preset-card.selected {
		border-color: var(--accent-purple-border);
		background-color: var(--accent-purple-bg);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05), 0 2px 8px rgba(0, 0, 0, 0.3);
	}

	.preset-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.preset-icon-box {
		color: var(--text-secondary);
	}

	.preset-desc {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		line-height: var(--line-height-normal);
	}

	/* Custom Permissions Category Blocks */
	.permissions-categories-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		margin-top: var(--space-3);
		max-height: 280px;
		overflow-y: auto;
		padding-right: var(--space-2);
	}

	.category-block {
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
	}

	.category-header {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		margin-bottom: var(--space-3);
	}

	.category-header h5 {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.permissions-items-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-2);
	}

	.permission-item-label {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		background-color: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-sm);
		padding: var(--space-2) var(--space-3);
		cursor: pointer;
		transition: border-color var(--transition-fast), background-color var(--transition-fast);
	}

	.permission-item-label:hover {
		border-color: var(--accent-purple-border);
	}

	.permission-item-label.checked {
		border-color: var(--accent-purple-border);
		background-color: var(--accent-purple-bg);
	}

	.permission-item-label input[type='checkbox'] {
		margin-top: 3px;
		accent-color: var(--accent-purple);
	}

	.perm-info {
		display: flex;
		flex-direction: column;
	}

	.perm-name {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.perm-desc {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 1px;
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-12) var(--space-6);
		gap: var(--space-3);
		color: var(--text-muted);
		text-align: center;
	}

	.empty-icon {
		opacity: 0.5;
	}

	.spinner {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

</style>
