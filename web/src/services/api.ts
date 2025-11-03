// HTTP клиент для работы с CFGY API

const API_BASE_URL = '/api';

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

export interface Environment {
  id: string;
  project_id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface CreateEnvironmentRequest {
  name: string;
}

export interface Secrets {
  [key: string]: string;
}

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

// Утилиты для работы с API
async function request<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const token = localStorage.getItem('session_token');
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  if (options.headers) {
    Object.assign(headers, options.headers);
  }

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers,
  });

  if (!response.ok) {
    // Если получили 401 Unauthorized, токен недействителен - очищаем его
    if (response.status === 401) {
      localStorage.removeItem('session_token');
      localStorage.removeItem('user');
      // Перенаправляем на страницу входа, если не находимся на ней
      if (window.location.pathname !== '/login' && window.location.pathname !== '/register') {
        window.location.href = '/login';
      }
    }
    
    const error: ApiError = await response.json().catch(() => ({
      error: `HTTP ${response.status}: ${response.statusText}`,
    }));
    throw new Error(error.error || `HTTP ${response.status}`);
  }

  // Если ответ пустой (204 No Content или 200 с пустым телом)
  if (response.status === 204 || response.headers.get('content-length') === '0') {
    return undefined as T;
  }

  // Проверяем, есть ли контент перед парсингом
  const text = await response.text();
  if (!text || text.trim() === '') {
    return undefined as T;
  }

  try {
    return JSON.parse(text) as T;
  } catch (e) {
    // Если не удалось распарсить, возможно это пустой ответ
    return undefined as T;
  }
}

// Аутентификация
export const authApi = {
  register: (data: RegisterRequest): Promise<LoginResponse> =>
    request('/auth/register', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  login: (data: LoginRequest): Promise<LoginResponse> =>
    request('/auth/login', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
};

// Проекты
export const projectsApi = {
  list: (): Promise<Project[]> => request('/projects'),

  get: (id: string): Promise<Project> => request(`/projects/${id}`),

  create: (data: CreateProjectRequest): Promise<Project> =>
    request('/projects', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateProjectRequest): Promise<Project> =>
    request(`/projects/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<void> =>
    request(`/projects/${id}`, {
      method: 'DELETE',
    }),
};

// Окружения
export const environmentsApi = {
  list: (projectId: string): Promise<Environment[]> =>
    request(`/projects/${projectId}/environments`),

  create: (projectId: string, data: CreateEnvironmentRequest): Promise<Environment> =>
    request(`/projects/${projectId}/environments`, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  delete: (projectId: string, envId: string): Promise<void> =>
    request(`/projects/${projectId}/environments/${envId}`, {
      method: 'DELETE',
    }),
};

// Секреты
export const secretsApi = {
  get: (projectId: string, envName: string): Promise<Secrets> =>
    request(`/projects/${projectId}/secrets/${envName}`),

  set: (projectId: string, envName: string, secrets: Secrets): Promise<void> =>
    request(`/projects/${projectId}/secrets/${envName}`, {
      method: 'PUT',
      body: JSON.stringify(secrets),
    }),

  delete: (projectId: string, envName: string, key: string): Promise<void> =>
    request(`/projects/${projectId}/secrets/${envName}/${encodeURIComponent(key)}`, {
      method: 'DELETE',
    }),
};

// Токены
export const tokensApi = {
  list: (): Promise<Token[]> => request('/tokens'),

  create: (data: CreateTokenRequest): Promise<CreateTokenResponse> =>
    request('/tokens', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<void> =>
    request(`/tokens/${id}`, {
      method: 'DELETE',
    }),
};

