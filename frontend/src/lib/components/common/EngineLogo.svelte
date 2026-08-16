<script>
	import { Cpu } from 'lucide-svelte';

	let {
		engine = '',
		size = 32,
		class: className = ''
	} = $props();

	const engineUpper = $derived((engine || '').trim().toUpperCase());
</script>

<div
	class="engine-logo-box engine-logo-{engineUpper.toLowerCase()} {className}"
	style="width: {size}px; height: {size}px; min-width: {size}px; min-height: {size}px;"
>
	{#if engineUpper === 'FABRIC'}
		<!-- Official FabricMC Spool & Loom SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<path d="M128 16C66.14 16 16 66.14 16 128s50.14 112 112 112 112-50.14 112-112S189.86 16 128 16z" fill="#E8D5B5" fill-opacity="0.15" />
			<!-- Fabric cross threads -->
			<path d="M72 80c0-13.25 10.75-24 24-24h64c13.25 0 24 10.75 24 24v20H72V80z" fill="#C9A368"/>
			<path d="M64 116h128v24H64v-24z" fill="#90B7DE"/>
			<path d="M72 156h112v20c0 13.25-10.75 24-24 24H96c-13.25 0-24-10.75-24-24v-20z" fill="#5F8CB8"/>
			<path d="M96 56v144M160 56v144" stroke="#486D94" stroke-width="8" stroke-linecap="round"/>
			<circle cx="128" cy="128" r="14" fill="#FFFFFF"/>
			<circle cx="128" cy="128" r="8" fill="#3B6590"/>
		</svg>

	{:else if engineUpper === 'PURPUR'}
		<!-- Official Purpur Squid Magenta SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#7B2CBF" fill-opacity="0.15"/>
			<path d="M64 88c0-35.35 28.65-64 64-64s64 28.65 64 64v80c0 12-8 24-20 24s-20-12-20-24v-40c0-6.63-5.37-12-12-12s-12 5.37-12 12v64c0 12-8 24-20 24s-20-12-20-24v-64c0-6.63-5.37-12-12-12s-12 5.37-12 12v40c0 12-8 24-20 24s-20-12-20-24V88z" fill="url(#purpur-grad)"/>
			<circle cx="104" cy="96" r="12" fill="#FFFFFF"/>
			<circle cx="104" cy="96" r="6" fill="#180033"/>
			<circle cx="152" cy="96" r="12" fill="#FFFFFF"/>
			<circle cx="152" cy="96" r="6" fill="#180033"/>
			<defs>
				<linearGradient id="purpur-grad" x1="64" y1="24" x2="192" y2="216" gradientUnits="userSpaceOnUse">
					<stop stop-color="#E0AAFF"/>
					<stop offset="0.5" stop-color="#C77DFF"/>
					<stop offset="1" stop-color="#9D4EDD"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'PAPER'}
		<!-- Official PaperMC Origami Swan/Paper SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#1F75FE" fill-opacity="0.15"/>
			<path d="M128 32L48 184h160L128 32z" fill="#3B82F6"/>
			<path d="M128 32L88 184h80L128 32z" fill="#60A5FA"/>
			<path d="M128 32l-40 152 40 40 40-40L128 32z" fill="#93C5FD"/>
			<path d="M128 184l-40 40h80l-40-40z" fill="#2563EB"/>
		</svg>

	{:else if engineUpper === 'FORGE'}
		<!-- Official Minecraft Forge Anvil & Hammer SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#DF6526" fill-opacity="0.15"/>
			<!-- Anvil Body -->
			<path d="M52 64h152c8 0 16 8 12 20l-16 44c-4 12-16 20-28 20h-16v28h32c6 0 12 6 12 12v16H52v-16c0-6 6-12 12-12h32v-28H80c-12 0-24-8-28-20L36 84c-4-12 4-20 16-20z" fill="url(#forge-grad)"/>
			<path d="M72 80h112v20H72V80z" fill="#272528"/>
			<!-- Forge Fire Accent -->
			<path d="M128 100c-16 24-8 40 0 56 8-16 16-32 0-56z" fill="#FBBF24"/>
			<defs>
				<linearGradient id="forge-grad" x1="36" y1="64" x2="220" y2="204" gradientUnits="userSpaceOnUse">
					<stop stop-color="#E26526"/>
					<stop offset="1" stop-color="#A5350F"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'NEOFORGE'}
		<!-- Official NeoForge Flame Shield SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#FF5722" fill-opacity="0.15"/>
			<path d="M128 24C80 72 48 112 48 160c0 44.18 35.82 80 80 80s80-35.82 80-80c0-48-32-88-80-136z" fill="url(#neoforge-grad)"/>
			<path d="M128 72c-28 32-48 60-48 88 0 26.51 21.49 48 48 48s48-21.49 48-48c0-28-20-56-48-88z" fill="#FFC107"/>
			<path d="M128 120c-12 16-20 28-20 40 0 11.05 8.95 20 20 20s20-8.95 20-20c0-12-8-24-20-40z" fill="#FFF"/>
			<defs>
				<linearGradient id="neoforge-grad" x1="48" y1="24" x2="208" y2="240" gradientUnits="userSpaceOnUse">
					<stop stop-color="#FF3D00"/>
					<stop offset="0.6" stop-color="#FF6E40"/>
					<stop offset="1" stop-color="#FF9100"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'SPIGOT'}
		<!-- Spigot MC Faucet / Tap SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#ED820A" fill-opacity="0.15"/>
			<path d="M108 40h40v36h-40V40z" fill="#F59E0B"/>
			<path d="M72 76h112v28H72V76z" fill="#D97706"/>
			<path d="M96 104h64v56c0 22.09-17.91 40-40 40H80v-28h40c6.63 0 12-5.37 12-12v-56z" fill="#B45309"/>
			<!-- Water Drop -->
			<path d="M96 200c-8 12 0 24 12 24s20-12 12-24c-4-6-8-10-12-16-4 6-8 10-12 16z" fill="#38BDF8"/>
		</svg>

	{:else if engineUpper === 'VANILLA'}
		<!-- Iconic Minecraft Grass Block Isometric 3D SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#5E9732" fill-opacity="0.15"/>
			<!-- Isometric Grass Top -->
			<path d="M128 32l84 48-84 48-84-48 84-48z" fill="#5EB839"/>
			<path d="M128 32l84 48-24 14-84-48 24-14z" fill="#75D44C"/>
			<!-- Dirt Left -->
			<path d="M44 80l84 48v96l-84-48V80z" fill="#866043"/>
			<!-- Dirt Right -->
			<path d="M128 128l84-48v96l-84 48V128z" fill="#573D26"/>
			<!-- Grass Side Fringe Left -->
			<path d="M44 80l84 48v20l-16-9-16 12-20-11-16 10-16-12V80z" fill="#4B8E2A"/>
			<!-- Grass Side Fringe Right -->
			<path d="M128 128l84-48v30l-20 8-16-12-18 10-16-9-14 8v-35z" fill="#3D7422"/>
		</svg>

	{:else if engineUpper === 'QUILT'}
		<!-- Official QuiltMC Kaleidoscope Purple / Cyan SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#9333EA" fill-opacity="0.15"/>
			<!-- Quilt 4 Squares -->
			<path d="M48 48h72v72H48V48z" fill="#8B5CF6"/>
			<path d="M136 48h72v72h-72V48z" fill="#06B6D4"/>
			<path d="M48 136h72v72H48v-72z" fill="#EC4899"/>
			<path d="M136 136h72v72h-72v-72z" fill="#F59E0B"/>
			<!-- Center Cross -->
			<path d="M128 32v192M32 128h192" stroke="#1E1B4B" stroke-width="12" stroke-linecap="round"/>
			<circle cx="128" cy="128" r="20" fill="#FFFFFF"/>
			<circle cx="128" cy="128" r="10" fill="#7C3AED"/>
		</svg>

	{:else if engineUpper === 'FOLIA'}
		<!-- Official Folia Multithreaded Leaf SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#10B981" fill-opacity="0.15"/>
			<!-- Leaf Body -->
			<path d="M56 184C48 120 88 48 200 48c0 112-72 152-136 144l-8-8z" fill="url(#folia-grad)"/>
			<!-- Leaf Veins (threads) -->
			<path d="M64 176C112 144 152 104 184 64M112 144c24-12 44-32 56-48M92 160c16-8 32-20 40-32" stroke="#FFFFFF" stroke-width="8" stroke-linecap="round" stroke-opacity="0.75"/>
			<defs>
				<linearGradient id="folia-grad" x1="56" y1="48" x2="200" y2="192" gradientUnits="userSpaceOnUse">
					<stop stop-color="#34D399"/>
					<stop offset="1" stop-color="#059669"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'MOHIST'}
		<!-- Official Mohist Cauldron & Flask SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#0D9488" fill-opacity="0.15"/>
			<!-- Cauldron Flask -->
			<path d="M104 36h48v36l48 84c16 28-4 64-36 64H92c-32 0-52-36-36-64l48-84V36z" fill="url(#mohist-grad)"/>
			<path d="M96 36h64v16H96V36z" fill="#0F766E"/>
			<!-- Liquid Bubbles -->
			<path d="M80 160c20-10 40 10 60 0 16-8 24-2 36 0v28c0 17.67-14.33 32-32 32h-36c-17.67 0-32-14.33-32-32v-28h4z" fill="#EF4444" fill-opacity="0.85"/>
			<circle cx="112" cy="180" r="6" fill="#FEF08A"/>
			<circle cx="140" cy="170" r="4" fill="#FEF08A"/>
			<defs>
				<linearGradient id="mohist-grad" x1="56" y1="36" x2="200" y2="220" gradientUnits="userSpaceOnUse">
					<stop stop-color="#2DD4BF"/>
					<stop offset="1" stop-color="#0F766E"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'ARCLIGHT'}
		<!-- Official Arclight Electric Arc Crystal SVG Logo -->
		<svg viewBox="0 0 256 256" width="{size}" height="{size}" fill="none" xmlns="http://www.w3.org/2000/svg">
			<rect width="256" height="256" rx="48" fill="#06B6D4" fill-opacity="0.15"/>
			<!-- Electric Diamond Crystal -->
			<path d="M128 24l72 64-72 144-72-144 72-64z" fill="url(#arclight-grad)"/>
			<path d="M128 24v208l72-144-72-64z" fill="#22D3EE" fill-opacity="0.4"/>
			<!-- Lightning Bolt Arc -->
			<path d="M136 64l-24 48h24l-16 48 32-56h-24l16-40h-8z" fill="#FACC15"/>
			<defs>
				<linearGradient id="arclight-grad" x1="56" y1="24" x2="200" y2="232" gradientUnits="userSpaceOnUse">
					<stop stop-color="#06B6D4"/>
					<stop offset="0.5" stop-color="#0891B2"/>
					<stop offset="1" stop-color="#0E7490"/>
				</linearGradient>
			</defs>
		</svg>

	{:else}
		<!-- Fallback General Engine Icon -->
		<div class="fallback-icon">
			<Cpu size={size * 0.75} />
		</div>
	{/if}
</div>

<style>
	.engine-logo-box {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-lg, 12px);
		overflow: hidden;
		flex-shrink: 0;
		transition: transform 0.2s ease, filter 0.2s ease;
	}

	.fallback-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		color: var(--text-secondary, #94a3b8);
	}
</style>
