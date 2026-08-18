<script>
	/**
	 * @typedef {'active' | 'hibernating' | 'busy' | 'stopped' | 'crash'} ServerStatus
	 */

	/**
	 * @type {{
	 *   status?: ServerStatus,
	 *   size?: number,
	 *   showLed?: boolean,
	 *   class?: string
	 * }}
	 */
	let {
		status = 'active',
		size = 32,
		showLed = true,
		class: className = ''
	} = $props();

	const statusLedClass = $derived(() => {
		switch (status) {
			case 'active':
				return 'telemetry-led-active';
			case 'hibernating':
				return 'telemetry-led-hibernating';
			case 'busy':
				return 'telemetry-led-busy';
			case 'stopped':
				return 'telemetry-led-stopped';
			case 'crash':
				return 'telemetry-led-crash';
			default:
				return 'telemetry-led-active';
		}
	});

	const statusLabel = $derived(() => {
		switch (status) {
			case 'active':
				return 'En ligne / Actif';
			case 'hibernating':
				return 'Veille lazymc';
			case 'busy':
				return 'Tâche de fond';
			case 'stopped':
				return 'Arrêté';
			case 'crash':
				return 'Incident / Crash';
			default:
				return 'État inconnu';
		}
	});
</script>

<div
	class="chipanel-logo-container {className}"
	style="width: {size}px; height: {size}px; min-width: {size}px; min-height: {size}px;"
	title="ChiPanel — {statusLabel()}"
	role="img"
	aria-label="ChiPanel Logo ({statusLabel()})"
>
	<svg
		class="chipanel-logo-svg"
		viewBox="0 0 32 32"
		fill="none"
		xmlns="http://www.w3.org/2000/svg"
	>
		<defs>
			<linearGradient id="hexPlateGrad" x1="16" y1="2" x2="16" y2="30" gradientUnits="userSpaceOnUse">
				<stop offset="0%" stop-color="#242836" />
				<stop offset="100%" stop-color="#141720" />
			</linearGradient>
			<linearGradient id="chiStrokeGrad" x1="8" y1="8" x2="24" y2="24" gradientUnits="userSpaceOnUse">
				<stop offset="0%" stop-color="#34D399" />
				<stop offset="100%" stop-color="#0FA968" />
			</linearGradient>
			<linearGradient id="bezelGrad" x1="16" y1="2" x2="16" y2="30" gradientUnits="userSpaceOnUse">
				<stop offset="0%" stop-color="rgba(255, 255, 255, 0.22)" />
				<stop offset="45%" stop-color="rgba(255, 255, 255, 0.06)" />
				<stop offset="100%" stop-color="rgba(0, 0, 0, 0.45)" />
			</linearGradient>
		</defs>

		<!-- Outer Machined Hexagon Plate -->
		<path
			d="M16 2.5 L28 9.428 V22.572 L16 29.5 L4 22.572 V9.428 L16 2.5 Z"
			fill="url(#hexPlateGrad)"
			stroke="url(#bezelGrad)"
			stroke-width="1.2"
			stroke-linejoin="round"
		/>

		<!-- Inner Stencil Inset -->
		<path
			d="M16 4.8 L26 10.57 V21.43 L16 27.2 L6 21.43 V10.57 L16 4.8 Z"
			fill="#111319"
			stroke="rgba(255, 255, 255, 0.06)"
			stroke-width="0.8"
			stroke-linejoin="round"
		/>

		<!-- Chi Main Diagonal (Top-Left to Bottom-Right) -->
		<path
			d="M10 9.5 L22 22.5"
			stroke="url(#chiStrokeGrad)"
			stroke-width="2.5"
			stroke-linecap="round"
		/>

		<!-- Chi Stencil Secondary Diagonal (Top-Right Segment) -->
		<path
			d="M22 9.5 L17.5 13.75"
			stroke="url(#chiStrokeGrad)"
			stroke-width="2.5"
			stroke-linecap="round"
		/>

		<!-- Chi Stencil Secondary Diagonal (Bottom-Left Segment) -->
		<path
			d="M14.5 17 L10 22.5"
			stroke="url(#chiStrokeGrad)"
			stroke-width="2.5"
			stroke-linecap="round"
		/>

		<!-- Stencil Micro-Bridge / Hardware Bolt Highlight -->
		<circle cx="16" cy="15.5" r="0.9" fill="#34D399" opacity="0.75" />
	</svg>

	{#if showLed}
		<span class="logo-led-dot telemetry-led {statusLedClass()}"></span>
	{/if}
</div>

<style>
	.chipanel-logo-container {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		position: relative;
		flex-shrink: 0;
		user-select: none;
	}

	.chipanel-logo-svg {
		width: 100%;
		height: 100%;
		display: block;
		filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.4));
		transition: transform 160ms var(--ease-out);
	}

	.chipanel-logo-container:hover .chipanel-logo-svg {
		transform: scale(1.04);
	}

	.logo-led-dot {
		position: absolute;
		bottom: -1px;
		right: -1px;
		width: 7px;
		height: 7px;
		border: 1.5px solid var(--bg-surface);
		border-radius: 50%;
		box-sizing: content-box;
	}
</style>
