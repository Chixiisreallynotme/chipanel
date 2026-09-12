<script>
	/** @import { Snippet } from 'svelte'; */
	/** @type {{ badge?: Snippet, title: string, titleExtra?: Snippet, visual?: Snippet, meta?: Snippet, kpis?: Snippet }} */
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
		gap: 1.5rem;
		padding: 1.75rem;
		border-radius: var(--radius-engine-lg);
		background: var(--bg-surface-2);
		border: 1px solid var(--border-engine-08);
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
	}
	.banner-primary { display: flex; flex-direction: column; gap: 0.875rem; }
	.banner-badge {
		display: inline-flex; align-items: center; gap: 0.45rem;
		font-size: 0.6875rem; font-weight: 700; letter-spacing: 0.05em;
		color: var(--accent-indigo-text);
	}
	.banner-identity { display: flex; align-items: flex-start; gap: 1.25rem; }
	.identity-info { display: flex; flex-direction: column; gap: 0.5rem; }
	.banner-title { font-size: 1.75rem; font-weight: 800; letter-spacing: -0.02em; color: var(--engine-slate-50); }
	.title-row { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
	@media (max-width: 960px) { .banner-card { grid-template-columns: 1fr; } }
</style>
