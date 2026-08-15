<script>
	import {
		Globe,
		Flame,
		Sparkles,
		Copy,
		Check,
		Compass,
		Shield,
		Save,
		HardDrive,
		MapPin,
		Maximize2,
		Layers,
		Play,
		Download,
		Trash2,
		Settings
	} from 'lucide-svelte';

	let {
		world = {
			folder_name: 'world',
			level_name: 'Main World',
			seed: 0,
			generator: 'default',
			size_bytes: 0,
			spawn_x: 0,
			spawn_y: 64,
			spawn_z: 0,
			is_nether: false,
			is_end: false,
			data_version: null
		},
		border = null,
		isActive = false,
		serverDataVersion = null,
		switchInFlight = false,
		onEditBorder = () => {},
		onStartPregen = () => {},
		onCreateBackup = () => {},
		onSwitch = () => {},
		onDelete = () => {},
		onDownload = () => {},
		onConfigure = () => {}
	} = $props();

	let copiedSeed = $state(false);

	// Compatibility of this world vs the configured server version.
	let compatInfo = $derived.by(() => {
		if (world.data_version == null || serverDataVersion == null) {
			return null; // unknown
		}
		if (world.data_version === serverDataVersion) {
			return { label: 'Compatible', badgeClass: 'badge-success' };
		}
		return { label: 'Incompatible', badgeClass: 'badge-warning' };
	});

	// Dimension info classification
	let dimensionInfo = $derived.by(() => {
		const folder = (world.folder_name || '').toLowerCase();
		if (world.is_nether || folder.endsWith('_nether') || folder === 'nether' || folder === 'dim-1') {
			return {
				name: 'Nether',
				icon: Flame,
				badgeClass: 'badge-danger',
				cardGlowClass: 'dimension-nether'
			};
		}
		if (world.is_end || folder.endsWith('_the_end') || folder.endsWith('_end') || folder === 'the_end' || folder === 'dim1') {
			return {
				name: 'The End',
				icon: Sparkles,
				badgeClass: 'badge-purple',
				cardGlowClass: 'dimension-end'
			};
		}
		return {
			name: 'Overworld',
			icon: Globe,
			badgeClass: 'badge-success',
			cardGlowClass: 'dimension-overworld'
		};
	});

	// Format disk size
	function formatBytes(bytes) {
		if (!bytes || bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i];
	}

	// Copy seed to clipboard with visual feedback
	async function copySeed() {
		try {
			await navigator.clipboard.writeText(String(world.seed ?? 0));
			copiedSeed = true;
			setTimeout(() => {
				copiedSeed = false;
			}, 2000);
		} catch (e) {
			console.error('Failed to copy seed:', e);
		}
	}

	// Format border summary text
	let borderSummary = $derived.by(() => {
		if (border && typeof border.size === 'number') {
			return `${border.size.toLocaleString()} blocks (Center: X ${border.center_x ?? 0}, Z ${border.center_z ?? 0})`;
		}
		return '60,000,000 blocks (Default)';
	});
</script>

