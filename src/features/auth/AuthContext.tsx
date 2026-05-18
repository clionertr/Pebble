import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react";
import { getAuthStatus, login as apiLogin, logout as apiLogout } from "@/lib/api-client";
import type { AuthStatus } from "@/lib/api-client";

type AuthState = "loading" | "authenticated" | "unauthenticated";

interface AuthContextValue {
  authState: AuthState;
  login: (password: string) => Promise<string | null>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [authState, setAuthState] = useState<AuthState>("loading");

  useEffect(() => {
    let cancelled = false;
    getAuthStatus()
      .then((status: AuthStatus) => {
        if (!cancelled) setAuthState(status.authenticated ? "authenticated" : "unauthenticated");
      })
      .catch(() => {
        if (!cancelled) setAuthState("unauthenticated");
      });
    return () => { cancelled = true; };
  }, []);

  const login = useCallback(async (password: string): Promise<string | null> => {
    try {
      const status = await apiLogin(password);
      if (status.authenticated) {
        setAuthState("authenticated");
        return null;
      }
      return "Login failed";
    } catch (err: unknown) {
      if (err && typeof err === "object" && "status" in err) {
        const apiErr = err as { status: number; body?: { error?: string } };
        if (apiErr.status === 429) return "Too many attempts. Please wait 15 minutes.";
        return apiErr.body?.error || "Wrong password";
      }
      return "Network error. Please try again.";
    }
  }, []);

  const logout = useCallback(async () => {
    await apiLogout().catch(() => {});
    setAuthState("unauthenticated");
  }, []);

  return (
    <AuthContext.Provider value={{ authState, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error("useAuth must be used within AuthProvider");
  return ctx;
}
