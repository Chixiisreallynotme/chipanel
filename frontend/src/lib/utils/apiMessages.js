/**
 * Backend error strings are authored in Rust. Localise the few that are surfaced
 * verbatim in the UI so the cockpit stays French end to end (Design Council VII).
 * Add an entry here rather than translating the same string in two routes.
 */
const TRANSLATIONS = {
	'Server engine configuration is not readable from ChiPanel':
		'La configuration du moteur est illisible depuis ChiPanel'
};

/**
 * @param {string | null | undefined} message
 * @returns {string}
 */
export function localizeServerError(message) {
	if (!message) return '';
	return TRANSLATIONS[message] ?? message;
}
