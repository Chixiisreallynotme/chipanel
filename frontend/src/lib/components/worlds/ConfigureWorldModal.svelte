<script>
	import { CheckCircle2, X, Settings, Sliders } from 'lucide-svelte';
	import { apiPost } from '$lib/api/client.js';

	let {
		isOpen = false,
		world = null,
		onClose = () => {},
		onToast = () => {}
	} = $props();

	// server.properties section
	let gamemode = $state('survival');
	let difficulty = $state('normal');
	let pvp = $state(true);
	let spawnProtection = $state('16');
	let maxPlayers = $state('20');
	let viewDistance = $state('10');
	let levelSeed = $state('');
	let levelType = $state('default');
	let savingProps = $state(false);

	// gamerules section
	let keepInventory = $state(false);
	let doFireTick = $state(true);
	let mobGriefing = $state(true);
	let doDaylightCycle = $state(true);
	let doWeatherCycle = $state(true);
	let randomTickSpeed = $state('3');
	let spawnX = $state('0');
	let spawnY = $state('64');
	let spawnZ = $state('0');
	let savingRules = $state(false);

	async function saveProperties() {
		if (savingProps) return;
		savingProps = true;
		try {
			/** @type {Record<string, string>} */
			const properties = {
				gamemode,
				difficulty,
				pvp: pvp ? 'true' : 'false',
				'spawn-protection': spawnProtection,
				'max-players': maxPlayers,
				'view-distance': viewDistance,
				'level-type': levelType
			};
			if (levelSeed.trim()) properties['level-seed'] = levelSeed.trim();

			const res = await apiPost('/api/worlds/configure', { properties });
			onToast('success', 'World Configured', res.message || 'Settings updated.');
			if (!res.restarted) {
				onToast('info', 'Applied on next start', 'The server is off — changes apply on next start.');
			}
		} catch (err) {
			onToast('error', 'Configure Failed', err.message || 'Could not update the world settings.');
		} finally {
			savingProps = false;
		}
	}

	async function saveGamerules() {
		if (savingRules) return;
		savingRules = true;
		try {
			const gamerules = [
				{ rule: 'keepInventory', value: keepInventory },
				{ rule: 'doFireTick', value: doFireTick },
				{ rule: 'mobGriefing', value: mobGriefing },
				{ rule: 'doDaylightCycle', value: doDaylightCycle },
				{ rule: 'doWeatherCycle', value: doWeatherCycle },
				{ rule: 'randomTickSpeed', value: parseInt(randomTickSpeed, 10) || 3 }
			];
			const payload = {
				gamerules,
				set_spawn: { x: parseInt(spawnX, 10) || 0, y: parseInt(spawnY, 10) || 64, z: parseInt(spawnZ, 10) || 0 }
			};
			const res = await apiPost('/api/worlds/gamerules', payload);
			onToast('success', 'Gamerules Applied', res.message || 'Gamerules and spawn updated.');
		} catch (err) {
			onToast('error', 'Gamerules Failed', err.message || 'Could not apply gamerules.');
		} finally {
			savingRules = false;
		}
	}
</script>

