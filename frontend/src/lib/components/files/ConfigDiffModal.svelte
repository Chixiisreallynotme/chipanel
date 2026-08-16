<script>
	import { apiFetch } from '$lib/api/client.js';
	import { FileCode, Plus, Minus, Check, X, AlertTriangle, Loader2 } from 'lucide-svelte';

	/**
	 * @type {{
	 *   open: boolean,
	 *   filePath: string,
	 *   newContent: string,
	 *   onConfirm: () => void,
	 *   onCancel: () => void
	 * }}
	 */
	let { open, filePath, newContent, onConfirm, onCancel } = $props();

	let loading = $state(false);
	let error = $state('');
	/** @type {{ has_changes: boolean, additions: number, deletions: number, unified_diff: string, lines: Array<{ tag: string, old_line: number|null, new_line: number|null, content: string }> } | null} */
	let diffData = $state(null);

	$effect(() => {
		if (open && filePath) {
			fetchDiff();
		} else {
			diffData = null;
			error = '';
		}
	});

	async function fetchDiff() {
		loading = true;
		error = '';
		try {
			const res = await apiFetch('/api/diff', {
				method: 'POST',
				body: { path: filePath, new_content: newContent }
			});

			if (!res.ok) {
				const errData = await res.json().catch(() => ({}));
				throw new Error(errData.message || 'Échec du calcul du diff');
			}

			diffData = await res.json();
		} catch (err) {
			error = err.message || 'Erreur de calcul du comparateur';
		} finally {
			loading = false;
		}
	}
</script>

{#if open}
	<div class="modal-backdrop" onclick={onCancel} role="presentation">
		<div class="modal-dialog" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="diff-modal-title">
			<div class="modal-header">
				<div class="header-left">
					<div class="icon-badge">
						<FileCode size={20} />
					</div>
					<div>
						<h2 id="diff-modal-title" class="modal-title">Comparateur de Modifications (Diff)</h2>
						<p class="modal-subtitle font-mono">{filePath}</p>
					</div>
				</div>

				<div class="header-right">
					{#if diffData}
						<div class="diff-badges">
							<span class="diff-badge badge-added">
								<Plus size={12} />
								<span>+{diffData.additions}</span>
							</span>
							<span class="diff-badge badge-deleted">
								<Minus size={12} />
								<span>-{diffData.deletions}</span>
							</span>
						</div>
					{/if}
					<button class="btn btn-ghost btn-sm btn-icon" onclick={onCancel} aria-label="Fermer">
						<X size={18} />
					</button>
				</div>
			</div>

			<div class="modal-body">
				{#if loading}
					<div class="loading-container">
						<Loader2 size={32} class="spinner" />
						<p>Calcul des différences en cours...</p>
					</div>
				{:else if error}
					<div class="error-container">
						<AlertTriangle size={24} class="text-danger" />
						<p>{error}</p>
					</div>
				{:else if diffData}
					{#if !diffData.has_changes}
						<div class="no-changes-container">
							<Check size={28} class="text-success" />
							<p>Aucune modification détectée par rapport à la version sur le disque.</p>
						</div>
					{:else}
						<div class="diff-container font-mono">
							{#each diffData.lines as line, idx}
								<div class="diff-line diff-{line.tag}">
									<span class="line-num line-num-old">{line.old_line ?? ''}</span>
									<span class="line-num line-num-new">{line.new_line ?? ''}</span>
									<span class="line-sign">
										{#if line.tag === 'insert'}+{/if}
										{#if line.tag === 'delete'}-{/if}
										{#if line.tag === 'equal'}&nbsp;{/if}
									</span>
									<span class="line-content">{line.content || ' '}</span>
								</div>
							{/each}
						</div>
					{/if}
				{/if}
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={onCancel}>
					Annuler
				</button>
				<button
					class="btn btn-primary"
					onclick={onConfirm}
					disabled={loading || (diffData && !diffData.has_changes)}
				>
					<Check size={16} />
					<span>Confirmer & Enregistrer</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-backdrop {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.75);
		backdrop-filter: blur(4px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: var(--z-modal, 100);
		padding: var(--space-4);
	}

	.modal-dialog {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: 0 20px 40px rgba(0, 0, 0, 0.6);
		width: 100%;
		max-width: 900px;
		max-height: 85vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border);
		background-color: var(--bg-card);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.icon-badge {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-sm);
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.modal-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		margin: 0;
	}

	.modal-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin: 2px 0 0 0;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.diff-badges {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.diff-badge {
		display: inline-flex;
		align-items: center;
		gap: 3px;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-bold);
		padding: 2px 8px;
		border-radius: 9999px;
	}

	.badge-added {
		background-color: rgba(16, 185, 129, 0.15);
		color: #34D399;
		border: 1px solid rgba(16, 185, 129, 0.3);
	}

	.badge-deleted {
		background-color: rgba(239, 68, 68, 0.15);
		color: #F87171;
		border: 1px solid rgba(239, 68, 68, 0.3);
	}

	.modal-body {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-4);
		background-color: #0d1117;
		min-height: 250px;
	}

	.loading-container,
	.error-container,
	.no-changes-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		padding: var(--space-12) 0;
		color: var(--text-muted);
		text-align: center;
	}

	.spinner {
		animation: spin 1s linear infinite;
		color: var(--accent-blue-text);
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.diff-container {
		font-size: 13px;
		line-height: 1.5;
	}

	.diff-line {
		display: flex;
		align-items: flex-start;
		padding: 1px 0;
		border-radius: 2px;
	}

	.diff-equal {
		color: #9CA3AF;
	}

	.diff-insert {
		background-color: rgba(16, 185, 129, 0.15);
		color: #A7F3D0;
	}

	.diff-delete {
		background-color: rgba(239, 68, 68, 0.15);
		color: #FECACA;
	}

	.line-num {
		width: 40px;
		text-align: right;
		padding-right: var(--space-2);
		color: #4B5563;
		user-select: none;
		flex-shrink: 0;
		font-size: 11px;
	}

	.line-sign {
		width: 18px;
		text-align: center;
		user-select: none;
		flex-shrink: 0;
		font-weight: bold;
	}

	.diff-insert .line-sign {
		color: #34D399;
	}

	.diff-delete .line-sign {
		color: #F87171;
	}

	.line-content {
		flex: 1;
		white-space: pre-wrap;
		word-break: break-all;
	}

	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-3);
		padding: var(--space-4) var(--space-6);
		border-top: 1px solid var(--border);
		background-color: var(--bg-card);
	}
</style>
