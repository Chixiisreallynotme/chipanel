<script>
	/**
	 * PageHeader — canonical page header for every cockpit page.
	 * Reference: /engine (Design Council I standard). One structure, one
	 * typography scale, tokenized. `icon` and `badge` are optional snippets;
	 * `children` renders the right-hand action area.
	 *
	 * @type {{
	 *   title: string,
	 *   subtitle?: string,
	 *   icon?: import('svelte').Snippet,
	 *   badge?: import('svelte').Snippet,
	 *   children?: import('svelte').Snippet
	 * }}
	 */
	let { title, subtitle = '', icon, badge, children } = $props();
</script>

<header class="page-header">
	<div class="header-main">
		<div class="header-titles">
			{#if icon}
				<span class="header-icon-box">{@render icon()}</span>
			{/if}
			<div class="header-text">
				<div class="title-row">
					<h1 class="page-title text-balance">{title}</h1>
					{#if badge}
						<span class="title-badge">{@render badge()}</span>
					{/if}
				</div>
				{#if subtitle}
					<p class="page-subtitle">{subtitle}</p>
				{/if}
			</div>
		</div>
		{#if children}
			<div class="header-actions">{@render children()}</div>
		{/if}
	</div>
</header>

<style>
	.page-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.header-main {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.header-titles {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		min-width: 0;
	}

	.header-icon-box {
		width: 40px;
		height: 40px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-btn);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		color: var(--accent-blue-text);
		box-shadow: var(--bevel-subtle);
	}

	.header-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.title-row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.page-title {
		font-size: 1.75rem;
		font-weight: var(--font-weight-bold);
		letter-spacing: -0.02em;
		line-height: 1.2;
		color: var(--text-primary);
		margin: 0;
	}

	.page-subtitle {
		font-size: var(--font-size-base);
		color: var(--text-secondary);
		margin: 0;
	}

	.title-badge {
		display: inline-flex;
		align-items: center;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}
</style>
