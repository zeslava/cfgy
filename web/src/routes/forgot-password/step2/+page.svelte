<script lang="ts">
	import { user, authLoading } from '$lib/auth';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	$effect(() => {
		if (page.url.pathname !== '/forgot-password/step2') return;
		if ($authLoading) return;
		if ($user) goto('/projects', { replaceState: true });
	});
</script>

<svelte:head><title>Восстановление пароля — CFGY</title></svelte:head>

{#if $authLoading}
	<div class="flex items-center justify-center min-h-screen">Загрузка...</div>
{:else if $user}
	<div class="flex items-center justify-center min-h-screen">Перенаправление...</div>
{:else}
	<div class="flex flex-col items-center justify-center min-h-screen gap-6">
		<h1 class="text-style-regular-24">Восстановление пароля (шаг 2)</h1>
		<p>Ввод кода и нового пароля (в Фазе 3).</p>
		<a href="/login" class="text-basic-accent cursor-pointer">Назад к входу</a>
	</div>
{/if}
