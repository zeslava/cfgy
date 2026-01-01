import { useContext } from "react";
import { AuthContext } from "./AuthContextValue";

/**
 * Hook for accessing authentication context.
 *
 * Kept in a separate file to satisfy React Fast Refresh rule:
 * the context file should export only components.
 */
export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth должен использоваться внутри AuthProvider");
  }
  return context;
};
