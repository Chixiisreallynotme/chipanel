<script>
	import { apiPost } from '$lib/api/client.js';
	import {
		Compass,
		Play,
		Pause,
		Square,
		Circle,
		Octagon,
		Triangle,
		Gauge,
		Clock,
		Layers,
		CheckCircle2,
		AlertCircle,
		RefreshCw,
		StopCircle,
		Activity
	} from '$lib/icons.js';

	let {
		chunkyStatus = {
			is_running: false,
			is_paused: false,
			percent_complete: 0,
			chunks_rendered: 0,
			chunks_total: 0,
			current_cps: 0,
			eta_seconds: 0,
			world: ''
		},
		worlds = [],
		selectedWorldName = '',
		onActionSuccess = () => {},
		onToast = () => {}
	} = $props();

	// Form State
	// svelte-ignore state_referenced_locally — initial world for the form; kept in sync via the $effect below
	let formWorld = $state(selectedWorldName || (worlds.length > 0 ? worlds[0].folder_name : 'world'));
	let shape = $state('square'); // 'square' | 'circle' | 'diamond' | 'pentagon'
	let radius = $state(5000);
	let centerX = $state(0);
	let centerZ = $state(0);
	let isSubmitting = $state(false);

	// Update formWorld if selectedWorldName prop changes
	$effect(() => {
		if (selectedWorldName) {
			formWorld = selectedWorldName;
		} else if (!formWorld && worlds.length > 0) {
			formWorld = worlds[0].folder_name;
		}
	});

	// Shape options
	const shapes = [
		{ id: 'square', label: 'Square', icon: Square },
		{ id: 'circle', label: 'Circle', icon: Circle },
		{ id: 'diamond', label: 'Diamond', icon: Octagon },
		{ id: 'pentagon', label: 'Pentagon', icon: Triangle }
	];

	// Preset Radius Options
	const presetRadii = [1000, 2500, 5000, 10000, 25000];

	// Format ETA seconds into readable duration
	function formatEta(seconds) {
		if (!seconds || seconds <= 0) return '00m 00s';
		const hrs = Math.floor(seconds / 3600);
		const mins = Math.floor((seconds % 3600) / 60);
		const secs = seconds % 60;
		if (hrs > 0) {
			return `${hrs}h ${mins.toString().padStart(2, '0')}m ${secs.toString().padStart(2, '0')}s`;
		}
		return `${mins.toString().padStart(2, '0')}m ${secs.toString().padStart(2, '0')}s`;
	}

	// Calculate Progress Percentage
	let progressPercent = $derived.by(() => {
		if (chunkyStatus.chunks_total > 0) {
			return Math.min(100, Math.max(0, (chunkyStatus.chunks_rendered / chunkyStatus.chunks_total) * 100));
		}
		return Math.min(100, Math.max(0, chunkyStatus.percent_complete || 0));
	});

	// Pre-generation Control Action Handler
	async function handleControlAction(action, payload = {}) {
		if (isSubmitting) return;
		isSubmitting = true;

		try {
			const body = { action, ...payload };
			const res = await apiPost('/api/worlds/chunky/control', body);
			const msg = res.message || res.status || `Chunky task ${action} executed.`;
			onToast('success', `Chunky ${action.toUpperCase()}`, msg);
			onActionSuccess(action, msg);
		} catch (err) {
			console.error(`Chunky ${action} failed:`, err);
			onToast('error', `Chunky ${action} Failed`, err.message || `Failed to execute ${action} command.`);
		} finally {
			isSubmitting = false;
		}
	}

	function handleStart(e) {
		if (e) e.preventDefault();
		if (!formWorld) {
			onToast('error', 'Validation Error', 'Please select a target world for pre-generation.');
			return;
		}
		if (radius <= 0) {
			onToast('error', 'Validation Error', 'Radius must be a positive block count.');
			return;
		}

		handleControlAction('start', {
			world: formWorld,
			shape,
			radius: Number(radius),
			center_x: Number(centerX),
			center_z: Number(centerZ)
		});
	}

	// Keyboard arrow key navigation handler for shape selection radiogroup
	function handleShapeKeydown(e) {
		if (chunkyStatus.is_running || chunkyStatus.is_paused) return;
		if (['ArrowRight', 'ArrowDown', 'ArrowLeft', 'ArrowUp'].includes(e.key)) {
			e.preventDefault();
			const currentIndex = shapes.findIndex((s) => s.id === shape);
			if (currentIndex === -1) return;
			let nextIndex;
			if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
				nextIndex = (currentIndex + 1) % shapes.length;
			} else {
				nextIndex = (currentIndex - 1 + shapes.length) % shapes.length;
			}
			shape = shapes[nextIndex].id;
			const buttons = e.currentTarget.querySelectorAll('button');
			if (buttons[nextIndex]) {
				buttons[nextIndex].focus();
			}
		}
	}
