import { createContext } from "react";
import type { LoginResponse } from "../services/api";

export type AuthUser = (LoginResponse & { username?: string | null }) | null;

export interface AuthContextType {
  user: AuthUser;
  isLoading: boolean;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  isAuthenticated: boolean;
}

/**
 * React Fast Refresh constraint:
 * this file intentionally contains only context types + context value,
 * while the provider component lives in a separate file.
 */
export const AuthContext = createContext<AuthContextType | undefined>(
  undefined,
);
