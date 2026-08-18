<script>
	import {
		Package,
		Box,
		Shield,
		Crown,
		RefreshCw,
		AlertCircle,
		Loader2,
		Sparkles,
		Layers,
		Swords,
		Shirt
	} from 'lucide-svelte';

	let {
		inventory = null,
		loading = false,
		error = '',
		onRefresh = () => {}
	} = $props();

	// Active tab state: 'inventory' | 'ender_chest'
	let activeView = $state('inventory');

	// Active hovered/focused item tooltip state
	let activeHoveredItem = $state(null);

	// Image fallback state for broken item icons
	let failedImages = $state({});

	function handleImageError(e, itemId) {
		if (!itemId) return;
		const clean = getCleanItemName(itemId);
		const img = e.currentTarget;
		if (!img) return;
		const currentSrc = img.src || '';

		if (currentSrc.includes('/textures/item/') && currentSrc.includes('mcasset.cloud')) {
			img.src = `https://assets.mcasset.cloud/1.20.4/assets/minecraft/textures/block/${clean}.png`;
			return;
		}
		if (currentSrc.includes('/textures/block/') && currentSrc.includes('mcasset.cloud')) {
			img.src = `https://cdn.jsdelivr.net/gh/InventivetalentDev/minecraft-assets@1.20.4/assets/minecraft/textures/item/${clean}.png`;
			return;
		}
		if (currentSrc.includes('/textures/item/') && currentSrc.includes('jsdelivr.net')) {
			img.src = `https://cdn.jsdelivr.net/gh/InventivetalentDev/minecraft-assets@1.20.4/assets/minecraft/textures/block/${clean}.png`;
			return;
		}

		failedImages = { ...failedImages, [itemId]: true };
	}

	// Tooltip state handlers
	function showTooltip(item, position, align, id) {
		if (!item) {
			activeHoveredItem = null;
			return;
		}
		activeHoveredItem = { item, position, align, id };
	}

	function hideTooltip(id) {
		if (activeHoveredItem?.id === id) {
			activeHoveredItem = null;
		}
	}

	function getAlign(colIndex) {
		if (colIndex <= 1) return 'left';
		if (colIndex >= 7) return 'right';
		return 'center';
	}

	// Derived armor slots array: [Helmet, Chestplate, Leggings, Boots]
	let armorSlots = $derived.by(() => {
		const raw = inventory?.armor || [];
		return [
			{ type: 'helmet', label: 'Helmet', item: raw[3] || null, placeholder: 'helmet' },
			{ type: 'chestplate', label: 'Chestplate', item: raw[2] || null, placeholder: 'chestplate' },
			{ type: 'leggings', label: 'Leggings', item: raw[1] || null, placeholder: 'leggings' },
			{ type: 'boots', label: 'Boots', item: raw[0] || null, placeholder: 'boots' }
		];
	});

	let offhandSlot = $derived(inventory?.offhand || null);

	// Derived 27 Main Storage slots (indices 9 to 35)
	let storageSlots = $derived.by(() => {
		const main = inventory?.main || [];
		const rawStorage = main.slice(9, 36);
		const slots = [];
		for (let i = 0; i < 27; i++) {
			slots.push(rawStorage[i] || null);
		}
		return slots;
	});

	// Derived 9 Hotbar slots (indices 0 to 8)
	let hotbarSlots = $derived.by(() => {
		const main = inventory?.main || [];
		const rawHotbar = main.slice(0, 9);
		const slots = [];
		for (let i = 0; i < 9; i++) {
			slots.push(rawHotbar[i] || null);
		}
		return slots;
	});

	// Derived 27 Ender Chest slots
	let enderChestSlots = $derived.by(() => {
		const rawEnder = inventory?.ender_chest || [];
		const slots = [];
		for (let i = 0; i < 27; i++) {
			slots.push(rawEnder[i] || null);
		}
		return slots;
	});

	// Item helper functions
	function getCleanItemName(itemId) {
		if (!itemId) return '';
		return itemId.replace(/^minecraft:/, '').toLowerCase();
	}

	// Cache getItemImageUrl string outputs to prevent redundant URL string operations
	const imageUrlCache = new Map();

	function getItemImageUrl(itemId) {
		if (!itemId) return '';
		if (imageUrlCache.has(itemId)) {
			return imageUrlCache.get(itemId);
		}
		const clean = getCleanItemName(itemId);
		const url = clean
			? `https://assets.mcasset.cloud/1.20.4/assets/minecraft/textures/item/${clean}.png`
			: '';
		imageUrlCache.set(itemId, url);
		return url;
	}

	function getFormattedItemName(item) {
		if (!item) return '';
		if (item.display_name) return item.display_name;
		const clean = getCleanItemName(item.item_id);
		return clean
			.split('_')
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(' ');
	}

	function toRoman(num) {
		if (!num || num <= 0) return '';
		const lookup = [
			[10, 'X'],
			[9, 'IX'],
			[5, 'V'],
			[4, 'IV'],
			[1, 'I']
		];
		let roman = '';
		let n = num;
		for (const [val, char] of lookup) {
			while (n >= val) {
				roman += char;
				n -= val;
			}
		}
		return roman || String(num);
	}

	function formatEnchantment(ench) {
		if (!ench) return '';
		const name = ench.name || ench.id;
		const singleLevelNames = [
			'Mending',
			'Silk Touch',
			'Aqua Affinity',
			'Curse of Binding',
			'Curse of Vanishing',
			'Flame',
			'Infinity',
			'Channeling',
			'Multishot'
		];
		if (ench.level === 1 && singleLevelNames.includes(name)) {
			return name;
		}
		const romanLevel = toRoman(ench.level);
		return romanLevel ? `${name} ${romanLevel}` : name;
	}

	function isCurse(ench) {
		if (!ench) return false;
		return ench.id?.includes('curse') || ench.name?.toLowerCase().includes('curse');
	}

	function getDurabilityData(item) {
		if (!item || !item.max_damage || item.max_damage <= 0) return null;
		const damage = item.damage || 0;
		const current = Math.max(0, item.max_damage - damage);
		const ratio = current / item.max_damage;
		const percent = Math.max(0, Math.min(100, ratio * 100));

		let colorClass = 'durability-high';
		if (ratio <= 0.2) {
			colorClass = 'durability-low';
		} else if (ratio <= 0.5) {
			colorClass = 'durability-medium';
		}

		return {
			current,
			max: item.max_damage,
			percent,
			ratio,
			colorClass
		};
	}

	// Total items summary counters
	let totalItemCount = $derived.by(() => {
		if (!inventory) return 0;
		let count = 0;
		const countSlots = (list) => {
			for (const slot of list || []) {
				if (slot) count += slot.count || 1;
			}
		};
		countSlots(inventory.main);
		countSlots(inventory.armor);
		if (inventory.offhand) count += inventory.offhand.count || 1;
		return count;
	});

	let enderItemCount = $derived.by(() => {
		if (!inventory?.ender_chest) return 0;
		let count = 0;
		for (const slot of inventory.ender_chest) {
			if (slot) count += slot.count || 1;
		}
		return count;
	});
