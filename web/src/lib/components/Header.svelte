<script lang="ts">
	import { goto } from '$app/navigation';
	import { user, logout } from '$lib/auth';
	import { getInitialsFromEmail, getDisplayName } from '$lib/avatar';
	import Avatar from './Avatar.svelte';
</script>

<header class="header">
	<div class="header-content">
		<a href="/projects" class="header-logo-link">
			<img src="/logo.svg" alt="CFGY" class="header-logo" />
		</a>
		<div class="header-right">
			<a href="/profile" class="header-user-link">
				<Avatar
					size="small"
					initials={getInitialsFromEmail($user?.email ?? getDisplayName($user) ?? 'U')}
					className="bg-basic-light text-text-dark"
				/>
				<span class="header-user-email">{getDisplayName($user)}</span>
			</a>
			<button
				type="button"
				class="header-logout-btn"
				aria-label="Выйти"
				onclick={async () => {
					await logout();
					goto('/login');
				}}
			>
				<svg class="size-6" width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true">
					<path
						d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4m7 14l5-5-5-5m5 5H9"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			</button>
		</div>
	</div>
</header>

<style>
	.header-logout-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: var(--color-basic-secondary);
		color: var(--color-text-dark);
		border: none;
		border-radius: 4px;
		cursor: pointer;
		padding: 8px;
	}
	.header-logout-btn:hover {
		background: var(--color-basic-secondary-hover);
	}
	.size-6 {
		width: 24px;
		height: 24px;
	}
</style>
