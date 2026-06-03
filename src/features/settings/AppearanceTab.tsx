import { useTranslation } from "react-i18next";
import { useThemeStore, type Theme, type Language } from "@/stores/theme.store";

const THEMES: { id: Theme; labelKey: string; descKey: string }[] = [
  { id: "light", labelKey: "settings.themeLight", descKey: "settings.themeLightDesc" },
  { id: "dark", labelKey: "settings.themeDark", descKey: "settings.themeDarkDesc" },
  { id: "system", labelKey: "settings.themeSystem", descKey: "settings.themeSystemDesc" },
];

const LANGUAGES: { id: Language; label: string }[] = [
  { id: "en", label: "English" },
  { id: "zh", label: "中文" },
];

export default function AppearanceTab() {
  const { t } = useTranslation();
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);
  const language = useThemeStore((s) => s.language);
  const setLanguage = useThemeStore((s) => s.setLanguage);

  return (
    <div>
      <h3 style={{ fontSize: "14px", fontWeight: 600, marginBottom: "16px" }}>
        {t("settings.theme")}
      </h3>
      <div style={{ display: "flex", gap: "12px" }}>
        {THEMES.map((th) => (
          <button
            key={th.id}
            onClick={() => setTheme(th.id)}
            style={{
              flex: 1,
              padding: "16px",
              borderRadius: "8px",
              border:
                theme === th.id ? "2px solid var(--color-accent)" : "1px solid var(--color-border)",
              backgroundColor: theme === th.id ? "var(--color-bg-hover)" : "transparent",
              cursor: "pointer",
              textAlign: "left",
              color: "var(--color-text-primary)",
            }}
          >
            <div style={{ fontWeight: 600, fontSize: "13px", marginBottom: "4px" }}>
              {t(th.labelKey)}
            </div>
            <div style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
              {t(th.descKey)}
            </div>
          </button>
        ))}
      </div>

      <h3 style={{ fontSize: "14px", fontWeight: 600, marginBottom: "16px", marginTop: "32px" }}>
        {t("settings.language")}
      </h3>
      <div style={{ display: "flex", gap: "12px" }}>
        {LANGUAGES.map((l) => (
          <button
            key={l.id}
            onClick={() => setLanguage(l.id)}
            style={{
              flex: 1,
              padding: "16px",
              borderRadius: "8px",
              border:
                language === l.id
                  ? "2px solid var(--color-accent)"
                  : "1px solid var(--color-border)",
              backgroundColor: language === l.id ? "var(--color-bg-hover)" : "transparent",
              cursor: "pointer",
              textAlign: "left",
              color: "var(--color-text-primary)",
            }}
          >
            <div style={{ fontWeight: 600, fontSize: "13px" }}>{l.label}</div>
          </button>
        ))}
      </div>
    </div>
  );
}
