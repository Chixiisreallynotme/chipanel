import { browser } from '$app/environment';

/**
 * Session-storage key holding the JWT. Owned here so the auth store and the request
 * wrapper cannot drift apart. `sessionStorage`, not `localStorage`: the token must die
 * with the browser session (CLAUDE.md).
 */
export const TOKEN_STORAGE_KEY = 'chipanel_token';

/**
 * Session-invalidation callback. Registered by the auth store (see
 * `$lib/stores/auth.svelte.js`) so this module never has to import it — importing the
 * store here would create a client <-> store import cycle, and a lazy `import()` inside
 * the 401 path would race with the redirect it is supposed to precede.
 *
 * The handler is expected to be idempotent: several in-flight requests can 401 at once.
 * @type {(() => void) | null}
 */
let onUnauthorized = null;

/**
 * Register the session-invalidation callback invoked on any 401 response.
 * @param {() => void} handler
 */
export function setUnauthorizedHandler(handler) {
	onUnauthorized = handler;
}

/**
 * Read the bearer token from session storage.
 * @returns {string | null}
 */
function readStoredToken() {
	if (!browser) return null;
	return sessionStorage.getItem(TOKEN_STORAGE_KEY);
}

/**
 * Base wrapper for fetch with Authorization header attachment and 401 session invalidation.
 * @param {string} path - Request endpoint URL or path
 * @param {RequestInit} [options={}] - Fetch configuration options
 * @returns {Promise<Response>}
 */
export async function apiFetch(path, options = {}) {
	const headers = new Headers(options.headers || {});

	const token = readStoredToken();
	if (token) {
		headers.set('Authorization', `Bearer ${token}`);
	}

	let body = options.body;
	if (body && typeof body === 'object' && !(body instanceof FormData)) {
		if (!headers.has('Content-Type')) {
			headers.set('Content-Type', 'application/json');
		}
		body = JSON.stringify(body);
	}

	const response = await fetch(path, {
		...options,
		headers,
		body
	});

	if (response.status === 401) {
		// Clear the session *state*, not just storage: leaving `auth.token` set made the
		// layout guard bounce straight back to `/`, remount the page and refetch — a 17 req/s
		// redirect loop in which the user was never actually logged out.
		// The redirect itself is owned by `routes/+layout.svelte`'s auth guard, which reacts
		// to the state change exactly once however many requests failed together.
		onUnauthorized?.();

		const errorData = await response
			.json()
			.catch(() => ({ message: 'Session expirée (24h écoulées)' }));
		throw new Error(errorData.error || errorData.message || 'Session expirée');
	}

	return response;
}

/**
 * Perform a GET request to the API.
 * @param {string} path - Endpoint path
 * @returns {Promise<any>} Parsed JSON response
 */
export async function apiGet(path) {
	const res = await apiFetch(path, { method: 'GET' });
	if (!res.ok) {
		const errorData = await res.json().catch(() => ({ message: 'Request failed' }));
		throw new Error(errorData.error || errorData.message || `GET ${path} failed (${res.status})`);
	}
	return res.json();
}

/**
 * Perform a POST request to the API with body.
 * @param {string} path - Endpoint path
 * @param {any} body - Payload object
 * @returns {Promise<any>} Parsed JSON response
 */
export async function apiPost(path, body) {
	const res = await apiFetch(path, {
		method: 'POST',
		body
	});
	if (!res.ok) {
		const errorData = await res.json().catch(() => ({ message: 'Request failed' }));
		throw new Error(errorData.error || errorData.message || `POST ${path} failed (${res.status})`);
	}
	return res.json();
}

/**
 * Perform a DELETE request to the API.
 * @param {string} path - Endpoint path
 * @returns {Promise<any>} Parsed JSON response
 */
export async function apiDelete(path) {
	const res = await apiFetch(path, { method: 'DELETE' });
	if (!res.ok) {
		const errorData = await res.json().catch(() => ({ message: 'Request failed' }));
		throw new Error(errorData.error || errorData.message || `DELETE ${path} failed (${res.status})`);
	}
	return res.json();
}

/**
 * GET a binary file (e.g. a world export zip) with the auth header attached and
 * trigger a browser download.
 * @param {string} path
 * @param {string} [filename]
 */
export async function apiDownload(path, filename) {
	const res = await apiFetch(path, { method: 'GET' });
	if (!res.ok) {
		const errorData = await res.json().catch(() => ({ message: 'Download failed' }));
		throw new Error(errorData.error || errorData.message || `GET ${path} failed (${res.status})`);
	}
	const blob = await res.blob();
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = filename || 'world.zip';
	document.body.appendChild(a);
	a.click();
	a.remove();
	URL.revokeObjectURL(url);
}
