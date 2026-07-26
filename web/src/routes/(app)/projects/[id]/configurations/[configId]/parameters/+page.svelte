<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { page } from '$app/state';
	import { projectsApi, configurationsApi, parametersApi } from '$lib/api';
	import type { Project, Configuration, Parameters } from '$lib/api';
	import { toastStore } from '$lib/toast';
	import Button from '$lib/components/Button.svelte';
	import Input from '$lib/components/Input.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

	const id = $derived(page.params.id);
	const configId = $derived(page.params.configId);

	let project = $state<Project | null>(null);
	let configuration = $state<Configuration | null>(null);
	let parameters = $state<Parameters>([]);
	let isLoading = $state(true);
	let error = $state('');
	let showAddForm = $state(false);
	let newKey = $state('');
	let newValue = $state('');
	let deleteConfirm = $state<string | null>(null);
	let editKey = $state<string | null>(null);
	let editValue = $state('');
	let revealed = $state<Record<string, boolean>>({});

	$effect(() => {
		if (editKey) {
			tick().then(() => {
				document.getElementById('edit-param-value')?.focus();
			});
		}
	});

	$effect(() => {
		if (showAddForm) {
			tick().then(() => {
				document.getElementById('new-param-key')?.focus();
			});
		}
	});

	onMount(() => {
		if (id && configId) loadData();
	});

	async function loadData() {
		if (!id || !configId) return;
		isLoading = true;
		error = '';
		try {
			project = await projectsApi.get(id);
			const configs = await configurationsApi.list(id);
			configuration = configs.find((c) => c.id === configId) ?? null;
			if (configuration) {
				parameters = await parametersApi.get(id, configuration.name);
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка загрузки';
		} finally {
			isLoading = false;
		}
	}

	async function handleAdd(e: Event) {
		e.preventDefault();
		if (!id || !configuration) return;
		if (!newKey.trim()) {
			toastStore.showError('Введите ключ');
			return;
		}
		try {
			const next = parameters.concat([{ key: newKey.trim(), value: newValue }]);
			await parametersApi.set(id, configuration.name, next);
			newKey = '';
			newValue = '';
			showAddForm = false;
			toastStore.showSuccess('Параметр добавлен');
			await loadData();
		} catch (err) {
			toastStore.showError(err instanceof Error ? err.message : 'Ошибка');
		}
	}

	function startEdit(key: string, value: string) {
		editKey = key;
		editValue = value;
	}

	function cancelEdit() {
		editKey = null;
		editValue = '';
	}

	async function handleEditSubmit(e: Event) {
		e.preventDefault();
		if (!id || !configuration || !editKey) return;
		try {
			await parametersApi.set(id, configuration.name, [{ key: editKey, value: editValue }]);
			cancelEdit();
			toastStore.showSuccess('Параметр обновлён');
			await loadData();
		} catch (err) {
			toastStore.showError(err instanceof Error ? err.message : 'Ошибка');
		}
	}

	function handleDelete(key: string) {
		deleteConfirm = key;
	}

	async function confirmDelete() {
		if (!id || !configuration || !deleteConfirm) return;
		try {
			await parametersApi.delete(id, configuration.name, deleteConfirm);
			toastStore.showSuccess('Параметр удалён');
			await loadData();
		} catch (err) {
			toastStore.showError(err instanceof Error ? err.message : 'Ошибка');
		} finally {
			deleteConfirm = null;
		}
	}
</script>

<svelte:head>
	<title>{configuration ? configuration.name : 'Параметры'} — CFGY</title>
</svelte:head>

{#if isLoading}
	<div class="text-center py-8">Загрузка...</div>
{:else if error || !project || !configuration}
	<div class="projects-error">{error || 'Конфигурация не найдена'}</div>
	<a href="/projects/{id}" class="text-basic-accent cursor-pointer">К конфигурациям</a>
{:else}
	<div class="projects-page">
		<nav class="flex gap-2 mb-4 text-regular-16">
			<a href="/projects" class="text-basic-accent">Проекты</a>
			<span>/</span>
			<a href="/projects/{id}" class="text-basic-accent">{project.name}</a>
			<span>/</span>
			<span>{configuration.name}</span>
		</nav>
		<div class="projects-header">
			<h1 class="projects-title">Параметры</h1>
			<Button variant="basic-secondary" size="big-text" onclick={() => (showAddForm = !showAddForm)}>
				{showAddForm ? 'Отмена' : 'Добавить параметр'}
			</Button>
		</div>

		{#if showAddForm}
			<form onsubmit={handleAdd} class="projects-form">
				<div class="flex gap-2 items-center w-full">
					<Input id="new-param-key" bind:value={newKey} placeholder="Ключ" className="flex-1" required />
					<Input bind:value={newValue} placeholder="Значение" className="flex-1" />
					<Button type="submit" variant="basic-accent" size="small-text">Сохранить</Button>
				</div>
			</form>
			<div class="projects-divider"></div>
		{/if}

		<div class="projects-list">
			{#if parameters.length === 0}
				<div class="projects-empty">Параметры не найдены. Добавьте первый.</div>
			{:else}
				{#each parameters as param (param.key)}
					<div class="projects-row">
						<span class="projects-item-link" style="pointer-events:none;">{param.key}</span>
						{#if editKey === param.key}
							<form onsubmit={handleEditSubmit} class="flex gap-2 items-center" style="flex:1;min-width:0;">
								<Input id="edit-param-value" bind:value={editValue} placeholder="Значение" className="flex-1" />
								<Button type="submit" variant="basic-accent" size="small-text">Сохранить</Button>
								<Button type="button" variant="basic-secondary" size="small-text" onclick={cancelEdit}>Отмена</Button>
							</form>
						{:else}
							<button
								type="button"
								class="text-regular-16 overflow-ellipsis text-left cursor-pointer"
								style="flex:1;min-width:0;background:none;border:none;"
								onclick={() => (revealed[param.key] = !revealed[param.key])}
							>
								{#if !param.value}(пусто){:else if revealed[param.key]}{param.value}{:else}••••••{/if}
							</button>
							<Button variant="basic-secondary" size="small-icon" onclick={() => startEdit(param.key, param.value)} aria-label="Редактировать">
								<svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M12 20h9M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
							</Button>
						{/if}
						<Button variant="basic-secondary" size="small-icon" onclick={() => handleDelete(param.key)} aria-label="Удалить">
							<svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
						</Button>
					</div>
				{/each}
			{/if}
		</div>
	</div>

	<ConfirmDialog
		open={deleteConfirm !== null}
		title="Удаление параметра"
		message={deleteConfirm ? `Удалить параметр "${deleteConfirm}"?` : ''}
		confirmLabel="Удалить"
		cancelLabel="Отмена"
		variant="danger"
		onConfirm={confirmDelete}
		onCancel={() => (deleteConfirm = null)}
	/>
{/if}
