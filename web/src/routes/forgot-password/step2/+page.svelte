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

	let email = $state(page.url.searchParams.get('email') ?? '');
	let code = $state('');
	let error = $state('');
	let isLoading = $state(false);
	let isResending = $state(false);

	$effect(() => {
		if (page.url.pathname !== '/forgot-password/step2') return;
		if ($authLoading) return;
		if ($user) goto('/projects', { replaceState: true });
	});

	async function handleResend(e: Event) {
		e.preventDefault();
		error = '';
		isResending = true;
		try {
			await authApi.requestPasswordReset(email);
			toastStore.showSuccess('Код отправлен повторно');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка отправки кода';
			toastStore.showError(error);
		} finally {
			isResending = false;
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		isLoading = true;
		try {
			await authApi.verifyCode(email, code);
			goto(`/forgot-password/step3?email=${encodeURIComponent(email)}&code=${encodeURIComponent(code)}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Неверный код';
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
				<div class="login-title">Введите почту, на которую зарегистрирован ваш аккаунт</div>

				<form onsubmit={handleResend} class="login-form">
					<div class="login-inputs">
						<Input
							type="email"
							placeholder="Email"
							bind:value={email}
							required
							disabled={isResending || isLoading}
							className="w-full"
						/>
					</div>

					<Button
						type="submit"
						variant="basic-accent"
						size="big-text"
						disabled={isResending || isLoading}
					>
						{isResending ? 'Отправка...' : 'Отправить код'}
					</Button>
				</form>

				<form onsubmit={handleSubmit} class="login-form">
					<div class="login-title">Введите код из письма:</div>

					<div class="login-inputs">
						<Input
							type="text"
							placeholder="Код"
							bind:value={code}
							required
							disabled={isLoading}
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
						{isLoading ? 'Проверка...' : 'Изменить пароль'}
					</Button>
				</form>
			</div>
		</div>
	</div>
{/if}
