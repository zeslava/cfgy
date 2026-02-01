<script lang="ts">
	import { goto } from '$app/navigation';
	import { user, authLoading } from '$lib/auth';
	import Header from '$lib/components/Header.svelte';

	let { children } = $props();

	// Защита маршрутов: редирект на /login если не авторизован
	$effect(() => {
		if ($authLoading) return;
		if (!$user) {
			goto('/login', { replaceState: true });
		}
	});
</script>

{#if $authLoading}
	<div class="flex items-center justify-center min-h-screen">
		Загрузка...
	</div>
{:else if $user}
	<div class="layout">
		<Header />
		<main class="layout-main">
			{@render children()}
		</main>
	</div>
{:else}
	<!-- редирект в $effect -->
	<div class="flex items-center justify-center min-h-screen">Перенаправление...</div>
{/if}
