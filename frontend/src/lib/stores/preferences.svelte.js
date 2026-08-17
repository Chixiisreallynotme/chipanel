import { browser } from '$app/environment';

const PREF_MODE_KEY = 'chipanel_user_mode';
const PREF_ONBOARDING_COMPLETED_KEY = 'chipanel_onboarding_completed';

/**
 * @typedef {'novice' | 'expert'} UserMode
 */

class PreferencesStore {
	/** @type {UserMode} */
	mode = $state('novice');

	/** @type {boolean} */
	onboardingCompleted = $state(false);

	/** @type {number} Total host RAM detected in GB (fallback 8.0) */
	hostTotalRamGb = $state(8.0);

	/** @type {boolean} */
	isDesktop = $state(false);

	/** @type {{ podman: boolean, docker: boolean, java: boolean, javaVersion: string | null }} */
	runtimeStatus = $state({
		podman: true,
		docker: false,
		java: true,
		javaVersion: '21.0.3 (OpenJDK)'
	});

	constructor() {
		if (browser) {
			const savedMode = localStorage.getItem(PREF_MODE_KEY);
			if (savedMode === 'novice' || savedMode === 'expert') {
				this.mode = savedMode;
			}
			const completed = localStorage.getItem(PREF_ONBOARDING_COMPLETED_KEY);
			if (completed === 'true') {
				this.onboardingCompleted = true;
			}
			// Detect Tauri environment
			this.isDesktop = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
		}
	}

	/**
	 * @param {UserMode} newMode
	 */
	setMode(newMode) {
		this.mode = newMode;
		if (browser) {
			localStorage.setItem(PREF_MODE_KEY, newMode);
		}
	}

	toggleMode() {
		this.setMode(this.mode === 'novice' ? 'expert' : 'novice');
	}

	/**
	 * Mark onboarding as completed
	 */
	completeOnboarding() {
		this.onboardingCompleted = true;
		if (browser) {
			localStorage.setItem(PREF_ONBOARDING_COMPLETED_KEY, 'true');
		}
	}

	/**
	 * Reset onboarding status
	 */
	resetOnboarding() {
		this.onboardingCompleted = false;
		if (browser) {
			localStorage.removeItem(PREF_ONBOARDING_COMPLETED_KEY);
		}
	}

	/**
	 * Update host RAM probe
	 * @param {number} ramGb
	 */
	setHostRam(ramGb) {
		if (ramGb > 0) {
			this.hostTotalRamGb = Number(ramGb.toFixed(1));
		}
	}
}

export const preferences = new PreferencesStore();
