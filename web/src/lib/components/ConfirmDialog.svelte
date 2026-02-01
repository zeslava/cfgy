<script lang="ts">
	import Modal from './Modal.svelte';
	import Button from './Button.svelte';

	interface Props {
		open: boolean;
		title?: string;
		message: string;
		confirmLabel?: string;
		cancelLabel?: string;
		variant?: 'danger' | 'primary';
		onConfirm: () => void;
		onCancel: () => void;
	}

	let {
		open,
		title = 'Подтверждение',
		message,
		confirmLabel = 'Подтвердить',
		cancelLabel = 'Отмена',
		variant = 'primary',
		onConfirm,
		onCancel
	}: Props = $props();
</script>

{#snippet footerContent()}
	<div class="flex gap-2 justify-end">
		<Button variant="secondary" size="small-text" onclick={onCancel}>{cancelLabel}</Button>
		<Button variant={variant} size="small-text" onclick={onConfirm}>{confirmLabel}</Button>
	</div>
{/snippet}

<Modal open={open} title={title} onClose={onCancel} width="max-w-md" footer={footerContent}>
	<p class="text-regular-16" style="margin:0;">{message}</p>
</Modal>
