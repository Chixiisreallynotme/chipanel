<script>
	import { untrack } from 'svelte';
	import uPlot from 'uplot';
	import 'uplot/dist/uPlot.min.css';
	import { Activity, Gauge, Cpu, HardDrive, Users } from 'lucide-svelte';

	/**
	 * @typedef {{
	 *   timestamps?: number[],
	 *   tps?: (number|null)[],
	 *   cpu?: (number|null)[],
	 *   ram_mb?: (number|null)[],
	 *   players?: (number|null)[]
	 * }} HistoryData
	 */

	let {
		/** @type {HistoryData} */
		historyData = { timestamps: [], tps: [], cpu: [], ram_mb: [], players: [] },
		title = '',
		/** @type {'tps' | 'cpu' | 'ram' | 'players'} */
		chartType = 'tps',
		height = 240
	} = $props();

	/**
	 * One series per chart, one Y scale per chart. CPU (%) and RAM (MB) used to
	 * share a chart with two Y scales, which made the crossings meaningless and
	 * pushed the right-hand axis outside the container — they are separate
	 * charts now (A14).
	 */
	const SERIES = {
		tps: { key: 'tps', label: 'TPS', suffix: ' TPS', color: '#0FA968', fill: 'rgba(15, 169, 104, 0.12)', digits: 2, axisDigits: 1, max: 20.5 },
		cpu: { key: 'cpu', label: 'CPU', suffix: ' %', color: '#6366F1', fill: 'rgba(99, 102, 241, 0.12)', digits: 1, axisDigits: 0, max: 100 },
		ram: { key: 'ram_mb', label: 'RAM', suffix: ' Mo', color: '#D97706', fill: 'rgba(217, 119, 6, 0.12)', digits: 0, axisDigits: 0, max: null },
		players: { key: 'players', label: 'Joueurs', suffix: '', color: '#8B5CF6', fill: 'rgba(139, 92, 246, 0.12)', digits: 0, axisDigits: 0, max: null }
	};

	let spec = $derived(SERIES[chartType] ?? SERIES.tps);
	let timestamps = $derived(historyData?.timestamps ?? []);
	/** @type {(number|null)[]} */
	let values = $derived(/** @type {any} */ (historyData)?.[spec.key] ?? []);

	// A chart with no samples, or with nothing but nulls, gets an explicit empty
	// state instead of a flat line pinned to the axis (B12).
	let hasData = $derived(timestamps.length > 0 && values.some((v) => v != null));

	let latestValue = $derived.by(() => {
		if (timestamps.length === 0) return 'aucune donnée';
		const last = values[timestamps.length - 1];
		if (last == null) return 'n/a';
		return `${last.toFixed(spec.digits)}${spec.suffix}`;
	});

	/** @param {number} ts */
	function fmtClock(ts) {
		const d = new Date(ts * (ts < 1e11 ? 1000 : 1));
		const pad = (/** @type {number} */ n) => String(n).padStart(2, '0');
		return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
	}

	function getChartData() {
		return [historyData?.timestamps ?? [], /** @type {any} */ (historyData)?.[spec.key] ?? []];
	}

	/**
	 * @param {number} width
	 * @param {number} h
	 * @returns {any}
	 */
	function getChartOpts(width, h) {
		const s = spec;
		return {
			width,
			height: h,
			cursor: {
				sync: { key: 'chipanel-metrics-sync' },
				focus: { prox: 16 },
				drag: { setScale: true }
			},
			scales: {
				x: { time: true },
				y: {
					// Hard floor at 0; hard ceiling only where the unit has one.
					range: (/** @type {any} */ _u, /** @type {number} */ _min, /** @type {number} */ max) => [
						0,
						s.max ?? Math.max(1, (max ?? 1) * 1.1)
					]
				}
			},
			axes: [
				{
					scale: 'x',
					// 24h HH:MM:SS labels need real breathing room or they run
					// together into "11:08:30 PM11:08:45 PM" (B11).
					space: 90,
					stroke: '#7070A0',
					grid: { stroke: 'rgba(255, 255, 255, 0.05)', width: 1 },
					ticks: { stroke: '#1E1E2A', width: 1 },
					font: '11px "JetBrains Mono", monospace',
					values: (/** @type {any} */ _self, /** @type {number[]} */ ticks) =>
						ticks.map((ts) => (ts == null ? '' : fmtClock(ts)))
				},
				{
					scale: 'y',
					stroke: s.color,
					grid: { stroke: 'rgba(255, 255, 255, 0.05)', width: 1 },
					ticks: { stroke: '#1E1E2A', width: 1 },
					font: '11px "JetBrains Mono", monospace',
					values: (/** @type {any} */ _self, /** @type {number[]} */ ticks) =>
						ticks.map((v) => (v == null ? '' : `${v.toFixed(s.axisDigits)}${s.suffix}`))
				}
			],
			series: [
				{
					label: 'Heure',
					value: (/** @type {any} */ _self, /** @type {number} */ raw) =>
						raw == null ? '--' : fmtClock(raw)
				},
				{
					label: s.label,
					scale: 'y',
					stroke: s.color,
					width: 2,
					fill: s.fill,
					// spanGaps stays off: a null sample is a hole in the data, not
					// a straight line between the points that surround it.
					spanGaps: false,
					points: { show: false },
					value: (/** @type {any} */ _self, /** @type {number} */ raw) =>
						raw == null ? 'n/a' : `${raw.toFixed(s.digits)}${s.suffix}`
				}
			]
		};
	}

	/** @type {HTMLDivElement | null} */
	let containerEl = $state(null);
	/** @type {any} */
	let uplotInstance = null;

	/** uPlot needs an explicit pixel width and never shrinks on its own (A15). */
	function measure(/** @type {HTMLElement} */ el) {
		return Math.max(120, Math.floor(el.clientWidth));
	}

	$effect(() => {
		const el = containerEl;
		const h = height;
		// re-create when the series definition changes
		void spec;
		if (!el) return;

		const chart = new uPlot(
			untrack(() => getChartOpts(measure(el), h)),
			untrack(() => getChartData()),
			el
		);
		uplotInstance = chart;

		let lastWidth = measure(el);
		const observer = new ResizeObserver(() => {
			const width = measure(el);
			if (width === lastWidth) return;
			lastWidth = width;
			chart.setSize({ width, height: h });
		});
		observer.observe(el);

		return () => {
			observer.disconnect();
			chart.destroy();
			uplotInstance = null;
		};
	});

	$effect(() => {
		const data = getChartData();
		if (uplotInstance) uplotInstance.setData(data);
	});
