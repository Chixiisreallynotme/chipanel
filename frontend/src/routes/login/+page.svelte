<script>
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth.svelte.js';
	import { Lock, User, AlertCircle, ArrowRight, Eye, EyeOff } from '$lib/icons.js';
	import ChiPanelLogo from '$lib/components/common/ChiPanelLogo.svelte';

	let username = $state('');
	let password = $state('');
	let showPassword = $state(false);
	let isSubmitting = $state(false);

	async function handleSubmit(event) {
		event.preventDefault();
		if (!username.trim() || !password.trim()) return;

		isSubmitting = true;
		try {
			await auth.login(username, password);
			goto('/');
		} catch (err) {
			// auth.error is handled and exposed by auth store
		} finally {
			isSubmitting = false;
		}
	}
</script>

<svelte:head>
	<title>Connexion - ChiPanel Admin</title>
</svelte:head>

<div class="login-container">
	<div class="hardware-shell login-shell">
		<div class="hardware-core login-card">
			<div class="login-header">
				<div class="logo-wrapper">
					<ChiPanelLogo size="lg" status="stopped" />
				</div>
				<h1 class="login-title">ChiPanel</h1>
				<p class="login-subtitle">Administration Homelab & Serveurs de Jeu</p>
			</div>

			{#if auth.error}
				<div class="error-banner" role="alert">
					<AlertCircle size={18} class="error-icon" />
					<span class="error-text">{auth.error}</span>
				</div>
			{/if}

			<form onsubmit={handleSubmit} class="login-form">
				<div class="form-group">
					<label for="username" class="label label-required">Nom d'utilisateur</label>
					<div class="input-container">
						<User size={16} class="field-icon" />
						<input
							id="username"
							type="text"
							class="input"
							placeholder="Entrez votre nom d'utilisateur"
							bind:value={username}
							disabled={isSubmitting}
							required
							autocomplete="username"
						/>
					</div>
				</div>

				<div class="form-group">
					<label for="password" class="label label-required">Mot de passe</label>
					<div class="input-container">
						<Lock size={16} class="field-icon" />
						<input
							id="password"
							type={showPassword ? 'text' : 'password'}
							class="input"
							placeholder="Entrez votre mot de passe"
							bind:value={password}
							disabled={isSubmitting}
							required
							 autocomplete="current-password"
						/>
						<button
							type="button"
							class="password-toggle"
							onclick={() => (showPassword = !showPassword)}
							aria-label={showPassword ? 'Masquer le mot de passe' : 'Afficher le mot de passe'}
							title={showPassword ? 'Masquer le mot de passe' : 'Afficher le mot de passe'}
						>
							{#if showPassword}
								<EyeOff size={16} />
							{:else}
								<Eye size={16} />
							{/if}
						</button>
					</div>
				</div>

				<button
					type="submit"
					class="btn btn-primary btn-lg submit-btn {isSubmitting ? 'btn-loading' : ''}"
					disabled={isSubmitting || !username.trim() || !password.trim()}
				>
					{#if !isSubmitting}
						<span>Se connecter</span>
						<ArrowRight size={18} />
					{/if}
				</button>
			</form>
		</div>
	</div>
</div>

<style>
	.login-container {
		min-height: 100vh;
		min-height: 100dvh;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--bg-base);
		padding: var(--space-4);
		box-sizing: border-box;
		width: 100%;
		overflow-x: hidden;
	}

	.login-shell {
		width: 100%;
		max-width: 420px;
		background-color: rgba(255, 255, 255, 0.02);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-card);
		padding: 6px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
		animation: loginEntrance 180ms var(--ease-out);
		box-sizing: border-box;
	}

	@keyframes loginEntrance {
		from {
			opacity: 0;
			transform: scale(0.95);
		}
		to {
			opacity: 1;
			transform: scale(1);
		}
	}

	.login-card {
		width: 100%;
		background-color: var(--bg-surface);
		border: 1px solid rgba(255, 255, 255, 0.04);
		border-radius: calc(var(--radius-card) - 6px);
		box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.09), 0 8px 32px rgba(0, 0, 0, 0.4);
		padding: var(--space-6);
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
	}

	@media (max-width: 480px) {
		.login-container {
			padding: var(--space-3);
		}
		.login-card {
			padding: var(--space-4);
		}
	}

	.login-header {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		margin-bottom: var(--space-6);
	}

	.logo-wrapper {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: var(--space-4);
	}

	.login-title {
		font-size: var(--font-size-2xl);
		font-weight: var(--font-weight-bold);
		letter-spacing: -0.02em;
		color: var(--text-primary);
		margin-bottom: var(--space-1);
	}

	.login-subtitle {
		font-size: var(--font-size-sm);
		color: var(--text-muted);
	}

	.error-banner {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-3) var(--space-4);
		border-radius: var(--radius-btn);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		color: var(--danger-text);
		font-size: var(--font-size-sm);
		font-weight: var(--font-weight-medium);
		margin-bottom: var(--space-4);
		animation: alertSlideIn 150ms var(--ease-out);
	}

	@keyframes alertSlideIn {
		from {
			opacity: 0;
			transform: translateY(-4px) scale(0.98);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.error-banner :global(.error-icon) {
		flex-shrink: 0;
	}

	.login-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}

	.input-container {
		position: relative;
		display: flex;
		align-items: center;
	}

	.input-container :global(.field-icon) {
		position: absolute;
		left: var(--space-3);
		color: var(--text-muted);
		pointer-events: none;
	}

	.input-container .input {
		width: 100%;
		padding-left: calc(var(--space-3) * 2 + 16px);
		border: 1px solid var(--border);
		background-color: var(--bg-base);
		color: var(--text-primary);
		padding-right: 44px;
		border-radius: var(--radius-input);
		transition: border-color 150ms var(--ease-out), box-shadow 150ms var(--ease-out);
	}

	.input-container .input:focus {
		outline: none;
		border-color: var(--accent-blue);
		box-shadow: 0 0 0 2px var(--accent-blue-border);
	}

	.password-toggle {
		position: absolute;
		right: var(--space-2);
		width: 32px;
		height: 32px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border: 0;
		border-radius: var(--radius-input);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: color var(--transition-fast), background-color var(--transition-fast), transform var(--transition-fast);
	}

	.password-toggle:hover {
		color: var(--text-primary);
		background-color: var(--bg-elevated);
	}

	.password-toggle:active {
		transform: scale(0.97);
	}

	.submit-btn {
		width: 100%;
		margin-top: var(--space-2);
		font-weight: var(--font-weight-semibold);
		transition: transform 160ms var(--ease-out),
					background-color 150ms var(--ease-out),
					border-color 150ms var(--ease-out),
					opacity 150ms var(--ease-out);
	}

	.submit-btn:active:not(:disabled) {
		transform: scale(0.97);
	}
</style>
