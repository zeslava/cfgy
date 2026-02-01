<script lang="ts">
	interface Props {
		size?: 'small' | 'big';
		initials?: string;
		className?: string;
	}
	let { size = 'small', initials, className = '' }: Props = $props();
</script>

<div
	class="avatar {size === 'small' ? 'avatar-small' : 'avatar-big'} {className}"
>
	{#if initials}
		<span class="avatar-initials">{initials}</span>
	{:else}
		<div class="w-full h-full bg-lines-gray" style="width:100%;height:100%;background:var(--color-lines-gray);border-radius:360px;"></div>
	{/if}
</div>
