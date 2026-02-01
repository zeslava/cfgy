<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { projectsApi } from '$lib/api';
	import type { Project } from '$lib/api';
	import { toastStore } from '$lib/toast';
	import { sortByString } from '$lib/sort';
	import Button from '$lib/components/Button.svelte';
	import Input from '$lib/components/Input.svelte';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

	let projects = $state<Project[]>([]);
	let isLoading = $state(true);
	let error = $state('');
	let showCreateForm = $state(false);
	let projectName = $state('');
	let createError = $state('');
	let deleteConfirm = $state<{ id: string; name: string } | null>(null);

	onMount(() => {
		loadProjects();
	});

	async function loadProjects() {
		isLoading = true;
		error = '';
		try {
			projects = await projectsApi.list();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Ошибка загрузки проектов';
		} finally {
			isLoading = false;
		}
	}

	async function handleCreate(e: Event) {
		e.preventDefault();
		createError = '';
		if (!projectName.trim()) {
			createError = 'Название проекта обязательно';
			return;
		}
		try {
			const project = await projectsApi.create({ name: projectName.trim() });
			showCreateForm = false;
			projectName = '';
			toastStore.showSuccess(`Проект "${project.name}" создан`);
			await loadProjects();
			goto(`/projects/${project.id}`);
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Ошибка создания проекта';
			createError = message;
			toastStore.showError(message);
		}
	}

	function handleDelete(id: string, name: string) {
		deleteConfirm = { id, name };
	}

	async function confirmDelete() {
		if (!deleteConfirm) return;
		try {
			await projectsApi.delete(deleteConfirm.id);
			toastStore.showSuccess(`Проект "${deleteConfirm.name}" удалён`);
			await loadProjects();
		} catch (err) {
			toastStore.showError(err instanceof Error ? err.message : 'Ошибка удаления проекта');
		} finally {
			deleteConfirm = null;
		}
	}

	const sortedProjects = $derived(sortByString(projects, (p) => p.name));
</script>

<svelte:head>
	<title>Проекты — CFGY</title>
</svelte:head>

<div class="projects-page">
	<div class="projects-header">
		<h1 class="projects-title">Мои проекты</h1>
		<Button
			variant="basic-secondary"
			size="big-text"
			onclick={() => {
				showCreateForm = true;
				createError = '';
			}}
		>
			Добавить проект
		</Button>
	</div>

	{#if error && !showCreateForm}
		<div class="projects-error">Ошибка: {error}</div>
	{/if}

	{#if showCreateForm}
		<div class="projects-create-wrapper">
			<form onsubmit={handleCreate} class="projects-form">
				<div class="projects-input-wrapper">
					<input
						type="text"
						bind:value={projectName}
						placeholder="Название проекта"
						class="projects-input"
						required
					/>
					<Button type="submit" variant="basic-accent" size="small-text">
						Сохранить
					</Button>
				</div>
			</form>
			{#if createError}
				<p class="text-states-error text-regular-16">{createError}</p>
			{/if}
			<div class="projects-divider"></div>
		</div>
	{/if}

	{#if isLoading}
		<div class="text-center py-8">Загрузка проектов...</div>
	{:else}
		<div class="projects-list">
			{#if sortedProjects.length === 0}
				<div class="projects-empty">Проекты не найдены. Создайте первый проект.</div>
			{:else}
				{#each sortedProjects as project (project.id)}
					<div
						class="projects-row"
						role="button"
						tabindex="0"
						onclick={() => goto(`/projects/${project.id}`)}
						onkeydown={(e) => e.key === 'Enter' && goto(`/projects/${project.id}`)}
					>
						<a
							href="/projects/{project.id}"
							class="projects-item-link"
							onclick={(e) => e.stopPropagation()}
						>
							{project.name}
						</a>
						<Button
							variant="basic-secondary"
							size="small-icon"
							onclick={(e) => {
								e.stopPropagation();
								handleDelete(project.id, project.name);
							}}
							aria-label="Удалить проект"
						>
							<svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
								<path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M10 11v6M14 11v6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
							</svg>
						</Button>
					</div>
				{/each}
			{/if}
		</div>
	{/if}
</div>

<ConfirmDialog
	open={deleteConfirm !== null}
	title="Удаление проекта"
	message={deleteConfirm ? `Вы уверены, что хотите удалить проект "${deleteConfirm.name}"? Это действие необратимо.` : ''}
	confirmLabel="Удалить"
	cancelLabel="Отмена"
	variant="danger"
	onConfirm={confirmDelete}
	onCancel={() => (deleteConfirm = null)}
/>
