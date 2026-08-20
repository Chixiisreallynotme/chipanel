<script>
	import { apiGet, apiPost } from '$lib/api/client.js';
	import EmptyState from '$lib/components/ui/EmptyState.svelte';
	import {
		Zap,
		Flame,
		Shield,
		Heart,
		Eye,
		EyeOff,
		Swords,
		ShieldCheck,
		Sparkles,
		Trash2,
		X,
		PlusCircle,
		Wand2,
		RefreshCw,
		AlertCircle,
		CheckCircle2,
		Loader2,
		Skull,
		Activity,
		Clock
	} from '$lib/icons.js';

	/**
	 * @typedef {Object} StatusEffectInfo
	 * @property {string} id
	 * @property {string} name
	 * @property {number} amplifier
	 * @property {number} duration_ticks
	 * @property {string} duration_formatted
	 * @property {string} icon_name
	 * @property {string} category
	 */

	let { uuid = '', isOnline = true } = $props();

	// Static dictionary mapping Minecraft effect IDs to Lucide Icon components
	const EFFECT_ICONS = {
		speed: Zap,
		haste: Zap,
		strength: Swords,
		regeneration: Heart,
		instant_health: Heart,
		health_boost: Heart,
		resistance: Shield,
		fire_resistance: Flame,
		invisibility: EyeOff,
		night_vision: Eye,
		absorption: ShieldCheck,
		poison: Skull,
		wither: Skull,
		instant_damage: Skull,
		slowness: Activity,
		mining_fatigue: Activity,
		weakness: Activity,
		nausea: Activity,
		blindness: Activity
	};

	// State
	let effects = $state([]);
	let loading = $state(true);
	let error = $state('');
	let actionMessage = $state('');
	let clearing = $state(null); // null | 'all' | effect_id
	let applying = $state(false);

	// Form inputs for Apply Status Effect
	let selectedEffect = $state('speed');
	let durationSeconds = $state(300);
	let amplifierLevel = $state(1); // 1-255 (UI 1-indexed: 1 => amp 0)

	const minecraftEffectsOptions = [
		{ id: 'speed', label: 'Speed' },
		{ id: 'haste', label: 'Haste' },
		{ id: 'strength', label: 'Strength' },
		{ id: 'regeneration', label: 'Regeneration' },
		{ id: 'resistance', label: 'Resistance' },
		{ id: 'fire_resistance', label: 'Fire Resistance' },
		{ id: 'invisibility', label: 'Invisibility' },
		{ id: 'night_vision', label: 'Night Vision' },
		{ id: 'absorption', label: 'Absorption' }
	];

	// Fetch effects when UUID changes or component mounts
	$effect(() => {
		if (uuid) {
			fetchEffects();
		}
	});

	async function fetchEffects() {
		if (!uuid) return;
		loading = true;
		error = '';
		try {
			const res = await apiGet(`/api/players/${uuid}/effects`);
			effects = Array.isArray(res) ? res : [];
		} catch (err) {
			console.error(`Failed to fetch status effects for player ${uuid}:`, err);
			error = err.message || 'Failed to load status effects';
		} finally {
			loading = false;
		}
	}

	async function handleClearEffect(effectId = null) {
		if (!uuid || clearing) return;
		clearing = effectId || 'all';
		error = '';
		actionMessage = '';
		try {
			const res = await apiPost(`/api/players/${uuid}/effects/clear`, {
				effect_id: effectId
			});
			actionMessage = res.message || (effectId ? 'Status effect cleared successfully.' : 'All status effects cleared.');
			await fetchEffects();
		} catch (err) {
			console.error(`Failed to clear effect:`, err);
			error = err.message || 'Failed to clear status effect';
		} finally {
			clearing = null;
		}
	}

	async function handleApplyEffect(e) {
		if (e) e.preventDefault();
		if (!uuid || applying) return;
		applying = true;
		error = '';
		actionMessage = '';

		const amp = Math.max(0, Math.min(255, Number(amplifierLevel) - 1));
		const dur = Math.max(1, Number(durationSeconds) || 60);

		try {
			const res = await apiPost(`/api/players/${uuid}/effects/apply`, {
				effect_id: selectedEffect,
				duration_seconds: dur,
				amplifier: amp
			});
			actionMessage = res.message || `Status effect '${selectedEffect}' applied successfully.`;
			await fetchEffects();
		} catch (err) {
			console.error(`Failed to apply effect:`, err);
			error = err.message || 'Failed to apply status effect';
		} finally {
			applying = false;
		}
	}

	function getRomanLevel(amplifier) {
		const level = (amplifier ?? 0) + 1;
		const numerals = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X'];
		if (level <= 10) return numerals[level - 1];
		return `${level}`;
	}

	function getEffectTitle(name, amplifier) {
		const roman = getRomanLevel(amplifier);
		return `${name} ${roman}`;
	}

	function getCategoryBadgeClass(category) {
		const cat = (category || '').toLowerCase();
		if (cat === 'beneficial') return 'badge-success';
		if (cat === 'harmful') return 'badge-danger';
		return 'badge-blue';
	}

	function getCategoryLabel(category) {
		const cat = (category || '').toLowerCase();
		if (cat === 'beneficial') return 'Beneficial';
		if (cat === 'harmful') return 'Harmful';
		return 'Neutral';
	}

	function formatDurationText(formatted, ticks) {
		if (!formatted || formatted === 'Infinite' || formatted === 'Permanent' || ticks >= 1000000000) {
			return 'Permanent';
		}
		return formatted;
	}

	function getEffectIconComponent(id) {
		const clean = (id || '').replace('minecraft:', '').toLowerCase();
		return EFFECT_ICONS[clean] || Sparkles;
	}
