// Контекст для управления аутентификацией (cookie-based сессия)
//
// Важно для React Fast Refresh:
// файл должен экспортировать только компоненты, поэтому контекст вынесен в отдельный модуль.
import React, { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { authApi } from "../services/api";
import type { LoginResponse } from "../services/api";
import { AuthContext } from "./AuthContextValue";
import type { AuthContextType } from "./AuthContextValue";

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<
    (LoginResponse & { username?: string | null }) | null
  >(null);
  // Для cookie-based сессии мы не можем синхронно "прочитать" токен из хранилища.
  // Поэтому при старте (и после SSO-redirect) подтягиваем пользователя через /api/auth/me.
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const bootstrap = async () => {
      setIsLoading(true);
      try {
        const me = await authApi.me();
        if (!cancelled) {
          setUser(me);
        }
      } catch {
        // Нет сессии / cookie невалидна — просто считаем пользователя неаутентифицированным
        if (!cancelled) {
          setUser(null);
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    };

    bootstrap();

    return () => {
      cancelled = true;
    };
  }, []);

  const login = async (email: string, password: string) => {
    // Сервер устанавливает httpOnly cookie сессии; session_token в UI не храним
    const response = await authApi.login({ email, password });
    setUser(response);
  };

  const register = async (email: string, password: string) => {
    // Сервер устанавливает httpOnly cookie сессии; session_token в UI не храним
    const response = await authApi.register({ email, password });
    setUser(response);
  };

  const logout = async () => {
    // Очищаем сессию на сервере (httpOnly cookie) + локальное состояние UI
    try {
      await authApi.logout();
    } finally {
      setUser(null);
    }
  };

  const value: AuthContextType = {
    user,
    isLoading,
    login,
    register,
    logout,
    // Для cookie-based схемы считаем аутентификацию подтверждённой после /api/auth/me
    isAuthenticated: !!user,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
