<script>
	import { ArrowRight, LoaderCircle } from 'lucide-svelte';

	/**
	 * @type {{
	 *   title: string,
	 *   description: string,
	 *   href: string,
	 *   actionLabel: string,
	 *   children?: import('svelte').Snippet
	 * }}
	 */
	let { title, description, href, actionLabel, children } = $props();
</script>

<section class="redirect-state" aria-live="polite">
	<div class="redirect-panel">
		<div class="redirect-icon">
			{#if children}
				{@render children()}
			{:else}
				<LoaderCircle size={22} />
			{/if}
		</div>
		<div class="redirect-copy">
			<span class="micro-stamp micro-stamp-active">ROUTE / RESOLVING</span>
			<h1>{title}</h1>
			<p>{description}</p>
		</div>
		<div class="redirect-progress" aria-hidden="true"><span></span></div>
		<a class="btn btn-secondary btn-sm" href={href}>
			<span>{actionLabel}</span>
			<ArrowRight size={15} />
		</a>
	</div>
</section>

<style>
	.redirect-state {
		display: grid;
		place-items: center;
		min-height: min(52vh, 460px);
	}

	.redirect-panel {
		width: min(100%, 520px);
		display: grid;
		grid-template-columns: auto minmax(0, 1fr);
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-6);
		background: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: var(--bevel-top);
	}

	.redirect-icon {
		width: 44px;
		height: 44px;
		display: grid;
		place-items: center;
		border: 1px solid var(--accent-blue-border);
		border-radius: var(--radius-btn);
		background: var(--accent-blue-bg);
		color: var(--accent-blue-text);
	}

	.redirect-icon :global(svg) {
		animation: redirect-spin 1.2s linear infinite;
	}

	.redirect-copy {
		min-width: 0;
	}

	h1 {
		margin: var(--space-2) 0 var(--space-1);
		font-size: var(--font-size-lg);
		font-weight: var(--font-weight-semibold);
		letter-spacing: -0.015em;
		text-wrap: balance;
	}

	p {
		margin: 0;
		font-size: var(--font-size-sm);
		color: var(--text-secondary);
	}

	.redirect-progress {
		grid-column: 1 / -1;
		height: 2px;
		overflow: hidden;
		background: var(--bg-base);
		border-radius: var(--radius-badge);
	}

	.redirect-progress span {
		display: block;
		width: 38%;
		height: 100%;
		background: var(--accent-blue);
		transform-origin: left;
		animation: redirect-progress 1.4s var(--ease-in-out) infinite;
	}

	.redirect-panel > .btn {
		grid-column: 2;
		justify-self: start;
	}

	@keyframes redirect-spin {
		to { transform: rotate(360deg); }
	}

	@keyframes redirect-progress {
		0% { transform: translateX(-120%) scaleX(0.7); }
		50% { transform: translateX(80%) scaleX(1); }
		100% { transform: translateX(260%) scaleX(0.7); }
	}

	@media (max-width: 520px) {
		.redirect-state {
			min-height: 42vh;
		}

		.redirect-panel {
			grid-template-columns: auto minmax(0, 1fr);
			padding: var(--space-4);
		}

		.redirect-panel > .btn {
			grid-column: 1 / -1;
			justify-self: stretch;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.redirect-icon :global(svg),
		.redirect-progress span {
			animation: none;
		}
	}
</style>