<div class="card world-card {dimensionInfo.cardGlowClass}">
	<!-- Card Header: Title & Dimension Badge -->
	<div class="card-header world-card-header">
		<div class="header-main-info">
			<div class="dimension-icon-box">
				{#if dimensionInfo.name === 'Nether'}
					<Flame size={20} class="nether-icon" />
				{:else if dimensionInfo.name === 'The End'}
					<Sparkles size={20} class="end-icon" />
				{:else}
					<Globe size={20} class="overworld-icon" />
				{/if}
			</div>
			<div>
				<h3 class="card-title world-title">{world.level_name || world.folder_name}</h3>
				<span class="world-folder-tag">{world.folder_name}</span>
			</div>
		</div>

		<div class="badges-row">
			{#if isActive}
				<span class="badge badge-success active-badge" title="This is the world the server currently loads">
					<Play size={12} />
					<span>Active</span>
				</span>
			{/if}

			{#if compatInfo}
				<span
					class="badge {compatInfo.badgeClass} active-badge"
					title={compatInfo.label === 'Compatible'
						? `Data version ${world.data_version} matches the server`
						: `Data version ${world.data_version} differs from the server (${serverDataVersion}) — downgrade risk`}
				>
					<span>{compatInfo.label}</span>
				</span>
			{/if}

			<span class="badge {dimensionInfo.badgeClass} dimension-badge">
				{#if dimensionInfo.name === 'Nether'}
					<Flame size={12} />
				{:else if dimensionInfo.name === 'The End'}
					<Sparkles size={12} />
				{:else}
					<Globe size={12} />
				{/if}
				<span>{dimensionInfo.name}</span>
			</span>
		</div>
	</div>

	<!-- Card Body: Key World Metrics -->
	<div class="card-body world-card-body">
		<div class="metrics-grid">
			<!-- World Seed -->
			<div class="metric-item seed-metric">
				<div class="metric-header">
					<span class="metric-label">World Seed</span>
					<button
						type="button"
						class="btn btn-ghost btn-icon btn-sm copy-seed-btn {copiedSeed ? 'copied' : ''}"
						onclick={copySeed}
						title="Copy Seed to Clipboard"
						aria-label="Copy World Seed"
					>
						{#if copiedSeed}
							<Check size={14} class="text-success" />
						{:else}
							<Copy size={14} />
						{/if}
					</button>
				</div>
				<div class="metric-value font-mono seed-val">
					{world.seed ?? '0'}
				</div>
			</div>

			<!-- Spawn Location -->
			<div class="metric-item">
				<div class="metric-header">
					<span class="metric-label">Spawn Coordinates</span>
				</div>
				<div class="metric-value font-mono spawn-val">
					<MapPin size={13} class="icon-subtle" />
					<span>X: {world.spawn_x ?? 0}, Y: {world.spawn_y ?? 64}, Z: {world.spawn_z ?? 0}</span>
				</div>
			</div>

			<!-- Disk Usage Size -->
			<div class="metric-item">
				<div class="metric-header">
					<span class="metric-label">Disk Storage</span>
				</div>
				<div class="metric-value storage-val">
					<HardDrive size={13} class="icon-subtle" />
					<span>{formatBytes(world.size_bytes)}</span>
				</div>
			</div>

			<!-- Data version -->
			<div class="metric-item">
				<div class="metric-header">
					<span class="metric-label">Data Version</span>
				</div>
				<div class="metric-value font-mono">
					<Layers size={13} class="icon-subtle" />
					<span>{world.data_version ?? '—'}</span>
				</div>
			</div>

			<!-- Worldborder Summary -->
			<div class="metric-item full-width">
				<div class="metric-header">
					<span class="metric-label">Worldborder Summary</span>
				</div>
				<div class="metric-value border-val">
					<Shield size={13} class="icon-subtle" />
					<span>{borderSummary}</span>
				</div>
			</div>
		</div>
	</div>

	<!-- Card Footer: Quick Action Buttons -->
	<div class="card-footer world-card-footer">
		{#if !isActive}
			<button
				type="button"
				class="btn btn-primary btn-sm action-btn {switchInFlight ? 'btn-loading' : ''}"
				onclick={() => onSwitch(world)}
				disabled={switchInFlight}
				title="Make this the active world (restarts the server)"
			>
				<Play size={14} />
				<span>Switch to</span>
			</button>
		{:else}
			<button
				type="button"
				class="btn btn-primary btn-sm action-btn"
				onclick={() => onConfigure(world)}
				title="Configure game rules, difficulty, and settings"
			>
				<Settings size={14} />
				<span>Configure</span>
			</button>
		{/if}

		<button
			type="button"
			class="btn btn-secondary btn-sm action-btn"
			onclick={() => onDownload(world)}
			title="Download this world as a ZIP file"
		>
			<Download size={14} />
			<span>Download</span>
		</button>

		<button
			type="button"
			class="btn btn-danger btn-sm action-btn"
			onclick={() => onDelete(world)}
			disabled={isActive}
			title={isActive ? 'Switch to another world before deleting this one' : 'Delete this world (a safety backup is created first)'}
		>
			<Trash2 size={14} />
			<span>Delete</span>
		</button>

		<button
			type="button"
			class="btn btn-secondary btn-sm action-btn"
			onclick={() => onEditBorder(world)}
			title="Edit Worldborder boundaries and options"
		>
			<Maximize2 size={14} />
			<span>Border</span>
		</button>

		<button
			type="button"
			class="btn btn-secondary btn-sm action-btn"
			onclick={() => onStartPregen(world)}
			title="Pre-generate chunks with Chunky plugin"
		>
			<Compass size={14} />
			<span>Pre-gen</span>
		</button>

		<button
			type="button"
			class="btn btn-secondary btn-sm action-btn"
			onclick={() => onCreateBackup(world)}
			title="Create ZIP backup archive of world files"
		>
			<Save size={14} />
			<span>Backup</span>
		</button>
	</div>
</div>

<style>
	.world-card {
		display: flex;
		flex-direction: column;
		height: 100%;
		transition: transform var(--transition-fast), border-color var(--transition-fast), box-shadow var(--transition-fast);
	}

	.world-card:hover {
		border-color: var(--border-focus);
		transform: translateY(-2px);
		box-shadow: 0 10px 30px -8px rgba(0, 0, 0, 0.5), 0 0 0 1px var(--border-focus);
	}

	/* Dimension-specific subtle glows */
	.dimension-overworld:hover {
		box-shadow: 0 10px 30px -8px rgba(74, 222, 128, 0.15), 0 0 0 1px var(--accent-green-border);
	}

	.dimension-nether:hover {
		box-shadow: 0 10px 30px -8px rgba(244, 63, 94, 0.15), 0 0 0 1px var(--danger-border);
	}

	.dimension-end:hover {
		box-shadow: 0 10px 30px -8px hsl(258 90% 66% / 0.15), 0 0 0 1px hsl(258 90% 66% / 0.3);
	}

	/* Header Styling */
	.world-card-header {
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}

	.header-main-info {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		min-width: 0;
	}

	.dimension-icon-box {
		width: 38px;
		height: 38px;
		border-radius: var(--radius-btn);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.overworld-icon { color: var(--accent-green); }
	.nether-icon { color: var(--danger-text); }
	.end-icon { color: var(--accent-purple-text); }

	.world-title {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.world-folder-tag {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		display: block;
	}

	.dimension-badge {
		font-size: var(--font-size-xs);
		padding: var(--space-1) var(--space-3);
		gap: var(--space-1);
	}

	.badges-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		flex-shrink: 0;
	}

	.active-badge {
		font-size: var(--font-size-xs);
		padding: var(--space-1) var(--space-3);
		gap: var(--space-1);
	}

	.badge-purple {
		background-color: hsl(258 90% 66% / 0.12);
		color: var(--accent-purple-text);
		border: 1px solid hsl(258 90% 66% / 0.3);
	}

	/* Body & Metrics */
	.world-card-body {
		padding: var(--space-6);
		flex: 1;
	}

	.metrics-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-4);
	}

	.metric-item {
		display: flex;
		flex-direction: column;
		gap: 4px;
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-input);
		padding: var(--space-3);
	}

	.metric-item.full-width {
		grid-column: span 2;
	}

	.metric-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.metric-label {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}

	.metric-value {
		font-size: var(--font-size-sm);
		color: var(--text-primary);
		font-weight: var(--font-weight-medium);
		display: flex;
		align-items: center;
		gap: 6px;
		word-break: break-all;
	}

	.seed-metric {
		grid-column: span 2;
	}

	.seed-val {
		color: var(--accent-blue-text);
		letter-spacing: 0.02em;
	}

	.copy-seed-btn {
		width: 24px;
		height: 24px;
		padding: 0;
		color: var(--text-muted);
	}

	.copy-seed-btn:hover {
		color: var(--text-primary);
	}

	.copy-seed-btn.copied {
		color: var(--accent-green);
	}

	.text-success {
		color: var(--accent-green);
	}

	.icon-subtle {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	/* Footer Styling */
	.world-card-footer {
		padding: var(--space-3) var(--space-4);
		background-color: rgba(0, 0, 0, 0.2);
		border-top: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-2);
		flex-wrap: wrap;
	}

	.action-btn {
		flex: 1;
		min-width: fit-content;
		font-size: var(--font-size-xs);
	}

	@media (max-width: 480px) {
		.metrics-grid {
			grid-template-columns: 1fr;
		}

		.metric-item.full-width,
		.seed-metric {
			grid-column: span 1;
		}

		.world-card-footer {
			flex-direction: column;
		}

		.action-btn {
			width: 100%;
		}
	}
</style>
