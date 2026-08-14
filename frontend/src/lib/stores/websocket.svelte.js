import { browser } from '$app/environment';

const BYTES_PER_MB = 1024 * 1024;
const MAX_LOG_ENTRIES = 500;
/** Backend broadcasts every 2s; throttle below that so a genuine frame is never dropped. */
const TELEMETRY_MIN_INTERVAL_MS = 1000;

/**
 * Every numeric field is `number | null`. `null` means "the source did not answer" and MUST
 * render as unavailable — never as 0, and never as a plausible-looking default. A fabricated
 * "20.0 TPS" is indistinguishable from a healthy server, which is the exact bug this shape
 * exists to prevent.
 */
function emptyTelemetry() {
	return {
		podmanAvailable: false,
		/** @type {boolean | null} */
		containerRunning: null,
		/** @type {number | null} */
		cpu: null,
		/** @type {number | null} MB */
		ram: null,
		/** @type {number | null} MB */
		ramMax: null,
		/** @type {number | null} */
		ramPercent: null,
		rconAvailable: false,
		/** @type {number | null} */
		tps: null,
		players: {
			/** @type {number | null} */
			online: null,
			/** @type {number | null} */
			max: null
		}
	};
}

/**
 * @param {unknown} value
 * @returns {number | null}
 */
