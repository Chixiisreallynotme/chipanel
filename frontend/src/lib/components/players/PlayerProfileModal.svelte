<script>
	import { onMount, onDestroy } from 'svelte';
	import { apiGet, apiPost, apiDelete } from '$lib/api/client.js';
	import { UNAVAILABLE } from '$lib/components/dashboard/serverState.js';
	import InventoryVisualizer from './InventoryVisualizer.svelte';
	import StatusEffectsPanel from './StatusEffectsPanel.svelte';
	import BasicPermissionsPanel from './BasicPermissionsPanel.svelte';
	import {
		X,
		Crown,
		UserX,
		ShieldAlert,
		ShieldCheck,
		Navigation,
		Heart,
		Utensils,
		Zap,
		Clock,
		MapPin,
		Globe,
		Calendar,
		Sparkles,
		CheckCircle2,
		AlertCircle,
		Loader2,
		Package,
		Activity,
		Key,
		Gift,
		RefreshCw,
		Flame,
		Skull,
		Trash2,
		UserCheck,
		Search,
		Plus,
		Check,
		Gamepad2,
		Sliders
	} from 'lucide-svelte';

	/**
	 * @typedef {Object} PlayerDetail
	 * @property {string} uuid
	 * @property {string} username
	 * @property {boolean} is_online
	 * @property {number} health
	 * @property {number} max_health
	 * @property {number} food
	 * @property {number} exp_level
	 * @property {number} exp_progress
	 * @property {number} position_x
	 * @property {number} position_y
	 * @property {number} position_z
	 * @property {string} dimension
	 * @property {number} playtime_seconds
	 * @property {number} first_joined_timestamp
	 * @property {number} last_joined_timestamp
	 * @property {boolean} is_op
	 * @property {boolean} is_banned
	 * @property {string|null} ban_reason
	 */

	let {
		open = false,
		player = null, // Initial summary player or object containing uuid
		onClose = () => {},
		onActionSuccess = (action, message) => {}
	} = $props();

	// Detailed state fetched from API
	let detail = $state(null);
	let loadingDetail = $state(false);
	let actionLoading = $state(null); // 'kick' | 'ban' | 'pardon' | 'teleport' | 'op' | 'deop' | 'gamemode' | 'heal' | 'feed' | 'kill' | 'clear' | 'give' | 'whitelist_add' | 'whitelist_remove'
	let errorMessage = $state('');

	// Inventory NBT state
	let inventoryData = $state(null);
	let loadingInventory = $state(false);
	let inventoryError = $state('');

	// Pending Commands queue state
	let pendingCommands = $state([]);
	let loadingPending = $state(false);

	// Action Form inputs
	let kickReason = $state('Kicked by admin via ChiPanel');
	let banReason = $state('Banned by admin via ChiPanel');
	let tpX = $state(0);
	let tpY = $state(64);
	let tpZ = $state(0);
	let activeTab = $state('overview'); // 'overview' | 'inventory' | 'effects' | 'give' | 'permissions' | 'moderation'

	// Gamemode selection
	let selectedGamemode = $state('survival');

	// Give Items state
	let giveSearch = $state('');
	let giveCategory = $state('all');
	let giveSelectedItemId = $state('minecraft:diamond');
	let giveSelectedCount = $state(64);
	let giveCustomItemId = $state('');

	// Auto-refresh timer inside modal
	let refreshInterval = null;

	const POPULAR_ITEMS = [
		// Resources & Ores
		{ id: 'minecraft:diamond', name: 'Diamond', cat: 'resources' },
		{ id: 'minecraft:diamond_block', name: 'Diamond Block', cat: 'resources' },
		{ id: 'minecraft:netherite_ingot', name: 'Netherite Ingot', cat: 'resources' },
		{ id: 'minecraft:netherite_block', name: 'Netherite Block', cat: 'resources' },
		{ id: 'minecraft:iron_ingot', name: 'Iron Ingot', cat: 'resources' },
		{ id: 'minecraft:iron_block', name: 'Iron Block', cat: 'resources' },
		{ id: 'minecraft:gold_ingot', name: 'Gold Ingot', cat: 'resources' },
		{ id: 'minecraft:gold_block', name: 'Gold Block', cat: 'resources' },
		{ id: 'minecraft:emerald', name: 'Emerald', cat: 'resources' },
		{ id: 'minecraft:emerald_block', name: 'Emerald Block', cat: 'resources' },
		{ id: 'minecraft:lapis_lazuli', name: 'Lapis Lazuli', cat: 'resources' },
		{ id: 'minecraft:redstone', name: 'Redstone', cat: 'resources' },
		{ id: 'minecraft:coal', name: 'Coal', cat: 'resources' },
		{ id: 'minecraft:copper_ingot', name: 'Copper Ingot', cat: 'resources' },
		{ id: 'minecraft:amethyst_shard', name: 'Amethyst Shard', cat: 'resources' },
		{ id: 'minecraft:ancient_debris', name: 'Ancient Debris', cat: 'resources' },

		// Weapons
		{ id: 'minecraft:netherite_sword', name: 'Netherite Sword', cat: 'weapons' },
		{ id: 'minecraft:diamond_sword', name: 'Diamond Sword', cat: 'weapons' },
		{ id: 'minecraft:iron_sword', name: 'Iron Sword', cat: 'weapons' },
		{ id: 'minecraft:netherite_axe', name: 'Netherite Axe', cat: 'weapons' },
		{ id: 'minecraft:diamond_axe', name: 'Diamond Axe', cat: 'weapons' },
		{ id: 'minecraft:bow', name: 'Bow', cat: 'weapons' },
		{ id: 'minecraft:crossbow', name: 'Crossbow', cat: 'weapons' },
		{ id: 'minecraft:trident', name: 'Trident', cat: 'weapons' },
		{ id: 'minecraft:mace', name: 'Mace', cat: 'weapons' },
		{ id: 'minecraft:arrow', name: 'Arrow', cat: 'weapons' },
		{ id: 'minecraft:spectral_arrow', name: 'Spectral Arrow', cat: 'weapons' },
		{ id: 'minecraft:shield', name: 'Shield', cat: 'weapons' },

		// Tools
		{ id: 'minecraft:netherite_pickaxe', name: 'Netherite Pickaxe', cat: 'tools' },
		{ id: 'minecraft:diamond_pickaxe', name: 'Diamond Pickaxe', cat: 'tools' },
		{ id: 'minecraft:netherite_shovel', name: 'Netherite Shovel', cat: 'tools' },
		{ id: 'minecraft:diamond_shovel', name: 'Diamond Shovel', cat: 'tools' },
		{ id: 'minecraft:netherite_hoe', name: 'Netherite Hoe', cat: 'tools' },
		{ id: 'minecraft:diamond_hoe', name: 'Diamond Hoe', cat: 'tools' },
		{ id: 'minecraft:flint_and_steel', name: 'Flint and Steel', cat: 'tools' },
		{ id: 'minecraft:shears', name: 'Shears', cat: 'tools' },
		{ id: 'minecraft:fishing_rod', name: 'Fishing Rod', cat: 'tools' },
		{ id: 'minecraft:spyglass', name: 'Spyglass', cat: 'tools' },
		{ id: 'minecraft:compass', name: 'Compass', cat: 'tools' },
		{ id: 'minecraft:clock', name: 'Clock', cat: 'tools' },
		{ id: 'minecraft:lead', name: 'Lead', cat: 'tools' },
		{ id: 'minecraft:name_tag', name: 'Name Tag', cat: 'tools' },

		// Armor
		{ id: 'minecraft:netherite_helmet', name: 'Netherite Helmet', cat: 'armor' },
		{ id: 'minecraft:netherite_chestplate', name: 'Netherite Chestplate', cat: 'armor' },
		{ id: 'minecraft:netherite_leggings', name: 'Netherite Leggings', cat: 'armor' },
		{ id: 'minecraft:netherite_boots', name: 'Netherite Boots', cat: 'armor' },
		{ id: 'minecraft:diamond_helmet', name: 'Diamond Helmet', cat: 'armor' },
		{ id: 'minecraft:diamond_chestplate', name: 'Diamond Chestplate', cat: 'armor' },
		{ id: 'minecraft:diamond_leggings', name: 'Diamond Leggings', cat: 'armor' },
		{ id: 'minecraft:diamond_boots', name: 'Diamond Boots', cat: 'armor' },
		{ id: 'minecraft:elytra', name: 'Elytra', cat: 'armor' },
		{ id: 'minecraft:turtle_helmet', name: 'Turtle Helmet', cat: 'armor' },

		// Food & Potions
		{ id: 'minecraft:enchanted_golden_apple', name: 'Enchanted Golden Apple', cat: 'food' },
		{ id: 'minecraft:golden_apple', name: 'Golden Apple', cat: 'food' },
		{ id: 'minecraft:golden_carrot', name: 'Golden Carrot', cat: 'food' },
		{ id: 'minecraft:cooked_beef', name: 'Steak (Cooked Beef)', cat: 'food' },
		{ id: 'minecraft:cooked_porkchop', name: 'Cooked Porkchop', cat: 'food' },
		{ id: 'minecraft:bread', name: 'Bread', cat: 'food' },
		{ id: 'minecraft:baked_potato', name: 'Baked Potato', cat: 'food' },
		{ id: 'minecraft:cake', name: 'Cake', cat: 'food' },
		{ id: 'minecraft:potion', name: 'Potion', cat: 'food' },
		{ id: 'minecraft:splash_potion', name: 'Splash Potion', cat: 'food' },

		// Utility & Magic
		{ id: 'minecraft:totem_of_undying', name: 'Totem of Undying', cat: 'utility' },
		{ id: 'minecraft:ender_pearl', name: 'Ender Pearl', cat: 'utility' },
		{ id: 'minecraft:eye_of_ender', name: 'Eye of Ender', cat: 'utility' },
		{ id: 'minecraft:experience_bottle', name: "Bottle o' Enchanting", cat: 'utility' },
		{ id: 'minecraft:firework_rocket', name: 'Firework Rocket', cat: 'utility' },
		{ id: 'minecraft:saddle', name: 'Saddle', cat: 'utility' },
		{ id: 'minecraft:shulker_box', name: 'Shulker Box', cat: 'utility' },
		{ id: 'minecraft:ender_chest', name: 'Ender Chest', cat: 'utility' },
		{ id: 'minecraft:beacon', name: 'Beacon', cat: 'utility' },
		{ id: 'minecraft:enchanting_table', name: 'Enchanting Table', cat: 'utility' },
		{ id: 'minecraft:anvil', name: 'Anvil', cat: 'utility' },
		{ id: 'minecraft:bookshelf', name: 'Bookshelf', cat: 'utility' },
		{ id: 'minecraft:nether_star', name: 'Nether Star', cat: 'utility' },
		{ id: 'minecraft:tnt', name: 'TNT', cat: 'utility' },
		{ id: 'minecraft:water_bucket', name: 'Water Bucket', cat: 'utility' },
		{ id: 'minecraft:lava_bucket', name: 'Lava Bucket', cat: 'utility' }
	];

	// Filtered catalog items for Give panel
	let filteredItems = $derived.by(() => {
		let list = POPULAR_ITEMS;
		if (giveCategory !== 'all') {
			list = list.filter((item) => item.cat === giveCategory);
		}
		if (giveSearch.trim()) {
			const q = giveSearch.toLowerCase().trim();
			list = list.filter(
				(item) => item.name.toLowerCase().includes(q) || item.id.toLowerCase().includes(q)
			);
		}
		return list;
	});

	// Fetch detailed player stats whenever modal opens or player UUID changes
	$effect(() => {
		if (open && player && player.uuid) {
			fetchPlayerDetail(player.uuid);
			if (activeTab === 'inventory') {
				fetchPlayerInventory(player.uuid);
			}

			// Clear previous timer and set 10s auto-refresh
			if (refreshInterval) clearInterval(refreshInterval);
			refreshInterval = setInterval(() => {
				if (open && player?.uuid) {
					refreshPlayerData(false);
				}
			}, 10000);
		} else if (!open) {
			if (refreshInterval) {
				clearInterval(refreshInterval);
				refreshInterval = null;
			}
			detail = null;
			inventoryData = null;
			inventoryError = '';
			errorMessage = '';
			actionLoading = null;
		}
	});

	onDestroy(() => {
		if (refreshInterval) clearInterval(refreshInterval);
	});

	// Reactively fetch inventory data when user switches to inventory tab
	$effect(() => {
		if (
			open &&
			(detail?.uuid || player?.uuid) &&
			activeTab === 'inventory' &&
			!inventoryData &&
			!loadingInventory
		) {
			fetchPlayerInventory((detail || player).uuid);
		}
	});

	async function refreshPlayerData(showSpinner = true) {
		const targetUuid = (detail || player)?.uuid;
		if (!targetUuid) return;
		if (showSpinner) loadingDetail = true;
		try {
			await fetchPlayerDetail(targetUuid, showSpinner);
			await fetchPendingCommands(targetUuid);
			if (activeTab === 'inventory') {
				await fetchPlayerInventory(targetUuid);
			}
		} finally {
			if (showSpinner) loadingDetail = false;
		}
	}

	async function fetchPendingCommands(uuid) {
		const targetUuid = uuid || (detail || player)?.uuid;
		if (!targetUuid) return;
		loadingPending = true;
		try {
			const res = await apiGet(`/api/players/${encodeURIComponent(targetUuid)}/pending-commands`);
			pendingCommands = Array.isArray(res) ? res : [];
		} catch (err) {
			console.error(`Failed to fetch pending commands for player ${targetUuid}:`, err);
		} finally {
			loadingPending = false;
		}
	}

	async function handleDeletePendingCommand(cmdId) {
		if (!cmdId || actionLoading) return;
		actionLoading = `del_${cmdId}`;
		errorMessage = '';
		try {
			await apiDelete(`/api/players/pending-commands/${encodeURIComponent(cmdId)}`);
			pendingCommands = pendingCommands.filter((c) => c.id !== cmdId);
			onActionSuccess('cancel_pending', 'Commande différée annulée avec succès.');
		} catch (err) {
			console.error(`Failed to delete pending command ${cmdId}:`, err);
			errorMessage = err.message || 'Impossible d\'annuler la commande';
		} finally {
			actionLoading = null;
		}
	}

	function formatRelativeTime(epochSecs) {
		if (!epochSecs) return '';
		const diff = Math.floor(Date.now() / 1000) - epochSecs;
		if (diff < 60) return `il y a ${Math.max(1, diff)}s`;
		if (diff < 3600) return `il y a ${Math.floor(diff / 60)} min`;
		if (diff < 86400) return `il y a ${Math.floor(diff / 3600)} h`;
		return `il y a ${Math.floor(diff / 86400)} j`;
	}

	async function fetchPlayerInventory(uuid) {
		if (!uuid) return;
		loadingInventory = true;
		inventoryError = '';
		try {
			const res = await apiGet(`/api/players/${uuid}/inventory`);
			inventoryData = res;
		} catch (err) {
			console.error(`Failed to fetch inventory for player ${uuid}:`, err);
			inventoryError = err.message || 'Failed to load player inventory';
		} finally {
			loadingInventory = false;
		}
	}

	async function fetchPlayerDetail(uuid, showSpinner = true) {
		if (showSpinner) loadingDetail = true;
		errorMessage = '';
		try {
			const res = await apiGet(`/api/players/${uuid}`);
			detail = res;
			if (res) {
				tpX = res.position_x == null ? 0 : Math.round(res.position_x);
				tpY = res.position_y == null ? 64 : Math.round(res.position_y);
				tpZ = res.position_z == null ? 0 : Math.round(res.position_z);
			}
		} catch (err) {
			console.error(`Failed to fetch detail for player ${uuid}:`, err);
			detail = null;
			errorMessage =
				err.message || 'Failed to load full player detail — showing summary data only.';
		} finally {
			if (showSpinner) loadingDetail = false;
		}
	}

	// Format helpers
	function formatPlaytime(seconds) {
		if (seconds == null) return UNAVAILABLE;
		if (seconds <= 0) return '0 mins';
		const hours = Math.floor(seconds / 3600);
		const mins = Math.floor((seconds % 3600) / 60);

		if (hours >= 24) {
			const days = Math.floor(hours / 24);
			const remHours = hours % 24;
			return `${days}d ${remHours}h ${mins}m`;
		}
		if (hours > 0) {
			return `${hours}h ${mins}m`;
		}
		return `${mins} mins`;
	}

	function formatDate(timestamp) {
		if (!timestamp || timestamp === 0) return 'Unknown';
		const date = new Date(timestamp * 1000);
		return date.toLocaleString(undefined, {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatDimension(dimStr) {
		if (!dimStr) return UNAVAILABLE;
		const d = dimStr.toLowerCase();
		if (d.includes('nether')) return 'Nether';
		if (d.includes('end')) return 'The End';
		if (d.includes('overworld')) return 'Overworld';
		return dimStr.split(':').pop();
	}

	function getDimensionBadgeClass(dimStr) {
		const d = (dimStr || '').toLowerCase();
		if (d.includes('nether')) return 'badge-danger';
		if (d.includes('end')) return 'badge-purple';
		if (d.includes('overworld')) return 'badge-success';
		return 'badge-secondary';
	}

	function vitalPercent(value, max) {
		if (value == null || max == null || !max) return null;
		return Math.min(100, Math.max(0, (value / max) * 100));
	}

	function fmtCoord(v) {
		return v == null ? UNAVAILABLE : v.toFixed(1);
	}

	function getItemIconUrl(itemId) {
		const clean = (itemId || '').replace('minecraft:', '').toLowerCase();
		return clean
			? `https://assets.mcasset.cloud/1.20.4/assets/minecraft/textures/item/${clean}.png`
			: '';
	}

	function handleGiveImageError(e, itemId) {
		const clean = (itemId || '').replace('minecraft:', '').toLowerCase();
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
		img.style.opacity = '0.3';
	}

	// Action Handler: Execute moderation commands via /api/players/action
	async function handleExecuteAction(actionType, extraParams = {}) {
		const target = detail || player;
		if (!target?.uuid || actionLoading) return;
		actionLoading = actionType;
		errorMessage = '';

		let body = {
			uuid: target.uuid,
			action: actionType,
			...extraParams
		};

		if (actionType === 'kick') {
			body.reason = kickReason.trim() || 'Kicked by admin via ChiPanel';
		} else if (actionType === 'ban') {
			body.reason = banReason.trim() || 'Banned by admin via ChiPanel';
		} else if (actionType === 'teleport') {
			body.target_coords = `${tpX} ${tpY} ${tpZ}`;
		} else if (actionType === 'gamemode') {
			body.gamemode = selectedGamemode;
		}

		try {
			const res = await apiPost('/api/players/action', body);

			if (actionType === 'ban') {
				target.is_banned = true;
				target.ban_reason = body.reason;
			} else if (actionType === 'pardon' || actionType === 'unban') {
				target.is_banned = false;
				target.ban_reason = null;
			} else if (actionType === 'op') {
				target.is_op = true;
			} else if (actionType === 'deop') {
				target.is_op = false;
			} else if (actionType === 'kick') {
				target.is_online = false;
			}

			const successMessage =
				res.message || res.output || `Action '${actionType}' executed successfully.`;
			onActionSuccess(actionType, successMessage);

			// Automatically refresh telemetry
			await refreshPlayerData(false);
		} catch (err) {
			console.error(`Failed to execute player action ${actionType}:`, err);
			errorMessage = err.message || `Failed to execute action ${actionType}`;
		} finally {
			actionLoading = null;
		}
	}

	async function handleGiveItemSubmit() {
		const targetItem = giveCustomItemId.trim() || giveSelectedItemId;
		if (!targetItem) {
			errorMessage = 'Please select or enter an item ID to give';
			return;
		}
		const count = parseInt(giveSelectedCount, 10) || 1;
		await handleExecuteAction('give', {
			item_id: targetItem,
			count: Math.min(64, Math.max(1, count))
		});
	}
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'Escape' && open) onClose();
	}}
