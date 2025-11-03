// Контекст для управления аутентификацией
import React, { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import { authApi } from '../services/api';
import type { LoginResponse } from '../services/api';

interface AuthContextType {
  user: LoginResponse | null;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  logout: () => void;
  isAuthenticated: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth должен использоваться внутри AuthProvider');
  }
  return context;
};

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<LoginResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Проверяем, есть ли сохраненный токен
    const token = localStorage.getItem('session_token');
    const userStr = localStorage.getItem('user');

    if (token && userStr) {
      try {
        const userData: LoginResponse = JSON.parse(userStr);
        setUser(userData);
      } catch (e) {
        // Если данные повреждены, очищаем
        localStorage.removeItem('session_token');
        localStorage.removeItem('user');
      }
    }
    setIsLoading(false);
  }, []);

  const login = async (email: string, password: string) => {
    const response = await authApi.login({ email, password });
    if (response.session_token) {
      localStorage.setItem('session_token', response.session_token);
      localStorage.setItem('user', JSON.stringify(response));
      setUser(response);
    } else {
      throw new Error('Не получен session token');
    }
  };

  const register = async (email: string, password: string) => {
    const response = await authApi.register({ email, password });
    if (response.session_token) {
      localStorage.setItem('session_token', response.session_token);
      localStorage.setItem('user', JSON.stringify(response));
      setUser(response);
    } else {
      throw new Error('Не получен session token');
    }
  };

  const logout = () => {
    localStorage.removeItem('session_token');
    localStorage.removeItem('user');
    setUser(null);
  };

  const value: AuthContextType = {
    user,
    isLoading,
    login,
    register,
    logout,
    isAuthenticated: !!user && !!localStorage.getItem('session_token'),
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