function num(value) {
	return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

/**
 * @param {unknown} bytes
 * @returns {number | null} megabytes, or null when the source reported nothing
 */
function bytesToMb(bytes) {
	const value = num(bytes);
	return value === null ? null : value / BYTES_PER_MB;
}

/**
 * @param {unknown} ts
 * @returns {string} ISO timestamp
 */
function normalizeTimestamp(ts) {
	if (typeof ts === 'string') return ts;
	const value = num(ts);
	if (value === null) return new Date().toISOString();
	// Backend emits epoch seconds; anything past 1e12 is already milliseconds.
	return new Date(value < 1e12 ? value * 1000 : value).toISOString();
}

class WebSocketStore {
	connected = $state(false);
	/**
	 * Connection lifecycle, for UI that needs to tell "never connected" from "retrying".
	 * @type {'idle' | 'unauthenticated' | 'connecting' | 'connected' | 'reconnecting'}
	 */
	status = $state('idle');
	telemetry = $state(emptyTelemetry());
	/** @type {Array<{ message: string, level: string, timestamp: string }>} */
	consoleLogs = $state([]);

	/** @type {WebSocket | null} */
	ws = null;
	/** @type {string | null} Last non-empty token handed to `connect()`. */
	authToken = null;
	reconnectDelay = 1000;
	maxReconnectDelay = 8000;
	/** @type {ReturnType<typeof setTimeout> | null} */
	reconnectTimer = null;
	/** Set only by `disconnect()`; suppresses reconnection for a deliberate teardown. */
	intentionalClose = false;
	lastTelemetryUpdate = 0;

	/**
	 * Open the WebSocket connection to `/ws?token=…`.
	 * Never opens a socket without a token — an empty `?token=` is rejected by the backend
	 * and just burns a reconnect cycle.
	 * @param {string | null} [token] - Auth token; reuses the last known one when omitted.
	 */
	connect(token) {
		if (!browser) return;

		const tokenChanged = !!token && token !== this.authToken;
		if (token) this.authToken = token;
		if (!this.authToken) {
			this.status = 'unauthenticated';
			this.connected = false;
			return;
		}

		this.intentionalClose = false;
		this.clearReconnectTimer();

		// Already live (or dialing) with the same credentials — don't churn the socket.
		// `/console` also calls connect() on mount, on top of the layout's lifecycle effect.
		if (
			!tokenChanged &&
			this.ws &&
			(this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)
		) {
			return;
		}

		this.teardownSocket();

		const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		const host = window.location.host;
		const wsUrl = `${protocol}//${host}/ws?token=${encodeURIComponent(this.authToken)}`;

		try {
			this.status = 'connecting';
			const socket = new WebSocket(wsUrl);
			this.ws = socket;

			socket.onopen = () => {
				this.connected = true;
				this.status = 'connected';
				this.reconnectDelay = 1000; // Reset exponential backoff on successful connect
				this.lastTelemetryUpdate = 0; // Never throttle away the first frame after a reconnect
			};

			socket.onmessage = (event) => this.handleMessage(event);

			socket.onclose = () => {
				this.connected = false;
				this.ws = null;
				// The socket is gone: the last telemetry frame is stale, not live data.
				this.telemetry = emptyTelemetry();
				if (this.intentionalClose) {
					this.status = 'idle';
				} else {
					this.status = 'reconnecting';
					this.scheduleReconnect();
				}
			};

			socket.onerror = (err) => {
				// Do NOT tear down here. `disconnect()` detaches `onclose`, which is the only
				// caller of `scheduleReconnect()` — doing that on the first error killed the
				// reconnect chain permanently. A `close` event always follows an `error`; let
				// it drive the retry.
				console.error('WebSocket connection error:', err);
			};
		} catch (err) {
			console.error('Failed to instantiate WebSocket:', err);
			this.ws = null;
			this.connected = false;
			this.status = 'reconnecting';
			this.scheduleReconnect();
		}
	}

	/**
	 * @param {MessageEvent} event
	 */
	handleMessage(event) {
		let data;
		try {
			data = JSON.parse(event.data);
		} catch {
			// Plain-text fallback for raw console streaming.
			this.pushLog({
				message: String(event.data),
				level: 'info',
				timestamp: new Date().toISOString()
			});
			return;
		}

		if (data.type === 'telemetry') {
			const now = Date.now();
			if (now - this.lastTelemetryUpdate < TELEMETRY_MIN_INTERVAL_MS) return;
			this.lastTelemetryUpdate = now;
			// No `?? this.telemetry.x` carry-forward: keeping the last good value alive after
			// RCON dies is exactly the "healthy-looking dead server" bug.
			this.telemetry = {
				podmanAvailable: data.podman_available === true,
				containerRunning: typeof data.container_running === 'boolean' ? data.container_running : null,
				cpu: num(data.cpu_percent),
				ram: bytesToMb(data.memory_bytes),
				ramMax: bytesToMb(data.memory_limit_bytes),
				ramPercent: num(data.memory_percent),
				rconAvailable: data.rcon_available === true,
				tps: num(data.tps),
				players: {
					online: num(data.online_players),
					max: num(data.max_players)
				}
			};
			return;
		}

		if (data.type === 'console_log') {
			this.pushLog({
				message: data.message ?? '',
				level: data.level || 'info',
				timestamp: normalizeTimestamp(data.timestamp)
			});
			return;
		}

		if (data.type === 'command_result') {
			this.pushLog({
				message: data.output ? `> ${data.command}\n${data.output}` : `> ${data.command}`,
				level: data.success ? 'info' : 'error',
				timestamp: new Date().toISOString()
			});
			return;
		}

		if (data.type === 'error') {
			this.pushLog({
				message: data.message ?? 'Erreur WebSocket',
				level: 'error',
				timestamp: new Date().toISOString()
			});
		}
	}

	/**
	 * Append one entry to the bounded log buffer.
	 * @param {{ message: string, level: string, timestamp: string }} entry
	 */
	pushLog(entry) {
		if (this.consoleLogs.length >= MAX_LOG_ENTRIES) {
			this.consoleLogs.shift();
		}
		this.consoleLogs.push(entry);
	}

	/**
	 * Schedule a connection retry with exponential backoff (1s, 2s, 4s, 8s max).
	 */
	scheduleReconnect() {
		if (this.intentionalClose || !this.authToken) return;
		this.clearReconnectTimer();
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			this.connect();
			this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
		}, this.reconnectDelay);
	}

	clearReconnectTimer() {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
	}

	/**
	 * Drop the current socket without letting it schedule a reconnect.
	 */
	teardownSocket() {
		const socket = this.ws;
		this.ws = null;
		if (!socket) return;
		socket.onopen = null;
		socket.onclose = null;
		socket.onmessage = null;
		socket.onerror = null;
		try {
			socket.close();
		} catch {
			/* already closing */
		}
	}

	/**
	 * Close the connection deliberately (logout, unmount). Suppresses reconnection until the
	 * next explicit `connect()`; a dropped connection goes through `onclose` instead and does
	 * reconnect.
	 */
	disconnect() {
		this.intentionalClose = true;
		this.clearReconnectTimer();
		this.teardownSocket();
		this.connected = false;
		this.status = 'idle';
		this.telemetry = emptyTelemetry();
	}

	/**
	 * Send an RCON or server command via WebSocket.
	 * @param {string} command - Command string to execute
	 * @returns {boolean} Success status of message sending
	 */
	sendCommand(command) {
		if (this.ws && this.ws.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify({ type: 'command', command }));
			return true;
		}
		return false;
	}
}

export const wsStore = new WebSocketStore();
