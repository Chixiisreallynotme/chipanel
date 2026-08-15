<script>
	import { onMount, onDestroy } from 'svelte';
	import { apiGet, apiPost, apiFetch, apiDownload } from '$lib/api/client.js';
	import WorldCard from '$lib/components/worlds/WorldCard.svelte';
	import ChunkyPanel from '$lib/components/worlds/ChunkyPanel.svelte';
	import WorldBackupModal from '$lib/components/worlds/WorldBackupModal.svelte';
	import CreateWorldModal from '$lib/components/worlds/CreateWorldModal.svelte';
	import ImportWorldModal from '$lib/components/worlds/ImportWorldModal.svelte';
	import ConfigureWorldModal from '$lib/components/worlds/ConfigureWorldModal.svelte';
	import {
		Globe,
		Compass,
		Archive,
		RefreshCw,
		CheckCircle2,
		AlertCircle,
		Info,
		X,
		Maximize2,
		Shield,
		Sliders,
		Plus,
		Upload,
		Play,
		Trash2
	} from 'lucide-svelte';

	// Main Page Data States
	let worlds = $state([]);
	let pendingWorlds = $state([]);
	let activeWorld = $state(null);
	let serverDataVersion = $state(null);
	let borders = $state({}); // Keyed by world folder_name
	let chunkyStatus = $state({
		is_running: false,
		is_paused: false,
		percent_complete: 0,
		chunks_rendered: 0,
		chunks_total: 0,
		current_cps: 0,
		eta_seconds: 0,
		world: ''
	});
	let backups = $state([]);
	let loading = $state(false);

	// Modals State
	let backupModalOpen = $state(false);
	let pregenTargetWorld = $state('');
	let createModalOpen = $state(false);
	let importModalOpen = $state(false);

	// Configure modal state
	let configureModalOpen = $state(false);
	let configureTarget = $state(null);

	// Delete confirmation state
	let deleteTarget = $state(null);
	let deleteInFlight = $state(false);
	let switchInFlight = $state(false);

	// Worldborder Edit Modal State
	let borderModalOpen = $state(false);
	let targetWorldForBorder = $state(null);
	let borderSize = $state(60000000);
	let borderCenterX = $state(0);
	let borderCenterZ = $state(0);
	let borderDamage = $state(0.2);
	let borderWarning = $state(5);
	let isSavingBorder = $state(false);

	// Toast Notification State
	/**
	 * @typedef {Object} ToastItem
	 * @property {string} id
	 * @property {'success' | 'error' | 'info'} type
	 * @property {string} title
	 * @property {string} message
	 */
	let toasts = $state([]);

	function addToast(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		const toast = { id, type, title, message };
		toasts = [...toasts, toast];

		setTimeout(() => {
			removeToast(id);
		}, 4500);
	}

	function removeToast(id) {
		toasts = toasts.filter((t) => t.id !== id);
	}

	// Chunky status polling timer reference
	let chunkyPollTimer = null;
	let worldsPollTimer = null;
	let currentIntervalMs = null;

	function resetChunkyPollTimer() {
		const isRunningOrPaused = chunkyStatus.is_running || chunkyStatus.is_paused;
		const targetInterval = isRunningOrPaused ? 2500 : 10000;

		if (currentIntervalMs !== targetInterval) {
			if (chunkyPollTimer) clearInterval(chunkyPollTimer);
			currentIntervalMs = targetInterval;
			chunkyPollTimer = setInterval(pollChunkyStatus, targetInterval);
		}
	}

	// Load all worlds data from API endpoints
	async function loadWorldsData() {
		loading = true;
		try {
			const [worldsRes, chunkyRes, backupsRes] = await Promise.all([
				apiGet('/api/worlds').catch((err) => {
					console.warn('Worlds API endpoint returned error or fallback:', err);
					return [];
				}),
				apiGet('/api/worlds/chunky/status').catch((err) => {
					console.warn('Chunky status endpoint error:', err);
					return {
						is_running: false,
						is_paused: false,
						percent_complete: 0,
						chunks_rendered: 0,
						chunks_total: 0,
						current_cps: 0,
						eta_seconds: 0,
						world: ''
					};
				}),
				apiGet('/api/worlds/backups').catch((err) => {
					console.warn('Backups endpoint error:', err);
					return [];
				})
			]);

			worlds = Array.isArray(worldsRes?.worlds) ? worldsRes.worlds : [];
			pendingWorlds = Array.isArray(worldsRes?.pending_worlds) ? worldsRes.pending_worlds : [];
			activeWorld = worldsRes?.active_world ?? null;
			serverDataVersion = worldsRes?.server_data_version ?? null;
			if (chunkyRes) {
				chunkyStatus = chunkyRes;
				resetChunkyPollTimer();
			}
			backups = Array.isArray(backupsRes) ? backupsRes : [];

			// Fetch detailed border info for each world
			for (const w of worlds) {
				try {
					const detail = await apiGet(`/api/worlds/${w.folder_name}`);
					if (Array.isArray(detail) && detail[1]) {
						borders[w.folder_name] = detail[1];
					}
				} catch (e) {
					// Fallback to default border info if detail fetch fails
					borders[w.folder_name] = {
						size: 60000000,
						center_x: 0,
						center_z: 0,
						damage_amount: 0.2,
						warning_distance: 5
					};
				}
			}
		} catch (err) {
			console.error('Failed to load worlds data:', err);
			addToast('error', 'Failed to Load Worlds', err.message || 'Could not fetch worlds data from server.');
		} finally {
			loading = false;
		}
	}

	// Poll Chunky status while task is running or paused
	async function pollChunkyStatus() {
		try {
			const chunkyRes = await apiGet('/api/worlds/chunky/status');
			if (chunkyRes) {
				chunkyStatus = chunkyRes;
				resetChunkyPollTimer();
			}
		} catch (e) {
			console.warn('Failed polling Chunky status:', e);
		}
	}

	onMount(() => {
		loadWorldsData();
		resetChunkyPollTimer();
		worldsPollTimer = setInterval(loadWorldsData, 10000);
	});

	onDestroy(() => {
		if (chunkyPollTimer) {
			clearInterval(chunkyPollTimer);
		}
		if (worldsPollTimer) {
			clearInterval(worldsPollTimer);
		}
	});

	// Derived Header Quick Indicators
	let totalWorldsCount = $derived(worlds.length + pendingWorlds.length);
	let chunkyQuickStatus = $derived.by(() => {
		if (chunkyStatus.is_running) {
			return {
				label: `Pre-generating (${(chunkyStatus.percent_complete || 0).toFixed(0)}%)`,
				badgeClass: 'badge-success',
				dotClass: 'status-dot-success status-dot-pulse'
			};
		}
		if (chunkyStatus.is_paused) {
			return {
				label: 'Chunky Paused',
				badgeClass: 'badge-warning',
				dotClass: 'status-dot-warning'
			};
		}
		return {
			label: 'Chunky Idle',
			badgeClass: 'badge-secondary',
			dotClass: 'status-dot'
		};
	});

	// Action Handlers
	function handleOpenEditBorder(world) {
		targetWorldForBorder = world;
		const borderInfo = borders[world.folder_name] || {};
		borderSize = borderInfo.size ?? 60000000;
		borderCenterX = borderInfo.center_x ?? 0;
		borderCenterZ = borderInfo.center_z ?? 0;
		borderDamage = borderInfo.damage_amount ?? 0.2;
		borderWarning = borderInfo.warning_distance ?? 5;
		borderModalOpen = true;
	}

	function closeBorderModal() {
		if (isSavingBorder) return;
		borderModalOpen = false;
		targetWorldForBorder = null;
	}

	async function saveWorldborder(e) {
		if (e) e.preventDefault();
		if (!targetWorldForBorder || isSavingBorder) return;

		isSavingBorder = true;
		const worldName = targetWorldForBorder.folder_name;

		try {
			const res = await apiPost('/api/worlds/' + encodeURIComponent(worldName) + '/worldborder', {
				size: Number(borderSize),
				center_x: Number(borderCenterX),
				center_z: Number(borderCenterZ)
			});

			borders[worldName] = {
				size: Number(borderSize),
				center_x: Number(borderCenterX),
				center_z: Number(borderCenterZ),
				damage_amount: Number(borderDamage),
				warning_distance: Number(borderWarning)
			};

			addToast('success', 'Worldborder Updated', res.message || `Worldborder for '${worldName}' updated to ${borderSize.toLocaleString()} blocks.`);
			closeBorderModal();
		} catch (err) {
			console.error('Failed to update worldborder:', err);
			addToast('error', 'Update Failed', err.message || `Could not update worldborder for '${worldName}'.`);
		} finally {
			isSavingBorder = false;
		}
	}

	function handleStartPregenFromCard(world) {
		pregenTargetWorld = world.folder_name;
		addToast('info', 'Pre-generator Target Selected', `Selected '${world.level_name}' in Chunky pre-generator below.`);
		// Scroll smoothly to chunky panel
		const element = document.getElementById('chunky-panel-section');
		if (element) {
			element.scrollIntoView({ behavior: 'smooth' });
		}
	}

	function handleCreateBackupFromCard(world) {
		backupModalOpen = true;
	}

	// --- World lifecycle actions ---

	async function switchTo(name) {
		if (switchInFlight) return;
		switchInFlight = true;
		try {
			// Optimistic: the active world updates immediately; the background
			// restart generates/loads it asynchronously (polled into place).
			activeWorld = name;
			const res = await apiPost('/api/worlds/switch', { name });
			addToast('success', 'World Switched', res.message || `Active world set to '${name}'.`);
			if (res.warning) addToast('info', 'Warning', res.warning);
			addToast('info', 'Generating in background', 'The server is restarting on this world in the background.');
			await loadWorldsData();
		} catch (err) {
			addToast('error', 'Switch Failed', err.message || `Could not switch to '${name}'.`);
			await loadWorldsData();
		} finally {
			switchInFlight = false;
		}
	}

	function handleSwitch(world) {
		switchTo(world.folder_name);
	}

	function handleDeleteRequest(world) {
		deleteTarget = world;
	}

	async function confirmDelete() {
		if (!deleteTarget || deleteInFlight) return;
		deleteInFlight = true;
		const world = deleteTarget;
		try {
			const res = await apiFetch('/api/worlds/' + encodeURIComponent(world.folder_name), {
				method: 'DELETE'
			});
			const data = await res.json().catch(() => ({}));
			if (!res.ok) {
				throw new Error(data?.error || `Delete failed (HTTP ${res.status})`);
			}
			addToast('success', 'World Deleted', data?.message || `World '${world.folder_name}' deleted.`);
			deleteTarget = null;
			await loadWorldsData();
		} catch (err) {
			addToast('error', 'Delete Failed', err.message || `Could not delete '${world.folder_name}'.`);
		} finally {
			deleteInFlight = false;
		}
	}

	async function handleDownload(world) {
		try {
			await apiDownload(
				'/api/worlds/' + encodeURIComponent(world.folder_name) + '/download',
				world.folder_name + '.zip'
			);
		} catch (err) {
			addToast('error', 'Download Failed', err.message || `Could not download '${world.folder_name}'.`);
		}
	}

	function handleOpenConfigure(world) {
		configureTarget = world;
		configureModalOpen = true;
	}

	function handleChunkyActionSuccess(action, message) {
		pollChunkyStatus();
	}

	function handleKeydown(e) {
		if (e.key === 'Escape') {
			if (borderModalOpen) {
				closeBorderModal();
			} else if (createModalOpen) {
				createModalOpen = false;
			} else if (importModalOpen) {
				importModalOpen = false;
			} else if (configureModalOpen) {
				configureModalOpen = false;
			} else if (deleteTarget) {
				deleteTarget = null;
			} else if (backupModalOpen) {
				backupModalOpen = false;
			}
		}
	}
