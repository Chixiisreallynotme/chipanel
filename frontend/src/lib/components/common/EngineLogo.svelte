<script>
	import { Cpu } from '$lib/icons.js';

	let {
		engine = '',
		size = 40,
		class: className = ''
	} = $props();

	const engineUpper = $derived((engine || '').trim().toUpperCase());

	const LOGO_MAP = {
		FABRIC: '/engines/fabric.png',
		PURPUR: '/engines/purpur.svg',
		PAPER: '/engines/paper.svg',
		FORGE: '/engines/forge.png',
		NEOFORGE: '/engines/neoforge.png',
		SPIGOT: '/engines/spigot.png',
		VANILLA: '/engines/vanilla.svg',
		QUILT: '/engines/quilt.svg',
		FOLIA: '/engines/folia.svg',
		MOHIST: '/engines/mohist.png',
		ARCLIGHT: '/engines/arclight.png'
	};

	const logoSrc = $derived(LOGO_MAP[engineUpper] || null);
</script>

<div
	class="engine-logo-container {className}"
	style="width: {size}px; height: {size}px; min-width: {size}px; min-height: {size}px;"
	title={engineUpper}
>
	{#if logoSrc}
		<img
			src={logoSrc}
			alt="{engineUpper} official logo"
			class="engine-logo-img"
			width="{size}"
			height="{size}"
			loading="lazy"
			decoding="async"
		/>
	{:else}
		<div class="engine-fallback-icon">
			<Cpu size={Math.round(size * 0.55)} />
		</div>
	{/if}
</div>

<style>
	.engine-logo-container {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		position: relative;
		border-radius: 10px;
		flex-shrink: 0;
		user-select: none;
	}

	.engine-logo-img {
		width: 100%;
		height: 100%;
		object-fit: contain;
		display: block;
		filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.25));
		transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.engine-fallback-icon {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(255, 255, 255, 0.05);
		border-radius: 10px;
		color: #94a3b8;
	}
</style>
