<script>
	import { onMount } from 'svelte';
	import { apiGet, apiFetch } from '$lib/api/client.js';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import {
		ShieldAlert,
		ShieldCheck,
		Search,
		RefreshCw,
		DownloadCloud,
		FileText,
		CheckCircle2,
		AlertCircle,
		AlertTriangle,
		User,
		Globe,
		Server,
		Database,
		Archive,
		Key,
		Layers,
		Sliders,
		Terminal,
		Check,
		Copy,
		X,
		Eye
	} from 'lucide-svelte';

	/** @type {Array<{ id: string, timestamp_secs: number, username: string, action: string, category: string, status: string, details: any, ip_address?: string }>} */
	let events = $state([]);
	let totalCount = $state(0);
	let loading = $state(true);
	let error = $state('');

	// Filters
	let searchQuery = $state('');
	let selectedCategory = $state('all');
	let selectedStatus = $state('ALL');

	// Inspect detail modal
	let showDetailModal = $state(false);
	/** @type {any} */
	let selectedEvent = $state(null);
	let copied = $state(false);

	const categories = [
		{ id: 'all', label: 'Toutes les catégories' },
		{ id: 'auth', label: 'Authentification & Sécurité' },
		{ id: 'server', label: 'Cycle Serveur' },
		{ id: 'backups', label: 'Sauvegardes' },
		{ id: 'maintenance', label: 'Maintenance & BDD' },
		{ id: 'permissions', label: 'Permissions & Rôles' },
		{ id: 'addons', label: 'Addons & Plugins' },
		{ id: 'config', label: 'Configuration' },
		{ id: 'rcon', label: 'Commandes RCON' }
	];

	onMount(() => {
		loadAuditLogs();
	});

	async function loadAuditLogs() {
		loading = true;
		error = '';
		try {
			const params = new URLSearchParams();
			if (searchQuery.trim()) params.set('search', searchQuery.trim());
			if (selectedCategory !== 'all') params.set('category', selectedCategory);
			if (selectedStatus !== 'ALL') params.set('status', selectedStatus);
			params.set('limit', '100');

			const res = await apiGet(`/api/audit?${params.toString()}`);
			events = res?.events || [];
			totalCount = res?.total || 0;
		} catch (err) {
			error = err.message || 'Impossible de charger le registre d\'audit.';
		} finally {
			loading = false;
		}
	}

	function handleSearchInput() {
		loadAuditLogs();
	}

	function handleCategoryChange(cat) {
		selectedCategory = cat;
		loadAuditLogs();
	}

	function handleStatusChange(st) {
		selectedStatus = st;
		loadAuditLogs();
	}

	async function handleExportCsv() {
		try {
			const token = localStorage.getItem('chipanel_token') || sessionStorage.getItem('chipanel_token');
			const response = await fetch('/api/audit/export/csv', {
				headers: {
					Authorization: `Bearer ${token}`
				}
			});
			if (!response.ok) throw new Error('Échec du téléchargement CSV');
			const blob = await response.blob();
			const url = window.URL.createObjectURL(blob);
			const a = document.createElement('a');
			a.href = url;
			a.download = `chipanel_audit_log_${Date.now()}.csv`;
			document.body.appendChild(a);
			a.click();
			window.URL.revokeObjectURL(url);
			a.remove();
		} catch (err) {
			alert(err.message || 'Erreur lors de l\'export CSV');
		}
	}

	function openDetailModal(event) {
		selectedEvent = event;
		showDetailModal = true;
		copied = false;
	}

	async function copyJsonDetail() {
		if (!selectedEvent) return;
		try {
			await navigator.clipboard.writeText(JSON.stringify(selectedEvent, null, 2));
			copied = true;
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			console.error('Failed to copy JSON:', err);
		}
	}

	function formatDate(timestampSecs) {
		if (!timestampSecs) return '—';
		return new Date(timestampSecs * 1000).toLocaleString('fr-FR', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});
	}

	function getCategoryBadgeClass(category) {
		switch (category) {
			case 'auth':
				return 'badge-primary';
			case 'backups':
				return 'badge-success';
			case 'maintenance':
				return 'badge-warning';
			case 'server':
				return 'badge-info';
			default:
				return 'badge-secondary';
		}
	}

	// Computed KPIs
	let successCount = $derived(events.filter((e) => e.status === 'SUCCESS').length);
	let failedCount = $derived(events.filter((e) => e.status === 'FAILED').length);
	let successRate = $derived(events.length > 0 ? Math.round((successCount / events.length) * 100) : 100);
</script>

<svelte:head>
	<title>Registre d'Audit & Sécurité | ChiPanel</title>
</svelte:head>