</script>

<svelte:head>
	<title>World Management & Chunky Control - ChiPanel</title>
</svelte:head>
<svelte:window onkeydown={handleKeydown} />

<div class="worlds-page">
	<!-- Top Hero Header Banner -->
	<div class="hero-header">
		<div class="hero-title-section">
			<div class="hero-icon-box">
				<Globe size={26} />
			</div>
			<div>
				<h1 class="page-title">World Management & Chunky Pre-generation</h1>
				<p class="page-subtitle">
					Inspect server dimensions, configure world borders, launch background chunk pre-generation, and manage ZIP backups.
				</p>
			</div>
		</div>

		<!-- Quick Status & Action Buttons -->
		<div class="hero-actions-row">
			<div class="quick-pills-group">
				<div class="quick-pill">
					<Globe size={16} class="pill-icon icon-blue" />
					<span class="pill-text font-mono"><strong>{totalWorldsCount}</strong> Worlds</span>
				</div>

				<div class="quick-pill badge {chunkyQuickStatus.badgeClass}">
					<span class="status-dot {chunkyQuickStatus.dotClass}"></span>
					<span>{chunkyQuickStatus.label}</span>
				</div>
			</div>

			<div class="buttons-group">
				<button
					type="button"
					class="btn btn-secondary"
					onclick={loadWorldsData}
					disabled={loading}
					title="Reload worlds & backups"
				>
					<RefreshCw size={16} class={loading ? 'spin-slow' : ''} />
					<span>Refresh</span>
				</button>

				<button
					type="button"
					class="btn btn-primary"
					onclick={() => (createModalOpen = true)}
				>
					<Plus size={16} />
					<span>New World</span>
				</button>

				<button
					type="button"
					class="btn btn-secondary"
					onclick={() => (importModalOpen = true)}
				>
					<Upload size={16} />
					<span>Import</span>
				</button>

				<button
					type="button"
					class="btn btn-secondary"
					onclick={() => (backupModalOpen = true)}
				>
					<Archive size={16} />
					<span>Backups ({backups.length})</span>
				</button>
			</div>
		</div>
	</div>

	<!-- Main Worlds Grid Section -->
	<div class="page-section">
		<div class="section-title-row">
			<h2 class="section-title">Worlds ({totalWorldsCount})</h2>
		</div>

		{#if loading && worlds.length === 0 && pendingWorlds.length === 0}
			<div class="empty-state-card card">
				<RefreshCw size={36} class="spin-slow empty-icon" />
				<p class="empty-title">Loading worlds...</p>
			</div>
		{:else if worlds.length === 0 && pendingWorlds.length === 0}
			<div class="empty-state-card card">
				<Globe size={36} class="empty-icon" />
				<p class="empty-title">No worlds on the server</p>
				<p class="empty-desc">Create a world to get started.</p>
			</div>
		{:else}
			<div class="worlds-grid">
				{#each pendingWorlds as pending (pending.name)}
					<div class="card world-card pending-card {activeWorld === pending.name ? 'pending-active' : ''}">
						<div class="pending-card-body">
							<div class="dimension-icon-box">
								<Plus size={18} class="pending-icon" />
							</div>
							<div class="pending-meta">
								<h3 class="pending-title">
									{pending.name}
									{#if activeWorld === pending.name}
										<span class="badge badge-warning active-badge">Active · generating…</span>
									{/if}
								</h3>
								<p class="pending-desc">
									Pas encore généré — sera créé quand tu le choisiras.
									{#if pending.level_type !== 'default'}
										· générateur <span class="font-mono">{pending.level_type}</span>
									{/if}
									{#if pending.seed}
										· seed <span class="font-mono">{pending.seed}</span>
									{/if}
								</p>
							</div>
						</div>
						<div class="card-footer world-card-footer">
							<button
								type="button"
								class="btn btn-primary btn-sm action-btn {switchInFlight ? 'btn-loading' : ''}"
								onclick={() => switchTo(pending.name)}
								disabled={switchInFlight}
								title="Choose this world (restarts the server to generate/load it)"
							>
								<Play size={14} />
								<span>Choose world</span>
							</button>
							<button
								type="button"
								class="btn btn-danger btn-sm action-btn"
								onclick={() => handleDeleteRequest({ folder_name: pending.name, level_name: pending.name })}
								title="Delete this pending world"
							>
								<Trash2 size={14} />
								<span>Delete</span>
							</button>
						</div>
					</div>
				{/each}

				{#each worlds as world (world.folder_name)}
					<WorldCard
						{world}
						border={borders[world.folder_name]}
						isActive={activeWorld === world.folder_name}
						{serverDataVersion}
						switchInFlight={switchInFlight}
						onEditBorder={handleOpenEditBorder}
						onStartPregen={handleStartPregenFromCard}
						onCreateBackup={handleCreateBackupFromCard}
						onSwitch={handleSwitch}
						onDelete={handleDeleteRequest}
						onDownload={handleDownload}
						onConfigure={handleOpenConfigure}
					/>
				{/each}
			</div>
		{/if}
	</div>

	<!-- Chunky Pre-generation Telemetry & Control Panel Section -->
	<div id="chunky-panel-section" class="page-section">
		<ChunkyPanel
			{chunkyStatus}
			{worlds}
			selectedWorldName={pregenTargetWorld}
			onActionSuccess={handleChunkyActionSuccess}
			onToast={addToast}
		/>
	</div>

	<!-- World Backup Manager Modal -->
	<WorldBackupModal
		isOpen={backupModalOpen}
		{backups}
		{worlds}
		{loading}
		onClose={() => (backupModalOpen = false)}
		onRefresh={loadWorldsData}
		onToast={addToast}
	/>

	<!-- Edit Worldborder Modal -->
	{#if borderModalOpen && targetWorldForBorder}
		<div
			class="modal-backdrop"
			onclick={closeBorderModal}
			onkeydown={handleKeydown}
			role="dialog"
			aria-modal="true"
			aria-labelledby="border-modal-title"
			tabindex="-1"
		>
			<div class="modal border-modal" onclick={(e) => e.stopPropagation()}>
				<div class="modal-header">
					<div class="title-with-icon">
						<Maximize2 size={20} class="icon-blue" />
						<h3 id="border-modal-title" class="modal-title">
							Edit Worldborder: <span class="font-mono text-blue">{targetWorldForBorder.level_name}</span>
						</h3>
					</div>
					<button
						type="button"
						class="btn btn-ghost btn-icon btn-sm"
						onclick={closeBorderModal}
						disabled={isSavingBorder}
						aria-label="Close Worldborder Modal"
					>
						<X size={18} />
					</button>
				</div>

				<form onsubmit={saveWorldborder}>
					<div class="modal-body border-modal-body">
						<div class="form-group">
							<label for="border-size-input" class="label label-required">Worldborder Diameter / Size (Blocks)</label>
							<input
								id="border-size-input"
								type="number"
								class="input font-mono"
								min="10"
								step="100"
								bind:value={borderSize}
								required
							/>
							<span class="field-hint">Sets boundary diameter. Common values: 60000000 (Default Unbounded), 10000, 25000.</span>
						</div>

						<div class="form-grid-2">
							<div class="form-group">
								<label for="border-center-x" class="label">Center X Coordinate</label>
								<input
									id="border-center-x"
									type="number"
									class="input font-mono"
									bind:value={borderCenterX}
								/>
							</div>

							<div class="form-group">
								<label for="border-center-z" class="label">Center Z Coordinate</label>
								<input
									id="border-center-z"
									type="number"
									class="input font-mono"
									bind:value={borderCenterZ}
								/>
							</div>
						</div>

						<div class="form-grid-2">
							<div class="form-group">
								<label for="border-damage" class="label">Damage Per Block (Outside Border)</label>
								<input
									id="border-damage"
									type="number"
									step="0.05"
									min="0"
									class="input font-mono"
									bind:value={borderDamage}
								/>
							</div>

							<div class="form-group">
								<label for="border-warning" class="label">Warning Distance (Blocks)</label>
								<input
									id="border-warning"
									type="number"
									min="0"
									class="input font-mono"
									bind:value={borderWarning}
								/>
							</div>
						</div>
					</div>

					<div class="modal-footer">
						<button
							type="button"
							class="btn btn-secondary"
							onclick={closeBorderModal}
							disabled={isSavingBorder}
						>
							Cancel
						</button>
						<button
							type="submit"
							class="btn btn-primary {isSavingBorder ? 'btn-loading' : ''}"
							disabled={isSavingBorder}
						>
							{#if !isSavingBorder}
								<CheckCircle2 size={16} />
							{/if}
							<span>Save Worldborder</span>
						</button>
					</div>
				</form>
			</div>
		</div>
	{/if}

	<!-- Create / Import / Configure world modals -->
	<CreateWorldModal
		isOpen={createModalOpen}
		onClose={() => (createModalOpen = false)}
		onDone={async () => {
			createModalOpen = false;
			await loadWorldsData();
		}}
		onToast={addToast}
	/>

	<ImportWorldModal
		isOpen={importModalOpen}
		{backups}
		onClose={() => (importModalOpen = false)}
		onDone={async () => {
			importModalOpen = false;
			await loadWorldsData();
		}}
		onToast={addToast}
	/>

	<ConfigureWorldModal
		isOpen={configureModalOpen}
		world={configureTarget}
		onClose={() => (configureModalOpen = false)}
		onToast={addToast}
	/>

	<!-- Delete world confirmation -->
	{#if deleteTarget}
		<div
			class="modal-backdrop"
			onclick={() => (deleteInFlight ? null : (deleteTarget = null))}
			role="dialog"
			aria-modal="true"
			aria-labelledby="delete-modal-title"
			tabindex="-1"
		>
			<div class="modal border-modal" onclick={(e) => e.stopPropagation()}>
				<div class="modal-header">
					<div class="title-with-icon">
						<AlertCircle size={20} class="text-danger" />
						<h3 id="delete-modal-title" class="modal-title">
							Delete World: <span class="font-mono text-blue">{deleteTarget.level_name}</span>
						</h3>
					</div>
					<button
						type="button"
						class="btn btn-ghost btn-icon btn-sm"
						onclick={() => (deleteTarget = null)}
						disabled={deleteInFlight}
						aria-label="Close Delete Modal"
					>
						<X size={18} />
					</button>
				</div>

				<div class="modal-body border-modal-body">
					<p class="delete-warning-text">
						This will permanently delete the world folder
						<span class="font-mono">{deleteTarget.folder_name}</span>. A safety ZIP backup is
						created automatically first and kept in the backups list.
					</p>
				</div>

				<div class="modal-footer">
					<button
						type="button"
						class="btn btn-secondary"
						onclick={() => (deleteTarget = null)}
						disabled={deleteInFlight}
					>
						Cancel
					</button>
					<button
						type="button"
						class="btn btn-danger {deleteInFlight ? 'btn-loading' : ''}"
						onclick={confirmDelete}
						disabled={deleteInFlight}
					>
						{#if !deleteInFlight}
							<AlertCircle size={16} />
						{/if}
						<span>Delete World</span>
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Toast Notifications Stack Container -->
	{#if toasts.length > 0}
		<div class="toast-container">
			{#each toasts as toast (toast.id)}
				<div class="toast-item toast-{toast.type}">
					<div class="toast-icon">
						{#if toast.type === 'success'}
							<CheckCircle2 size={18} />
						{:else if toast.type === 'error'}
							<AlertCircle size={18} />
						{:else}
							<Info size={18} />
						{/if}
					</div>

					<div class="toast-body">
						<h4 class="toast-title">{toast.title}</h4>
						<p class="toast-message">{toast.message}</p>
					</div>

					<button
						type="button"
						class="btn btn-ghost btn-icon btn-sm toast-close-btn"
						onclick={() => removeToast(toast.id)}
						aria-label="Close Notification"
					>
						<X size={14} />
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.worlds-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		position: relative;
	}

	/* Hero Header */
	.hero-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		border-bottom: 1px solid var(--border);
		padding-bottom: var(--space-6);
	}

	.hero-title-section {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.hero-icon-box {
		width: 52px;
		height: 52px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	.page-title {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		letter-spacing: -0.015em;
		line-height: 1.2;
	}

	.page-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.hero-actions-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.quick-pills-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.quick-pill {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: var(--space-2) var(--space-4);
		font-size: var(--font-size-sm);
	}

	.pill-icon {
		flex-shrink: 0;
	}

	.buttons-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	/* Section Layout */
	.page-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.section-title-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.section-title {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.worlds-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
		gap: var(--space-6);
	}

	/* Empty State Card */
	.empty-state-card {
		padding: var(--space-12) var(--space-6);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		text-align: center;
		color: var(--text-muted);
	}

	.empty-icon {
		color: var(--text-muted);
	}

	.empty-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.empty-desc {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}

	/* Pending (not-yet-generated) world card */
	.pending-card {
		border-style: dashed;
		border-color: var(--accent-blue-border);
	}

	.pending-card.pending-active {
		border-style: solid;
		border-color: var(--accent-green-border);
	}

	.pending-card-body {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		padding: var(--space-6);
		flex: 1;
	}

	.pending-icon {
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	.pending-meta {
		flex: 1;
		min-width: 0;
	}

	.pending-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-wrap: wrap;
	}

	.pending-desc {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		margin-top: 2px;
	}

	/* Border Modal */
	.border-modal {
		max-width: 540px;
		width: 90vw;
	}

	.title-with-icon {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.text-blue {
		color: var(--accent-blue-text);
	}

	.text-danger {
		color: var(--danger-text);
	}

	.delete-warning-text {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		line-height: 1.6;
	}

	.border-modal-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.form-grid-2 {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-4);
	}

	.field-hint {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 4px;
		display: block;
	}

	.spin-slow {
		animation: spin 3s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* Toast Container */
	.toast-container {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		z-index: var(--z-toast);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		max-width: 380px;
		width: 100%;
		pointer-events: none;
	}

	.toast-item {
		pointer-events: auto;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-focus);
		border-radius: var(--radius-card);
		padding: var(--space-4);
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
		box-shadow: var(--elevation-shadow);
		animation: toastSlideIn var(--transition-fast) ease-out;
	}

	@keyframes toastSlideIn {
		from {
			opacity: 0;
			transform: translateY(16px) scale(0.96);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.toast-success {
		border-color: var(--accent-green-border);
		background: linear-gradient(135deg, var(--bg-surface) 0%, rgba(74, 222, 128, 0.05) 100%);
	}

	.toast-success .toast-icon {
		color: var(--accent-green);
	}

	.toast-error {
		border-color: var(--danger-border);
		background: linear-gradient(135deg, var(--bg-surface) 0%, rgba(244, 63, 94, 0.05) 100%);
	}

	.toast-error .toast-icon {
		color: var(--danger-text);
	}

	.toast-info {
		border-color: var(--accent-blue-border);
		background: linear-gradient(135deg, var(--bg-surface) 0%, rgba(99, 102, 241, 0.05) 100%);
	}

	.toast-info .toast-icon {
		color: var(--accent-blue-text);
	}

	.toast-body {
		flex: 1;
		min-width: 0;
	}

	.toast-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		line-height: 1.3;
	}

	.toast-message {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin-top: 2px;
		word-break: break-word;
	}

	.toast-close-btn {
		color: var(--text-muted);
		padding: 2px;
		margin-top: -2px;
		margin-right: -4px;
	}

	.toast-close-btn:hover {
		color: var(--text-primary);
	}

	@media (max-width: 640px) {
		.hero-actions-row {
			flex-direction: column;
			align-items: stretch;
		}

		.buttons-group {
			justify-content: stretch;
		}

		.buttons-group .btn {
			flex: 1;
		}

		.form-grid-2 {
			grid-template-columns: 1fr;
		}

		.toast-container {
			left: var(--space-4);
			right: var(--space-4);
			bottom: var(--space-4);
			max-width: none;
		}
	}
</style>
