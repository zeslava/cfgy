// Контекст для управления аутентификацией (token-based: session_token в localStorage, Authorization: Bearer)
//
// Важно для React Fast Refresh:
// файл должен экспортировать только компоненты, поэтому контекст вынесен в отдельный модуль.
import React, { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { authApi, getSessionToken, setSessionToken, clearSessionToken } from "../services/api";
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
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const bootstrap = async () => {
      setIsLoading(true);
      try {
        // После OAuth redirect сервер редиректит на /projects#session_token=<jwt> — читаем и сохраняем
        const hash = window.location.hash.slice(1);
        const params = new URLSearchParams(hash);
        const tokenFromHash = params.get("session_token");
        if (tokenFromHash) {
          setSessionToken(decodeURIComponent(tokenFromHash));
          window.history.replaceState(null, "", window.location.pathname + window.location.search);
        }
        // Если токена нет в hash — пробуем обмен cookie на токен (GET /api/auth/session)
        if (!getSessionToken()) {
          const session = await authApi.session();
          if (!cancelled && session?.session_token) {
            setSessionToken(session.session_token);
          }
        }
        const token = getSessionToken();
        if (!token) {
          if (!cancelled) setUser(null);
        } else {
          const me = await authApi.me();
          if (!cancelled) setUser(me);
        }
      } catch {
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
    const response = await authApi.login({ email, password });
    if (response.session_token) {
      setSessionToken(response.session_token);
    }
    setUser(response);
  };

  const register = async (email: string, password: string) => {
    const response = await authApi.register({ email, password });
    if (response.session_token) {
      setSessionToken(response.session_token);
    }
    setUser(response);
  };

  const logout = async () => {
    try {
      await authApi.logout();
    } finally {
      clearSessionToken();
      setUser(null);
    }
  };

  const value: AuthContextType = {
    user,
    isLoading,
    login,
    register,
    logout,
    isAuthenticated: !!user,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
