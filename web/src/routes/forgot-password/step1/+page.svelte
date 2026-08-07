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

	let email = $state('');
	let error = $state('');
	let isLoading = $state(false);

	$effect(() => {
		if (page.url.pathname !== '/forgot-password/step1') return;
		if ($authLoading) return;
		if ($user) goto('/projects', { replaceState: true });
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		isLoading = true;
		try {
			await authApi.requestPasswordReset(email);
			goto(`/forgot-password/step2?email=${encodeURIComponent(email)}`);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка отправки кода';
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
	<div class="auth-page">
		<div class="auth-container">
			<div class="auth-logo-wrapper">
				<img src="/logo.svg" alt="CFGY" class="auth-logo" />
			</div>

			<div class="auth-form-wrapper">
				<div class="auth-title">Введите почту, на которую зарегистрирован ваш аккаунт</div>

				<form onsubmit={handleSubmit} class="auth-form">
					<div class="auth-inputs">
						<Input
							type="email"
							placeholder="Email"
							bind:value={email}
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
						{isLoading ? 'Отправка...' : 'Отправить код'}
					</Button>
				</form>
			</div>
		</div>
	</div>
{/if}