</script>

<div class="card metrics-chart-card">
	<div class="card-header chart-card-header">
		<div class="header-title-group">
			{#if chartType === 'tps'}
				<Gauge size={18} class="chart-header-icon icon-tps" />
			{:else if chartType === 'cpu'}
				<Cpu size={18} class="chart-header-icon icon-cpu" />
			{:else if chartType === 'ram'}
				<HardDrive size={18} class="chart-header-icon icon-ram" />
			{:else if chartType === 'players'}
				<Users size={18} class="chart-header-icon icon-players" />
			{:else}
				<Activity size={18} class="chart-header-icon" />
			{/if}
			<h3 class="card-title">{title}</h3>
		</div>
		<div class="header-badge-group">
			<span class="badge badge-blue">{latestValue}</span>
		</div>
	</div>

	<div class="card-body chart-card-body">
		{#if hasData}
			<div class="uplot-wrapper" style="min-height: {height}px" bind:this={containerEl}></div>
		{:else}
			<div class="chart-empty-state" style="min-height: {height}px">
				<Activity size={24} />
				<span class="empty-title">Aucune donnée mesurée</span>
				<span class="empty-desc">
					Les échantillons apparaîtront dès que la télémétrie sera disponible. Les périodes non
					mesurées restent vides — elles ne sont pas comblées.
				</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.metrics-chart-card {
		width: 100%;
		/* Grid/flex children default to min-width:auto and refuse to shrink below
		   their content, which is how the chart escaped its column (A15/B2). */
		min-width: 0;
		display: flex;
		flex-direction: column;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		overflow: hidden;
	}

	.chart-card-header {
		padding: var(--space-4) var(--space-6);
		border-bottom: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
	}

	.header-title-group {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		min-width: 0;
	}

	.chart-header-icon.icon-tps {
		color: #0fa968;
	}

	.chart-header-icon.icon-cpu {
		color: #6366f1;
	}

	.chart-header-icon.icon-ram {
		color: #d97706;
	}

	/* Non-text icon (3:1), deliberately kept identical to the `players` SERIES
	   colour above so the header matches the plotted line. Not the UI text token. */
	.chart-header-icon.icon-players {
		color: #8b5cf6;
	}

	.chart-card-body {
		padding: var(--space-4);
		position: relative;
		width: 100%;
		min-width: 0;
		overflow: hidden;
	}

	.uplot-wrapper {
		width: 100%;
		min-width: 0;
		overflow: hidden;
	}

	.chart-empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		text-align: center;
		padding: var(--space-6);
		color: var(--text-muted);
	}

	.empty-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		color: var(--text-secondary);
	}

	.empty-desc {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		max-width: 42ch;
	}

	/* uPlot dark theme overrides */
	:global(.uplot) {
		font-family: var(--font-mono) !important;
	}

	:global(.u-legend) {
		color: var(--text-secondary) !important;
		font-size: var(--font-size-xs) !important;
		padding: 4px 8px !important;
	}

	:global(.u-legend .u-series) {
		padding: 2px 6px !important;
		border-radius: var(--radius-sm);
	}

	:global(.u-legend .u-label) {
		color: var(--text-muted) !important;
		font-weight: var(--font-weight-medium);
	}

	:global(.u-legend .u-value) {
		color: var(--text-primary) !important;
		font-weight: var(--font-weight-bold);
	}

	:global(.u-cursor-x),
	:global(.u-cursor-y) {
		border-color: rgba(255, 255, 255, 0.25) !important;
	}
</style>
