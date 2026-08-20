<script>
	import { preferences } from '$lib/stores/preferences.svelte.js';
	import { HardDrive, Zap, AlertTriangle, CheckCircle2, Info, Sparkles } from '$lib/icons.js';

	let {
		allocatedRamGb = $bindable(4),
		hostRamGb = preferences.hostTotalRamGb
	} = $props();

	// Calculate recommended RAM based on host machine hardware
	let recommendedRamGb = $derived.by(() => {
		if (hostRamGb <= 4) return 2;
		if (hostRamGb <= 8) return 4;
		if (hostRamGb <= 12) return 6;
		return 8;
	});

	// OS Reserve (at least 2 GB or 25% of host memory)
	const osReserveGb = 2.0;

	// Memory breakdown percentages and normalized scales
	let heapPercentage = $derived(Math.min(100, (allocatedRamGb / hostRamGb) * 100));
	let osPercentage = $derived(Math.min(100, (osReserveGb / hostRamGb) * 100));
	let freePercentage = $derived(Math.max(0, 100 - heapPercentage - osPercentage));

	let osScale = $derived(Math.max(0, Math.min(1, osPercentage / 100)));
	let heapScale = $derived(Math.max(0, Math.min(1, heapPercentage / 100)));
	let freeScale = $derived(Math.max(0, Math.min(1, freePercentage / 100)));

	// High memory usage alert threshold (> 75% of host RAM)
	let isOverAllocated = $derived(allocatedRamGb > hostRamGb * 0.75);

	let maxSliderValue = $derived(Math.min(16, Math.max(8, hostRamGb)));

	function applyRecommendation() {
		allocatedRamGb = recommendedRamGb;
	}

	// Player capacity estimate based on RAM
	let estimatedPlayers = $derived.by(() => {
		if (allocatedRamGb <= 2) return '1 à 4 joueurs (Survie vanille légère)';
		if (allocatedRamGb <= 4) return '5 à 12 joueurs (Purpur/Paper avec plugins)';
		if (allocatedRamGb <= 6) return '10 à 25 joueurs (Communauté ou mods légers)';
		return '25+ joueurs ou gros modpack technique';
	});
</script>