</script>

<div class="inventory-visualizer">
	<!-- Control Bar: Toggle Viewers & Actions -->
	<div class="visualizer-header">
		<div class="view-toggle-group">
			<button
				class="toggle-btn {activeView === 'inventory' ? 'active' : ''}"
				onclick={() => (activeView = 'inventory')}
			>
				<Package size={15} />
				<span>Player Inventory</span>
				{#if totalItemCount > 0}
					<span class="count-pill">{totalItemCount}</span>
				{/if}
			</button>
			<button
				class="toggle-btn toggle-ender {activeView === 'ender_chest' ? 'active' : ''}"
				onclick={() => (activeView = 'ender_chest')}
			>
				<Box size={15} class="ender-icon" />
				<span>Ender Chest</span>
				{#if enderItemCount > 0}
					<span class="count-pill ender-pill">{enderItemCount}</span>
				{/if}
			</button>
		</div>

		<button class="btn btn-ghost btn-sm refresh-btn" onclick={onRefresh} disabled={loading} title="Reload NBT Inventory Data">
			<RefreshCw size={14} class={loading ? 'spinner' : ''} />
			<span>Refresh</span>
		</button>
	</div>

	<!-- Loading State -->
	{#if loading}
		<div class="loading-state">
			<Loader2 size={28} class="spinner" />
			<span>Reading player NBT inventory data...</span>
		</div>
	{:else if error}
		<!-- Error State -->
		<div class="error-state">
			<AlertCircle size={24} class="error-icon" />
			<div class="error-info">
				<strong>Unable to load inventory data</strong>
				<p>{error}</p>
			</div>
			<button class="btn btn-secondary btn-sm" onclick={onRefresh}>
				<RefreshCw size={14} />
				<span>Retry</span>
			</button>
		</div>
	{:else}
		<!-- Main Player Inventory View -->
		{#if activeView === 'inventory'}
			<div class="inventory-layout">
				<!-- Equipment Header Section: Armor + Silhouette + Offhand -->
				<div class="equipment-section">
					<!-- Armor Column -->
					<div class="armor-column">
						<span class="column-label">Armor</span>
						<div class="armor-slots">
							{#each armorSlots as slot, i (i)}
								{@const slotId = `armor-${slot.type}`}
								<div
									class="item-slot-wrapper"
									tabindex="0"
									aria-label="{slot.label} slot"
									onmouseenter={() => showTooltip(slot.item, 'bottom', 'left', slotId)}
									onmouseleave={() => hideTooltip(slotId)}
									onfocus={() => showTooltip(slot.item, 'bottom', 'left', slotId)}
									onblur={() => hideTooltip(slotId)}
								>
									<div class="slot-box armor-slot {slot.item ? 'has-item' : 'empty'}">
										{#if slot.item}
											{@render itemContent(slot.item)}
										{:else}
											<div class="placeholder-icon">
												{#if slot.placeholder === 'helmet'}
													<Crown size={18} />
												{:else if slot.placeholder === 'chestplate'}
													<Shirt size={18} />
												{:else if slot.placeholder === 'leggings'}
													<Layers size={18} />
												{:else}
													<Shield size={18} />
												{/if}
											</div>
										{/if}
									</div>
									{#if activeHoveredItem?.id === slotId && activeHoveredItem.item}
										{@render itemTooltip(activeHoveredItem.item, activeHoveredItem.position, activeHoveredItem.align)}
									{/if}
								</div>
							{/each}
						</div>
					</div>

					<!-- Player Silhouette Frame -->
					<div class="player-silhouette-box">
						<div class="silhouette-bg">
							<Shield size={56} class="silhouette-icon" />
						</div>
						<span class="silhouette-label">Equipment Layout</span>
					</div>

					<!-- Offhand Column -->
					<div class="offhand-column">
						<span class="column-label">Offhand</span>
						<div
							class="item-slot-wrapper"
							tabindex="0"
							aria-label="Offhand slot"
							onmouseenter={() => showTooltip(offhandSlot, 'bottom', 'right', 'offhand-slot')}
							onmouseleave={() => hideTooltip('offhand-slot')}
							onfocus={() => showTooltip(offhandSlot, 'bottom', 'right', 'offhand-slot')}
							onblur={() => hideTooltip('offhand-slot')}
						>
							<div class="slot-box offhand-slot {offhandSlot ? 'has-item' : 'empty'}">
								{#if offhandSlot}
									{@render itemContent(offhandSlot)}
								{:else}
									<div class="placeholder-icon">
										<Shield size={18} />
									</div>
								{/if}
							</div>
							{#if activeHoveredItem?.id === 'offhand-slot' && activeHoveredItem.item}
								{@render itemTooltip(activeHoveredItem.item, activeHoveredItem.position, activeHoveredItem.align)}
							{/if}
						</div>
					</div>
				</div>

				<!-- 27-Slot Main Storage Grid (3x9) -->
				<div class="grid-section">
					<div class="section-label">
						<Package size={13} />
						<span>Main Storage (27 Slots)</span>
					</div>
					<div class="minecraft-grid storage-grid">
						{#each storageSlots as item, i (i)}
							{@const slotId = `storage-${i}`}
							{@const col = i % 9}
							{@const pos = i < 9 ? 'bottom' : 'top'}
							{@const align = getAlign(col)}
							<div
								class="item-slot-wrapper"
								tabindex="0"
								aria-label="Storage slot {i + 1}"
								onmouseenter={() => showTooltip(item, pos, align, slotId)}
								onmouseleave={() => hideTooltip(slotId)}
								onfocus={() => showTooltip(item, pos, align, slotId)}
								onblur={() => hideTooltip(slotId)}
							>
								<div class="slot-box {item ? 'has-item' : 'empty'}">
									{#if item}
										{@render itemContent(item)}
									{/if}
								</div>
								{#if activeHoveredItem?.id === slotId && activeHoveredItem.item}
									{@render itemTooltip(activeHoveredItem.item, activeHoveredItem.position, activeHoveredItem.align)}
								{/if}
							</div>
						{/each}
					</div>
				</div>

				<!-- 9-Slot Hotbar Grid -->
				<div class="grid-section hotbar-section">
					<div class="section-label">
						<Swords size={13} />
						<span>Hotbar (9 Slots)</span>
					</div>
					<div class="minecraft-grid hotbar-grid">
						{#each hotbarSlots as item, i (i)}
							{@const slotId = `hotbar-${i}`}
							{@const col = i}
							{@const align = getAlign(col)}
							<div
								class="item-slot-wrapper"
								tabindex="0"
								aria-label="Hotbar slot {i + 1}"
								onmouseenter={() => showTooltip(item, 'top', align, slotId)}
								onmouseleave={() => hideTooltip(slotId)}
								onfocus={() => showTooltip(item, 'top', align, slotId)}
								onblur={() => hideTooltip(slotId)}
							>
								<div class="slot-box hotbar-slot {item ? 'has-item' : 'empty'}">
									{#if item}
										{@render itemContent(item)}
									{/if}
									<span class="slot-key-num">{i + 1}</span>
								</div>
								{#if activeHoveredItem?.id === slotId && activeHoveredItem.item}
									{@render itemTooltip(activeHoveredItem.item, activeHoveredItem.position, activeHoveredItem.align)}
								{/if}
							</div>
						{/each}
					</div>
				</div>
			</div>
		{:else}
			<!-- Ender Chest Storage View -->
			<div class="ender-chest-layout">
				<div class="ender-banner">
					<Box size={22} class="ender-banner-icon" />
					<div>
						<h4 class="ender-title">Ender Chest Vault</h4>
						<p class="ender-subtitle">Dimensions-wide secure player ender storage grid (27 slots).</p>
					</div>
				</div>

				<div class="grid-section">
					<div class="minecraft-grid ender-grid">
						{#each enderChestSlots as item, i (i)}
							{@const slotId = `ender-${i}`}
							{@const col = i % 9}
							{@const pos = i < 9 ? 'bottom' : 'top'}
							{@const align = getAlign(col)}
							<div
								class="item-slot-wrapper"
								tabindex="0"
								aria-label="Ender Chest slot {i + 1}"
								onmouseenter={() => showTooltip(item, pos, align, slotId)}
								onmouseleave={() => hideTooltip(slotId)}
								onfocus={() => showTooltip(item, pos, align, slotId)}
								onblur={() => hideTooltip(slotId)}
							>
								<div class="slot-box ender-slot {item ? 'has-item' : 'empty'}">
									{#if item}
										{@render itemContent(item)}
									{/if}
								</div>
								{#if activeHoveredItem?.id === slotId && activeHoveredItem.item}
									{@render itemTooltip(activeHoveredItem.item, activeHoveredItem.position, activeHoveredItem.align)}
								{/if}
							</div>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	{/if}
</div>

<!-- Item Slot Content Snippet -->
{#snippet itemContent(item)}
	{@const dur = getDurabilityData(item)}
	{@const cleanName = getCleanItemName(item.item_id)}
	{@const hasImgError = failedImages[item.item_id]}

	<div class="item-icon-container">
		{#if !hasImgError && item.item_id}
			<img
				src={getItemImageUrl(item.item_id)}
				alt={getFormattedItemName(item)}
				class="item-icon"
				onerror={(e) => handleImageError(e, item.item_id)}
				loading="lazy"
			/>
		{:else}
			<div class="item-fallback-icon">
				<Box size={20} />
			</div>
		{/if}
	</div>

	{#if item.count > 1}
		<span class="stack-badge">{item.count}</span>
	{/if}

	{#if dur}
		<div class="durability-bar-container" title="Durability: {dur.current} / {dur.max}">
			<div
				class="durability-bar-fill {dur.colorClass}"
				style="transform: scaleX({dur.ratio});"
			></div>
		</div>
	{/if}
{/snippet}

<!-- Rich Item Tooltip Snippet -->
{#snippet itemTooltip(item, position = 'top', align = 'center')}
	{@const dur = getDurabilityData(item)}
	{@const formattedName = getFormattedItemName(item)}

	<div class="item-tooltip tooltip-{position} align-{align}">
		<div class="tooltip-header">
			<span class="item-display-title {item.display_name ? 'is-custom-name' : ''}">
				{formattedName}
			</span>
			{#if item.is_trimmed}
				<span class="trim-tag" title="Armor Trimmed">
					✨ Trimmed
				</span>
			{/if}
		</div>

		{#if dur}
			<div class="tooltip-durability">
				<div class="durability-text">
					<span>Durability:</span>
					<strong>{dur.current} / {dur.max}</strong>
				</div>
				<div class="tooltip-dur-gauge">
					<div class="gauge-fill {dur.colorClass}" style="width: {dur.percent}%;"></div>
				</div>
			</div>
		{/if}

		{#if item.enchantments && item.enchantments.length > 0}
			<div class="tooltip-enchantments">
				{#each item.enchantments as ench, i (i)}
					<div class="ench-row {isCurse(ench) ? 'curse-text' : 'aqua-text'}">
						<span>{formatEnchantment(ench)}</span>
					</div>
				{/each}
			</div>
		{/if}

		<div class="tooltip-footer">
			<span class="item-raw-id">{item.item_id}</span>
		</div>
	</div>
{/snippet}

<style>
	.inventory-visualizer {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		width: 100%;
	}

	/* Header & Controls */
	.visualizer-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		background-color: var(--bg-base);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-card);
		border: 1px solid var(--border);
	}

	.view-toggle-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.toggle-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-2) var(--space-3);
		border-radius: var(--radius-btn);
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: background-color var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast);
	}

	.toggle-btn:hover {
		color: var(--text-primary);
		background-color: rgba(255, 255, 255, 0.04);
	}

	.toggle-btn.active {
		color: var(--accent-blue-text);
		background-color: var(--accent-blue-bg);
		border-color: var(--accent-blue-border);
	}

	.toggle-ender.active {
		color: var(--accent-purple-text);
		background-color: rgba(168, 85, 247, 0.12);
		border-color: rgba(168, 85, 247, 0.3);
	}

	.count-pill {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		background-color: rgba(255, 255, 255, 0.08);
		color: var(--text-secondary);
		padding: 1px 6px;
		border-radius: var(--radius-badge);
	}

	.ender-pill {
		background-color: rgba(168, 85, 247, 0.2);
		color: #D8B4FE;
	}

	.refresh-btn {
		color: var(--text-muted);
	}

	/* Loading & Error States */
	.loading-state,
	.error-state {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		padding: var(--space-8);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		color: var(--text-muted);
		font-size: var(--font-size-sm);
	}

	.error-state {
		background-color: var(--danger-bg);
		border-color: var(--danger-border);
		color: var(--danger-text);
		justify-content: space-between;
	}

	.error-info p {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin-top: 2px;
	}

	.spinner {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* Layout Container */
	.inventory-layout,
	.ender-chest-layout {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		align-items: center;
	}

	/* Equipment Header Section */
	.equipment-section {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-6);
		background-color: var(--bg-base);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		padding: var(--space-4) var(--space-6);
		width: 100%;
		max-width: 480px;
	}

	.armor-column,
	.offhand-column {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
	}

	.column-label,
	.section-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.armor-slots {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.player-silhouette-box {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-2);
		flex: 1;
		height: 196px;
		background: radial-gradient(circle, rgba(99, 102, 241, 0.06) 0%, rgba(0, 0, 0, 0) 70%);
		border: 1px dashed var(--border);
		border-radius: var(--radius-card);
	}

	.silhouette-bg {
		opacity: 0.15;
		color: var(--accent-blue-text);
	}

	.silhouette-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	/* Grids */
	.grid-section {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
		width: 100%;
	}

	.section-label {
		align-self: flex-start;
		margin-left: 2px;
	}

	.minecraft-grid {
		display: grid;
		grid-template-columns: repeat(9, 44px);
		gap: var(--space-1);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-3);
		box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.4);
	}

	.hotbar-section .minecraft-grid {
		border-color: var(--border-focus);
		background-color: rgba(17, 17, 24, 0.8);
	}

	.ender-grid {
		border-color: rgba(139, 92, 246, 0.3);
		background-color: rgba(15, 10, 25, 0.9);
	}

	/* Item Slot Wrapper & Box */
	.item-slot-wrapper {
		position: relative;
		outline: none;
	}

	.item-slot-wrapper:focus-visible .slot-box {
		outline: 2px solid var(--accent-blue);
		outline-offset: 2px;
	}

	.slot-box {
		width: 44px;
		height: 44px;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: border-color var(--transition-fast), background-color var(--transition-fast), box-shadow var(--transition-fast);
		cursor: default;
		user-select: none;
	}

	.slot-box.has-item {
		cursor: pointer;
		background-color: rgba(255, 255, 255, 0.03);
	}

	.item-slot-wrapper:hover .slot-box.has-item,
	.item-slot-wrapper:focus .slot-box.has-item {
		border-color: var(--accent-blue);
		background-color: var(--accent-blue-bg);
		box-shadow: 0 0 10px rgba(99, 102, 241, 0.25);
	}

	.ender-slot.has-item:hover,
	.item-slot-wrapper:focus .ender-slot.has-item {
		border-color: var(--accent-purple, #A855F7);
		background-color: rgba(168, 85, 247, 0.15);
		box-shadow: 0 0 10px rgba(168, 85, 247, 0.25);
	}

	.placeholder-icon {
		color: var(--text-muted);
		opacity: 0.3;
	}

	.slot-key-num {
		position: absolute;
		top: 2px;
		left: 3px;
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--text-muted);
		opacity: 0.5;
		pointer-events: none;
	}

	/* Item Rendering */
	.item-icon-container {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.item-icon {
		width: 32px;
		height: 32px;
		image-rendering: pixelated;
		object-fit: contain;
		filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.5));
	}

	.item-fallback-icon {
		color: var(--accent-blue-text);
		opacity: 0.8;
	}

	.stack-badge {
		position: absolute;
		bottom: 2px;
		right: 3px;
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: var(--font-weight-bold);
		color: #FFFFFF;
		text-shadow: 1px 1px 0 #000, -1px -1px 0 #000, 1px -1px 0 #000, -1px 1px 0 #000;
		line-height: 1;
		pointer-events: none;
	}

	/* Durability Bar */
	.durability-bar-container {
		position: absolute;
		bottom: 2px;
		left: 4px;
		right: 4px;
		height: 3px;
		background-color: rgba(0, 0, 0, 0.7);
		border-radius: 1px;
		overflow: hidden;
		pointer-events: none;
	}

	.durability-bar-fill {
		width: 100%;
		height: 100%;
		transform-origin: left;
		transition: transform var(--transition-fast) var(--ease-out);
	}

	.durability-high {
		background-color: var(--accent-green, #4ADE80);
	}

	.durability-medium {
		background-color: var(--warning, #F59E0B);
	}

	.durability-low {
		background-color: var(--danger, #F43F5E);
	}

	/* Ender Banner */
	.ender-banner {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background: linear-gradient(135deg, rgba(139, 92, 246, 0.15) 0%, rgba(17, 17, 24, 0.9) 100%);
		border: 1px solid rgba(139, 92, 246, 0.25);
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
		width: 100%;
		max-width: 480px;
	}

	.ender-banner-icon {
		color: var(--accent-purple-text);
	}

	.ender-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.ender-subtitle {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	/* Rich Tooltip Component */
	.item-tooltip {
		position: absolute;
		z-index: var(--z-dropdown);
		pointer-events: none;
		background-color: var(--bg-elevated);
		border: 1px solid var(--border-focus);
		border-radius: var(--radius-btn);
		padding: var(--space-3);
		box-shadow: 0 10px 30px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.05);
		min-width: 180px;
		max-width: 240px;
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.tooltip-top {
		bottom: calc(100% + 8px);
	}

	.tooltip-bottom {
		top: calc(100% + 8px);
	}

	.align-center {
		left: 50%;
		transform: translateX(-50%);
	}

	.align-left {
		left: 0;
		transform: translateX(0);
	}

	.align-right {
		right: 0;
		left: auto;
		transform: translateX(0);
	}

	.tooltip-header {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.item-display-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: #FFFFFF;
		line-height: 1.3;
	}

	.item-display-title.is-custom-name {
		color: #FFAA00;
		font-style: italic;
	}

	.trim-tag {
		font-size: 11px;
		font-family: var(--font-mono);
		color: #FFAA00;
		background-color: rgba(255, 170, 0, 0.1);
		border: 1px solid rgba(255, 170, 0, 0.25);
		border-radius: var(--radius-sm);
		padding: 1px 6px;
		align-self: flex-start;
		margin-top: 2px;
	}

	.tooltip-durability {
		display: flex;
		flex-direction: column;
		gap: 4px;
		background-color: rgba(0, 0, 0, 0.3);
		padding: 4px 6px;
		border-radius: var(--radius-sm);
		border: 1px solid var(--border-subtle);
	}

	.durability-text {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 11px;
		color: var(--text-muted);
	}

	.durability-text strong {
		color: var(--text-primary);
		font-family: var(--font-mono);
	}

	.tooltip-dur-gauge {
		height: 4px;
		background-color: rgba(0, 0, 0, 0.5);
		border-radius: 2px;
		overflow: hidden;
	}

	.gauge-fill {
		height: 100%;
	}

	.tooltip-enchantments {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.ench-row {
		font-size: 12px;
		font-family: var(--font-mono);
		line-height: 1.3;
	}

	.aqua-text {
		color: #55FFFF;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
	}

	.curse-text {
		color: #FF5555;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.8);
	}

	.tooltip-footer {
		border-top: 1px solid var(--border-subtle);
		padding-top: 4px;
		margin-top: 2px;
	}

	.item-raw-id {
		font-size: 10px;
		font-family: var(--font-mono);
		color: var(--text-muted);
	}

	/* Responsive grid scaling */
	@media (max-width: 520px) {
		.minecraft-grid {
			grid-template-columns: repeat(9, 36px);
			gap: 4px;
		}

		.slot-box {
			width: 36px;
			height: 36px;
		}

		.item-icon-container,
		.item-icon {
			width: 26px;
			height: 26px;
		}

		.equipment-section {
			padding: var(--space-3);
			gap: var(--space-3);
		}
	}
</style>
