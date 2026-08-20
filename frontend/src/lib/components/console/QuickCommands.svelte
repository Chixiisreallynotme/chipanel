<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import {
		Sun,
		CloudSun,
		Save,
		Users,
		Activity,
		RotateCcw,
		Zap,
		CornerDownLeft,
		Check,
		AlertTriangle,
		X
	} from '$lib/icons.js';

	let { onFillCommand, onExecuteCommand } = $props();

	let confirmModalAction = $state(null);
	let feedbackNotice = $state(null);
	let noticeTimer = null;

	const QUICK_COMMANDS = [
		{
			id: 'weather_clear',
			label: 'Weather Clear',
			command: 'weather clear',
			icon: Sun,
			badge: 'World',
			badgeClass: 'badge-blue',
			requiresConfirmation: false
		},
		{
			id: 'time_day',
			label: 'Time Day',
			command: 'time set day',
			icon: CloudSun,
			badge: 'World',
			badgeClass: 'badge-blue',
			requiresConfirmation: false
		},
		{
			id: 'save_all',
			label: 'Save All',
			command: 'save-all',
			icon: Save,
			badge: 'Disk',
			badgeClass: 'badge-warning',
			requiresConfirmation: true,
			confirmMessage: 'Force save all world data and player states to disk storage now?'
		},
		{
			id: 'list_players',
			label: 'List Players',
			command: 'list',
			icon: Users,
			badge: 'Players',
			badgeClass: 'badge-success',
			requiresConfirmation: false
		},
		{
			id: 'tps_check',
			label: 'TPS Check',
			command: 'tps',
			icon: Activity,
			badge: 'Perf',
			badgeClass: 'badge-blue',
			requiresConfirmation: false
		},
		{
			id: 'reload_plugins',
			label: 'Reload Plugins',
			command: 'reload confirm',
			icon: RotateCcw,
			badge: 'System',
			badgeClass: 'badge-danger',
			requiresConfirmation: true,
			confirmMessage: 'Reloading server plugins may cause temporary lag or unexpected plugin state changes. Continue?'
		}
	];

	function handleChipClick(item) {
		if (item.requiresConfirmation) {
			confirmModalAction = item;
		} else {
			executeCommand(item);
		}
	}

	function confirmAndExecute() {
		if (confirmModalAction) {
			executeCommand(confirmModalAction);
			confirmModalAction = null;
		}
	}

	function executeCommand(item) {
		const success = wsStore.sendCommand(item.command);
		if (onExecuteCommand) {
			onExecuteCommand(item.command);
		}

		if (noticeTimer) clearTimeout(noticeTimer);
		feedbackNotice = {
			title: success ? `Executed /${item.command}` : `Failed to execute /${item.command}`,
			success
		};
		noticeTimer = setTimeout(() => {
			feedbackNotice = null;
		}, 3000);
	}

	function fillToInput(item, e) {
		e.stopPropagation();
		if (onFillCommand) {
			onFillCommand(item.command);
		}
	}

	function handleWindowKeydown(e) {
		if (e.key === 'Escape' && confirmModalAction) {
			confirmModalAction = null;
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="quick-commands-container">
	<div class="section-header">
		<div class="header-title">
			<Zap size={16} class="zap-icon" />
			<span>Quick Command Actions</span>
		</div>
		{#if feedbackNotice}
			<div class="feedback-badge badge {feedbackNotice.success ? 'badge-success' : 'badge-danger'}">
				<Check size={12} />
				<span>{feedbackNotice.title}</span>
			</div>
		{/if}
	</div>

	<!-- Quick Action Chips Grid -->
	<div class="chips-grid">
		{#each QUICK_COMMANDS as item, idx (item.id)}
			{@const IconComponent = item.icon}
			<div
				class="chip-card"
				role="button"
				onclick={() => handleChipClick(item)}
				tabindex="0"
				onkeydown={(e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						handleChipClick(item);
					}
				}}
			>
				<div class="chip-left">
					<div class="chip-icon-box">
						<IconComponent size={16} />
					</div>
					<div class="chip-text">
						<span class="chip-label">{item.label}</span>
						<span class="chip-cmd">/{item.command}</span>
					</div>
				</div>

				<div class="chip-right">
					<span class="badge {item.badgeClass} chip-badge">{item.badge}</span>
					<kbd class="chip-shortcut tabular-nums" title="Raccourci #{idx + 1}">#{idx + 1}</kbd>
					<button
						type="button"
						class="btn-fill-input"
						onclick={(e) => fillToInput(item, e)}
						title="Remplir dans le prompt"
						aria-label="Remplir dans le prompt"
					>
						<CornerDownLeft size={12} />
					</button>
				</div>
			</div>
		{/each}
	</div>
</div>

<!-- Confirmation Modal Dialog -->
{#if confirmModalAction}
	<div class="modal-backdrop" onclick={() => (confirmModalAction = null)} onkeydown={(e) => e.stopPropagation()}  aria-hidden="true">
		<div
			class="modal confirm-modal"
			onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			tabindex="-1"
			aria-modal="true"
			aria-labelledby="confirm-modal-title"
		>
			<div class="modal-header">
				<div class="modal-title-group">
					<AlertTriangle size={20} class="warning-icon" />
					<h3 class="modal-title" id="confirm-modal-title">Confirmer l'action rapide</h3>
				</div>
				<button
					class="btn btn-ghost btn-icon btn-sm"
					onclick={() => (confirmModalAction = null)}
					aria-label="Fermer la confirmation"
				>
					<X size={16} />
				</button>
			</div>

			<div class="modal-body">
				<p class="confirm-message">{confirmModalAction.confirmMessage}</p>
				<div class="cmd-preview-box">
					<span class="cmd-label">Commande cible :</span>
					<code class="cmd-code">/{confirmModalAction.command}</code>
				</div>
			</div>

			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={() => (confirmModalAction = null)}>
					Annuler
				</button>
				<button class="btn btn-danger" onclick={confirmAndExecute}>
					Confirmer & Exécuter
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.quick-commands-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.header-title {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.zap-icon {
		color: var(--warning);
	}

	.feedback-badge {
		font-size: var(--font-size-xs);
		animation: fadeIn var(--transition-fast) ease-out;
	}

	/* Chips Grid */
	.chips-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
		gap: var(--space-3);
	}

	.chip-card {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
		cursor: pointer;
		user-select: none;
		transition: transform 160ms var(--ease-out),
			background-color 150ms var(--ease-out),
			border-color 150ms var(--ease-out),
			box-shadow 150ms var(--ease-out);
	}

	.chip-card:hover {
		background-color: var(--bg-elevated);
		border-color: var(--border-focus);
		transform: translateY(-1px);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08), 0 4px 12px rgba(0, 0, 0, 0.3);
	}

	.chip-card:active {
		transform: scale(0.97);
	}

	.chip-left {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		min-width: 0;
	}

	.chip-icon-box {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-btn);
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
		flex-shrink: 0;
	}

	.chip-text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.chip-label {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.chip-cmd {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--text-muted);
	}

	.chip-right {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-shrink: 0;
	}

	.chip-badge {
		font-size: 10px;
		padding: 1px 7px;
	}

	.chip-shortcut {
		font-family: var(--font-mono);
		font-size: 10px;
		padding: 1px 5px;
		border-radius: var(--radius-sm);
		background-color: rgba(255, 255, 255, 0.06);
		border: 1px solid var(--border-subtle);
		color: var(--text-muted);
		font-variant-numeric: tabular-nums;
		user-select: none;
	}

	.btn-fill-input {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: var(--radius-sm);
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast), transform 160ms var(--ease-out);
	}

	.btn-fill-input:hover {
		background-color: rgba(255, 255, 255, 0.1);
		color: var(--text-primary);
	}

	.btn-fill-input:active {
		transform: scale(0.95);
	}

	/* Modal Confirmation Styling */
	.confirm-modal {
		max-width: 440px;
	}

	.modal-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.warning-icon {
		color: var(--warning);
	}

	.confirm-message {
		color: var(--text-primary);
		font-size: var(--font-size-base);
		margin-bottom: var(--space-4);
	}

	.cmd-preview-box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		font-size: var(--font-size-sm);
	}

	.cmd-label {
		color: var(--text-muted);
	}

	.cmd-code {
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-semibold);
		font-family: var(--font-mono);
	}
</style>
