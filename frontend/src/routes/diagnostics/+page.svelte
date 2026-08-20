<script>
	import { onMount } from 'svelte';
	import { apiGet, apiPost } from '$lib/api/client.js';
	import PageHeader from '$lib/components/ui/PageHeader.svelte';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import {
		Activity,
		ShieldCheck,
		ShieldAlert,
		AlertTriangle,
		AlertCircle,
		CheckCircle2,
		Cpu,
		Zap,
		Wrench,
		RefreshCw,
		Copy,
		Check,
		FileText,
		Layers,
		Flame,
		Terminal,
		Sliders,
		Info,
		X
	} from '$lib/icons.js';

	let health = $state({
		score: 100,
		status: 'EXCELLENT',
		total_crashes_detected: 0,
		last_crash_timestamp: null,
		issues: [],
		recommendations: []
	});

	let autotune = $state(null);
	let crashes = $state([]);
	let loading = $state(true);
	let applyingAutotune = $state(false);
	let remediatingCrashId = $state(null);
	let selectedCrash = $state(null);
	let copiedJvm = $state(false);

	// Toasts
	let toasts = $state([]);
	function addToast(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		toasts = [...toasts, { id, type, title, message }];
		setTimeout(() => {
			toasts = toasts.filter((t) => t.id !== id);
		}, 4500);
	}

	onMount(() => {
		loadDiagnostics();
	});

	async function loadDiagnostics() {
		loading = true;
		try {
			const [healthRes, autotuneRes, crashesRes] = await Promise.all([
				apiGet('/api/diagnostics/health').catch(() => null),
				apiGet('/api/autotune/recommendations').catch(() => null),
				apiGet('/api/diagnostics/crashes').catch(() => [])
			]);

			if (healthRes) health = healthRes;
			if (autotuneRes) autotune = autotuneRes;
			if (crashesRes) crashes = crashesRes;
		} catch (err) {
			console.error('Failed to load diagnostics:', err);
		} finally {
			loading = false;
		}
	}

	async function handleApplyAutotune() {
		applyingAutotune = true;
		try {
			const res = await apiPost('/api/autotune/apply', { apply_properties: true });
			addToast('success', 'Auto-Tuning Appliqué', res.message);
			await loadDiagnostics();
		} catch (err) {
			addToast('danger', 'Erreur Auto-Tuning', err.message || 'Impossible d\'appliquer les optimisations.');
		} finally {
			applyingAutotune = false;
		}
	}

	async function handleRemediateCrash(id) {
		remediatingCrashId = id;
		try {
			const res = await apiPost(`/api/diagnostics/crashes/${encodeURIComponent(id)}/remediate`, {});
			addToast('success', 'Action Corrective Exécutée', res.message);
			await loadDiagnostics();
		} catch (err) {
			addToast('danger', 'Échec de la correction', err.message || 'Erreur lors de la remédiation.');
		} finally {
			remediatingCrashId = null;
		}
	}

	function copyJvmFlags() {
		if (!autotune?.recommended_jvm_flags) return;
		navigator.clipboard.writeText(autotune.recommended_jvm_flags);
		copiedJvm = true;
		setTimeout(() => (copiedJvm = false), 2000);
	}

	function formatDate(timestampSecs) {
		if (!timestampSecs) return '—';
		return new Date(timestampSecs * 1000).toLocaleString('fr-FR', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>Diagnostics & Auto-Tuner | ChiPanel</title>
</svelte:head>

<div class="diagnostics-page-layout">
	<!-- Header -->
	<PageHeader
		title="Diagnostics & Auto-Tuner"
		subtitle="Analyse heuristique des plantages Java, auto-remédiation 1-clic et optimisations matérielles JVM / cgroups."
	>
		{#snippet icon()}
			<Zap size={22} />
		{/snippet}
		{#snippet badge()}
			<span class="badge">Intelligence & Remédiation</span>
		{/snippet}
		<button class="btn btn-ghost btn-sm" onclick={loadDiagnostics} disabled={loading}>
			<RefreshCw size={14} class={loading ? 'spin' : ''} />
			<span>Actualiser</span>
		</button>
	</PageHeader>

	<!-- Health Score & Status Summary -->
	<div class="health-grid">
		<div class="card health-card">
			<div class="health-gauge">
				<div class="score-circle {health.score >= 80 ? 'score-good' : health.score >= 50 ? 'score-warn' : 'score-bad'}">
					<span class="score-num font-mono">{health.score}</span>
					<span class="score-unit">/ 100</span>
				</div>
				<div class="health-meta">
					<span class="health-badge badge {health.status === 'EXCELLENT' ? 'badge-success' : health.status === 'GOOD' ? 'badge-primary' : 'badge-warning'}">
						État du Serveur : {health.status}
					</span>
					<span class="text-sm text-muted">
						{health.total_crashes_detected} rapport(s) d'incident répertorié(s).
					</span>
				</div>
			</div>

			<div class="health-details">
				{#if (health?.issues?.length || 0) > 0}
					<div class="issue-box">
						<strong class="text-warning flex items-center gap-1.5 text-xs uppercase">
							<AlertTriangle size={14} />
							<span>Alertes actives :</span>
						</strong>
						<ul class="issue-list">
							{#each health.issues as issue}
								<li>{issue}</li>
							{/each}
						</ul>
					</div>
				{:else}
					<div class="issue-box good-box">
						<CheckCircle2 size={16} class="text-success flex-shrink-0" />
						<span class="text-sm">Aucun incident critique détecté au cours des dernières 24 heures.</span>
					</div>
				{/if}
			</div>
		</div>

		<!-- Auto-Tuner Quick Summary Card -->
		{#if autotune}
			<div class="card autotune-hero-card">
				<div class="autotune-header">
					<div class="flex items-center gap-2">
						<Cpu size={18} class="text-primary" />
						<h2 class="card-title">Profil Matériel & Moteur Détecté</h2>
					</div>
					<span class="badge badge-secondary font-mono">{autotune.hardware?.logical_cpu_cores ?? 'N/A'} Coeurs CPU</span>
				</div>

				<div class="specs-grid">
					<div class="spec-item">
						<span class="spec-label">Mémoire Heap Cible</span>
						<span class="spec-val font-mono">{autotune.recommended_heap_mb ?? 'N/A'} Mo</span>
					</div>
					<div class="spec-item">
						<span class="spec-label">Collecteur GC Recommandé</span>
						<span class="spec-val text-primary font-semibold">{autotune.gc_collector ?? 'G1GC'}</span>
					</div>
					<div class="spec-item">
						<span class="spec-label">Gain de Stabilité</span>
						<span class="spec-val text-success">{autotune.expected_tps_gain ?? 'Optimal'}</span>
					</div>
				</div>

				<div class="autotune-actions">
					<button class="btn btn-primary btn-sm" onclick={handleApplyAutotune} disabled={applyingAutotune}>
						<Wrench size={14} class={applyingAutotune ? 'spin' : ''} />
						<span>Appliquer les {autotune.property_tweaks?.length ?? 0} Optimisations Properties</span>
					</button>
				</div>
			</div>
		{/if}
	</div>

	<!-- Hardware Auto-Tuner Recommendations -->
	{#if autotune}
		<div class="card autotune-detail-card">
			<div class="card-header-row">
				<div class="flex items-center gap-2">
					<Sliders size={18} class="text-primary" />
					<h2 class="card-title">Optimisations des Propriétés Serveur (server.properties)</h2>
				</div>
			</div>

			<div class="table-wrapper">
				<table class="data-table">
					<thead>
						<tr>
							<th>Paramètre</th>
							<th>Valeur Actuelle</th>
							<th>Valeur Recommandée</th>
							<th>Justification & Impact</th>
						</tr>
					</thead>
					<tbody>
						{#each autotune.property_tweaks as tweak}
							<tr>
								<td class="font-mono text-sm font-semibold">{tweak.key}</td>
								<td class="font-mono text-sm text-muted">{tweak.current_value}</td>
								<td>
									<span class="badge badge-success font-mono">{tweak.recommended_value}</span>
								</td>
								<td class="text-sm">{tweak.rationale}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<div class="jvm-flags-box">
				<div class="jvm-header">
					<div class="flex items-center gap-2">
						<Terminal size={16} class="text-primary" />
						<span class="text-sm font-semibold">Drapeaux JVM Recommandés (Aikar / ZGC) :</span>
					</div>
					<button class="btn btn-secondary btn-sm" onclick={copyJvmFlags}>
						{#if copiedJvm}
							<Check size={14} class="text-success" />
							<span>Copié !</span>
						{:else}
							<Copy size={14} />
							<span>Copier les Drapeaux</span>
						{/if}
					</button>
				</div>
				<pre class="jvm-code font-mono">{autotune.recommended_jvm_flags}</pre>
			</div>
		</div>
	{/if}

	<!-- Crash Diagnostic Reports -->
	<div class="card crash-reports-card">
		<div class="card-header-row">
			<div class="flex items-center gap-2">
				<Flame size={18} class="text-danger" />
				<h2 class="card-title">Rapports d'Incidents & Diagnostic Automatique</h2>
			</div>
			<span class="badge badge-secondary">{crashes.length} rapport(s)</span>
		</div>

		{#if crashes.length === 0}
<EmptyState>
			{#snippet icon()}
				<ShieldCheck size={36} class="text-success" />
			{/snippet}
			{#snippet description()}
				Aucun rapport de crash enregistré dans le dossier <code>crash-reports/</code>.
			{/snippet}
		</EmptyState>
		{:else}
			<div class="crash-list">
				{#each crashes as crash}
					<div class="crash-item card">
						<div class="crash-item-header">
							<div class="flex items-center gap-3">
								<span class="badge {crash.severity === 'CRITICAL' ? 'badge-danger' : 'badge-warning'}">
									{crash.category}
								</span>
								<strong class="crash-title">{crash.title}</strong>
							</div>
							<span class="text-xs text-muted font-mono">{formatDate(crash.created_at_secs)}</span>
						</div>

						<p class="crash-root-cause text-sm">
							<strong>Cause racine identifiée :</strong> {crash.root_cause}
						</p>

						<div class="crash-recommendation text-sm alert alert-info flex items-center gap-2">
							<Info size={14} class="flex-shrink-0" />
							<span><strong>Conseil :</strong> {crash.recommendation}</span>
						</div>

						<div class="crash-footer">
							<div class="flex items-center gap-2">
								{#if crash.suspected_source}
									<span class="text-xs text-muted">
										Source suspectée : <code class="font-mono">{crash.suspected_source}</code>
									</span>
								{/if}
							</div>

							<div class="flex items-center gap-2">
								<button class="btn btn-secondary btn-sm" onclick={() => (selectedCrash = crash)}>
									<FileText size={14} />
									<span>Détails & Stack Trace</span>
								</button>

								{#if crash.remediation_action}
									<button
										class="btn btn-primary btn-sm"
										onclick={() => handleRemediateCrash(crash.id)}
										disabled={remediatingCrashId === crash.id}
									>
										<Wrench size={14} class={remediatingCrashId === crash.id ? 'spin' : ''} />
										<span>Auto-Remédiation</span>
									</button>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

<!-- Modal Détail Stack Trace -->
{#if selectedCrash}
	<div class="modal-backdrop" onclick={() => (selectedCrash = null)} onkeydown={(e) => e.stopPropagation()}  role="presentation">
		<div class="modal-card card" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()} role="dialog" tabindex="-1">
			<div class="modal-header">
				<h2 class="modal-title flex items-center gap-2 text-danger">
					<Flame size={18} />
					<span>Rapport de Crash #{selectedCrash.id}</span>
				</h2>
				<button class="btn btn-ghost btn-sm btn-icon" onclick={() => (selectedCrash = null)} aria-label="Fermer">
					<X size={16} />
				</button>
			</div>

			<div class="modal-body">
				<div class="meta-grid">
					<div><strong>Fichier :</strong> <span class="font-mono text-xs">{selectedCrash.filename}</span></div>
					<div><strong>Version MC :</strong> <span>{selectedCrash.minecraft_version || 'Inconnue'}</span></div>
					<div><strong>Catégorie :</strong> <span class="badge badge-warning">{selectedCrash.category}</span></div>
					<div><strong>Sévérité :</strong> <span class="badge badge-danger">{selectedCrash.severity}</span></div>
				</div>

				<div class="divider"></div>

				<h3 class="text-sm font-semibold mb-1">Extrait de la Stack Trace Java :</h3>
				<pre class="stack-box font-mono text-xs">{selectedCrash.stack_trace_snippet}</pre>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={() => (selectedCrash = null)}>Fermer</button>
			</div>
		</div>
	</div>
{/if}

<!-- Toasts -->
{#if toasts.length > 0}
	<div class="toast-stack">
		{#each toasts as toast (toast.id)}
			<div class="toast toast-{toast.type} card">
				{#if toast.type === 'success'}<CheckCircle2 size={16} class="text-success" />{/if}
				{#if toast.type === 'danger'}<AlertCircle size={16} class="text-danger" />{/if}
				<div>
					<strong>{toast.title}</strong>
					<p>{toast.message}</p>
				</div>
			</div>
		{/each}
	</div>
{/if}

<style>
	.diagnostics-page-layout {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
		padding-bottom: var(--space-8);
	}

	/* Health Grid */
	.health-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-4);
	}

	.health-card {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		padding: var(--space-5);
		background-color: var(--bg-surface);
	}

	.health-gauge {
		display: flex;
		align-items: center;
		gap: var(--space-4);
	}

	.score-circle {
		width: 64px;
		height: 64px;
		border-radius: 9999px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		border: 3px solid;
	}

	.score-good { border-color: #10B981; color: #10B981; background: rgba(16, 185, 129, 0.1); }
	.score-warn { border-color: #F59E0B; color: #F59E0B; background: rgba(245, 158, 11, 0.1); }
	.score-bad { border-color: #EF4444; color: #EF4444; background: rgba(239, 68, 68, 0.1); }

	.score-num {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		line-height: 1;
	}

	.score-unit {
		font-size: 10px;
		color: var(--text-muted);
	}

	.health-meta {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}

	.issue-box {
		padding: var(--space-3);
		border-radius: var(--radius-md);
		background-color: rgba(245, 158, 11, 0.08);
		border: 1px solid rgba(245, 158, 11, 0.2);
	}

	.good-box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		background-color: rgba(16, 185, 129, 0.08);
		border-color: rgba(16, 185, 129, 0.2);
	}

	.issue-list {
		margin: var(--space-2) 0 0 var(--space-4);
		padding: 0;
		font-size: var(--font-size-xs);
	}

	/* Autotune hero */
	.autotune-hero-card {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		padding: var(--space-5);
		background-color: var(--bg-surface);
	}

	.autotune-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.card-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		margin: 0;
	}

	.specs-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-3);
		margin: var(--space-3) 0;
	}

	.spec-item {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.spec-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.spec-val {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
	}

	/* Tables & JVM boxes */
	.autotune-detail-card,
	.crash-reports-card {
		padding: var(--space-5);
		background-color: var(--bg-surface);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.card-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
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
		padding: var(--space-3) var(--space-4);
		font-size: var(--font-size-xs);
		text-transform: uppercase;
		color: var(--text-muted);
		border-bottom: 1px solid var(--border);
	}

	.data-table td {
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
	}

	.jvm-flags-box {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		padding: var(--space-4);
		background-color: var(--bg-card);
		border-radius: var(--radius-md);
	}

	.jvm-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.jvm-code {
		font-size: var(--font-size-xs);
		color: var(--text-primary);
		white-space: pre-wrap;
		word-break: break-all;
		margin: 0;
		padding: var(--space-2) var(--space-3);
		background-color: rgba(0, 0, 0, 0.4);
		border-radius: var(--radius-sm);
	}

	/* Crash list */
	.crash-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.crash-item {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		padding: var(--space-4);
		background-color: var(--bg-card);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-md);
	}

	.crash-item-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.crash-title {
		font-size: var(--font-size-sm);
	}

	.crash-root-cause {
		margin: 0;
		color: var(--text-muted);
	}

	.crash-recommendation {
		margin: 0;
		padding: var(--space-2) var(--space-3);
	}

	.crash-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		border-top: 1px solid var(--border-subtle);
		padding-top: var(--space-3);
	}

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
		max-width: 680px;
		background-color: var(--bg-surface);
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
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
		gap: var(--space-3);
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		padding: var(--space-4) var(--space-6);
		border-top: 1px solid var(--border);
	}

	.meta-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
	}

	.stack-box {
		padding: var(--space-3);
		background-color: rgba(0, 0, 0, 0.5);
		border-radius: var(--radius-sm);
		overflow-x: auto;
		max-height: 280px;
		margin: 0;
	}

	.divider {
		height: 1px;
		background-color: var(--border);
		margin: var(--space-2) 0;
	}

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.toast-stack {
		position: fixed;
		bottom: var(--space-6);
		right: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		z-index: var(--z-toast);
	}

	.toast {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-surface);
		box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4);
		border: 1px solid var(--border);
	}

	.toast-success { border-color: var(--accent-green-border); }
	.toast-danger { border-color: var(--danger-border); }

	.toast p {
		margin: 2px 0 0 0;
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}
</style>
