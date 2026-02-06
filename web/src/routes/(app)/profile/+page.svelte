<script lang="ts">
	import { onMount } from 'svelte';
	import { user } from '$lib/auth';
	import { getDisplayName, getInitialsFromEmail } from '$lib/avatar';
	import { tokensApi } from '$lib/api';
	import type { Token, CreateTokenResponse } from '$lib/api';
	import { toastStore } from '$lib/toast';
	import Avatar from '$lib/components/Avatar.svelte';
	import Button from '$lib/components/Button.svelte';
	import Input from '$lib/components/Input.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

	let tokens = $state<Token[]>([]);
	let isLoading = $state(true);
	let error = $state('');
	let showCreateForm = $state(false);
	let newToken = $state<CreateTokenResponse | null>(null);
	let tokenName = $state('');
	let deleteConfirm = $state<{ id: string; name: string } | null>(null);

	onMount(() => {
		loadTokens();
	});

	async function loadTokens() {
		isLoading = true;
		error = '';
		try {
			tokens = await tokensApi.list();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка загрузки токенов';
		} finally {
			isLoading = false;
		}
	}

	async function handleCreate() {
		if (!tokenName.trim()) {
			toastStore.showError('Введите имя токена');
			return;
		}
		error = '';
		try {
			const response = await tokensApi.create({ name: tokenName.trim() });
			newToken = response;
			await loadTokens();
			toastStore.showSuccess('Токен создан');
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Ошибка создания токена';
			error = message;
			toastStore.showError(message);
		}
	}

	function handleSaveAndClose() {
		showCreateForm = false;
		newToken = null;
		tokenName = '';
		error = '';
	}

	function handleDelete(id: string, name: string) {
		deleteConfirm = { id, name };
	}

	async function confirmDelete() {
		if (!deleteConfirm) return;
		try {
			await tokensApi.delete(deleteConfirm.id);
			toastStore.showSuccess('Токен удалён');
			await loadTokens();
		} catch (err) {
			toastStore.showError(err instanceof Error ? err.message : 'Ошибка');
		} finally {
			deleteConfirm = null;
		}
	}

	async function copyToken(token: string) {
		try {
			await navigator.clipboard.writeText(token);
			toastStore.showSuccess('Токен скопирован');
		} catch {
			toastStore.showError('Не удалось скопировать');
		}
	}
</script>

<svelte:head>
	<title>Профиль — CFGY</title>
</svelte:head>

<div class="projects-page">
	<h1 class="projects-title">Профиль</h1>

	<div class="flex gap-4 items-center mb-6">
		<Avatar size="big" initials={getInitialsFromEmail($user?.email ?? 'U')} />
		<div>
			<p class="text-style-regular-24">{getDisplayName($user)}</p>
			{#if $user?.email}
				<p class="text-regular-16 text-basic-gray">{$user.email}</p>
			{/if}
		</div>
	</div>

	<h2 class="text-style-regular-20 mb-4">API-токены</h2>
	{#if isLoading}
		<div class="text-center py-4">Загрузка...</div>
	{:else}
		{#if error}
			<p class="text-states-error text-regular-16">{error}</p>
		{/if}
		<Button variant="basic-secondary" size="big-text" onclick={() => { showCreateForm = true; newToken = null; tokenName = ''; error = ''; }}>
			Создать токен
		</Button>
		<div class="projects-list mt-4">
			{#if tokens.length === 0}
				<div class="projects-empty">Токенов нет. Создайте токен для CLI/CI.</div>
			{:else}
				{#each tokens as token (token.id)}
					<div class="projects-row">
						<span class="projects-item-link">{token.name}</span>
						<Button variant="basic-secondary" size="small-icon" onclick={() => handleDelete(token.id, token.name)} aria-label="Удалить токен">
							<svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
						</Button>
					</div>
				{/each}
			{/if}
		</div>
	{/if}
</div>

<Modal open={showCreateForm} title={newToken ? 'Токен создан' : 'Новый токен'} onClose={handleSaveAndClose}>
	{#if newToken}
		<p class="text-regular-16 mb-4">Скопируйте токен — он больше не будет показан.</p>
		<div class="flex gap-2 items-center">
			<Input value={newToken.token} readonly className="flex-1" />
			<Button variant="basic-secondary" size="small-text" onclick={() => copyToken(newToken!.token)}>Копировать</Button>
		</div>
		<p class="text-regular-16 mt-4 text-basic-gray">Используйте в CLI: CFGY_TOKEN=... CFGY_SERVER_URL=...</p>
	{:else}
		<form onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
			<Input bind:value={tokenName} placeholder="Имя токена" className="w-full mb-4" required />
			{#if error}
				<p class="text-states-error text-regular-16 mb-2">{error}</p>
			{/if}
			<div class="flex gap-2 justify-end">
				<Button type="button" variant="secondary" onclick={handleSaveAndClose}>Отмена</Button>
				<Button type="submit" variant="basic-accent">Создать</Button>
			</div>
		</form>
	{/if}
</Modal>

<ConfirmDialog
	open={deleteConfirm !== null}
	title="Удаление токена"
	message={deleteConfirm ? `Удалить токен "${deleteConfirm.name}"?` : ''}
	confirmLabel="Удалить"
	cancelLabel="Отмена"
	variant="danger"
	onConfirm={confirmDelete}
	onCancel={() => (deleteConfirm = null)}
/>
