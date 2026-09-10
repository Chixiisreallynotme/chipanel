/**
 * @typedef {'success' | 'error' | 'warning' | 'info'} ToastType
 * @typedef {{ id: string, type: ToastType, title: string, message: string }} ToastItem
 */
class ToastStore {
	/** @type {ToastItem[]} */
	items = $state([]);

	/** @param {ToastType} type @param {string} title @param {string} message */
	show(type, title, message) {
		const id = Math.random().toString(36).substring(2, 9);
		this.items = [...this.items, { id, type, title, message }];
		setTimeout(() => this.dismiss(id), 5000);
	}

	/** @param {string} title @param {string} message */
	success(title, message) { this.show('success', title, message); }
	/** @param {string} title @param {string} message */
	error(title, message) { this.show('error', title, message); }
	/** @param {string} title @param {string} message */
	warning(title, message) { this.show('warning', title, message); }
	/** @param {string} title @param {string} message */
	info(title, message) { this.show('info', title, message); }

	/** @param {string} id */
	dismiss(id) {
		this.items = this.items.filter((t) => t.id !== id);
	}
}

export const toast = new ToastStore();
