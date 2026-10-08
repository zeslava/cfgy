// HTTP клиент для работы с CFGY API
// Аутентификация: session_token в localStorage, запросы с Authorization: Bearer.

const API_BASE_URL = '/api';
const SESSION_TOKEN_KEY = 'cfgy_session_token';

export function getSessionToken(): string | null {
	if (typeof window === 'undefined') return null;
	return localStorage.getItem(SESSION_TOKEN_KEY);
}

export function setSessionToken(token: string): void {
	localStorage.setItem(SESSION_TOKEN_KEY, token);
}

export function clearSessionToken(): void {
	localStorage.removeItem(SESSION_TOKEN_KEY);
}

export interface ApiError {
	error: string;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
}

export interface LoginResponse {
	user_id: string;
	email: string;
	session_token?: string;
}

export interface Project {
	id: string;
	name: string;
	description?: string;
	owner_id: string;
	created_at: string;
	updated_at: string;
}

export interface CreateProjectRequest {
	name: string;
	description?: string;
}

export interface UpdateProjectRequest {
	name?: string;
	description?: string;
}

export interface Configuration {
	id: string;
	project_id: string;
	name: string;
	created_at: string;
	updated_at: string;
}

export interface CreateConfigurationRequest {
	name: string;
}

export interface Parameter {
	key: string;
	value: string;
}

export type Parameters = Parameter[];

export interface Token {
	id: string;
	name: string;
	project_id?: string;
	expires_at?: string;
	created_at: string;
	last_used_at?: string;
}

export interface CreateTokenRequest {
	name: string;
	project_id?: string;
	expires_in_days?: number;
}

export interface CreateTokenResponse {
	token: string;
	token_id: string;
	name: string;
	expires_at?: string;
}

async function request<T>(
	endpoint: string,
	options: RequestInit & { skipAuth?: boolean } = {}
): Promise<T> {
	const { skipAuth, ...fetchOptions } = options;
	const headers: Record<string, string> = {
		'Content-Type': 'application/json'
	};

	const token = getSessionToken();
	if (token && !skipAuth) {
		headers['Authorization'] = `Bearer ${token}`;
	}
	if (options.headers) {
		Object.assign(headers, options.headers);
	}

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		...fetchOptions,
		credentials: 'include',
		headers
	});

	if (!response.ok) {
		if (response.status === 401) {
			clearSessionToken();
			if (
				typeof window !== 'undefined' &&
				window.location.pathname !== '/login' &&
				window.location.pathname !== '/register'
			) {
				window.location.href = '/login';
			}
		}

		const error: ApiError = await response.json().catch(() => ({
			error: `HTTP ${response.status}: ${response.statusText}`
		}));
		throw new Error(error.error || `HTTP ${response.status}`);
	}

	if (
		response.status === 204 ||
		response.headers.get('content-length') === '0'
	) {
		return undefined as T;
	}

	const text = await response.text();
	if (!text || text.trim() === '') {
		return undefined as T;
	}

	try {
		return JSON.parse(text) as T;
	} catch {
		return undefined as T;
	}
}

export const authApi = {
	register: (data: RegisterRequest): Promise<LoginResponse> =>
		request('/auth/register', {
			method: 'POST',
			body: JSON.stringify(data)
		}),

	registrationStatus: (): Promise<{ open: boolean }> =>
		request('/auth/registration', { skipAuth: true }),

	login: (data: LoginRequest): Promise<LoginResponse> =>
		request('/auth/login', {
			method: 'POST',
			body: JSON.stringify(data)
		}),

	session: (): Promise<{ session_token: string } | undefined> =>
		request('/auth/session', { skipAuth: true }),

	me: (): Promise<LoginResponse & { username?: string | null }> =>
		request('/auth/me'),

	logout: (): Promise<void> =>
		request('/auth/logout', {
			method: 'POST'
		})
};

export const projectsApi = {
	list: (): Promise<Project[]> => request('/projects'),
	get: (id: string): Promise<Project> => request(`/projects/${id}`),
	create: (data: CreateProjectRequest): Promise<Project> =>
		request('/projects', {
			method: 'POST',
			body: JSON.stringify(data)
		}),
	update: (id: string, data: UpdateProjectRequest): Promise<Project> =>
		request(`/projects/${id}`, {
			method: 'PATCH',
			body: JSON.stringify(data)
		}),
	delete: (id: string): Promise<void> =>
		request(`/projects/${id}`, {
			method: 'DELETE'
		})
};

export const configurationsApi = {
	list: (projectId: string): Promise<Configuration[]> =>
		request(`/projects/${projectId}/configurations`),
	create: (
		projectId: string,
		data: CreateConfigurationRequest
	): Promise<Configuration> =>
		request(`/projects/${projectId}/configurations`, {
			method: 'POST',
			body: JSON.stringify(data)
		}),
	delete: (projectId: string, configId: string): Promise<void> =>
		request(`/projects/${projectId}/configurations/${configId}`, {
			method: 'DELETE'
		})
};

export const parametersApi = {
	get: async (projectId: string, configName: string): Promise<Parameters> => {
		const data = await request<Record<string, string> | Parameters>(
			`/projects/${projectId}/configurations/${configName}/parameters`
		);
		let result: Parameters = [];
		if (Array.isArray(data)) {
			result = data;
		} else if (data && typeof data === 'object' && !Array.isArray(data)) {
			result = Object.entries(data).map(([key, value]) => ({ key, value }));
		}
		return result.sort((a, b) =>
			a.key.localeCompare(b.key, 'ru', { sensitivity: 'base' })
		);
	},
	set: async (
		projectId: string,
		configName: string,
		parameters: Parameters
	): Promise<void> => {
		const paramsObject = parameters.reduce(
			(acc, param) => {
				acc[param.key] = param.value;
				return acc;
			},
			{} as Record<string, string>
		);
		return request(
			`/projects/${projectId}/configurations/${configName}/parameters`,
			{
				method: 'PUT',
				body: JSON.stringify(paramsObject)
			}
		);
	},
	delete: (
		projectId: string,
		configName: string,
		key: string
	): Promise<void> =>
		request(
			`/projects/${projectId}/configurations/${configName}/parameters/${encodeURIComponent(key)}`,
			{ method: 'DELETE' }
		)
};

export const tokensApi = {
	list: (): Promise<Token[]> => request('/tokens'),
	create: (data: CreateTokenRequest): Promise<CreateTokenResponse> =>
		request('/tokens', {
			method: 'POST',
			body: JSON.stringify(data)
		}),
	delete: (id: string): Promise<void> =>
		request(`/tokens/${id}`, { method: 'DELETE' })
};
