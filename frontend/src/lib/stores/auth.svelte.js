import { browser } from '$app/environment';
import { apiPost, apiGet, setUnauthorizedHandler, TOKEN_STORAGE_KEY } from '$lib/api/client.js';

class AuthStore {
	/** @type {{ id?: string, username: string, role: string | null } | null} */
	user = $state(null);
	/** @type {string | null} */
	token = $state(null);
	isLoading = $state(false);
	/** @type {string | null} */
	error = $state(null);

	/**
	 * True until `init()` has settled. The route guard and every data-fetching child must
	 * wait on this: running them against a not-yet-restored session produced a `/login`
	 * flash plus a burst of parasitic 401s on every page load.
	 */
	isInitializing = $state(true);

	get isAuthenticated() {
		return !!this.token && !!this.user;
	}

	constructor() {
		if (browser) {
			// A token left in localStorage by the previous version (same key, wrong storage)
			// is a ghost credential: drop it instead of adopting it, so the session really
			// ends with the browser.
			localStorage.removeItem(TOKEN_STORAGE_KEY);
			this.token = sessionStorage.getItem(TOKEN_STORAGE_KEY);
		} else {
			// Nothing to restore during SSR; never leave the guard blocked server-side.
			this.isInitializing = false;
		}
	}

	/**
	 * Initialize session state from the stored token and validate it against the API.
	 * Always clears `isInitializing`, including on failure.
	 */
	async init() {
		if (!browser) return;

		const savedToken = sessionStorage.getItem(TOKEN_STORAGE_KEY);
		if (!savedToken) {
			this.logout();
			this.isInitializing = false;
			return;
		}

		this.token = savedToken;
		this.isLoading = true;
		this.error = null;

		try {
			const res = await apiGet('/api/auth/me');
			this.user = res.user || res;
		} catch (err) {
			console.warn('Auth token validation check failed:', err);
			this.logout();
		} finally {
			this.isLoading = false;
			this.isInitializing = false;
		}
	}

	/**
	 * Authenticate user with credentials and store token.
	 * @param {string} username
	 * @param {string} password
	 */
	async login(username, password) {
		this.isLoading = true;
		this.error = null;

		try {
			const data = await apiPost('/api/auth/login', { username, password });
			if (data && data.token) {
				if (browser) {
					sessionStorage.setItem(TOKEN_STORAGE_KEY, data.token);
				}
				this.token = data.token;
				// The API returns the real profile (id/username/role). Never invent a role:
				// a fabricated `role: 'admin'` is indistinguishable from a genuine one.
				this.user = data.user ?? { username, role: null };
				return true;
			} else {
				throw new Error(data?.message || data?.error || 'Identifiants invalides');
			}
		} catch (err) {
			const msg = err instanceof Error ? err.message : 'Identifiants invalides';
			this.error = msg;
			throw err;
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Log out current session and clear stored tokens. Idempotent — several concurrent
	 * 401s must not produce several logouts (or several redirects downstream).
	 */
	logout() {
		this.token = null;
		this.user = null;
		this.error = null;
		if (browser) {
			sessionStorage.removeItem(TOKEN_STORAGE_KEY);
			localStorage.removeItem(TOKEN_STORAGE_KEY);
		}
	}
}

export const auth = new AuthStore();

// Any 401 invalidates the session state itself, not just storage. The redirect is left to
// the layout guard, which reacts once to `isAuthenticated` flipping regardless of how many
// requests failed simultaneously.
setUnauthorizedHandler(() => {
	if (!auth.token && !auth.user) return;
	auth.logout();
});
