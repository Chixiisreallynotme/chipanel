<script>
	import { onMount } from 'svelte';
	import { preferences } from '$lib/stores/preferences.svelte.js';
	import { Sparkles, Terminal } from 'lucide-svelte';

	let { compact = false } = $props();

	function handleKeydown(event) {
		// Alt+M / Option+M toggles mode instantly (0ms)
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

<div
	class="mode-switch-wrapper"
	class:compact
	role="radiogroup"
	aria-label="Mode d'utilisation de ChiPanel"
>
	<button
		type="button"
		class="mode-btn mode-novice"
		class:active={preferences.mode === 'novice'}
		onclick={() => preferences.setMode('novice')}
		title="Mode Novice (1-Click / Guidé) — Raccourci : Alt+M"
		role="radio"
		aria-checked={preferences.mode === 'novice'}
		aria-label="Mode Novice 1-Click (Raccourci : Alt+M)"
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
		role="radio"
		aria-checked={preferences.mode === 'expert'}
		aria-label="Mode Power User Expert (Raccourci : Alt+M)"
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
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4);
		flex-shrink: 0;
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
		transition: transform var(--duration-fast) var(--ease-hypr-snap),
					background-color var(--duration-fast) var(--ease-out),
					border-color var(--duration-fast) var(--ease-out),
					color var(--duration-fast) var(--ease-out),
					box-shadow var(--duration-fast) var(--ease-out);
	}

	.compact .mode-btn {
		height: 26px;
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
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.08);
	}

	.mode-novice.active {
		border-color: var(--accent-blue-border);
	}

	.mode-novice.active :global(.mode-icon) {
		color: var(--accent-blue-text);
	}

	.mode-expert.active {
		border-color: var(--accent-orange-border);
	}

	.mode-expert.active :global(.mode-icon) {
		color: var(--accent-orange-text);
	}

	.mode-label {
		line-height: 1;
	}

	/* Responsive: Icon-only on mobile < 640px to eliminate horizontal header overflow */
	@media (max-width: 640px) {
		.mode-label {
			display: none;
		}
		.mode-btn {
			width: 32px;
			height: 32px;
			padding: 0;
			justify-content: center;
		}
	}
</style>