<div class="ram-step-container">
	<div class="step-header">
		<h2 class="step-title">3. Allouez la mémoire vive (RAM)</h2>
		<p class="step-subtitle">
			Une allocation équilibrée garantit un jeu fluide (20 TPS) tout en préservant la stabilité du système hôte.
		</p>
	</div>

	<!-- Hardware Awareness Box -->
	<div class="hardware-probe-card">
		<div class="probe-header">
			<div class="probe-meta">
				<span class="probe-icon"><HardDrive size={18} /></span>
				<span class="probe-title">Mémoire système détectée :</span>
				<span class="probe-val font-mono tabular-nums">{hostRamGb.toFixed(1)} Go RAM</span>
			</div>

			<button
				type="button"
				class="recommendation-chip"
				class:active={allocatedRamGb === recommendedRamGb}
				onclick={applyRecommendation}
				title="Appliquer automatiquement l'allocation optimale calculée pour votre matériel"
			>
				<Sparkles size={13} />
				<span>Appliquer recommandation (<span class="font-mono tabular-nums">{recommendedRamGb} Go</span>)</span>
			</button>
		</div>

		<!-- Segmented Memory Allocation Bar (GPU-safe transform: scaleX) -->
		<div class="memory-meter-container">
			<div class="memory-meter-bar">
				<div
					class="meter-segment segment-free"
					style="transform: translate3d({(osPercentage + heapPercentage).toFixed(2)}%, 0, 0) scaleX({freeScale.toFixed(4)});"
					title={`Mémoire Libre Restante : ${(hostRamGb - allocatedRamGb - osReserveGb).toFixed(1)} Go`}
				></div>
				<div
					class="meter-segment segment-heap"
					style="transform: translate3d({osPercentage.toFixed(2)}%, 0, 0) scaleX({heapScale.toFixed(4)});"
					title={`Mémoire Allouée au Serveur : ${allocatedRamGb} Go`}
				></div>
				<div
					class="meter-segment segment-os"
					style="transform: scaleX({osScale.toFixed(4)});"
					title={`Réserve Système OS & ChiPanel : ${osReserveGb} Go`}
				></div>
			</div>

			<div class="meter-legend">
				<div class="legend-item">
					<span class="legend-dot dot-os"></span>
					<span class="legend-label">OS & ChiPanel (<span class="num tabular-nums font-mono">{osReserveGb.toFixed(1)} Go</span>)</span>
				</div>
				<div class="legend-item">
					<span class="legend-dot dot-heap"></span>
					<span class="legend-label">Serveur Minecraft (<span class="num tabular-nums font-mono">{allocatedRamGb} Go</span>)</span>
				</div>
				<div class="legend-item">
					<span class="legend-dot dot-free"></span>
					<span class="legend-label">
						Marge disponible (<span class="num tabular-nums font-mono">{Math.max(0, hostRamGb - allocatedRamGb - osReserveGb).toFixed(1)} Go</span>)
					</span>
				</div>
			</div>
		</div>
	</div>

	<!-- Interactive RAM Slider -->
	<div class="slider-control-card">
		<div class="slider-header">
			<span class="slider-label">Allocation pour le conteneur Minecraft</span>
			<div class="slider-display-badge">
				<span class="slider-number font-mono tabular-nums">{allocatedRamGb}</span>
				<span class="slider-unit">Go</span>
			</div>
		</div>

		<div class="slider-input-wrapper">
			<input
				type="range"
				min="1"
				max={maxSliderValue}
				step="0.5"
				bind:value={allocatedRamGb}
				class="ram-range-slider"
				aria-label="Allocation de mémoire RAM en Gigaoctets"
			/>
		</div>

		<div class="slider-ticks-row font-mono tabular-nums">
			<span>1 Go</span>
			<span>2 Go</span>
			<span>4 Go</span>
			<span>6 Go</span>
			<span>8 Go</span>
			{#if maxSliderValue > 8}
				<span>{maxSliderValue} Go</span>
			{/if}
		</div>

		<!-- Capacity Indicator -->
		<div class="capacity-info-box">
			<div class="capacity-row">
				<span class="probe-icon"><Info size={14} /></span>
				<span class="capacity-title">Capacité estimée :</span>
				<span class="capacity-value">{estimatedPlayers}</span>
			</div>
		</div>

		{#if isOverAllocated}
			<div class="warning-alert-box">
				<span class="text-warning-icon"><AlertTriangle size={15} /></span>
				<div class="warning-text">
					<strong>Attention :</strong> Vous allouez plus de 75% de la mémoire de la machine. Si d'autres services fonctionnent sur l'hôte, cela peut entraîner des ralentissements système.
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.ram-step-container {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.step-header {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.step-title {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		letter-spacing: -0.015em;
	}

	.step-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		line-height: var(--line-height-normal);
		max-width: 65ch;
	}

	.hardware-probe-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 14px;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04), 0 4px 16px rgba(0, 0, 0, 0.2);
	}

	.probe-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}

	.probe-meta {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.probe-icon {
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	.probe-title {
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
		font-weight: var(--font-weight-medium);
	}

	.probe-val {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		background-color: var(--bg-base);
		padding: 2px 8px;
		border-radius: var(--radius-input);
		border: 1px solid var(--border);
	}

	.recommendation-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 12px;
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		border-radius: var(--radius-badge);
		color: var(--accent-blue-text);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		cursor: pointer;
		transition: transform 160ms var(--ease-out), background-color 150ms var(--ease-out), border-color 150ms var(--ease-out);
	}

	.recommendation-chip:hover {
		background-color: rgba(59, 130, 246, 0.2);
		border-color: var(--accent-blue);
	}

	.recommendation-chip:active {
		transform: scale(0.97);
	}

	.recommendation-chip.active {
		background-color: var(--accent-green-bg);
		border-color: var(--accent-green-border);
		color: var(--accent-green-text);
	}

	.memory-meter-container {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.memory-meter-bar {
		position: relative;
		height: 12px;
		background-color: var(--bg-base);
		border-radius: 6px;
		overflow: hidden;
		border: 1px solid var(--border);
		box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4);
	}

	.meter-segment {
		position: absolute;
		top: 0;
		left: 0;
		bottom: 0;
		width: 100%;
		height: 100%;
		transform-origin: left;
		transition: transform 160ms var(--ease-out);
		will-change: transform;
	}

	.segment-os {
		background-color: #4B5563; /* Cool Slate Gray OS reserve */
		z-index: 2;
	}

	.segment-heap {
		background-color: var(--accent-blue-solid);
		z-index: 1;
	}

	.segment-free {
		background-color: var(--bg-elevated);
		z-index: 0;
	}

	.meter-legend {
		display: flex;
		align-items: center;
		gap: 16px;
		flex-wrap: wrap;
		font-size: 11px;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--text-secondary);
	}

	.legend-dot {
		width: 8px;
		height: 8px;
		border-radius: 2px;
	}

	.dot-os { background-color: #4B5563; }
	.dot-heap { background-color: var(--accent-blue-solid); }
	.dot-free { background-color: var(--bg-elevated); border: 1px solid var(--border-focus); }

	.tabular-nums {
		font-variant-numeric: tabular-nums;
	}

	.slider-control-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: 18px;
		display: flex;
		flex-direction: column;
		gap: 14px;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04), 0 4px 16px rgba(0, 0, 0, 0.2);
	}

	.slider-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.slider-label {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.slider-display-badge {
		display: flex;
		align-items: baseline;
		gap: 3px;
		background-color: var(--bg-base);
		border: 1px solid var(--accent-blue);
		padding: 4px 12px;
		border-radius: var(--radius-btn);
		box-shadow: 0 0 12px rgba(59, 130, 246, 0.15), inset 0 1px 0 rgba(255, 255, 255, 0.06);
	}

	.slider-number {
		font-size: var(--font-size-xl);
		font-weight: var(--font-weight-bold);
		color: var(--accent-blue-text);
	}

	.slider-unit {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.slider-input-wrapper {
		width: 100%;
		padding: 4px 0;
	}

	.ram-range-slider {
		-webkit-appearance: none;
		appearance: none;
		width: 100%;
		height: 8px;
		background: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: 4px;
		outline: none;
		cursor: pointer;
	}

	.ram-range-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 22px;
		height: 22px;
		border-radius: 50%;
		background: #FFFFFF;
		border: 3px solid var(--accent-blue);
		cursor: pointer;
		box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
		transition: transform 120ms var(--ease-out);
	}

	.ram-range-slider::-webkit-slider-thumb:hover {
		transform: scale(1.15);
	}

	.ram-range-slider::-webkit-slider-thumb:active {
		transform: scale(0.95);
	}

	.ram-range-slider::-moz-range-thumb {
		width: 22px;
		height: 22px;
		border-radius: 50%;
		background: #FFFFFF;
		border: 3px solid var(--accent-blue);
		cursor: pointer;
		box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
	}

	.slider-ticks-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: 11px;
		color: var(--text-muted);
		padding: 0 2px;
	}

	.capacity-info-box {
		padding: 10px 12px;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
	}

	.capacity-row {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: var(--font-size-xs);
	}

	.capacity-title {
		color: var(--text-secondary);
		font-weight: var(--font-weight-medium);
	}

	.capacity-value {
		color: var(--text-primary);
		font-weight: var(--font-weight-semibold);
	}

	.warning-alert-box {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		padding: 10px 12px;
		background-color: var(--accent-orange-bg);
		border: 1px solid var(--accent-orange-border);
		border-radius: var(--radius-input);
	}

	.text-warning-icon {
		color: var(--accent-orange);
		flex-shrink: 0;
		margin-top: 1px;
	}

	.warning-text {
		font-size: var(--font-size-xs);
		color: var(--accent-orange-text);
		line-height: 1.4;
	}
</style>
