<script>
	/** @import { Snippet } from 'svelte'; */
	/**
	 * Identity banner for the active engine/server. `kpis` usually hosts a KpiGrid.
	 * @type {{
	 *   badge?: Snippet,
	 *   title: string,
	 *   titleExtra?: Snippet,
	 *   visual?: Snippet,
	 *   meta?: Snippet,
	 *   kpis?: Snippet
	 * }}
	 */
	let { badge, title, titleExtra, visual, meta, kpis } = $props();
</script>

<section class="banner-card">
	<div class="banner-primary">
		{#if badge}<div class="banner-badge">{@render badge()}</div>{/if}
		<div class="banner-identity">
			{#if visual}{@render visual()}{/if}
			<div class="identity-info">
				<div class="title-row">
					<span class="banner-title">{title}</span>
					{#if titleExtra}{@render titleExtra()}{/if}
				</div>
				{#if meta}{@render meta()}{/if}
			</div>
		</div>
	</div>
	{#if kpis}{@render kpis()}{/if}
</section>

<style>
	.banner-card {
		display: grid;
		grid-template-columns: 1.35fr 1fr;
		gap: var(--space-6);
		padding: var(--space-6);
		border-radius: var(--radius-card);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		box-shadow: var(--card-shadow);
	}

	.banner-primary {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}

	.banner-badge {
		display: inline-flex;
		align-items: center;
		gap: var(--space-2);
		font-size: var(--font-size-xs);
		font-weight: var(--font-weight-bold);
		letter-spacing: 0.05em;
		color: var(--accent-blue-text);
	}

	.banner-identity {
		display: flex;
		align-items: flex-start;
		gap: var(--space-5);
	}

	.identity-info {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}

	.banner-title {
		font-size: 1.75rem;
		font-weight: var(--font-weight-bold);
		letter-spacing: -0.02em;
		color: var(--text-primary);
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	@media (max-width: 960px) {
		.banner-card { grid-template-columns: 1fr; }
	}
</style>
