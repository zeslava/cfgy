// Состояние аутентификации (store + bootstrap/login/logout)
import { writable } from 'svelte/store';
import {
	authApi,
	getSessionToken,
	setSessionToken,
	clearSessionToken
} from '$lib/api';
import type { LoginResponse } from '$lib/api';

export type AuthUser = (LoginResponse & { username?: string | null }) | null;

export const user = writable<AuthUser>(null);
export const authLoading = writable<boolean>(true);

let bootstrapped = false;

/** Один раз при загрузке: hash → session() → me() */
export async function bootstrapAuth(): Promise<void> {
	if (bootstrapped) return;
	bootstrapped = true;
	authLoading.set(true);
	try {
		// После OAuth redirect: /projects#session_token=<jwt>
		if (typeof window !== 'undefined') {
			const hash = window.location.hash.slice(1);
			const params = new URLSearchParams(hash);
			const tokenFromHash = params.get('session_token');
			if (tokenFromHash) {
				setSessionToken(decodeURIComponent(tokenFromHash));
				window.history.replaceState(
					null,
					'',
					window.location.pathname + window.location.search
				);
			}
		}
		if (!getSessionToken()) {
			const session = await authApi.session();
			if (session?.session_token) {
				setSessionToken(session.session_token);
			}
		}
		const token = getSessionToken();
		if (!token) {
			user.set(null);
		} else {
			const me = await authApi.me();
			user.set(me);
		}
	} catch {
		user.set(null);
	} finally {
		authLoading.set(false);
	}
}

export async function login(email: string, password: string): Promise<void> {
	const response = await authApi.login({ email, password });
	if (response.session_token) {
		setSessionToken(response.session_token);
	}
	user.set(response);
}

export async function register(email: string, password: string): Promise<void> {
	const response = await authApi.register({ email, password });
	if (response.session_token) {
		setSessionToken(response.session_token);
	}
	user.set(response);
}

export async function logout(): Promise<void> {
	try {
		await authApi.logout();
	} finally {
		clearSessionToken();
		user.set(null);
	}
}
