<script>
	import { onMount } from 'svelte';
	import { preferences } from '$lib/stores/preferences.svelte.js';
	import { Sparkles, Terminal, Sliders } from 'lucide-svelte';

	let { compact = false } = $props();

	function handleKeydown(event) {
		// Alt+M toggles mode instantly (0ms)
		if (event.altKey && (event.key === 'm' || event.key === 'M' || event.code === 'KeyM')) {
			event.preventDefault();
			preferences.toggleMode();
		}
	}

	onMount(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<div class="mode-switch-wrapper" class:compact>
	<button
		type="button"
		class="mode-btn mode-novice"
		class:active={preferences.mode === 'novice'}
		onclick={() => preferences.setMode('novice')}
		title="Mode Novice (1-Click / Guidé) — Raccourci : Alt+M"
		aria-pressed={preferences.mode === 'novice'}
	>
		<Sparkles size={14} class="mode-icon" />
		<span class="mode-label">Novice 1-Click</span>
	</button>

	<button
		type="button"
		class="mode-btn mode-expert"
		class:active={preferences.mode === 'expert'}
		onclick={() => preferences.setMode('expert')}
		title="Mode Power User (Expert / Quadlet / RCON) — Raccourci : Alt+M"
		aria-pressed={preferences.mode === 'expert'}
	>
		<Terminal size={14} class="mode-icon" />
		<span class="mode-label">Power User</span>
	</button>
</div>

<style>
	.mode-switch-wrapper {
		display: inline-flex;
		align-items: center;
		padding: 3px;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		gap: 2px;
		position: relative;
		user-select: none;
	}

	.mode-switch-wrapper.compact {
		padding: 2px;
	}

	.mode-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 4px 10px;
		height: 28px;
		border-radius: var(--radius-input);
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-secondary);
		font-family: var(--font-ui);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		white-space: nowrap;
		transition: transform 160ms var(--ease-out),
					background-color 150ms var(--ease-out),
					border-color 150ms var(--ease-out),
					color 150ms var(--ease-out);
	}

	.compact .mode-btn {
		height: 24px;
		padding: 2px 8px;
		font-size: 11px;
	}

	.mode-btn:hover:not(.active) {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
	}

	.mode-btn:active {
		transform: scale(0.97);
	}

	.mode-btn.active {
		background-color: var(--bg-elevated);
		color: var(--text-primary);
		border-color: var(--border-focus);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.06);
	}

	.mode-novice.active :global(.mode-icon) {
		color: var(--accent-blue-text);
	}

	.mode-expert.active :global(.mode-icon) {
		color: var(--accent-orange-text);
	}

	.mode-label {
		line-height: 1;
	}
</style>