<div class="audit-page-layout">
	<!-- Page Header -->
	<PageHeader
		title="Registre d'Audit & Journal des Actions"
		subtitle="Historique chronologique et exhaustif des actions sensibles effectuées par les administrateurs et le système."
	>
		{#snippet icon()}
			<ShieldAlert size={22} />
		{/snippet}
		{#snippet badge()}
			<span class="badge">Traçabilité Immuable</span>
		{/snippet}
		<button class="btn btn-secondary btn-sm" onclick={handleExportCsv}>
			<DownloadCloud size={14} />
			<span>Exporter CSV</span>
		</button>
		<button class="btn btn-ghost btn-sm" onclick={loadAuditLogs} disabled={loading}>
			<RefreshCw size={14} class={loading ? 'spin' : ''} />
			<span>Actualiser</span>
		</button>
	</PageHeader>

	<!-- KPI Stats Grid -->
	<div class="kpi-grid">
		<div class="kpi-card card">
			<div class="kpi-icon-wrapper kpi-primary">
				<FileText size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Événements Enregistrés</span>
				<span class="kpi-value">{totalCount}</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper kpi-success">
				<ShieldCheck size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Taux de Succès</span>
				<span class="kpi-value">{successRate}%</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper {failedCount > 0 ? 'kpi-danger' : 'kpi-info'}">
				<AlertTriangle size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Échecs / Alertes</span>
				<span class="kpi-value">{failedCount}</span>
			</div>
		</div>

		<div class="kpi-card card">
			<div class="kpi-icon-wrapper kpi-info">
				<Globe size={22} />
			</div>
			<div class="kpi-content">
				<span class="kpi-label">Dernière Action</span>
				<span class="kpi-value text-xs font-mono">
					{events.length > 0 ? formatDate(events[0].timestamp_secs) : 'Aucune'}
				</span>
			</div>
		</div>
	</div>

	<!-- Main Filter & Table Card -->
	<div class="card table-card">
		<div class="toolbar-row">
			<div class="search-box">
				<Search size={16} class="search-icon text-muted" />
				<input
					type="text"
					class="search-input"
					placeholder="Filtrer par action, utilisateur, IP ou détail JSON..."
					bind:value={searchQuery}
					oninput={handleSearchInput}
				/>
				{#if searchQuery}
					<button class="clear-btn" onclick={() => { searchQuery = ''; loadAuditLogs(); }}>
						<X size={14} />
					</button>
				{/if}
			</div>

			<div class="filter-controls">
				<select class="select select-sm" bind:value={selectedCategory} onchange={() => handleCategoryChange(selectedCategory)}>
					{#each categories as cat}
						<option value={cat.id}>{cat.label}</option>
					{/each}
				</select>

				<select class="select select-sm" bind:value={selectedStatus} onchange={() => handleStatusChange(selectedStatus)}>
					<option value="ALL">Tous statuts</option>
					<option value="SUCCESS">Succès uniquement</option>
					<option value="FAILED">Échecs uniquement</option>
				</select>
			</div>
		</div>

		{#if loading}
			<div class="loading-state">
				<RefreshCw size={24} class="spin text-muted" />
				<span>Chargement des événements d'audit...</span>
			</div>
		{:else if events.length === 0}
			<div class="empty-state">
				<ShieldCheck size={40} class="text-muted" />
				<p>Aucun événement ne correspond aux critères de recherche.</p>
				<button class="btn btn-secondary btn-sm" onclick={() => { searchQuery = ''; selectedCategory = 'all'; selectedStatus = 'ALL'; loadAuditLogs(); }}>
					Réinitialiser les filtres
				</button>
			</div>
		{:else}
			<div class="table-wrapper">
				<table class="data-table">
					<thead>
						<tr>
							<th>Date & Heure</th>
							<th>Utilisateur</th>
							<th>Catégorie</th>
							<th>Action</th>
							<th>Statut</th>
							<th>Détails</th>
							<th class="text-right">Inspecter</th>
						</tr>
					</thead>
					<tbody>
						{#each events as event (event.id)}
							<tr>
								<td class="text-sm font-mono text-muted whitespace-nowrap">
									{formatDate(event.timestamp_secs)}
								</td>
								<td>
									<div class="user-cell">
										<User size={14} class="text-muted flex-shrink-0" />
										<span class="font-semibold text-sm">{event.username}</span>
									</div>
								</td>
								<td>
									<span class="badge {getCategoryBadgeClass(event.category)}">
										{event.category}
									</span>
								</td>
								<td>
									<span class="action-tag font-mono">{event.action}</span>
								</td>
								<td>
									{#if event.status === 'SUCCESS'}
										<span class="status-pill status-success">
											<CheckCircle2 size={12} />
											<span>Succès</span>
										</span>
									{:else}
										<span class="status-pill status-danger">
											<AlertCircle size={12} />
											<span>Échec</span>
										</span>
									{/if}
								</td>
								<td class="details-cell">
									<span class="details-preview font-mono text-xs text-muted">
										{JSON.stringify(event.details)}
									</span>
								</td>
								<td class="text-right">
									<button class="btn btn-ghost btn-sm btn-icon" onclick={() => openDetailModal(event)} title="Inspecter l'événement">
										<Eye size={15} />
									</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</div>

<!-- Modal Détail Événement JSON -->
{#if showDetailModal && selectedEvent}
	<div class="modal-backdrop" onclick={() => (showDetailModal = false)} role="presentation">
		<div class="modal-card card" onclick={(e) => e.stopPropagation()} role="dialog">
			<div class="modal-header">
				<div class="modal-title-with-badge">
					<h2 class="modal-title font-mono">{selectedEvent.action}</h2>
					<span class="badge {getCategoryBadgeClass(selectedEvent.category)}">{selectedEvent.category}</span>
				</div>
				<button class="btn btn-ghost btn-sm btn-icon" onclick={() => (showDetailModal = false)}>
					<X size={16} />
				</button>
			</div>

			<div class="modal-body">
				<div class="meta-row-grid">
					<div class="meta-item">
						<span class="meta-label">ID Événement :</span>
						<span class="meta-val font-mono text-xs">{selectedEvent.id}</span>
					</div>
					<div class="meta-item">
						<span class="meta-label">Horodatage :</span>
						<span class="meta-val text-xs">{formatDate(selectedEvent.timestamp_secs)}</span>
					</div>
					<div class="meta-item">
						<span class="meta-label">Utilisateur :</span>
						<span class="meta-val font-semibold text-xs">{selectedEvent.username}</span>
					</div>
					<div class="meta-item">
						<span class="meta-label">Statut :</span>
						<span class="meta-val font-semibold text-xs {selectedEvent.status === 'SUCCESS' ? 'text-success' : 'text-danger'}">
							{selectedEvent.status}
						</span>
					</div>
				</div>

				<div class="json-viewer-header">
					<span class="form-label">Payload & Détails JSON :</span>
					<button class="btn btn-ghost btn-xs btn-icon" onclick={copyJsonDetail} title="Copier le JSON">
						{#if copied}<Check size={13} class="text-success" />{:else}<Copy size={13} />{/if}
					</button>
				</div>

				<pre class="json-code-box font-mono text-xs">{JSON.stringify(selectedEvent.details, null, 2)}</pre>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary btn-sm" onclick={() => (showDetailModal = false)}>Fermer</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.audit-page-layout {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		padding-bottom: var(--space-8);
	}

	/* KPI Grid */
	.kpi-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-4);
	}

	.kpi-card {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-4);
		background-color: var(--bg-surface);
	}

	.kpi-icon-wrapper {
		width: 42px;
		height: 42px;
		border-radius: var(--radius-md);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.kpi-primary { background-color: var(--accent-blue-bg); color: var(--accent-blue-text); }
	.kpi-success { background-color: rgba(16, 185, 129, 0.15); color: #10B981; }
	.kpi-info { background-color: rgba(59, 130, 246, 0.15); color: #60A5FA; }
	.kpi-danger { background-color: rgba(239, 68, 68, 0.15); color: #EF4444; }

	.kpi-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.kpi-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.kpi-value {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}

	/* Table Card */
	.table-card {
		background-color: var(--bg-surface);
	}

	.toolbar-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
	}

	.search-box {
		position: relative;
		flex: 1;
		max-width: 480px;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: 12px;
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: 8px 32px 8px 36px;
		background-color: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
	}

	.search-input:focus {
		outline: none;
		border-color: var(--accent-blue-border);
	}

	.clear-btn {
		position: absolute;
		right: 8px;
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
	}

	.filter-controls {
		display: flex;
		gap: var(--space-3);
	}

	.table-wrapper {
		overflow-x: auto;
	}

	.data-table {
		width: 100%;
		border-collapse: collapse;
		text-align: left;
	}

	.data-table th {
		padding: var(--space-3) var(--space-6);
		font-size: var(--font-size-xs);
		text-transform: uppercase;
		color: var(--text-muted);
		border-bottom: 1px solid var(--border);
	}

	.data-table td {
		padding: var(--space-3) var(--space-6);
		border-bottom: 1px solid var(--border-subtle);
	}

	.user-cell {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.action-tag {
		padding: 2px 6px;
		background-color: var(--bg-card);
		border: 1px solid var(--border-subtle);
		border-radius: 4px;
		font-size: var(--font-size-xs);
	}

	.status-pill {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		padding: 2px 8px;
		border-radius: 9999px;
	}

	.status-success {
		background-color: rgba(16, 185, 129, 0.15);
		color: #10B981;
	}

	.status-danger {
		background-color: rgba(239, 68, 68, 0.15);
		color: #EF4444;
	}

	.details-cell {
		max-width: 280px;
	}

	.details-preview {
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-12);
		gap: var(--space-3);
		color: var(--text-muted);
	}

	/* Modal */
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.7);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal);
	}

	.modal-card {
		width: 100%;
		max-width: 580px;
		background-color: var(--bg-surface);
		box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
	}

	.modal-title-with-badge {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.modal-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin: 0;
	}

	.modal-body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.meta-row-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-card);
		border-radius: var(--radius-sm);
	}

	.meta-item {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.meta-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.json-viewer-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.json-code-box {
		padding: var(--space-4);
		background-color: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		overflow-x: auto;
		max-height: 280px;
		color: var(--text-primary);
		line-height: 1.4;
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		padding: var(--space-4) var(--space-6);
		border-top: 1px solid var(--border);
	}

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
