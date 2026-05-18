import { useState, useRef, useEffect } from "react";
import { useAuth } from "./AuthContext";
import { useUIStore } from "@/stores/ui.store";
import { Eye, EyeOff, LogIn } from "lucide-react";
import { useTranslation } from "react-i18next";
import iconUrl from "@/assets/app-icon.png";

export default function LoginView() {
  const { login, authState } = useAuth();
  const setActiveView = useUIStore((s) => s.setActiveView);
  const { t } = useTranslation();

  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!password.trim() || submitting) return;

    setError(null);
    setSubmitting(true);
    try {
      const err = await login(password);
      if (err) {
        setError(err);
        setPassword("");
        inputRef.current?.focus();
      } else {
        setActiveView("inbox");
      }
    } finally {
      setSubmitting(false);
    }
  };

  // Already authenticated (shouldn't normally render, but just in case)
  if (authState === "authenticated") return null;

  return (
    <div className="flex items-center justify-center h-screen bg-[var(--color-bg-primary)]">
      <div className="w-full max-w-sm px-6">
        <div className="text-center mb-8">
          <img
            src={iconUrl}
            alt="Pebble"
            className="w-16 h-16 mx-auto mb-4 rounded-xl"
            onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
          />
          <h1 className="text-xl font-semibold text-[var(--color-text-primary)]">
            Pebble
          </h1>
          <p className="text-sm text-[var(--color-text-secondary)] mt-1">
            {t("login.subtitle", "Sign in to your self-hosted mailbox")}
          </p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="relative">
            <input
              ref={inputRef}
              type={showPassword ? "text" : "password"}
              value={password}
              onChange={(e) => { setPassword(e.target.value); setError(null); }}
              placeholder={t("login.passwordPlaceholder", "Enter password")}
              disabled={submitting}
              className="w-full px-3 py-2.5 pr-10 text-sm rounded-lg border
                border-[var(--color-border-primary)] bg-[var(--color-bg-secondary)]
                text-[var(--color-text-primary)] placeholder:text-[var(--color-text-secondary)]
                focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]/40
                disabled:opacity-50"
              autoComplete="current-password"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
              tabIndex={-1}
            >
              {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
          </div>

          {error && (
            <p className="text-xs text-red-500 text-center">{error}</p>
          )}

          <button
            type="submit"
            disabled={submitting || !password.trim()}
            className="w-full flex items-center justify-center gap-2 py-2.5 px-4 rounded-lg
              bg-[var(--color-accent)] text-white text-sm font-medium
              hover:opacity-90 disabled:opacity-50 transition-opacity"
          >
            <LogIn size={16} />
            {submitting ? t("common.loading", "Loading...") : t("login.submit", "Sign in")}
          </button>
        </form>
      </div>
    </div>
  );
}
