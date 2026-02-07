<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { projectsApi, configurationsApi } from '$lib/api';
	import type { Project, Configuration } from '$lib/api';
	import { toastStore } from '$lib/toast';
	import { sortByString } from '$lib/sort';
	import Button from '$lib/components/Button.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

	const id = $derived(page.params.id);

	let project = $state<Project | null>(null);
	let configurations = $state<Configuration[]>([]);
	let isLoading = $state(true);
	let error = $state('');
	let showCreateForm = $state(false);
	let configName = $state('');
	let createError = $state('');
	let deleteConfirm = $state<{ configId: string; name: string } | null>(null);

	onMount(() => {
		if (id) {
			loadProject();
			loadConfigurations();
		}
	});

	async function loadProject() {
		if (!id) return;
		isLoading = true;
		error = '';
		try {
			project = await projectsApi.get(id);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка загрузки проекта';
		} finally {
			isLoading = false;
		}
	}

	async function loadConfigurations() {
		if (!id) return;
		try {
			configurations = await configurationsApi.list(id);
		} catch (err) {
			console.error('Ошибка загрузки конфигураций:', err);
		}
	}

	async function handleCreate(e: Event) {
		e.preventDefault();
		if (!id) return;
		createError = '';
		if (!configName.trim()) {
			createError = 'Название конфигурации обязательно';
			return;
		}
		try {
			await configurationsApi.create(id, { name: configName.trim() });
			configName = '';
			showCreateForm = false;
			toastStore.showSuccess('Конфигурация создана');
			await loadConfigurations();
		} catch (err) {
			createError = err instanceof Error ? err.message : 'Ошибка создания конфигурации';
		}
	}

	function handleDelete(configId: string, name: string) {
		deleteConfirm = { configId, name };
	}

	async function confirmDelete() {
		if (!id || !deleteConfirm) return;
		try {
			await configurationsApi.delete(id, deleteConfirm.configId);
			toastStore.showSuccess(`Конфигурация "${deleteConfirm.name}" удалена`);
			await loadConfigurations();
		} catch (err) {
			toastStore.showError(err instanceof Error ? err.message : 'Ошибка удаления');
		} finally {
			deleteConfirm = null;
		}
	}

	const sortedConfigs = $derived(sortByString(configurations, (c) => c.name));
</script>

<svelte:head>
	<title>{project ? project.name : 'Конфигурации'} — CFGY</title>
</svelte:head>

{#if isLoading}
	<div class="text-center py-8">Загрузка...</div>
{:else if error || !project}
	<div class="projects-error">{error || 'Проект не найден'}</div>
	<a href="/projects" class="text-basic-accent cursor-pointer">К списку проектов</a>
{:else}
	<div class="projects-page">
		<nav class="flex gap-2 mb-4 text-regular-16">
			<a href="/projects" class="text-basic-accent">Проекты</a>
			<span>/</span>
			<span>{project.name}</span>
		</nav>
		<div class="projects-header">
			<h1 class="projects-title">Конфигурации</h1>
			<Button variant="basic-secondary" size="big-text" onclick={() => { showCreateForm = true; createError = ''; }}>
				Добавить конфигурацию
			</Button>
		</div>

		{#if showCreateForm}
			<form onsubmit={handleCreate} class="projects-form">
				<div class="projects-input-wrapper">
					<input type="text" bind:value={configName} placeholder="Название конфигурации" class="projects-input" required />
					<Button type="submit" variant="basic-accent" size="small-text">Сохранить</Button>
				</div>
			</form>
			{#if createError}
				<p class="text-states-error text-regular-16">{createError}</p>
			{/if}
			<div class="projects-divider"></div>
		{/if}

		<div class="projects-list">
			{#if sortedConfigs.length === 0}
				<div class="projects-empty">Конфигурации не найдены. Создайте первую.</div>
			{:else}
				{#each sortedConfigs as config (config.id)}
					<div class="projects-row">
						<a href="/projects/{id}/configurations/{config.id}/parameters" class="projects-item-link">{config.name}</a>
						<Button
							variant="basic-secondary"
							size="small-icon"
							onclick={(e) => { e.stopPropagation(); handleDelete(config.id, config.name); }}
							aria-label="Удалить конфигурацию"
						>
							<svg width="24" height="24" viewBox="0 0 24 24" fill="none"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
						</Button>
					</div>
				{/each}
			{/if}
		</div>
	</div>

	<ConfirmDialog
		open={deleteConfirm !== null}
		title="Удаление конфигурации"
		message={deleteConfirm ? `Удалить конфигурацию "${deleteConfirm.name}"? Это действие необратимо и удалит все параметры.` : ''}
		confirmLabel="Удалить"
		cancelLabel="Отмена"
		variant="danger"
		onConfirm={confirmDelete}
		onCancel={() => (deleteConfirm = null)}
	/>
{/if}
