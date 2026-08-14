<script>
	import { wsStore } from '$lib/stores/websocket.svelte.js';
	import { Send, Terminal, CornerDownLeft, Sparkles } from 'lucide-svelte';

	let { commandValue = $bindable(''), onCommandExecuted } = $props();

	let inputEl = $state(null);
	let showSuggestions = $state(false);
	let selectedSuggestionIndex = $state(-1);
	let isSubmitting = $state(false);

	// Command history navigation state
	let commandHistory = $state([]);
	let historyIndex = $state(-1);
	let draftInput = $state('');

	// Standard Minecraft commands auto-complete suggestions list
	const SUGGESTIONS = [
		{ command: 'op', description: 'Grant operator permissions to player' },
		{ command: 'deop', description: 'Revoke operator permissions from player' },
		{ command: 'tp', description: 'Teleport player to target coordinates or player' },
		{ command: 'gamemode creative', description: 'Set game mode to Creative' },
		{ command: 'gamemode survival', description: 'Set game mode to Survival' },
		{ command: 'weather clear', description: 'Set world weather to clear and sunny' },
		{ command: 'time set day', description: 'Set in-game time to morning (1000)' },
		{ command: 'difficulty hard', description: 'Set game difficulty to Hard' },
		{ command: 'stop', description: 'Gracefully save and stop the server' },
		{ command: 'save-all', description: 'Force save all server state to disk' },
		{ command: 'list', description: 'Show list of online players' },
		{ command: 'whitelist add', description: 'Add player to server whitelist' },
		{ command: 'ban', description: 'Ban player from server' },
		{ command: 'pardon', description: 'Unban player from server' }
	];

	// Filtered suggestion list based on user input
	let filteredSuggestions = $derived.by(() => {
		const val = commandValue.trim().toLowerCase();
		if (!val) return [];
		// Strip leading slash if present for matching
		const cleanVal = val.startsWith('/') ? val.slice(1) : val;

		return SUGGESTIONS.filter((s) => {
			const cmd = s.command.toLowerCase();
			return cmd.startsWith(cleanVal) || cmd.includes(cleanVal);
		});
	});

	function handleInput(e) {
		commandValue = e.target.value;
		historyIndex = -1; // Reset history position when typing
		showSuggestions = filteredSuggestions.length > 0;
		selectedSuggestionIndex = -1;
	}

	function handleKeyDown(e) {
		const hasSuggestions = showSuggestions && filteredSuggestions.length > 0;

		if (e.key === 'ArrowDown') {
			if (hasSuggestions && selectedSuggestionIndex < filteredSuggestions.length - 1) {
				e.preventDefault();
				selectedSuggestionIndex++;
			} else if (commandHistory.length > 0 && historyIndex !== -1) {
				e.preventDefault();
				if (historyIndex < commandHistory.length - 1) {
					historyIndex++;
					commandValue = commandHistory[historyIndex];
				} else {
					historyIndex = -1;
					commandValue = draftInput;
				}
				showSuggestions = false;
			}
		} else if (e.key === 'ArrowUp') {
			if (hasSuggestions && selectedSuggestionIndex > 0) {
				e.preventDefault();
				selectedSuggestionIndex--;
			} else if (commandHistory.length > 0) {
				e.preventDefault();
				if (historyIndex === -1) {
					draftInput = commandValue;
					historyIndex = commandHistory.length - 1;
				} else if (historyIndex > 0) {
					historyIndex--;
				}
				commandValue = commandHistory[historyIndex];
				showSuggestions = false;
			}
		} else if (e.key === 'Tab') {
			if (hasSuggestions) {
				e.preventDefault();
				const idx = selectedSuggestionIndex >= 0 ? selectedSuggestionIndex : 0;
				selectSuggestion(filteredSuggestions[idx]);
			}
		} else if (e.key === 'Enter') {
			if (hasSuggestions && selectedSuggestionIndex >= 0) {
				e.preventDefault();
				selectSuggestion(filteredSuggestions[selectedSuggestionIndex]);
			} else {
				e.preventDefault();
				submitCommand();
			}
		} else if (e.key === 'Escape') {
			showSuggestions = false;
			selectedSuggestionIndex = -1;
		}
	}

	function selectSuggestion(suggestion) {
		commandValue = suggestion.command;
		showSuggestions = false;
		selectedSuggestionIndex = -1;
		if (inputEl) inputEl.focus();
	}

	function submitCommand() {
		const cmd = commandValue.trim();
		if (!cmd || isSubmitting) return;

		isSubmitting = true;

		try {
			// Strip leading slash if present for standard rcon execution
			const finalCmd = cmd.startsWith('/') ? cmd.slice(1) : cmd;
			wsStore.sendCommand(finalCmd);

			// Add to command history if distinct from previous
			if (commandHistory[commandHistory.length - 1] !== cmd) {
				commandHistory = [...commandHistory, cmd].slice(-50);
			}

			if (onCommandExecuted) {
				onCommandExecuted(finalCmd);
			}

			commandValue = '';
			historyIndex = -1;
			draftInput = '';
			showSuggestions = false;
			selectedSuggestionIndex = -1;
		} catch (err) {
			console.error('Error submitting console command:', err);
		} finally {
			isSubmitting = false;
		}
	}

	function handleBlur() {
		// Delay closing suggestions to allow click events to process
		setTimeout(() => {
			showSuggestions = false;
		}, 180);
	}

	$effect(() => {
		if (showSuggestions && selectedSuggestionIndex >= 0) {
			const selectedEl = document.querySelector('.suggestions-dropdown .suggestion-item.selected');
			if (selectedEl) {
				selectedEl.scrollIntoView({ block: 'nearest' });
			}
		}
	});