{#if isOpen && world}
	<div
		class="modal-backdrop"
		onclick={() => (savingProps || savingRules ? null : onClose())}
		role="dialog"
		aria-modal="true"
		aria-labelledby="configure-world-title"
		tabindex="-1"
	>
		<div class="modal world-modal" onclick={(e) => e.stopPropagation()}>
			<div class="modal-header">
				<div class="m-title">
					<Settings size={20} class="icon-blue" />
					<h3 id="configure-world-title" class="modal-title">
						Configure: <span class="font-mono text-blue">{world.level_name}</span>
					</h3>
				</div>
				<button
					type="button"
					class="btn btn-ghost btn-icon btn-sm"
					onclick={onClose}
					disabled={savingProps || savingRules}
					aria-label="Close"
				>
					<X size={18} />
				</button>
			</div>

			<div class="modal-body world-modal-body">
				<h4 class="section-label"><Sliders size={14} /> Server settings (server.properties)</h4>
				<div class="form-grid-2">
					<div class="form-group">
						<label class="label" for="cf-gamemode">Gamemode</label>
						<select id="cf-gamemode" class="input" bind:value={gamemode}>
							<option value="survival">Survival</option>
							<option value="creative">Creative</option>
							<option value="adventure">Adventure</option>
							<option value="spectator">Spectator</option>
						</select>
					</div>
					<div class="form-group">
						<label class="label" for="cf-difficulty">Difficulty</label>
						<select id="cf-difficulty" class="input" bind:value={difficulty}>
							<option value="peaceful">Peaceful</option>
							<option value="easy">Easy</option>
							<option value="normal">Normal</option>
							<option value="hard">Hard</option>
						</select>
					</div>
				</div>
				<div class="form-grid-2">
					<div class="form-group">
						<label class="label" for="cf-maxplayers">Max players</label>
						<input id="cf-maxplayers" type="number" class="input font-mono" bind:value={maxPlayers} />
					</div>
					<div class="form-group">
						<label class="label" for="cf-viewdistance">View distance</label>
						<input id="cf-viewdistance" type="number" class="input font-mono" bind:value={viewDistance} />
					</div>
				</div>
				<div class="form-grid-2">
					<div class="form-group">
						<label class="label" for="cf-spawnprot">Spawn protection</label>
						<input id="cf-spawnprot" type="number" class="input font-mono" bind:value={spawnProtection} />
					</div>
					<div class="form-group">
						<label class="label" for="cf-leveltype">Generator</label>
						<select id="cf-leveltype" class="input" bind:value={levelType}>
							<option value="default">Default</option>
							<option value="flat">Superflat</option>
							<option value="largebiomes">Large Biomes</option>
							<option value="amplified">Amplified</option>
							<option value="caves">Caves</option>
						</select>
					</div>
				</div>
				<div class="form-grid-2">
					<div class="form-group">
						<label class="label" for="cf-seed">Seed</label>
						<input id="cf-seed" class="input font-mono" bind:value={levelSeed} placeholder="(keep current)" />
					</div>
					<label class="checkbox-row">
						<input type="checkbox" bind:checked={pvp} />
						<span>Enable PvP</span>
					</label>
				</div>

				<div class="modal-footer">
					<button
						type="button"
						class="btn btn-primary {savingProps ? 'btn-loading' : ''}"
						onclick={saveProperties}
						disabled={savingProps}
					>
						{#if !savingProps}
							<CheckCircle2 size={16} />
						{/if}
						<span>Save Settings</span>
					</button>
				</div>

				<hr class="section-divider" />

				<h4 class="section-label"><Sliders size={14} /> Gamerules & spawn (live, via RCON)</h4>
				<div class="gamerules-grid">
					<label class="checkbox-row">
						<input type="checkbox" bind:checked={keepInventory} />
						<span>keepInventory</span>
					</label>
					<label class="checkbox-row">
						<input type="checkbox" bind:checked={doFireTick} />
						<span>doFireTick</span>
					</label>
					<label class="checkbox-row">
						<input type="checkbox" bind:checked={mobGriefing} />
						<span>mobGriefing</span>
					</label>
					<label class="checkbox-row">
						<input type="checkbox" bind:checked={doDaylightCycle} />
						<span>doDaylightCycle</span>
					</label>
					<label class="checkbox-row">
						<input type="checkbox" bind:checked={doWeatherCycle} />
						<span>doWeatherCycle</span>
					</label>
					<div class="form-group">
						<label class="label" for="cf-tickspeed">randomTickSpeed</label>
						<input id="cf-tickspeed" type="number" class="input font-mono" bind:value={randomTickSpeed} />
					</div>
				</div>

				<div class="form-grid-3">
					<div class="form-group">
						<label class="label" for="cf-spawnx">Spawn X</label>
						<input id="cf-spawnx" type="number" class="input font-mono" bind:value={spawnX} />
					</div>
					<div class="form-group">
						<label class="label" for="cf-spawny">Spawn Y</label>
						<input id="cf-spawny" type="number" class="input font-mono" bind:value={spawnY} />
					</div>
					<div class="form-group">
						<label class="label" for="cf-spawnz">Spawn Z</label>
						<input id="cf-spawnz" type="number" class="input font-mono" bind:value={spawnZ} />
					</div>
				</div>

				<div class="modal-footer">
					<button
						type="button"
						class="btn btn-primary {savingRules ? 'btn-loading' : ''}"
						onclick={saveGamerules}
						disabled={savingRules}
					>
						{#if !savingRules}
							<CheckCircle2 size={16} />
						{/if}
						<span>Apply Gamerules</span>
					</button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.world-modal {
		max-width: 640px;
		width: 92vw;
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

	.text-blue {
		color: var(--accent-blue-text);
	}

	.section-label {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.form-grid-2 {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-4);
	}

	.form-grid-3 {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-4);
	}

	.gamerules-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-3);
	}

	.checkbox-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		cursor: pointer;
	}

	.section-divider {
		border: none;
		border-top: 1px solid var(--border-subtle);
		margin: var(--space-2) 0;
	}

	@media (max-width: 640px) {
		.form-grid-2,
		.form-grid-3,
		.gamerules-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
