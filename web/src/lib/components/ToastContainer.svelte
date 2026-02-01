<script lang="ts">
	import { toastStore } from '$lib/toast';
	import type { ToastVariant } from '$lib/toast';

	const variantClasses: Record<ToastVariant, string> = {
		success: 'toast-success',
		error: 'toast-error',
		info: 'toast-info',
		warning: 'toast-warning'
	};
	const iconClasses: Record<ToastVariant, string> = {
		success: 'toast-icon-success',
		error: 'toast-icon-error',
		info: 'toast-icon-info',
		warning: 'toast-icon-warning'
	};
</script>

<div class="toast-container">
	{#each $toastStore as toast (toast.id)}
		<div
			class="toast {variantClasses[toast.variant ?? 'info']}"
			role="alert"
		>
			{#if toast.variant === 'success'}
				<svg class="toast-icon {iconClasses[toast.variant]}" width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
					<path d="M20 6L9 17l-5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
				</svg>
			{:else if toast.variant === 'error'}
				<svg class="toast-icon {iconClasses[toast.variant]}" width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
					<path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
				</svg>
			{:else if toast.variant === 'info'}
				<span class="toast-icon {iconClasses[toast.variant] ?? iconClasses.info}" style="font-size: 20px">ℹ</span>
			{:else if toast.variant === 'warning'}
				<span class="toast-icon {iconClasses[toast.variant] ?? iconClasses.warning}" style="font-size: 20px">⚠</span>
			{:else}
				<span class="toast-icon toast-icon-info" style="font-size: 20px">ℹ</span>
			{/if}
			<p class="toast-message">{toast.message}</p>
			<button
				type="button"
				class="toast-close-button"
				aria-label="Закрыть"
				onclick={() => toastStore.remove(toast.id)}
			>
				<svg class="toast-close-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" aria-hidden="true">
					<path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
				</svg>
			</button>
		</div>
	{/each}
</div>