</script>

<div class="chunky-panel card">
	<!-- Panel Header -->
	<div class="card-header chunky-header">
		<div class="header-title-box">
			<div class="chunky-icon-box">
				<Compass size={22} />
			</div>
			<div>
				<h3 class="card-title">Chunky World Pre-generator</h3>
				<p class="card-subtitle">Generate world terrain chunks in background to eliminate server lag during exploration</p>
			</div>
		</div>

		<!-- Status Badge HUD Indicator -->
		<div class="chunky-status-badge">
			{#if chunkyStatus.is_running}
				<span class="badge badge-success">
					<span class="status-dot status-dot-success status-dot-pulse"></span>
					<span>Running ({chunkyStatus.world || 'Active World'})</span>
				</span>
			{:else if chunkyStatus.is_paused}
				<span class="badge badge-warning">
					<span class="status-dot status-dot-warning status-dot-pulse"></span>
					<span>Paused</span>
				</span>
			{:else}
				<span class="badge badge-secondary">
					<span class="status-dot"></span>
					<span>Idle</span>
				</span>
			{/if}
		</div>
	</div>

	<!-- Telemetry HUD Bar (Visible whenever task is running or paused, or shows overall status) -->
	<div class="chunky-hud-section">
		<div class="hud-metrics-row">
			<!-- Chunks Rendered -->
			<div class="hud-metric-card">
				<div class="hud-metric-icon">
					<Layers size={18} class="icon-blue" />
				</div>
				<div class="hud-metric-info">
					<span class="hud-metric-label">Rendered Chunks</span>
					<span class="hud-metric-value font-mono">
						{(chunkyStatus.chunks_rendered || 0).toLocaleString()} / {(chunkyStatus.chunks_total || 0).toLocaleString()}
					</span>
				</div>
			</div>

			<!-- Speed CPS -->
			<div class="hud-metric-card">
				<div class="hud-metric-icon">
					<Gauge size={18} class="icon-green" />
				</div>
				<div class="hud-metric-info">
					<span class="hud-metric-label">Generation Speed</span>
					<span class="hud-metric-value font-mono">
						{(chunkyStatus.current_cps || 0).toFixed(1)} <span class="unit">CPS</span>
					</span>
				</div>
			</div>

			<!-- ETA Duration -->
			<div class="hud-metric-card">
				<div class="hud-metric-icon">
					<Clock size={18} class="icon-purple" />
				</div>
				<div class="hud-metric-info">
					<span class="hud-metric-label">Estimated Time Left</span>
					<span class="hud-metric-value font-mono">
						{formatEta(chunkyStatus.eta_seconds)}
					</span>
				</div>
			</div>

			<!-- Target World -->
			<div class="hud-metric-card">
				<div class="hud-metric-icon">
					<Compass size={18} class="icon-warning" />
				</div>
				<div class="hud-metric-info">
					<span class="hud-metric-label">Target Dimension</span>
					<span class="hud-metric-value font-mono">
						{chunkyStatus.world || (formWorld || 'None')}
					</span>
				</div>
			</div>
		</div>

		<!-- Progress Bar HUD -->
		<div class="hud-progress-container">
			<div class="progress-info-row">
				<span class="progress-title">Overall Generation Progress</span>
				<span class="progress-percentage font-mono">{progressPercent.toFixed(1)}%</span>
			</div>
			<div class="gauge-bar chunky-gauge-bar">
				<div
					class="gauge-fill {chunkyStatus.is_paused ? 'gauge-fill-warning' : 'gauge-fill-success'}"
					style="transform: scaleX({progressPercent / 100});"
				></div>
			</div>
		</div>

		<!-- Live Telemetry Controls (If running or paused) -->
		{#if chunkyStatus.is_running || chunkyStatus.is_paused}
			<div class="live-controls-bar">
				<span class="live-status-text">
					{#if chunkyStatus.is_running}
						<Activity size={16} class="icon-green spin-slow" />
						<span>Pre-generation in progress...</span>
					{:else}
						<Pause size={16} class="icon-warning" />
						<span>Task paused.</span>
					{/if}
				</span>

				<div class="live-actions-group">
					{#if chunkyStatus.is_running}
						<button
							type="button"
							class="btn btn-secondary btn-sm {isSubmitting ? 'btn-loading' : ''}"
							onclick={() => handleControlAction('pause')}
							disabled={isSubmitting}
						>
							<Pause size={14} />
							<span>Pause Task</span>
						</button>
					{:else if chunkyStatus.is_paused}
						<button
							type="button"
							class="btn btn-primary btn-sm {isSubmitting ? 'btn-loading' : ''}"
							onclick={() => handleControlAction('continue')}
							disabled={isSubmitting}
						>
							<Play size={14} />
							<span>Resume Generation</span>
						</button>
					{/if}

					<button
						type="button"
						class="btn btn-danger btn-sm {isSubmitting ? 'btn-loading' : ''}"
						onclick={() => handleControlAction('cancel')}
						disabled={isSubmitting}
					>
						<StopCircle size={14} />
						<span>Cancel Generation</span>
					</button>
				</div>
			</div>
		{/if}
	</div>

	<!-- Start Pre-generation Form -->
	<div class="card-body chunky-form-body">
		<h4 class="form-section-title">Configure & Start New Pre-generation</h4>

		<form onsubmit={handleStart} class="pregen-form">
			<div class="form-grid-3">
				<!-- Target World Selector -->
				<div class="form-group">
					<label for="chunky-world-select" class="label label-required">Target World</label>
					<select
						id="chunky-world-select"
						class="select"
						bind:value={formWorld}
						disabled={chunkyStatus.is_running || chunkyStatus.is_paused}
					>
						{#if worlds.length === 0}
							<option value="world">world</option>
						{:else}
							{#each worlds as w}
								<option value={w.folder_name}>
									{w.level_name} ({w.folder_name})
								</option>
							{/each}
						{/if}
					</select>
				</div>

				<!-- Center Coordinates (X, Z) -->
				<div class="form-group">
					<label for="chunky-center-x" class="label">Center Coordinates (X / Z)</label>
					<div class="coords-input-row">
						<div class="coord-field">
							<span class="coord-prefix">X:</span>
							<input
								id="chunky-center-x"
								type="number"
								class="input font-mono"
								bind:value={centerX}
								placeholder="0"
								disabled={chunkyStatus.is_running || chunkyStatus.is_paused}
							/>
						</div>
						<div class="coord-field">
							<span class="coord-prefix">Z:</span>
							<input
								id="chunky-center-z"
								type="number"
								class="input font-mono"
								bind:value={centerZ}
								placeholder="0"
								aria-label="Center Z Coordinate"
								disabled={chunkyStatus.is_running || chunkyStatus.is_paused}
							/>
						</div>
					</div>
				</div>

				<!-- Radius Input -->
				<div class="form-group">
					<label for="chunky-radius-input" class="label label-required">Radius (Blocks)</label>
					<input
						id="chunky-radius-input"
						type="number"
						class="input font-mono"
						min="100"
						step="500"
						bind:value={radius}
						placeholder="5000"
						disabled={chunkyStatus.is_running || chunkyStatus.is_paused}
					/>
					<div class="preset-pills">
						{#each presetRadii as r}
							<button
								type="button"
								class="preset-pill {radius === r ? 'active' : ''}"
								onclick={() => (radius = r)}
								disabled={chunkyStatus.is_running || chunkyStatus.is_paused}
							>
								{r >= 1000 ? `${r / 1000}k` : r}
							</button>
						{/each}
					</div>
				</div>
			</div>

			<!-- Shape Selector -->
			<div class="form-group shape-form-group">
				<span class="label">Pre-generation Shape</span>
				<div
					class="shape-selector-grid"
					role="radiogroup"
					tabindex="-1"
					aria-label="Pre-generation shape selector"
					onkeydown={handleShapeKeydown}
				>
					{#each shapes as s}
						{@const ShapeIcon = s.icon}
						<button
							type="button"
							class="shape-card {shape === s.id ? 'active' : ''}"
							onclick={() => (shape = s.id)}
							disabled={chunkyStatus.is_running || chunkyStatus.is_paused}
							role="radio"
							aria-checked={shape === s.id}
							tabindex={shape === s.id ? 0 : -1}
						>
							<ShapeIcon size={20} class="shape-icon" />
							<span class="shape-label">{s.label}</span>
						</button>
					{/each}
				</div>
			</div>

			<!-- Form Submit Button -->
			<div class="form-actions-row">
				<button
					type="submit"
					class="btn btn-primary btn-lg start-gen-btn {isSubmitting ? 'btn-loading' : ''}"
					disabled={isSubmitting || chunkyStatus.is_running || chunkyStatus.is_paused}
				>
					{#if !isSubmitting}
						<Play size={18} />
					{/if}
					<span>Start Generation</span>
				</button>
			</div>
		</form>
	</div>
</div>

<style>
	.chunky-panel {
		display: flex;
		flex-direction: column;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		overflow: hidden;
	}

	/* Header */
	.chunky-header {
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
	}

	.header-title-box {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.chunky-icon-box {
		width: 44px;
		height: 44px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
		flex-shrink: 0;
	}

	/* HUD Section */
	.chunky-hud-section {
		background-color: var(--bg-base);
		border-bottom: 1px solid var(--border);
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.hud-metrics-row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: var(--space-4);
	}

	.hud-metric-card {
		background-color: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.hud-metric-icon {
		width: 36px;
		height: 36px;
		border-radius: var(--radius-btn);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.icon-blue { color: var(--accent-blue-text); }
	.icon-green { color: var(--accent-green); }
	.icon-purple { color: var(--accent-purple-text); }
	.icon-warning { color: var(--warning); }

	.hud-metric-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.hud-metric-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.hud-metric-value {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.unit {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: normal;
	}

	/* Progress Bar */
	.hud-progress-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.progress-info-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: var(--font-size-sm);
	}

	.progress-title {
		color: var(--text-secondary);
		font-weight: var(--font-weight-medium);
	}

	.progress-percentage {
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-bold);
	}

	.chunky-gauge-bar {
		height: 10px;
	}

	/* Live Controls Bar */
	.live-controls-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		background-color: var(--bg-surface);
		border: 1px solid var(--border-focus);
		border-radius: var(--radius-btn);
		padding: var(--space-3) var(--space-4);
		gap: var(--space-3);
	}

	.live-status-text {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		color: var(--text-primary);
		font-weight: var(--font-weight-medium);
	}

	.spin-slow {
		animation: spin 3s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.live-actions-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	/* Form Body */
	.chunky-form-body {
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.form-section-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.pregen-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-6);
	}

	.form-grid-3 {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
		gap: var(--space-6);
	}

	.coords-input-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.coord-field {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		flex: 1;
	}

	.coord-prefix {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-semibold);
	}

	.preset-pills {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		margin-top: var(--space-2);
		flex-wrap: wrap;
	}

	.preset-pill {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-badge);
		padding: 3px 10px;
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		font-variant-numeric: tabular-nums;
		color: var(--text-secondary);
		cursor: pointer;
		user-select: none;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
		transition: transform 160ms var(--ease-out),
		            background-color 150ms var(--ease-out),
		            border-color 150ms var(--ease-out),
		            color 150ms var(--ease-out),
		            box-shadow 150ms var(--ease-out);
	}

	.preset-pill:hover:not(:disabled) {
		border-color: var(--accent-blue-border);
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
	}

	.preset-pill:active:not(:disabled) {
		transform: scale(0.97);
	}

	.preset-pill.active {
		background-color: var(--accent-blue-bg);
		border-color: var(--accent-blue);
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-semibold);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12), 0 0 0 1px var(--accent-blue);
	}

	/* Shape Selector */
	.shape-form-group {
		margin-bottom: 0;
	}

	.shape-selector-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: var(--space-3);
		margin-top: var(--space-2);
	}

	.shape-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		padding: var(--space-4) var(--space-2);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-btn);
		color: var(--text-muted);
		cursor: pointer;
		user-select: none;
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
		transition: transform 160ms var(--ease-out),
		            background-color 150ms var(--ease-out),
		            border-color 150ms var(--ease-out),
		            color 150ms var(--ease-out),
		            box-shadow 150ms var(--ease-out);
	}

	.shape-card:hover:not(:disabled) {
		border-color: var(--border-focus);
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.02);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
	}

	.shape-card:active:not(:disabled) {
		transform: scale(0.97);
	}

	.shape-card.active {
		background-color: var(--accent-blue-bg);
		border-color: var(--accent-blue);
		color: var(--accent-blue-text);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12), 0 0 0 1px var(--accent-blue);
	}

	.shape-card.active .shape-icon {
		color: var(--accent-blue-text);
	}

	.shape-label {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
	}

	.form-actions-row {
		display: flex;
		justify-content: flex-end;
		padding-top: var(--space-2);
	}

	.start-gen-btn {
		min-width: 200px;
	}

	@media (max-width: 640px) {
		.shape-selector-grid {
			grid-template-columns: repeat(2, 1fr);
		}

		.live-controls-bar {
			flex-direction: column;
			align-items: stretch;
		}

		.live-actions-group {
			justify-content: flex-end;
		}
	}
</style>
