<script>
	/**
	 * Application confirmation dialog — the rendering replacement for `window.confirm()`.
	 *
	 * The native dialog was accessible for free (modal, focus-managed, Escape-cancellable,
	 * announced). Anything replacing it has to earn that back, so this component is
	 * deliberately not just a styled div: `role="alertdialog"` + `aria-modal`, labelled by
	 * its own title and described by its message, focus moved in on open and restored to
	 * the trigger on close, Tab trapped inside, Escape cancels, and the destructive button
	 * is never the initially-focused control.
	 *
	 * The confirmation semantics are unchanged: nothing runs until `onconfirm` fires.
	 */

	/**
	 * @type {{
	 *   open: boolean,
	 *   title: string,
	 *   message: string,
	 *   confirmLabel?: string,
	 *   cancelLabel?: string,
	 *   destructive?: boolean,
	 *   busy?: boolean,
	 *   onconfirm: () => void,
	 *   oncancel: () => void
	 * }}
	 */
	let {
		open,
		title,
		message,
		confirmLabel = 'Confirmer',
		cancelLabel = 'Annuler',
		destructive = true,
		busy = false,
		onconfirm,
		oncancel
	} = $props();

	/** @type {HTMLDivElement | null} */
	let dialogEl = $state(null);
	/** @type {HTMLButtonElement | null} */
	let cancelEl = $state(null);
	/** @type {Element | null} */
	let previouslyFocused = null;

	const titleId = `confirm-title-${Math.random().toString(36).slice(2, 9)}`;
	const messageId = `confirm-message-${Math.random().toString(36).slice(2, 9)}`;

	$effect(() => {
		if (!open) return;

		previouslyFocused = document.activeElement;
		// Focus the non-destructive choice: an alertdialog that opens with "Supprimer"
		// pre-focused turns a stray Enter into the very action it is guarding.
		queueMicrotask(() => cancelEl?.focus());

		return () => {
			if (previouslyFocused instanceof HTMLElement) previouslyFocused.focus();
			previouslyFocused = null;
		};
	});

	/** @param {KeyboardEvent} event */
	function handleKeydown(event) {
		if (event.key === 'Escape') {
			event.preventDefault();
			oncancel();
			return;
		}
		if (event.key !== 'Tab' || !dialogEl) return;

		const focusable = /** @type {HTMLElement[]} */ ([
			...dialogEl.querySelectorAll('button:not([disabled]), [href], input, select, textarea')
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
	<div class="modal-backdrop confirm-backdrop" onclick={() => oncancel()}>
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			bind:this={dialogEl}
			class="modal confirm-modal"
			role="alertdialog"
			tabindex="-1"
			aria-modal="true"
			aria-labelledby={titleId}
			aria-describedby={messageId}
			onclick={(e) => e.stopPropagation()}
		>
			<div class="modal-header">
				<h2 class="modal-title" id={titleId}>{title}</h2>
			</div>
			<div class="modal-body">
				<p class="confirm-message" id={messageId}>{message}</p>
			</div>
			<div class="modal-footer">
				<button
					bind:this={cancelEl}
					type="button"
					class="btn btn-secondary"
					onclick={() => oncancel()}
					disabled={busy}
				>
					{cancelLabel}
				</button>
				<button
					type="button"
					class="btn {destructive ? 'btn-danger' : 'btn-primary'} {busy ? 'btn-loading' : ''}"
					onclick={() => onconfirm()}
					disabled={busy}
				>
					{confirmLabel}
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	/* Confirmations routinely fire from inside another modal (profile delete, group delete),
	   so this sits one step above the modal tier rather than inventing a new number. */
	.confirm-backdrop {
		z-index: calc(var(--z-modal) + 1);
	}

	.confirm-modal {
		max-width: 440px;
	}

	.confirm-message {
		color: var(--text-secondary);
		font-size: var(--font-size-base);
		line-height: var(--line-height-relaxed);
		margin: 0;
	}
</style>
