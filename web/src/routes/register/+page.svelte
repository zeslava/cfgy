<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { user, authLoading, register } from '$lib/auth';
	import { authApi } from '$lib/api';
	import Button from '$lib/components/Button.svelte';
	import Input from '$lib/components/Input.svelte';

	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let isLoading = $state(false);
	let registrationOpen = $state<boolean | null>(null);

	authApi
		.registrationStatus()
		.then((s) => (registrationOpen = s.open))
		.catch(() => (registrationOpen = true));

	$effect(() => {
		if (page.url.pathname !== '/register') return;
		if ($authLoading) return;
		if ($user) goto('/projects', { replaceState: true });
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		if (password !== confirmPassword) {
			error = 'Пароли не совпадают';
			return;
		}
		if (password.length < 8) {
			error = 'Пароль должен быть не менее 8 символов';
			return;
		}
		isLoading = true;
		try {
			await register(email, password);
			goto('/projects');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка регистрации';
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Регистрация — CFGY</title>
</svelte:head>

{#if $authLoading || registrationOpen === null}
	<div class="flex items-center justify-center min-h-screen">Загрузка...</div>
{:else if $user}
	<div class="flex items-center justify-center min-h-screen">Перенаправление...</div>
{:else if !registrationOpen}
	<div class="auth-page">
		<div class="auth-container">
			<div class="auth-logo-wrapper">
				<img src="/logo.svg" alt="CFGY" class="auth-logo" />
			</div>
			<p class="text-regular-16 text-center">Регистрация закрыта. Обратитесь к администратору.</p>
			<p class="text-regular-16">
				<a href="/login" class="text-basic-accent cursor-pointer">Войти</a>
			</p>
		</div>
	</div>
{:else}
	<div class="auth-page">
		<div class="auth-container">
			<div class="auth-logo-wrapper">
				<img src="/logo.svg" alt="CFGY" class="auth-logo" />
			</div>
			<div class="auth-form-wrapper auth-form-wrapper--start">
				<div class="auth-title">Регистрация:</div>
				<form onsubmit={handleSubmit} class="auth-form auth-form--wide">
					<div class="auth-inputs">
						<Input
							type="email"
							placeholder="Email"
							bind:value={email}
							required
							disabled={isLoading}
							state={error ? 'error' : 'default'}
							className="w-full"
						/>
						<Input
							type="password"
							placeholder="Пароль"
							bind:value={password}
							required
							disabled={isLoading}
							minlength={8}
							showPasswordToggle
							state={error ? 'error' : 'default'}
							className="w-full"
						/>
						<Input
							type="password"
							placeholder="Повторите пароль"
							bind:value={confirmPassword}
							required
							disabled={isLoading}
							minlength={8}
							showPasswordToggle
							state={error ? 'error' : 'default'}
							className="w-full"
						/>
					</div>
					<Button type="submit" variant="basic-secondary" size="big-text" disabled={isLoading}>
						{isLoading ? 'Регистрация...' : 'Зарегистрироваться'}
					</Button>
					{#if error}
						<p class="text-states-error text-regular-16 text-center w-full">{error}</p>
					{/if}
				</form>
			</div>
			<p class="text-regular-16">
				Уже есть аккаунт? <a href="/login" class="text-basic-accent cursor-pointer">Войти</a>
			</p>
		</div>
	</div>
{/if}
