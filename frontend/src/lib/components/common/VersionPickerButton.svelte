<script>
	import { Layers, ChevronDown, Sparkles } from '$lib/icons.js';
	import VersionPickerModal from './VersionPickerModal.svelte';

	let {
		selectedVersion = 'all',
		releaseVersions = [],
		snapshotVersions = [],
		recommendedVersions = [],
		minVersion = null,
		maxVersion = null,
		allowAll = true,
		label = '',
		disabled = false,
		onSelect = () => {}
	} = $props();

	let modalOpen = $state(false);

	function handleVersionChosen(newVer) {
		onSelect(newVer);
	}
</script>

<div class="version-picker-wrapper">
	{#if label}
		<span class="picker-label">{label}</span>
	{/if}

	<button
		type="button"
		class="version-picker-trigger-btn {modalOpen ? 'active' : ''}"
		{disabled}
		onclick={() => (modalOpen = true)}
		aria-haspopup="dialog"
		aria-expanded={modalOpen}
	>
		<div class="trigger-left">
			<div class="trigger-icon-box">
				<Layers size={15} />
			</div>
			<div class="trigger-info">
				<span class="trigger-version-display">
					{selectedVersion === 'all' || !selectedVersion ? 'Toutes les versions' : selectedVersion}
				</span>
			</div>
		</div>

		<div class="trigger-right">
			{#if recommendedVersions.includes(selectedVersion)}
				<span class="trigger-tag tag-recommended" title="Version recommandée">
					<Sparkles size={10} />
					<span>Top</span>
				</span>
			{/if}
			<ChevronDown size={16} class="chevron-icon {modalOpen ? 'rotated' : ''}" />
		</div>
	</button>

	<!-- Modal -->
	<VersionPickerModal
		open={modalOpen}
		{selectedVersion}
		{releaseVersions}
		{snapshotVersions}
		{recommendedVersions}
		{minVersion}
		{maxVersion}
		{allowAll}
		onSelect={handleVersionChosen}
		onClose={() => (modalOpen = false)}
	/>
</div>

<style>
	.version-picker-wrapper {
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.picker-label {
		font-size: 0.8125rem;
		font-weight: 500;
		color: #94a3b8;
	}

	.version-picker-trigger-btn {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
		padding: 0.5rem 0.875rem;
		background: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		color: var(--text-primary);
		font-size: var(--font-size-sm);
		font-weight: 600;
		cursor: pointer;
		transition: background-color var(--transition-fast),
					border-color var(--transition-fast),
					box-shadow var(--transition-fast),
					transform var(--transition-fast);
		min-width: 180px;
	}

	.version-picker-trigger-btn:hover:not(:disabled) {
		background: var(--bg-elevated);
		border-color: var(--border-focus);
	}

	.version-picker-trigger-btn:active:not(:disabled) {
		transform: scale(0.97);
	}

	.version-picker-trigger-btn.active {
		border-color: var(--accent-blue-border);
		background-color: var(--accent-blue-bg);
	}

	.version-picker-trigger-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.trigger-left {
		display: flex;
		align-items: center;
		gap: 0.625rem;
	}

	.trigger-icon-box {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: 6px;
		background: rgba(99, 102, 241, 0.2);
		color: #818cf8;
	}

	.trigger-info {
		display: flex;
		flex-direction: column;
	}

	.trigger-version-display {
		font-family: monospace;
		font-size: 0.875rem;
		color: #ffffff;
		letter-spacing: 0.02em;
	}

	.trigger-right {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.trigger-tag {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		padding: 0.125rem 0.375rem;
		font-size: 0.6875rem;
		font-weight: 700;
		border-radius: 4px;
		text-transform: uppercase;
	}

	.tag-recommended {
		background: rgba(16, 185, 129, 0.2);
		color: #34d399;
		border: 1px solid rgba(16, 185, 129, 0.3);
	}

	:global(.chevron-icon) {
		color: #64748b;
		transition: transform 0.2s ease;
	}

	:global(.chevron-icon.rotated) {
		transform: rotate(180deg);
		color: #ffffff;
	}
</style>
