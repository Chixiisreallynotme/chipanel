<script>
	import { Cpu } from 'lucide-svelte';

	let {
		engine = '',
		size = 40,
		class: className = ''
	} = $props();

	const engineUpper = $derived((engine || '').trim().toUpperCase());
</script>

<div
	class="engine-logo-container {className}"
	style="width: {size}px; height: {size}px; min-width: {size}px; min-height: {size}px;"
	title={engineUpper}
>
	{#if engineUpper === 'FABRIC'}
		<!-- Official FabricMC Loom / Spool Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#1e2433" stroke="#2d3748" stroke-width="2"/>
			<circle cx="50" cy="50" r="34" fill="#dfb675" fill-opacity="0.12"/>
			<!-- Spool Structure -->
			<rect x="30" y="24" width="40" height="10" rx="3" fill="#dfb675"/>
			<rect x="30" y="66" width="40" height="10" rx="3" fill="#dfb675"/>
			<rect x="38" y="34" width="24" height="32" fill="#60a5fa"/>
			<!-- Cross threads -->
			<line x1="38" y1="40" x2="62" y2="40" stroke="#93c5fd" stroke-width="2.5"/>
			<line x1="38" y1="50" x2="62" y2="50" stroke="#93c5fd" stroke-width="2.5"/>
			<line x1="38" y1="60" x2="62" y2="60" stroke="#93c5fd" stroke-width="2.5"/>
			<line x1="44" y1="34" x2="44" y2="66" stroke="#2563eb" stroke-width="2"/>
			<line x1="56" y1="34" x2="56" y2="66" stroke="#2563eb" stroke-width="2"/>
			<circle cx="50" cy="50" r="4.5" fill="#ffffff"/>
		</svg>

	{:else if engineUpper === 'PURPUR'}
		<!-- Official Purpur Squid Silhouette Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#241434" stroke="#442366" stroke-width="2"/>
			<!-- Squid Head -->
			<path d="M28 42 C28 26, 40 18, 50 18 C60 18, 72 26, 72 42 L72 68 L64 68 L64 54 C64 52, 60 52, 60 54 L60 76 L54 76 L54 54 C54 52, 50 52, 50 54 L50 78 L46 78 L46 54 C46 52, 42 52, 42 54 L42 76 L36 76 L36 54 C36 52, 32 52, 32 54 L32 68 L28 68 Z" fill="url(#purpur-svg-grad)"/>
			<!-- Eyes -->
			<ellipse cx="40" cy="38" rx="4" ry="4" fill="#ffffff"/>
			<circle cx="40" cy="38" r="2" fill="#1a0b2e"/>
			<ellipse cx="60" cy="38" rx="4" ry="4" fill="#ffffff"/>
			<circle cx="60" cy="38" r="2" fill="#1a0b2e"/>
			<defs>
				<linearGradient id="purpur-svg-grad" x1="28" y1="18" x2="72" y2="78" gradientUnits="userSpaceOnUse">
					<stop stop-color="#e879f9"/>
					<stop offset="0.6" stop-color="#c084fc"/>
					<stop offset="1" stop-color="#9333ea"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'PAPER'}
		<!-- Official PaperMC Geometric Origami Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#0f223d" stroke="#1e3a5f" stroke-width="2"/>
			<!-- Origami Swan/Paper Wings -->
			<path d="M50 20 L22 72 L78 72 Z" fill="#2563eb"/>
			<path d="M50 20 L36 72 L64 72 Z" fill="#3b82f6"/>
			<path d="M50 20 L44 72 L50 82 L56 72 Z" fill="#60a5fa"/>
			<path d="M50 72 L36 82 L64 82 Z" fill="#1d4ed8"/>
		</svg>

	{:else if engineUpper === 'FORGE'}
		<!-- Official Minecraft Forge Heavy Anvil & Ember Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#2c1a14" stroke="#522b1f" stroke-width="2"/>
			<!-- Anvil -->
			<path d="M22 28 H78 C81 28, 83 31, 81 35 L74 48 C72 52, 68 55, 63 55 H58 V66 H68 C70 66, 72 68, 72 70 V74 H28 V70 C28 68, 30 66, 32 66 H42 V55 H37 C32 55, 28 52, 26 48 L19 35 C17 31, 19 28, 22 28 Z" fill="url(#forge-svg-grad)"/>
			<rect x="28" y="33" width="44" height="6" rx="2" fill="#1e1815"/>
			<!-- Glowing Core Flame -->
			<path d="M50 42 C44 48, 46 54, 50 58 C54 54, 56 48, 50 42 Z" fill="#fbbf24"/>
			<defs>
				<linearGradient id="forge-svg-grad" x1="20" y1="28" x2="80" y2="74" gradientUnits="userSpaceOnUse">
					<stop stop-color="#f97316"/>
					<stop offset="0.7" stop-color="#ea580c"/>
					<stop offset="1" stop-color="#9a3412"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'NEOFORGE'}
		<!-- Official NeoForge Fiery Crest Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#2b1713" stroke="#54251d" stroke-width="2"/>
			<!-- Outer Flame -->
			<path d="M50 18 C34 34, 24 50, 24 64 C24 78, 36 84, 50 84 C64 84, 76 78, 76 64 C76 50, 66 34, 50 18 Z" fill="url(#neoforge-svg-grad)"/>
			<!-- Inner Flame -->
			<path d="M50 36 C40 48, 34 58, 34 68 C34 76, 41 80, 50 80 C59 80, 66 76, 66 68 C66 58, 60 48, 50 36 Z" fill="#fbbf24"/>
			<!-- Core Spark -->
			<path d="M50 54 C46 60, 44 64, 44 70 C44 74, 47 76, 50 76 C53 76, 56 74, 56 70 C56 64, 54 60, 50 54 Z" fill="#ffffff"/>
			<defs>
				<linearGradient id="neoforge-svg-grad" x1="24" y1="18" x2="76" y2="84" gradientUnits="userSpaceOnUse">
					<stop stop-color="#ef4444"/>
					<stop offset="0.5" stop-color="#f97316"/>
					<stop offset="1" stop-color="#ea580c"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'SPIGOT'}
		<!-- Official SpigotMC Brass Tap / Faucet Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#291e13" stroke="#4d351b" stroke-width="2"/>
			<!-- Faucet Top Handle -->
			<rect x="42" y="20" width="16" height="10" rx="3" fill="#f59e0b"/>
			<rect x="30" y="30" width="40" height="8" rx="2" fill="#d97706"/>
			<!-- Pipe & Spout -->
			<path d="M38 38 H62 V58 C62 64, 56 68, 50 68 H42 V60 H50 C52 60, 54 58, 54 56 V38 H38 Z" fill="#b45309"/>
			<!-- Water Droplet -->
			<path d="M46 74 C43 78, 42 81, 46 84 C50 84, 51 81, 46 74 Z" fill="#38bdf8"/>
		</svg>

	{:else if engineUpper === 'VANILLA'}
		<!-- Official Minecraft Isometric 3D Grass Block Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#192617" stroke="#2c4228" stroke-width="2"/>
			<!-- Isometric Grass Top -->
			<polygon points="50,22 78,38 50,54 22,38" fill="#4ade80"/>
			<polygon points="50,22 78,38 72,41 50,29 28,41 22,38" fill="#86efac"/>
			<!-- Left Dirt Wall -->
			<polygon points="22,38 50,54 50,80 22,64" fill="#78350f"/>
			<!-- Right Dirt Wall -->
			<polygon points="50,54 78,38 78,64 50,80" fill="#451a03"/>
			<!-- Grass Edge Fringe Left -->
			<polygon points="22,38 50,54 50,60 44,57 40,61 34,57 28,60 22,56" fill="#22c55e"/>
			<!-- Grass Edge Fringe Right -->
			<polygon points="50,54 78,38 78,56 72,60 66,56 60,61 56,57 50,60" fill="#16a34a"/>
		</svg>

	{:else if engineUpper === 'QUILT'}
		<!-- Official QuiltMC 4-Panel Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#20152e" stroke="#3b2457" stroke-width="2"/>
			<!-- 4 Quilt Squares -->
			<rect x="26" y="26" width="22" height="22" rx="4" fill="#8b5cf6"/>
			<rect x="52" y="26" width="22" height="22" rx="4" fill="#06b6d4"/>
			<rect x="26" y="52" width="22" height="22" rx="4" fill="#ec4899"/>
			<rect x="52" y="52" width="22" height="22" rx="4" fill="#f59e0b"/>
			<!-- Center Node -->
			<circle cx="50" cy="50" r="7" fill="#ffffff"/>
			<circle cx="50" cy="50" r="3.5" fill="#7c3aed"/>
		</svg>

	{:else if engineUpper === 'FOLIA'}
		<!-- Official Folia Multithreaded Leaf Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#132a20" stroke="#204a37" stroke-width="2"/>
			<!-- Folia Leaf -->
			<path d="M26 72 C22 46, 38 22, 76 22 C76 60, 52 76, 26 72 Z" fill="url(#folia-svg-grad)"/>
			<!-- Multithread Veins -->
			<path d="M30 68 C46 54, 60 40, 72 26" stroke="#ffffff" stroke-width="2.5" stroke-linecap="round" stroke-opacity="0.8"/>
			<path d="M46 54 C54 48, 62 44, 66 38" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-opacity="0.8"/>
			<path d="M38 62 C44 58, 50 56, 54 50" stroke="#ffffff" stroke-width="2" stroke-linecap="round" stroke-opacity="0.8"/>
			<defs>
				<linearGradient id="folia-svg-grad" x1="26" y1="22" x2="76" y2="72" gradientUnits="userSpaceOnUse">
					<stop stop-color="#34d399"/>
					<stop offset="1" stop-color="#059669"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'MOHIST'}
		<!-- Official Mohist Alchemical Flask / Crucible Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#132726" stroke="#1d4543" stroke-width="2"/>
			<!-- Flask Body -->
			<path d="M42 22 H58 V36 L74 66 C78 74, 72 82, 64 82 H36 C28 82, 22 74, 26 66 L42 36 V22 Z" fill="url(#mohist-svg-grad)"/>
			<rect x="40" y="20" width="20" height="5" rx="1.5" fill="#0f766e"/>
			<!-- Crimson Solution in Flask -->
			<path d="M32 66 L38 54 H62 L68 66 C70 72, 66 78, 60 78 H40 C34 78, 30 72, 32 66 Z" fill="#ef4444" fill-opacity="0.9"/>
			<circle cx="45" cy="68" r="2.5" fill="#fef08a"/>
			<circle cx="55" cy="64" r="1.5" fill="#fef08a"/>
			<defs>
				<linearGradient id="mohist-svg-grad" x1="26" y1="20" x2="74" y2="82" gradientUnits="userSpaceOnUse">
					<stop stop-color="#2dd4bf"/>
					<stop offset="1" stop-color="#0f766e"/>
				</linearGradient>
			</defs>
		</svg>

	{:else if engineUpper === 'ARCLIGHT'}
		<!-- Official Arclight Electric Arc Crystal Vector -->
		<svg viewBox="0 0 100 100" width="{size}" height="{size}" class="engine-svg">
			<rect width="100" height="100" rx="22" fill="#112933" stroke="#1c4759" stroke-width="2"/>
			<!-- Crystal Diamond -->
			<polygon points="50,18 78,42 50,84 22,42" fill="url(#arclight-svg-grad)"/>
			<polygon points="50,18 78,42 50,84" fill="#22d3ee" fill-opacity="0.3"/>
			<!-- Lightning Spark -->
			<path d="M52 30 L44 48 H52 L46 68 L60 46 H50 L56 30 Z" fill="#facc15"/>
			<defs>
				<linearGradient id="arclight-svg-grad" x1="22" y1="18" x2="78" y2="84" gradientUnits="userSpaceOnUse">
					<stop stop-color="#06b6d4"/>
					<stop offset="1" stop-color="#0e7490"/>
				</linearGradient>
			</defs>
		</svg>

	{:else}
		<!-- Fallback General Clean Chip Icon -->
		<div class="engine-fallback-box">
			<Cpu size={size * 0.55} />
		</div>
	{/if}
</div>

<style>
	.engine-logo-container {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: 14px;
		flex-shrink: 0;
		user-select: none;
		position: relative;
	}

	.engine-svg {
		display: block;
		width: 100%;
		height: 100%;
		border-radius: 14px;
	}

	.engine-fallback-box {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 100%;
		height: 100%;
		background: #1e293b;
		border: 1px solid #334155;
		border-radius: 14px;
		color: #94a3b8;
	}
</style>
