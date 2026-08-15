<script>
	import { CheckCircle2, X, Globe } from 'lucide-svelte';
	import { apiPost } from '$lib/api/client.js';

	let {
		isOpen = false,
		onClose = () => {},
		onDone = () => {},
		onToast = () => {}
	} = $props();

	let name = $state('');
	let seed = $state('');
	let levelType = $state('default');
	let gamemode = $state('survival');
	let difficulty = $state('normal');
	let submitting = $state(false);

	async function submit() {
		if (!name.trim() || submitting) return;
		submitting = true;
		try {
			/** @type {Record<string, string>} */
			const payload = {
				name: name.trim(),
				level_type: levelType,
				gamemode,
				difficulty
			};
			if (seed.trim()) payload.seed = seed.trim();
			const res = await apiPost('/api/worlds/create', payload);
			onToast('success', 'World Created', res.message || `World '${name.trim()}' created.`);
			if (res.warning) onToast('info', 'Warning', res.warning);
			if (!res.restarted) {
				onToast('info', 'Applied on next start', 'The server is off — the world will generate on next start.');
			}
			onDone();
		} catch (err) {
			onToast('error', 'Create Failed', err.message || 'Could not create the world.');
		} finally {
			submitting = false;
		}
	}
</script>

{#if isOpen}
	<div
		class="modal-backdrop"
		onclick={() => (submitting ? null : onClose())}
		role="dialog"
		aria-modal="true"
		aria-labelledby="create-world-title"
		tabindex="-1"
	>
		<div class="modal world-modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<div class="m-title">
					<Globe size={20} class="icon-blue" />
					<h3 id="create-world-title" class="modal-title">Create New World</h3>
				</div>
				<button
					type="button"
					class="btn btn-ghost btn-icon btn-sm"
					onclick={onClose}
					disabled={submitting}
					aria-label="Close"
				>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body world-modal-body">
				<div class="form-group">
					<label class="label label-required" for="cw-name">World name (folder)</label>
					<input
						id="cw-name"
						class="input font-mono"
						bind:value={name}
						placeholder="myworld"
					/>
					<span class="field-hint">Lowercase letters, numbers, `-` and `_` only.</span>
				</div>

				<div class="form-group">
					<label class="label" for="cw-seed">Seed (optional)</label>
					<input
						id="cw-seed"
						class="input font-mono"
						bind:value={seed}
						placeholder="Leave empty for a random seed"
					/>
				</div>

				<div class="form-grid-2">
					<div class="form-group">
						<label class="label" for="cw-type">Generator</label>
						<select id="cw-type" class="input" bind:value={levelType}>
							<option value="default">Default</option>
							<option value="flat">Superflat</option>
							<option value="largebiomes">Large Biomes</option>
							<option value="amplified">Amplified</option>
							<option value="caves">Caves</option>
						</select>
					</div>
					<div class="form-group">
						<label class="label" for="cw-gamemode">Gamemode</label>
						<select id="cw-gamemode" class="input" bind:value={gamemode}>
							<option value="survival">Survival</option>
							<option value="creative">Creative</option>
							<option value="adventure">Adventure</option>
							<option value="spectator">Spectator</option>
						</select>
					</div>
				</div>

				<div class="form-group">
					<label class="label" for="cw-difficulty">Difficulty</label>
					<select id="cw-difficulty" class="input" bind:value={difficulty}>
						<option value="peaceful">Peaceful</option>
						<option value="easy">Easy</option>
						<option value="normal">Normal</option>
						<option value="hard">Hard</option>
					</select>
				</div>
			</div>

			<div class="modal-footer">
				<button type="button" class="btn btn-secondary" onclick={onClose} disabled={submitting}>
					Cancel
				</button>
				<button
					type="button"
					class="btn btn-primary {submitting ? 'btn-loading' : ''}"
					onclick={submit}
					disabled={submitting || !name.trim()}
				>
					{#if !submitting}
						<CheckCircle2 size={16} />
					{/if}
					<span>Create World</span>
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.world-modal {
		max-width: 520px;
		width: 90vw;
	}

	.world-modal-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.m-title {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.icon-blue {
		color: var(--accent-blue-text);
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

	@media (max-width: 640px) {
		.form-grid-2 {
			grid-template-columns: 1fr;
		}
	}
</style>