</script>

<div class="command-bar-wrapper">
	<!-- Auto-complete Suggestions Dropdown Overlay -->
	{#if showSuggestions && filteredSuggestions.length > 0}
		<div class="suggestions-dropdown" id="suggestions-listbox" role="listbox">
			<div class="dropdown-header">
				<Sparkles size={12} class="sparkle-icon" />
				<span>Suggested Commands</span>
			</div>
			{#each filteredSuggestions as suggestion, idx (suggestion.command)}
				<button
					type="button"
					class="suggestion-item {idx === selectedSuggestionIndex ? 'selected' : ''}"
					onclick={() => selectSuggestion(suggestion)}
					role="option"
					aria-selected={idx === selectedSuggestionIndex}
				>
					<span class="suggestion-cmd">/{suggestion.command}</span>
					<span class="suggestion-desc">{suggestion.description}</span>
				</button>
			{/each}
		</div>
	{/if}

	<!-- Command Input Controls Form -->
	<form class="command-form" onsubmit={(e) => { e.preventDefault(); submitCommand(); }}>
		<div class="command-input-container">
			<span class="prompt-prefix">&gt;</span>
			<input
				bind:this={inputEl}
				type="text"
				class="input command-input"
				placeholder="Enter server command... (e.g. say Hello, op player, time set day)"
				value={commandValue}
				oninput={handleInput}
				onkeydown={handleKeyDown}
				onfocus={() => {
					if (filteredSuggestions.length > 0) showSuggestions = true;
				}}
				onblur={handleBlur}
				aria-label="Minecraft console command input"
				role="combobox"
				aria-expanded={showSuggestions && filteredSuggestions.length > 0}
				aria-autocomplete="list"
				aria-controls="suggestions-listbox"
				autocomplete="off"
				spellcheck="false"
			/>
		</div>

		<button
			type="submit"
			class="btn btn-primary send-button {isSubmitting ? 'btn-loading' : ''}"
			disabled={!commandValue.trim() || isSubmitting}
			title="Execute console command"
		>
			<Send size={16} />
			<span>Send</span>
		</button>
	</form>
</div>

<style>
	.command-bar-wrapper {
		position: relative;
		width: 100%;
	}

	.command-form {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.command-input-container {
		position: relative;
		flex: 1;
		display: flex;
		align-items: center;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		overflow: hidden;
		transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
	}

	.command-input-container:focus-within {
		border-color: var(--accent-blue);
		box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.15);
	}

	.prompt-prefix {
		padding: 0 0 0 var(--space-4);
		font-family: var(--font-mono);
		font-weight: var(--font-weight-bold);
		font-size: var(--font-size-md);
		color: var(--accent-blue-text);
		user-select: none;
	}

	.command-input {
		flex: 1;
		height: 44px;
		border: none;
		background: transparent;
		box-shadow: none;
		font-family: var(--font-mono);
		font-size: var(--font-size-base);
		color: var(--text-primary);
		padding: 0 var(--space-3);
	}

	.command-input:focus {
		outline: none;
		box-shadow: none;
		border: none;
	}

	.send-button {
		height: 44px;
		padding: 0 var(--space-6);
		font-weight: var(--font-weight-semibold);
		flex-shrink: 0;
	}

	/* Auto-complete Suggestions Dropdown */
	.suggestions-dropdown {
		position: absolute;
		bottom: calc(100% + 8px);
		left: 0;
		right: 0;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-focus);
		border-radius: var(--radius-card);
		box-shadow: var(--elevation-shadow);
		overflow: hidden;
		z-index: var(--z-dropdown);
		max-height: 260px;
		overflow-y: auto;
		animation: modalSlideUp var(--transition-fast) ease-out;
	}

	.dropdown-header {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-4);
		background-color: var(--bg-base);
		border-bottom: 1px solid var(--border-subtle);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.sparkle-icon {
		color: var(--accent-blue-text);
	}

	.suggestion-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-2) var(--space-4);
		border: none;
		background: transparent;
		color: var(--text-primary);
		text-align: left;
		cursor: pointer;
		font-family: var(--font-ui);
		transition: background-color var(--transition-fast);
	}

	.suggestion-item:hover,
	.suggestion-item.selected {
		background-color: rgba(99, 102, 241, 0.12);
	}

	.suggestion-cmd {
		font-family: var(--font-mono);
		font-weight: var(--font-weight-medium);
		color: var(--accent-blue-text);
		font-size: var(--font-size-sm);
	}

	.suggestion-desc {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}
</style>
