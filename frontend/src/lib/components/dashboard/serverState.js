import { wsStore } from '$lib/stores/websocket.svelte.js';

/** Placeholder rendered wherever a value was not measured. */
export const UNAVAILABLE = '—';

/**
 * Single source of truth for "is the server up, and what do we actually know".
 *
 * `wsStore.telemetry` already carries the backend contract faithfully: every
 * numeric field is `number | null`, and `null` means the source did not answer.
 * This adds the two pieces the raw store cannot express on its own:
 *
 *  - an unopened/dropped WebSocket makes `podmanAvailable`/`rconAvailable` read
 *    `false` (their initial value), which is "unknown", not "down" → `null`;
 *  - with Podman unreachable the container state is unknowable, so
 *    `containerRunning` collapses to `null` rather than "stopped".
 *
 * Nothing here substitutes a plausible value for a missing one — `null` must
 * render as "indisponible"/"n/a" at the call site (B1).
 *
 * @returns {{
 *   wsConnected: boolean,
 *   podmanAvailable: boolean | null,
 *   containerRunning: boolean | null,
 *   rconAvailable: boolean | null,
 *   cpuPercent: number | null,
 *   memoryMb: number | null,
 *   memoryLimitMb: number | null,
 *   memoryPercent: number | null,
 *   tps: number | null,
 *   onlinePlayers: number | null,
 *   maxPlayers: number | null
 * }}
 */
export function serverState() {
	const t = wsStore.telemetry;
	const live = wsStore.connected;

	const podmanAvailable = live ? t.podmanAvailable : null;
	const memoryMb = t.ram;
	const memoryLimitMb = t.ramMax;

	let memoryPercent = t.ramPercent;
	if (memoryPercent == null && memoryMb != null && memoryLimitMb) {
		memoryPercent = (memoryMb / memoryLimitMb) * 100;
	}

	return {
		wsConnected: live,
		podmanAvailable,
		containerRunning: podmanAvailable === true ? t.containerRunning : null,
		rconAvailable: live ? t.rconAvailable : null,
		cpuPercent: t.cpu,
		memoryMb,
		memoryLimitMb,
		memoryPercent,
		tps: t.tps,
		onlinePlayers: t.players?.online ?? null,
		maxPlayers: t.players?.max ?? null
	};
}

/**
 * Format a possibly-null number, never inventing one.
 * @param {number | null | undefined} value
 * @param {number} [digits=1]
 * @returns {string}
 */
export function fmtNum(value, digits = 1) {
	return value == null ? UNAVAILABLE : value.toFixed(digits);
}

/**
 * Clamp a possibly-null percentage into a CSS bar width (0 when unknown).
 * @param {number | null | undefined} percent
 * @returns {number}
 */
export function barWidth(percent) {
	if (percent == null) return 0;
	return Math.max(0, Math.min(100, percent));
}
