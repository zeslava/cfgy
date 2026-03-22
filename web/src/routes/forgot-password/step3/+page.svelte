<script lang="ts">
	import { user, authLoading } from '$lib/auth';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authApi } from '$lib/api';
	import { toastStore } from '$lib/toast';
	import { setDocumentTitle } from '$lib/documentTitle';
	import Button from '$lib/components/Button.svelte';
	import Input from '$lib/components/Input.svelte';

	setDocumentTitle('Восстановление пароля');

	const email = page.url.searchParams.get('email') ?? '';
	const code = page.url.searchParams.get('code') ?? '';

	let newPassword = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let isLoading = $state(false);

	$effect(() => {
		if (page.url.pathname !== '/forgot-password/step3') return;
		if ($authLoading) return;
		if ($user) goto('/projects', { replaceState: true });
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		if (newPassword !== confirmPassword) {
			error = 'Пароли не совпадают';
			return;
		}

		isLoading = true;
		try {
			await authApi.resetPassword(email, code, newPassword);
			toastStore.showSuccess('Пароль успешно изменён');
			goto('/login');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка смены пароля';
			toastStore.showError(error);
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head><title>Восстановление пароля — CFGY</title></svelte:head>

{#if $authLoading}
	<div class="flex items-center justify-center min-h-screen">Загрузка...</div>
{:else if $user}
	<div class="flex items-center justify-center min-h-screen">Перенаправление...</div>
{:else}
	<div class="login-page">
		<div class="login-container">
			<div class="login-logo-wrapper">
				<img src="/logo.svg" alt="CFGY" class="login-logo" />
			</div>

			<div class="login-form-wrapper">
				<div class="login-title">Создайте новый пароль:</div>

				<form onsubmit={handleSubmit} class="login-form">
					<div class="login-inputs">
						<Input
							type="password"
							placeholder="Новый пароль"
							bind:value={newPassword}
							required
							minlength={8}
							disabled={isLoading}
							showPasswordToggle
							state={error ? 'error' : 'default'}
							className="w-full"
						/>
						<Input
							type="password"
							placeholder="Подтвердите пароль"
							bind:value={confirmPassword}
							required
							minlength={8}
							disabled={isLoading}
							showPasswordToggle
							state={error ? 'error' : 'default'}
							description={error || undefined}
							className="w-full"
						/>
					</div>

					<Button
						type="submit"
						variant="basic-accent"
						size="big-text"
						disabled={isLoading}
					>
						{isLoading ? 'Сохранение...' : 'Сменить пароль'}
					</Button>
				</form>
			</div>
		</div>
	</div>
{/if}