</script>

<div class="effects-panel-container">
	<!-- Top Alerts -->
	{#if error}
		<div class="alert alert-danger" role="alert">
			<AlertCircle size={18} />
			<span>{error}</span>
		</div>
	{/if}

	{#if actionMessage}
		<div class="alert alert-success" role="alert">
			<CheckCircle2 size={18} />
			<span>{actionMessage}</span>
		</div>
	{/if}

	<!-- Header with Actions -->
	<div class="panel-header-bar">
		<div class="header-left">
			<h3 class="panel-title">Active Status Effects</h3>
			<span class="effects-count-pill">{effects.length} {effects.length === 1 ? 'effect' : 'effects'}</span>
		</div>

		<div class="header-actions">
			<button
				class="btn btn-ghost btn-sm"
				onclick={fetchEffects}
				disabled={loading}
				title="Refresh status effects"
			>
				<RefreshCw size={14} class={loading ? 'spinner' : ''} />
				<span>Refresh</span>
			</button>

			<button
				class="btn btn-danger btn-sm {clearing === 'all' ? 'btn-loading' : ''}"
				onclick={() => handleClearEffect(null)}
				disabled={effects.length === 0 || clearing !== null}
			>
				{#if clearing !== 'all'}
					<Trash2 size={14} />
				{/if}
				<span>Clear All Effects</span>
			</button>
		</div>
	</div>

	<!-- Status Effects Grid -->
	{#if loading && effects.length === 0}
		<div class="loading-state">
			<Loader2 size={28} class="spinner" />
			<span>Loading active player status effects...</span>
		</div>
	{:else if effects.length === 0}
		<EmptyState dashed title="No Active Status Effects" description="This player currently has no active status potion effects.">
			{#snippet icon()}
				<Sparkles size={36} class="icon-muted" />
			{/snippet}
		</EmptyState>
	{:else}
		<div class="effects-grid">
			{#each effects as effect (effect.id)}
				{@const IconComp = getEffectIconComponent(effect.id)}
				<div class="effect-card category-{effect.category}">
					<div class="effect-card-header">
						<div class="effect-title-box">
							<div class="effect-icon-wrapper category-{effect.category}">
								<IconComp size={18} />
							</div>
							<span class="effect-title">{getEffectTitle(effect.name, effect.amplifier)}</span>
						</div>
						<span class="badge {getCategoryBadgeClass(effect.category)}">
							{getCategoryLabel(effect.category)}
						</span>
					</div>

					<div class="effect-card-body">
						<div class="timer-display">
							<Clock size={14} class="timer-icon" />
							<span class="timer-value">{formatDurationText(effect.duration_formatted, effect.duration_ticks)}</span>
						</div>

						<button
							class="btn btn-ghost btn-sm clear-single-btn {clearing === effect.id ? 'btn-loading' : ''}"
							onclick={() => handleClearEffect(effect.id)}
							disabled={clearing !== null}
							title="Clear this effect"
							aria-label="Clear {effect.name} effect"
						>
							{#if clearing !== effect.id}
								<X size={14} />
							{/if}
							<span>Clear</span>
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	<!-- Apply Status Effect Section -->
	<div class="card apply-effect-card">
		<div class="card-header">
			<div class="apply-card-title">
				<Wand2 size={18} class="icon-blue" />
				<div>
					<h4 class="card-title">Apply Status Effect</h4>
					<p class="card-subtitle">Grant temporary or custom status effect to the player.</p>
				</div>
			</div>
		</div>

		<form class="card-body apply-effect-form" onsubmit={handleApplyEffect}>
			<div class="form-grid">
				<!-- Minecraft Effect Selector -->
				<div class="form-group">
					<label for="effect-select" class="label label-required">Minecraft Effect</label>
					<select id="effect-select" class="select" bind:value={selectedEffect}>
						{#each minecraftEffectsOptions as option}
							<option value={option.id}>{option.label}</option>
						{/each}
					</select>
				</div>

				<!-- Duration Input (Seconds) -->
				<div class="form-group">
					<label for="effect-duration" class="label label-required">Duration (Seconds)</label>
					<input
						id="effect-duration"
						type="number"
						class="input"
						min="1"
						max="1000000"
						placeholder="300"
						bind:value={durationSeconds}
						required
					/>
				</div>

				<!-- Amplifier Level Input (1-255) -->
				<div class="form-group">
					<label for="effect-amplifier" class="label label-required">Amplifier Level (1 - 255)</label>
					<input
						id="effect-amplifier"
						type="number"
						class="input"
						min="1"
						max="255"
						placeholder="1"
						bind:value={amplifierLevel}
						required
					/>
				</div>
			</div>

			<div class="apply-action-bar">
				<button
					type="submit"
					class="btn btn-primary {applying ? 'btn-loading' : ''}"
					disabled={applying}
				>
					{#if !applying}
						<PlusCircle size={16} />
					{/if}
					<span>Apply Effect</span>
				</button>
			</div>
		</form>
	</div>
</div>

<style>
	.effects-panel-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.alert {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-input);
		font-size: var(--font-size-sm);
	}

	.alert-danger {
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
	}

	.alert-success {
		background-color: var(--accent-green-bg);
		border: 1px solid var(--accent-green-border);
		color: var(--accent-green);
	}

	.panel-header-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.panel-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.effects-count-pill {
		font-size: var(--font-size-xs);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		color: var(--text-muted);
		padding: 2px 8px;
		border-radius: var(--radius-badge);
		font-family: var(--font-mono);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-8);
		background-color: var(--bg-base);
		border: 1px dashed var(--border-focus);
		border-radius: var(--radius-card);
		color: var(--text-muted);
		text-align: center;
	}

	.effects-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: var(--space-3);
	}

	.effect-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		transition: border-color var(--transition-fast);
	}

	.effect-card:hover {
		border-color: var(--border-focus);
	}

	.effect-card-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
	}

	.effect-title-box {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		overflow: hidden;
	}

	.effect-icon-wrapper {
		width: 32px;
		height: 32px;
		border-radius: var(--radius-input);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.effect-icon-wrapper.category-beneficial {
		background-color: var(--accent-green-bg);
		color: var(--accent-green);
	}

	.effect-icon-wrapper.category-harmful {
		background-color: var(--danger-bg);
		color: var(--danger-text);
	}

	.effect-icon-wrapper.category-neutral {
		background-color: var(--accent-blue-bg);
		color: var(--accent-blue-text);
	}

	.effect-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.effect-card-body {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-2);
		padding-top: var(--space-2);
		border-top: 1px solid var(--border-subtle);
	}

	.timer-display {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		color: var(--text-secondary);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
	}

	.timer-icon {
		color: var(--text-muted);
	}

	.clear-single-btn {
		color: var(--danger-text);
		padding: 0 var(--space-2);
		height: 28px;
		font-size: var(--font-size-xs);
	}

	.clear-single-btn:hover {
		background-color: var(--danger-bg);
		color: var(--danger-hover);
	}

	/* Apply Effect Section */
	.apply-effect-card {
		margin-top: var(--space-2);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
	}

	.apply-card-title {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.form-grid {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-3);
	}

	@media (max-width: 640px) {
		.form-grid {
			grid-template-columns: 1fr;
		}
	}

	.apply-action-bar {
		display: flex;
		justify-content: flex-end;
		margin-top: var(--space-2);
	}

	.icon-blue {
		color: var(--accent-blue-text);
	}

	.icon-muted {
		color: var(--text-muted);
	}

	.spinner {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}
</style>
