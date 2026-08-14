<script>
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth.svelte.js';
	import { Server, Lock, User, AlertCircle, ArrowRight } from 'lucide-svelte';

	let username = $state('');
	let password = $state('');
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
	<title>Login - ChiPanel Admin</title>
</svelte:head>

<div class="login-container">
	<div class="login-card">
		<div class="login-header">
			<div class="logo-wrapper">
				<Server size={32} class="logo-icon" />
			</div>
			<h1 class="login-title">ChiPanel</h1>
			<p class="login-subtitle">Game Server & Homelab Administration</p>
		</div>

		{#if auth.error}
			<div class="error-banner" role="alert">
				<AlertCircle size={18} class="error-icon" />
				<span class="error-text">{auth.error}</span>
			</div>
		{/if}

		<form onsubmit={handleSubmit} class="login-form">
			<div class="form-group">
				<label for="username" class="label label-required">Username</label>
				<div class="input-container">
					<User size={16} class="field-icon" />
					<input
						id="username"
						type="text"
						class="input"
						placeholder="Enter your username"
						bind:value={username}
						disabled={isSubmitting}
						required
						autocomplete="username"
					/>
				</div>
			</div>

			<div class="form-group">
				<label for="password" class="label label-required">Password</label>
				<div class="input-container">
					<Lock size={16} class="field-icon" />
					<input
						id="password"
						type="password"
						class="input"
						placeholder="Enter your password"
						bind:value={password}
						disabled={isSubmitting}
						required
						autocomplete="current-password"
					/>
				</div>
			</div>

			<button
				type="submit"
				class="btn btn-primary btn-lg submit-btn {isSubmitting ? 'btn-loading' : ''}"
				disabled={isSubmitting || !username.trim() || !password.trim()}
			>
				{#if !isSubmitting}
					<span>Sign In</span>
					<ArrowRight size={18} />
				{/if}
			</button>
		</form>
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
	}

	.login-card {
		width: 100%;
		max-width: 420px;
		background-color: var(--bg-surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		box-shadow: var(--elevation-shadow);
		padding: var(--space-8);
		display: flex;
		flex-direction: column;
	}

	.login-header {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		margin-bottom: var(--space-6);
	}

	.logo-wrapper {
		width: 56px;
		height: 56px;
		border-radius: var(--radius-card);
		background-color: var(--accent-blue-bg);
		border: 1px solid var(--accent-blue-border);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-blue-text);
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
		animation: modalSlideUp var(--transition-fast) ease-out;
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
		padding-left: calc(var(--space-3) * 2 + 16px);
	}

	.submit-btn {
		width: 100%;
		margin-top: var(--space-2);
		font-weight: var(--font-weight-semibold);
	}
</style>
