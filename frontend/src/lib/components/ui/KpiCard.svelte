<script>
	/** @import { Snippet } from 'svelte'; */
	/**
	 * Canonical KPI tile. `icon` fills the 42px status tile; `children` is the value
	 * (preferred when it needs formatting). `tone` only colours the icon tile.
	 * @type {{
	 *   label: string,
	 *   value?: string,
	 *   icon?: Snippet,
	 *   tone?: 'info' | 'success' | 'warning' | 'danger',
	 *   mono?: boolean,
	 *   children?: Snippet
	 * }}
	 */
	let { label, value = '', icon, tone = 'info', mono = false, children } = $props();
</script>

<div class="kpi-card">
	{#if icon}
		<div class="kpi-icon kpi-{tone}">{@render icon()}</div>
	{/if}
	<div class="kpi-content">
		<span class="kpi-label">{label}</span>
		<div class="kpi-value {mono ? 'kpi-mono' : ''}">
			{#if children}{@render children()}{:else}{value}{/if}
		</div>
	</div>
</div>

<style>
	.kpi-card {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-4);
		border-radius: var(--radius-card);
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		min-width: 0;
	}

	.kpi-icon {
		width: 42px;
		height: 42px;
		flex-shrink: 0;
		border-radius: var(--radius-md);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.kpi-info { background-color: var(--accent-blue-bg); color: var(--accent-blue-text); }
	.kpi-success { background-color: var(--accent-green-bg); color: var(--accent-green-text); }
	.kpi-warning { background-color: var(--warning-bg); color: var(--warning-text); }
	.kpi-danger { background-color: var(--danger-bg); color: var(--danger-text); }

	.kpi-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.kpi-label {
		font-size: var(--font-size-xs);
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.kpi-value {
		font-size: var(--font-size-base);
		font-weight: var(--font-weight-bold);
		color: var(--text-primary);
		font-variant-numeric: tabular-nums;
		overflow-wrap: anywhere;
	}

	.kpi-mono {
		font-family: var(--font-mono);
		font-size: var(--font-size-sm);
	}
</style>