/>

{#if open && (detail || player)}
	{@const p = detail || player}
	{@const healthPct = vitalPercent(p.health, p.max_health)}
	{@const foodPct = vitalPercent(p.food, 20)}
	{@const expPct = vitalPercent(p.exp_progress, 1)}
	<div class="modal-backdrop" onclick={onClose}>
		<div
			class="modal profile-modal-drawer"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => {
				e.stopPropagation();
				if (e.key === 'Escape') onClose();
			}}
			role="dialog"
			aria-modal="true"
			aria-labelledby="profile-modal-title"
		>
			<!-- Modal Header -->
			<div class="modal-header">
				<div class="player-header-info">
					<img
						src="https://mc-heads.net/avatar/{encodeURIComponent(p.uuid || p.username)}/32"
						alt="{p.username} avatar"
						class="header-avatar"
						width="32"
						height="32"
					/>
					<div>
						<div class="title-row">
							<h2 id="profile-modal-title" class="modal-title">{p.username}</h2>
							{#if p.is_op}
								<span class="badge badge-op">
									<Crown size={12} />
									OP
								</span>
							{/if}
						</div>
						<span class="header-uuid">{p.uuid}</span>
					</div>
				</div>

				<div class="header-right-tools">
					{#if p.is_online}
						<span class="badge badge-success">
							<span class="status-dot status-dot-success status-dot-pulse"></span>
							ONLINE
						</span>
					{:else if p.is_banned}
						<span class="badge badge-danger">BANNED</span>
					{:else}
						<span class="badge badge-secondary">OFFLINE</span>
					{/if}

					<button
						class="btn btn-ghost btn-icon btn-sm"
						onclick={() => refreshPlayerData(true)}
						title="Refresh player telemetry"
						disabled={loadingDetail}
					>
						<RefreshCw size={16} class={loadingDetail ? 'spinning' : ''} />
					</button>

					<button
						class="btn btn-ghost btn-icon btn-sm close-modal-btn"
						onclick={onClose}
						aria-label="Close modal"
					>
						<X size={18} />
					</button>
				</div>
			</div>

			<!-- Sub-Navigation Tabs -->
			<div class="modal-nav-tabs" role="tablist">
				<button
					id="tab-overview"
					class="nav-tab-btn {activeTab === 'overview' ? 'active' : ''}"
					onclick={() => (activeTab = 'overview')}
					role="tab"
					aria-selected={activeTab === 'overview'}
					aria-controls="tabpanel-overview"
				>
					<Sparkles size={15} />
					<span>Overview</span>
				</button>

				<button
					id="tab-inventory"
					class="nav-tab-btn {activeTab === 'inventory' ? 'active' : ''}"
					onclick={() => (activeTab = 'inventory')}
					role="tab"
					aria-selected={activeTab === 'inventory'}
					aria-controls="tabpanel-inventory"
				>
					<Package size={15} />
					<span>Inventory</span>
				</button>

				<button
					id="tab-effects"
					class="nav-tab-btn {activeTab === 'effects' ? 'active' : ''}"
					onclick={() => (activeTab = 'effects')}
					role="tab"
					aria-selected={activeTab === 'effects'}
					aria-controls="tabpanel-effects"
				>
					<Activity size={15} />
					<span>Effects</span>
				</button>

				<button
					id="tab-give"
					class="nav-tab-btn {activeTab === 'give' ? 'active' : ''}"
					onclick={() => (activeTab = 'give')}
					role="tab"
					aria-selected={activeTab === 'give'}
					aria-controls="tabpanel-give"
				>
					<Gift size={15} />
					<span>Give Item</span>
				</button>

				<button
					id="tab-permissions"
					class="nav-tab-btn {activeTab === 'permissions' ? 'active' : ''}"
					onclick={() => (activeTab = 'permissions')}
					role="tab"
					aria-selected={activeTab === 'permissions'}
					aria-controls="tabpanel-permissions"
				>
					<Key size={15} />
					<span>Permissions</span>
				</button>

				<button
					id="tab-moderation"
					class="nav-tab-btn {activeTab === 'moderation' ? 'active' : ''}"
					onclick={() => (activeTab = 'moderation')}
					role="tab"
					aria-selected={activeTab === 'moderation'}
					aria-controls="tabpanel-moderation"
				>
					<ShieldAlert size={15} />
					<span>Moderation</span>
					{#if pendingCommands.length > 0}
						<span class="tab-badge-count">{pendingCommands.length}</span>
					{/if}
				</button>
			</div>

			<!-- Error Alert Display -->
			{#if errorMessage}
				<div class="error-banner">
					<AlertCircle size={18} />
					<span>{errorMessage}</span>
				</div>
			{/if}

			<!-- Modal Scrollable Body -->
			<div class="modal-body profile-body">
				{#if loadingDetail}
					<div class="loading-overlay">
						<Loader2 size={32} class="spinner" />
						<span>Fetching player telemetry data...</span>
					</div>
				{/if}

				{#if activeTab === 'overview'}
					<!-- Overview Tab: 3D Body Viewer + Stats Grid -->
					<div role="tabpanel" id="tabpanel-overview" aria-labelledby="tab-overview">
						<div class="overview-layout">
							<!-- Left Box: 3D Full Body Skin Viewer -->
							<div class="skin-viewer-card">
								<div class="skin-viewer-container">
									<img
										src="https://mc-heads.net/body/{encodeURIComponent(p.uuid || p.username)}/150"
										alt="{p.username}'s 3D full body skin viewer"
										class="skin-body-image"
										width="150"
										height="220"
										loading="eager"
									/>
								</div>
								<span class="skin-caption">3D Skin View</span>
							</div>

							<!-- Right Box: Detailed Stats Grid -->
							<div class="stats-grid">
								<!-- Health -->
								<div class="stat-box">
									<div class="stat-header">
										<Heart size={16} class="icon-danger" />
										<span class="stat-title">Health</span>
									</div>
									<div class="stat-value-container">
										<span class="stat-main-value">
											{p.health == null ? UNAVAILABLE : p.health.toFixed(1)} / {p.max_health ??
												UNAVAILABLE} HP
										</span>
										{#if healthPct == null}
											<div class="gauge-bar gauge-unknown" title="Health not reported for this player"></div>
										{:else}
											<div class="gauge-bar">
												<div class="gauge-fill gauge-fill-success" style="transform: scaleX({(healthPct ?? 0) / 100});"></div>
											</div>
										{/if}
									</div>
								</div>

								<!-- Food -->
								<div class="stat-box">
									<div class="stat-header">
										<Utensils size={16} class="icon-warning" />
										<span class="stat-title">Food Level</span>
									</div>
									<div class="stat-value-container">
										<span class="stat-main-value">{p.food ?? UNAVAILABLE} / 20</span>
										{#if foodPct == null}
											<div class="gauge-bar gauge-unknown" title="Food level not reported for this player"></div>
										{:else}
											<div class="gauge-bar">
												<div class="gauge-fill gauge-fill-warning" style="transform: scaleX({(foodPct ?? 0) / 100});"></div>
											</div>
										{/if}
									</div>
								</div>

								<!-- Experience -->
								<div class="stat-box">
									<div class="stat-header">
										<Zap size={16} class="icon-green" />
										<span class="stat-title">Experience</span>
									</div>
									<div class="stat-value-container">
										<span class="stat-main-value">
											Level {p.exp_level ?? UNAVAILABLE}
											<span class="stat-small-text">
												({expPct == null ? UNAVAILABLE : `${Math.round(expPct)}%`})
											</span>
										</span>
										{#if expPct == null}
											<div class="gauge-bar gauge-unknown" title="Experience not reported for this player"></div>
										{:else}
											<div class="gauge-bar">
												<div class="gauge-fill gauge-fill-primary" style="transform: scaleX({(expPct ?? 0) / 100});"></div>
											</div>
										{/if}
									</div>
								</div>

								<!-- Playtime -->
								<div class="stat-box">
									<div class="stat-header">
										<Clock size={16} class="icon-blue" />
										<span class="stat-title">Total Playtime</span>
									</div>
									<div class="stat-value-container">
										<span class="stat-large-text">{formatPlaytime(p.playtime_seconds)}</span>
									</div>
								</div>

								<!-- Coordinates -->
								<div class="stat-box stat-box-wide">
									<div class="stat-header">
										<MapPin size={16} class="icon-purple" />
										<span class="stat-title">Current Coordinates</span>
									</div>
									<div class="coords-display">
										<div class="coord-pill">
											<span class="coord-axis">X:</span>
											<span class="coord-val">{fmtCoord(p.position_x)}</span>
										</div>
										<div class="coord-pill">
											<span class="coord-axis">Y:</span>
											<span class="coord-val">{fmtCoord(p.position_y)}</span>
										</div>
										<div class="coord-pill">
											<span class="coord-axis">Z:</span>
											<span class="coord-val">{fmtCoord(p.position_z)}</span>
										</div>
									</div>
								</div>

								<!-- Dimension -->
								<div class="stat-box">
									<div class="stat-header">
										<Globe size={16} class="icon-green" />
										<span class="stat-title">Dimension</span>
									</div>
									<div class="stat-badge">
										<span class="badge {getDimensionBadgeClass(p.dimension)}">
											{formatDimension(p.dimension)}
										</span>
									</div>
								</div>

								<!-- First & Last Joined -->
								<div class="stat-box stat-box-wide">
									<div class="stat-header">
										<Calendar size={16} class="icon-muted" />
										<span class="stat-title">Activity Timeline</span>
									</div>
									<div class="timeline-row">
										<div>
											<span class="timeline-lbl">First Seen:</span>
											<span class="timeline-val">{formatDate(p.first_joined_timestamp)}</span>
										</div>
										<div>
											<span class="timeline-lbl">Last Seen:</span>
											<span class="timeline-val">{formatDate(p.last_joined_timestamp || p.last_seen)}</span>
										</div>
									</div>
								</div>
							</div>
						</div>
					</div>
				{:else if activeTab === 'inventory'}
					<!-- Inventory & Ender Chest Visualizer Tab -->
					<div role="tabpanel" id="tabpanel-inventory" aria-labelledby="tab-inventory">
						<InventoryVisualizer
							inventory={inventoryData}
							loading={loadingInventory}
							error={inventoryError}
						/>
					</div>
				{:else if activeTab === 'effects'}
					<!-- Active Status Effects & Potion Modifiers Tab -->
					<div role="tabpanel" id="tabpanel-effects" aria-labelledby="tab-effects">
						<StatusEffectsPanel
							playerUuid={p.uuid}
							playerName={p.username}
							isOnline={p.is_online}
							onActionSuccess={(msg) => onActionSuccess('effects', msg)}
						/>
					</div>
				{:else if activeTab === 'give'}
					<!-- Give Item Visual Catalog Tab -->
					<div role="tabpanel" id="tabpanel-give" aria-labelledby="tab-give">
						<div class="give-panel">
							<!-- Top Controls: Search, Category, Count & Direct Give Action -->
							<div class="give-header-card">
								<div class="give-search-row">
									<div class="search-input-wrapper">
										<Search size={16} class="search-icon" />
										<input
											type="text"
											class="input search-input"
											placeholder="Search item name or ID (e.g. Diamond, Netherite, Apple)..."
											bind:value={giveSearch}
										/>
										{#if giveSearch}
											<button class="clear-search-btn" onclick={() => (giveSearch = '')}>
												<X size={14} />
											</button>
										{/if}
									</div>

									<div class="count-selector-group">
										<span class="count-label">Quantity:</span>
										<div class="count-buttons">
											{#each [1, 16, 32, 64] as cnt}
												<button
													class="btn btn-sm count-btn {giveSelectedCount === cnt ? 'active' : ''}"
													onclick={() => (giveSelectedCount = cnt)}
												>
													{cnt}
												</button>
											{/each}
										</div>
										<input
											type="number"
											min="1"
											max="64"
											class="input count-custom-input"
											bind:value={giveSelectedCount}
										/>
									</div>
								</div>

								<!-- Category Filter Pills -->
								<div class="category-pills">
									{#each [{ id: 'all', label: 'All Items' }, { id: 'resources', label: 'Resources' }, { id: 'weapons', label: 'Weapons' }, { id: 'tools', label: 'Tools' }, { id: 'armor', label: 'Armor' }, { id: 'food', label: 'Food' }, { id: 'utility', label: 'Utility' }] as cat}
										<button
											class="cat-pill {giveCategory === cat.id ? 'active' : ''}"
											onclick={() => (giveCategory = cat.id)}
										>
											{cat.label}
										</button>
									{/each}
								</div>

								<!-- Selected Item Action Strip -->
								<div class="give-action-strip">
									<div class="selected-item-preview">
										<div class="item-icon-box">
											<img
												src={getItemIconUrl(giveCustomItemId || giveSelectedItemId)}
												alt={giveSelectedItemId}
												class="give-preview-img"
												onerror={(e) => handleGiveImageError(e, giveCustomItemId || giveSelectedItemId)}
											/>
										</div>
										<div class="selected-item-info">
											<span class="selected-item-title">
												{POPULAR_ITEMS.find((i) => i.id === giveSelectedItemId)?.name ||
													giveCustomItemId ||
													giveSelectedItemId}
											</span>
											<span class="selected-item-id"
												>{giveCustomItemId.trim() || giveSelectedItemId} × {giveSelectedCount}</span
											>
										</div>
									</div>

									<div class="give-action-buttons">
										<div class="custom-id-input-box">
											<input
												type="text"
												class="input input-sm"
												placeholder="Or type custom ID (e.g. mod:item)..."
												bind:value={giveCustomItemId}
											/>
										</div>
										<button
											class="btn btn-primary {actionLoading === 'give' ? 'btn-loading' : ''}"
											onclick={handleGiveItemSubmit}
											disabled={actionLoading === 'give'}
										>
											{#if actionLoading !== 'give'}
												{#if p.is_online}
													<Gift size={16} />
												{:else}
													<Clock size={16} />
												{/if}
											{/if}
											<span
												>{p.is_online
													? `Give to ${p.username}`
													: `Queue Give for ${p.username} (on join)`}</span
											>
										</button>
									</div>
								</div>
							</div>

							<!-- Item Grid Cards -->
							<div class="items-grid-catalog">
								{#each filteredItems as item}
									<button
										class="item-grid-card {giveSelectedItemId === item.id && !giveCustomItemId
											? 'selected'
											: ''}"
										onclick={() => {
											giveSelectedItemId = item.id;
											giveCustomItemId = '';
										}}
									>
										<div class="item-card-icon">
											<img
												src={getItemIconUrl(item.id)}
												alt={item.name}
												class="item-card-img"
												loading="lazy"
												onerror={(e) => handleGiveImageError(e, item.id)}
											/>
										</div>
										<div class="item-card-details">
											<span class="item-card-name">{item.name}</span>
											<span class="item-card-id">{item.id.replace('minecraft:', '')}</span>
										</div>
										{#if giveSelectedItemId === item.id && !giveCustomItemId}
											<div class="selected-check-badge">
												<Check size={12} />
											</div>
										{/if}
									</button>
								{/each}
							</div>
						</div>
					</div>
				{:else if activeTab === 'permissions'}
					<!-- LuckPerms Permissions Manager Tab -->
					<div role="tabpanel" id="tabpanel-permissions" aria-labelledby="tab-permissions">
						<BasicPermissionsPanel
							playerUuid={p.uuid}
							playerName={p.username}
							onActionSuccess={(msg) => onActionSuccess('permissions', msg)}
						/>
					</div>
				{:else if activeTab === 'moderation'}
					<!-- Comprehensive Moderation & Actions Tab -->
					<div role="tabpanel" id="tabpanel-moderation" aria-labelledby="tab-moderation">
						<div class="moderation-panel">
							<!-- Pending Commands Queue Card -->
							<div class="mod-card pending-queue-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<Clock size={18} class="icon-warning" />
										<div>
											<div class="pending-title-line">
												<h4 class="mod-title">Commandes en attente (File d'exécution)</h4>
												{#if pendingCommands.length > 0}
													<span class="count-pill queue-count-pill">{pendingCommands.length}</span>
												{/if}
											</div>
											<p class="mod-desc">
												Commandes enregistrées qui s'exécuteront automatiquement dès la connexion du joueur.
											</p>
										</div>
									</div>
									<button
										class="btn btn-ghost btn-icon btn-sm"
										onclick={() => fetchPendingCommands()}
										title="Rafraîchir la file d'attente"
										disabled={loadingPending}
									>
										<RefreshCw size={14} class={loadingPending ? 'spin' : ''} />
									</button>
								</div>
								<div class="mod-card-body">
									{#if loadingPending && pendingCommands.length === 0}
										<div class="pending-loading-row">
											<Loader2 size={16} class="spinner" />
											<span>Chargement des commandes en attente...</span>
										</div>
									{:else if pendingCommands.length === 0}
										<div class="pending-empty-state">
											<CheckCircle2 size={16} class="icon-green" />
											<span>Aucune commande en attente. Toutes les actions ont été appliquées.</span>
										</div>
									{:else}
										<div class="pending-commands-list">
											{#each pendingCommands as cmd}
												<div class="pending-cmd-row">
													<div class="pending-cmd-info">
														<div class="pending-cmd-meta">
															<span class="badge badge-warning badge-sm">{cmd.action.toUpperCase()}</span>
															<span class="pending-cmd-time">{formatRelativeTime(cmd.created_at)}</span>
														</div>
														<code class="pending-cmd-code">{cmd.command}</code>
													</div>
													<button
														class="btn btn-ghost btn-icon btn-sm btn-del-pending"
														onclick={() => handleDeletePendingCommand(cmd.id)}
														title="Annuler cette commande"
														disabled={actionLoading === `del_${cmd.id}`}
													>
														{#if actionLoading === `del_${cmd.id}`}
															<Loader2 size={14} class="spinner" />
														{:else}
															<Trash2 size={14} />
														{/if}
													</button>
												</div>
											{/each}
										</div>
									{/if}
								</div>
							</div>

							<!-- Gamemode Switcher Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<Gamepad2 size={18} class="icon-blue" />
										<div>
											<h4 class="mod-title">Gamemode Switcher</h4>
											<p class="mod-desc">Change player's active game mode instantly (runs on join if offline).</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<div class="gamemode-buttons-grid">
										{#each [{ id: 'survival', label: 'Survival', desc: 'Default survival mode' }, { id: 'creative', label: 'Creative', desc: 'Unlimited items & flight' }, { id: 'adventure', label: 'Adventure', desc: 'Block break restrictions' }, { id: 'spectator', label: 'Spectator', desc: 'Fly through blocks invisible' }] as gm}
											<button
												class="gamemode-select-btn {selectedGamemode === gm.id ? 'active' : ''}"
												onclick={() => (selectedGamemode = gm.id)}
											>
												<span class="gm-label">{gm.label}</span>
												<span class="gm-desc">{gm.desc}</span>
											</button>
										{/each}
									</div>
								</div>
								<div class="mod-card-footer">
									<button
										class="btn btn-primary {actionLoading === 'gamemode' ? 'btn-loading' : ''}"
										onclick={() => handleExecuteAction('gamemode')}
										disabled={actionLoading === 'gamemode'}
									>
										{#if actionLoading !== 'gamemode'}
											<Gamepad2 size={16} />
										{/if}
										<span
											>{p.is_online
												? `Set to ${selectedGamemode}`
												: `Queue Gamemode ${selectedGamemode} (on join)`}</span
										>
									</button>
								</div>
							</div>

							<!-- Quick Vitality & Inventory Actions -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<Flame size={18} class="icon-warning" />
										<div>
											<h4 class="mod-title">Quick Vitality & Actions</h4>
											<p class="mod-desc">Instantly restore health, satiate food, kill, or wipe player inventory (queued if offline).</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<div class="quick-actions-row">
										<button
											class="btn btn-secondary {actionLoading === 'heal' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('heal')}
											disabled={actionLoading === 'heal'}
											title="Applies Instant Health 255 & Saturation (queued if offline)"
										>
											<Heart size={16} class="icon-danger" />
											<span>{p.is_online ? 'Heal & Feed' : 'Queue Heal'}</span>
										</button>

										<button
											class="btn btn-secondary {actionLoading === 'feed' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('feed')}
											disabled={actionLoading === 'feed'}
											title="Applies Max Saturation (queued if offline)"
										>
											<Utensils size={16} class="icon-warning" />
											<span>{p.is_online ? 'Feed' : 'Queue Feed'}</span>
										</button>

										<button
											class="btn btn-secondary {actionLoading === 'kill' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('kill')}
											disabled={actionLoading === 'kill'}
											title="Executes /kill command (queued if offline)"
										>
											<Skull size={16} class="icon-danger" />
											<span>{p.is_online ? 'Kill Player' : 'Queue Kill'}</span>
										</button>

										<button
											class="btn btn-danger {actionLoading === 'clear' ? 'btn-loading' : ''}"
											onclick={() => {
												if (confirm(`Are you sure you want to completely clear ${p.username}'s inventory?`)) {
													handleExecuteAction('clear');
												}
											}}
											disabled={actionLoading === 'clear'}
											title="Clears player inventory (queued if offline)"
										>
											<Trash2 size={16} />
											<span>{p.is_online ? 'Clear Inventory' : 'Queue Clear'}</span>
										</button>
									</div>
								</div>
							</div>

							<!-- Whitelist Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<UserCheck size={18} class="icon-green" />
										<div>
											<h4 class="mod-title">Whitelist Access</h4>
											<p class="mod-desc">Add or remove player from the server whitelist.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<p class="op-desc-text">
										Allow this player to join when whitelist enforcement is enabled on the server.
									</p>
								</div>
								<div class="mod-card-footer">
									<div class="whitelist-btns">
										<button
											class="btn btn-secondary {actionLoading === 'whitelist_remove' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('whitelist_remove')}
											disabled={actionLoading === 'whitelist_remove'}
										>
											<X size={16} />
											<span>Remove Whitelist</span>
										</button>
										<button
											class="btn btn-primary {actionLoading === 'whitelist_add' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('whitelist_add')}
											disabled={actionLoading === 'whitelist_add'}
										>
											<Check size={16} />
											<span>Add to Whitelist</span>
										</button>
									</div>
								</div>
							</div>

							<!-- Kick Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<UserX size={18} class="icon-warning" />
										<div>
											<h4 class="mod-title">Kick Player from Server</h4>
											<p class="mod-desc">Disconnects active player immediately with an optional reason.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<div class="form-group mb-0">
										<label for="kick-reason" class="label">Reason for Kick</label>
										<input
											id="kick-reason"
											type="text"
											class="input"
											placeholder="Reason displayed on disconnect screen..."
											bind:value={kickReason}
										/>
									</div>
								</div>
								<div class="mod-card-footer">
									<button
										class="btn btn-secondary {actionLoading === 'kick' ? 'btn-loading' : ''}"
										onclick={() => handleExecuteAction('kick')}
										disabled={!p.is_online || actionLoading === 'kick'}
									>
										{#if actionLoading !== 'kick'}
											<UserX size={16} />
										{/if}
										<span>{p.is_online ? 'Kick Player' : 'Player Offline'}</span>
									</button>
								</div>
							</div>

							<!-- Ban / Pardon Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<ShieldAlert size={18} class="icon-danger" />
										<div>
											<h4 class="mod-title">Ban & Blacklist Management</h4>
											<p class="mod-desc">Prevent player from joining the Minecraft server.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									{#if p.is_banned}
										<div class="banned-status-box">
											<AlertCircle size={18} class="icon-danger" />
											<div>
												<strong>Player is currently Banned</strong>
												<p class="ban-reason-text">Reason: {p.ban_reason || 'No reason specified'}</p>
											</div>
										</div>
									{:else}
										<div class="form-group mb-0">
											<label for="ban-reason" class="label">Ban Reason</label>
											<input
												id="ban-reason"
												type="text"
												class="input"
												placeholder="Reason recorded in ban list..."
												bind:value={banReason}
											/>
										</div>
									{/if}
								</div>
								<div class="mod-card-footer">
									{#if p.is_banned}
										<button
											class="btn btn-primary {actionLoading === 'pardon' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('pardon')}
											disabled={actionLoading === 'pardon'}
										>
											{#if actionLoading !== 'pardon'}
												<ShieldCheck size={16} />
											{/if}
											<span>Pardon Player (Unban)</span>
										</button>
									{:else}
										<button
											class="btn btn-danger {actionLoading === 'ban' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('ban')}
											disabled={actionLoading === 'ban'}
										>
											{#if actionLoading !== 'ban'}
												<ShieldAlert size={16} />
											{/if}
											<span>Ban Player</span>
										</button>
									{/if}
								</div>
							</div>

							<!-- Teleport Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<Navigation size={18} class="icon-blue" />
										<div>
											<h4 class="mod-title">Teleport Coordinates</h4>
											<p class="mod-desc">Teleports online player to specified X, Y, Z target location.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<div class="tp-coords-inputs">
										<div class="form-group">
											<label for="tp-x" class="label">X Coord</label>
											<input id="tp-x" type="number" class="input" bind:value={tpX} />
										</div>
										<div class="form-group">
											<label for="tp-y" class="label">Y Coord</label>
											<input id="tp-y" type="number" class="input" bind:value={tpY} />
										</div>
										<div class="form-group">
											<label for="tp-z" class="label">Z Coord</label>
											<input id="tp-z" type="number" class="input" bind:value={tpZ} />
										</div>
									</div>
								</div>
								<div class="mod-card-footer">
									<button
										class="btn btn-primary {actionLoading === 'teleport' ? 'btn-loading' : ''}"
										onclick={() => handleExecuteAction('teleport')}
										disabled={!p.is_online || actionLoading === 'teleport'}
									>
										{#if actionLoading !== 'teleport'}
											<Navigation size={16} />
										{/if}
										<span>{p.is_online ? 'Teleport Player' : 'Player Offline'}</span>
									</button>
								</div>
							</div>

							<!-- OP / DeOP Section -->
							<div class="mod-card">
								<div class="mod-card-header">
									<div class="mod-title-group">
										<Crown size={18} class="icon-purple" />
										<div>
											<h4 class="mod-title">Operator Permissions (OP)</h4>
											<p class="mod-desc">Grant or revoke operator privileges on the Minecraft server.</p>
										</div>
									</div>
								</div>
								<div class="mod-card-body">
									<p class="op-desc-text">
										Operators have full administrator permissions to execute all server commands in-game.
									</p>
								</div>
								<div class="mod-card-footer">
									{#if p.is_op}
										<button
											class="btn btn-secondary {actionLoading === 'deop' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('deop')}
											disabled={actionLoading === 'deop'}
										>
											{#if actionLoading !== 'deop'}
												<Crown size={16} />
											{/if}
											<span>Revoke Operator (De-OP)</span>
										</button>
									{:else}
										<button
											class="btn btn-primary btn-purple {actionLoading === 'op' ? 'btn-loading' : ''}"
											onclick={() => handleExecuteAction('op')}
											disabled={actionLoading === 'op'}
										>
											{#if actionLoading !== 'op'}
												<Crown size={16} />
											{/if}
											<span>Grant Operator (OP)</span>
										</button>
									{/if}
								</div>
							</div>
						</div>
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="modal-footer">
				<button class="btn btn-secondary" onclick={onClose}>
					Close Drawer
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.profile-modal-drawer {
		max-width: 900px;
		width: 95vw;
		height: 90vh;
		display: flex;
		flex-direction: column;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-modal);
		box-shadow: 0 20px 45px rgba(0, 0, 0, 0.7);
		overflow: hidden;
		animation: modalFadeIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
	}

	@keyframes modalFadeIn {
		from {
			opacity: 0;
			transform: scale(0.98);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-4) var(--space-5);
		border-bottom: 1px solid var(--border);
		background-color: var(--bg-elevated);
	}

	.player-header-info {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.header-avatar {
		border-radius: var(--radius-sm);
		image-rendering: pixelated;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.modal-title {
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}

	.header-uuid {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--text-muted);
	}

	.header-right-tools {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.spinning {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		100% {
			transform: rotate(360deg);
		}
	}

	.status-dot {
		display: inline-block;
		width: 7px;
		height: 7px;
		border-radius: 50%;
		margin-right: 4px;
	}

	.status-dot-success {
		background-color: var(--accent-green);
	}

	.status-dot-pulse {
		box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7);
		animation: pulse 1.6s infinite;
	}

	@keyframes pulse {
		0% {
			transform: scale(0.95);
			box-shadow: 0 0 0 0 rgba(34, 197, 94, 0.7);
		}
		70% {
			transform: scale(1);
			box-shadow: 0 0 0 6px rgba(34, 197, 94, 0);
		}
		100% {
			transform: scale(0.95);
			box-shadow: 0 0 0 0 rgba(34, 197, 94, 0);
		}
	}

	/* Navigation Tabs */
	.modal-nav-tabs {
		display: flex;
		background-color: var(--bg-base);
		border-bottom: 1px solid var(--border);
		padding: 0 var(--space-3);
		overflow-x: auto;
	}

	.nav-tab-btn {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-4);
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--text-muted);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		transition: color var(--transition-fast), border-color var(--transition-fast), background-color var(--transition-fast);
		white-space: nowrap;
	}

	.nav-tab-btn:hover {
		color: var(--text-primary);
	}

	.nav-tab-btn.active {
		color: var(--accent-blue-text);
		border-bottom-color: var(--accent-blue);
		background-color: rgba(59, 130, 246, 0.05);
	}

	/* Modal Body */
	.modal-body {
		padding: var(--space-5);
		overflow-y: auto;
		flex: 1;
		position: relative;
	}

	.loading-overlay {
		position: absolute;
		inset: 0;
		background-color: rgba(15, 23, 42, 0.75);
		backdrop-filter: blur(2px);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		z-index: 10;
		color: var(--text-secondary);
		font-size: var(--font-size-sm);
	}

	.spinner {
		animation: spin 1s linear infinite;
		color: var(--accent-blue);
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-input);
		margin: var(--space-3) var(--space-5) 0;
		font-size: var(--font-size-sm);
	}

	/* Overview Layout */
	.overview-layout {
		display: grid;
		grid-template-columns: 200px 1fr;
		gap: var(--space-5);
		align-items: start;
	}

	@media (max-width: 700px) {
		.overview-layout {
			grid-template-columns: 1fr;
		}
	}

	.skin-viewer-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
	}

	.skin-viewer-container {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 220px;
	}

	.skin-body-image {
		image-rendering: pixelated;
		filter: drop-shadow(0 8px 12px rgba(0, 0, 0, 0.4));
	}

	.skin-caption {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-3);
	}

	.stat-box {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-3) var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.stat-box-wide {
		grid-column: span 2;
	}

	.stat-header {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.stat-title {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.stat-value-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
	}

	.stat-main-value {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
		font-family: var(--font-mono);
	}

	.stat-large-text {
		font-size: var(--font-size-md);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
	}

	.stat-small-text {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.stat-badge {
		align-self: flex-start;
		margin-top: 2px;
	}

	.gauge-bar {
		width: 100%;
		height: 6px;
		background-color: var(--bg-elevated);
		border-radius: var(--radius-full);
		overflow: hidden;
	}

	.gauge-fill {
		width: 100%;
		height: 100%;
		border-radius: var(--radius-full);
		transform-origin: left;
		transition: transform var(--transition-normal) var(--ease-out);
	}

	.gauge-fill-success {
		background-color: var(--accent-green);
	}
	.gauge-fill-warning {
		background-color: var(--warning);
	}
	.gauge-fill-primary {
		background-color: var(--accent-blue);
	}

	.gauge-unknown {
		background: repeating-linear-gradient(
			45deg,
			var(--bg-elevated),
			var(--bg-elevated) 4px,
			var(--border) 4px,
			var(--border) 8px
		);
	}

	.coords-display {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.coord-pill {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		padding: 2px 8px;
		border-radius: var(--radius-sm);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
	}

	.coord-axis {
		color: var(--accent-blue-text);
		font-weight: var(--font-weight-bold);
	}

	.coord-val {
		color: var(--text-primary);
	}

	.timeline-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		font-size: var(--font-size-xs);
	}

	.timeline-lbl {
		color: var(--text-muted);
		margin-right: var(--space-1);
	}

	.timeline-val {
		color: var(--text-secondary);
		font-family: var(--font-mono);
	}

	/* Give Item Panel Styles */
	.give-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.give-header-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-4);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.give-search-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.search-input-wrapper {
		position: relative;
		flex: 1;
		min-width: 250px;
	}

	.search-icon {
		position: absolute;
		left: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		color: var(--text-muted);
		pointer-events: none;
	}

	.search-input {
		padding-left: calc(var(--space-3) + 22px);
		padding-right: var(--space-8);
		width: 100%;
	}

	.clear-search-btn {
		position: absolute;
		right: var(--space-3);
		top: 50%;
		transform: translateY(-50%);
		background: none;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.count-selector-group {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.count-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		font-weight: var(--font-weight-medium);
	}

	.count-buttons {
		display: flex;
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		overflow: hidden;
	}

	.count-btn {
		border: none;
		border-radius: 0;
		background: none;
		color: var(--text-secondary);
		padding: 4px 10px;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-semibold);
		cursor: pointer;
	}

	.count-btn.active {
		background-color: var(--accent-blue);
		color: #ffffff;
	}

	.count-custom-input {
		width: 60px;
		padding: 4px 8px;
		font-size: var(--font-size-xs);
		text-align: center;
	}

	.category-pills {
		display: flex;
		gap: var(--space-2);
		overflow-x: auto;
		padding-bottom: 2px;
	}

	.cat-pill {
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-full);
		color: var(--text-secondary);
		padding: 4px 12px;
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		cursor: pointer;
		white-space: nowrap;
		transition: color var(--transition-fast), border-color var(--transition-fast), background-color var(--transition-fast);
	}

	.cat-pill:hover {
		border-color: var(--border-hover);
		color: var(--text-primary);
	}

	.cat-pill.active {
		background-color: rgba(59, 130, 246, 0.15);
		border-color: var(--accent-blue);
		color: var(--accent-blue-text);
	}

	.give-action-strip {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		flex-wrap: wrap;
	}

	.selected-item-preview {
		display: flex;
		align-items: center;
		gap: var(--space-3);
	}

	.item-icon-box {
		width: 40px;
		height: 40px;
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.give-preview-img {
		width: 28px;
		height: 28px;
		image-rendering: pixelated;
	}

	.selected-item-info {
		display: flex;
		flex-direction: column;
	}

	.selected-item-title {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.selected-item-id {
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		color: var(--accent-blue-text);
	}

	.give-action-buttons {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.custom-id-input-box {
		min-width: 200px;
	}

	.items-grid-catalog {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
		gap: var(--space-3);
		max-height: 420px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.item-grid-card {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: var(--space-2) var(--space-3);
		cursor: pointer;
		text-align: left;
		position: relative;
		transition: border-color var(--transition-fast), background-color var(--transition-fast), transform var(--transition-fast);
	}

	.item-grid-card:hover {
		border-color: var(--border-hover);
		background-color: var(--bg-elevated);
	}

	.item-grid-card:active {
		transform: scale(0.97);
	}

	.item-grid-card.selected {
		border-color: var(--accent-blue);
		background-color: rgba(59, 130, 246, 0.1);
	}

	.item-card-icon {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
	}

	.item-card-img {
		width: 28px;
		height: 28px;
		image-rendering: pixelated;
	}

	.item-card-details {
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.item-card-name {
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-medium);
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.item-card-id {
		font-size: 10px;
		font-family: var(--font-mono);
		color: var(--text-muted);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.selected-check-badge {
		position: absolute;
		top: 4px;
		right: 4px;
		background-color: var(--accent-blue);
		color: #ffffff;
		border-radius: 50%;
		width: 16px;
		height: 16px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	/* Moderation Panel */
	.moderation-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.mod-card {
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		overflow: hidden;
	}

	.mod-card-header {
		padding: var(--space-3) var(--space-4);
		border-bottom: 1px solid var(--border-subtle);
		background-color: rgba(255, 255, 255, 0.01);
	}

	.mod-title-group {
		display: flex;
		align-items: flex-start;
		gap: var(--space-3);
	}

	.mod-title {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.mod-desc {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		margin-top: 2px;
	}

	.mod-card-body {
		padding: var(--space-4);
	}

	.mod-card-footer {
		padding: var(--space-3) var(--space-4);
		background-color: rgba(0, 0, 0, 0.2);
		border-top: 1px solid var(--border-subtle);
		display: flex;
		align-items: center;
		justify-content: flex-end;
	}

	.gamemode-buttons-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
		gap: var(--space-3);
	}

	.gamemode-select-btn {
		display: flex;
		flex-direction: column;
		padding: var(--space-3);
		background-color: var(--bg-elevated);
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		cursor: pointer;
		text-align: left;
		transition: border-color var(--transition-fast), background-color var(--transition-fast), transform var(--transition-fast);
	}

	.gamemode-select-btn:hover {
		border-color: var(--border-hover);
	}

	.gamemode-select-btn:active {
		transform: scale(0.97);
	}

	.gamemode-select-btn.active {
		border-color: var(--accent-blue);
		background-color: rgba(59, 130, 246, 0.15);
	}

	.gm-label {
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-semibold);
		color: var(--text-primary);
	}

	.gm-desc {
		font-size: 11px;
		color: var(--text-muted);
		margin-top: 2px;
	}

	.quick-actions-row {
		display: flex;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.whitelist-btns {
		display: flex;
		gap: var(--space-3);
	}

	.tp-coords-inputs {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-3);
	}

	.mb-0 {
		margin-bottom: 0;
	}

	.banned-status-box {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-input);
		padding: var(--space-3);
		color: var(--danger-text);
	}

	.ban-reason-text {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
		margin-top: 2px;
	}

	.op-desc-text {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}

	.btn-purple {
		background-color: var(--accent-purple-solid);
		border-color: var(--accent-purple-solid);
		color: #ffffff;
	}

	.btn-purple:hover:not(:disabled) {
		background-color: var(--accent-purple-solid-hover);
		border-color: var(--accent-purple-solid-hover);
		box-shadow: 0 0 16px rgba(139, 92, 246, 0.3);
	}

	.badge-op {
		background-color: rgba(168, 85, 247, 0.15);
		color: var(--accent-purple-text);
		border-color: rgba(168, 85, 247, 0.3);
	}

	.badge-purple {
		background-color: rgba(168, 85, 247, 0.12);
		color: var(--accent-purple-text);
		border-color: rgba(168, 85, 247, 0.25);
	}

	/* Icon Accents */
	.icon-danger {
		color: var(--danger-text);
	}
	.icon-warning {
		color: var(--warning);
	}
	.icon-green {
		color: var(--accent-green);
	}
	.icon-blue {
		color: var(--accent-blue-text);
	}
	.icon-purple {
		color: var(--accent-purple-text);
	}
	.icon-muted {
		color: var(--text-muted);
	}

	/* Tab Badge Count */
	.tab-badge-count {
		background-color: var(--warning, #F59E0B);
		color: #000000;
		font-size: 10px;
		font-weight: 700;
		padding: 1px 5px;
		border-radius: 9999px;
		line-height: 1;
		margin-left: 4px;
	}

	/* Pending Queue Card */
	.pending-queue-card {
		border-color: rgba(245, 158, 11, 0.25);
		background-color: rgba(245, 158, 11, 0.02);
	}

	.pending-title-line {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.queue-count-pill {
		background-color: rgba(245, 158, 11, 0.2);
		color: var(--warning, #F59E0B);
		font-size: 11px;
		font-weight: 700;
		padding: 2px 7px;
		border-radius: 9999px;
	}

	.pending-loading-row,
	.pending-empty-state {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-sm);
		color: var(--text-muted);
		padding: var(--space-3);
		background-color: var(--bg-base);
		border-radius: var(--radius-input);
		border: 1px dashed var(--border);
	}

	.pending-commands-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.pending-cmd-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-3);
		background-color: var(--bg-base);
		border: 1px solid var(--border);
		border-radius: var(--radius-input);
		transition: border-color var(--transition-fast);
	}

	.pending-cmd-row:hover {
		border-color: rgba(245, 158, 11, 0.35);
	}

	.pending-cmd-info {
		display: flex;
		flex-direction: column;
		gap: 3px;
		min-width: 0;
		flex: 1;
	}

	.pending-cmd-meta {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.pending-cmd-time {
		font-size: 11px;
		color: var(--text-muted);
	}

	.pending-cmd-code {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--accent-blue-text);
		background-color: rgba(0, 0, 0, 0.25);
		padding: 2px 6px;
		border-radius: 3px;
		word-break: break-all;
	}

	.btn-del-pending {
		color: var(--text-muted);
	}

	.btn-del-pending:hover:not(:disabled) {
		color: var(--danger-text);
		background-color: var(--danger-bg);
	}
</style>
