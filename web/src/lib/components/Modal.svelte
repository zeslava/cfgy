<script lang="ts">
	import Button from './Button.svelte';

	interface Props {
		open: boolean;
		title?: string;
		onClose: () => void;
		children?: import('svelte').Snippet;
		footer?: import('svelte').Snippet;
		width?: string;
	}

	let { open, title, onClose, children, footer, width = 'max-w-lg' }: Props = $props();

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}

	$effect(() => {
		if (open) {
			window.addEventListener('keydown', handleKeydown);
			return () => window.removeEventListener('keydown', handleKeydown);
		}
	});
</script>

{#if open}
	<div class="modal-overlay" role="dialog" aria-modal="true" aria-labelledby={title ? 'modal-title' : undefined}>
		<div class="modal-backdrop" onclick={onClose} aria-hidden="true"></div>
		<div
			class="modal-content {width === 'max-w-md' ? 'max-w-md' : ''}"
			role="document"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.key === 'Escape' && onClose()}
		>
			{#if title}
				<div class="modal-header">
					<div id="modal-title" class="modal-title">{title}</div>
					<Button variant="secondary" size="small-icon" aria-label="Закрыть" onclick={onClose} class="modal-close-button">
						<svg width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
							<path d="M18 6L6 18M6 6l12 12" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
						</svg>
					</Button>
				</div>
			{/if}
			<div class="modal-body">
				{#if children}
					{@render children()}
				{/if}
			</div>
			{#if footer}
				<div class="modal-footer">
					{@render footer()}
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.max-w-md { max-width: 448px; }
</style>
