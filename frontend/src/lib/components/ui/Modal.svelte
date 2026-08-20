<script>
	/**
	 * Shared modal shell — backdrop + dialog panel with the full a11y contract:
	 * `role`/`aria-modal`/`aria-labelledby`, `tabindex="-1"`, focus moved in on open
	 * (first focusable control) and restored to the trigger on close, Escape closes,
	 * Tab is trapped inside the panel. Styles come from the global `.modal-backdrop`
	 * and `.modal` classes; every consumer passes its own panel classes via `class`
	 * (e.g. "modal world-modal", "modal-dialog", "modal-card version-picker-modal").
	 *
	 * The behaviour contract mirrors ConfirmDialog so that every modal in the app
	 * earns back what a native dialog gives for free. Close guards (busy states)
	 * belong in `onclose`: it is called for backdrop clicks AND Escape alike.
	 */
	let {
		open = false,
		onclose = () => {},
		class: klass = '',
		backdropClass = '',
		role = 'dialog',
		ariaLabelledBy = undefined,
		initialFocus = '',
		closeOnBackdrop = true,
		content
	} = $props();

	/** @type {HTMLDivElement | null} */
	let panelEl = $state(null);
	/** @type {Element | null} */
	let previouslyFocused = null;

	$effect(() => {
		if (!open) return;

		previouslyFocused = document.activeElement;
		queueMicrotask(() => {
			if (!panelEl) return;
			const target = initialFocus
				? panelEl.querySelector(initialFocus)
				: panelEl.querySelector('button:not([disabled]), [href], input, select, textarea');
			if (target instanceof HTMLElement) target.focus();
			else panelEl.focus();
		});

		return () => {
			if (previouslyFocused instanceof HTMLElement) previouslyFocused.focus();
			previouslyFocused = null;
		};
	});

	/** @param {KeyboardEvent} event */
	function handleKeydown(event) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onclose();
			return;
		}
		if (event.key !== 'Tab' || !panelEl) return;

		const focusable = /** @type {HTMLElement[]} */ ([
			...panelEl.querySelectorAll('button:not([disabled]), [href], input, select, textarea')
		]);
		if (focusable.length === 0) return;

		const first = focusable[0];
		const last = focusable[focusable.length - 1];
		const active = document.activeElement;

		if (event.shiftKey && active === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && active === last) {
			event.preventDefault();
			first.focus();
		}
	}
</script>

<svelte:window onkeydown={open ? handleKeydown : undefined} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="modal-backdrop {backdropClass}"
		onclick={() => (closeOnBackdrop ? onclose() : undefined)}
	>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			bind:this={panelEl}
			class={klass}
			role={role}
			tabindex="-1"
			aria-modal="true"
			aria-labelledby={ariaLabelledBy}
			onclick={(e) => e.stopPropagation()}
		>
			{@render content?.()}
		</div>
	</div>
{/if}