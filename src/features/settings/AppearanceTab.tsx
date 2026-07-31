import { useTranslation } from "react-i18next";
import { useThemeStore, type Theme, type MotionPref, type Language } from "@/stores/theme.store";

const THEMES: { id: Theme; labelKey: string; descKey: string }[] = [
  { id: "light", labelKey: "settings.themeLight", descKey: "settings.themeLightDesc" },
  { id: "dark", labelKey: "settings.themeDark", descKey: "settings.themeDarkDesc" },
  { id: "system", labelKey: "settings.themeSystem", descKey: "settings.themeSystemDesc" },
];

const MOTIONS: { id: MotionPref; labelKey: string; descKey: string }[] = [
  { id: "system", labelKey: "settings.motionSystem", descKey: "settings.motionSystemDesc" },
  { id: "on", labelKey: "settings.motionOn", descKey: "settings.motionOnDesc" },
  { id: "off", labelKey: "settings.motionOff", descKey: "settings.motionOffDesc" },
];

const LANGUAGES: { id: Language; label: string }[] = [
  { id: "en", label: "English" },
  { id: "zh", label: "中文" },
];

export default function AppearanceTab() {
  const { t } = useTranslation();
  const theme = useThemeStore((s) => s.theme);
  const setTheme = useThemeStore((s) => s.setTheme);
  const motion = useThemeStore((s) => s.motion);
  const setMotion = useThemeStore((s) => s.setMotion);
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
              border: "1px solid " +
                (theme === th.id ? "var(--color-accent)" : "var(--color-border)"),
              boxShadow: theme === th.id ? "inset 0 0 0 1px var(--color-accent)" : "none",
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
        {t("settings.motion")}
      </h3>
      <div style={{ display: "flex", gap: "12px" }}>
        {MOTIONS.map((mo) => (
          <button
            key={mo.id}
            onClick={() => setMotion(mo.id)}
            style={{
              flex: 1,
              padding: "16px",
              borderRadius: "8px",
              border: "1px solid " +
                (motion === mo.id ? "var(--color-accent)" : "var(--color-border)"),
              boxShadow: motion === mo.id ? "inset 0 0 0 1px var(--color-accent)" : "none",
              backgroundColor: motion === mo.id ? "var(--color-bg-hover)" : "transparent",
              cursor: "pointer",
              textAlign: "left",
              color: "var(--color-text-primary)",
            }}
          >
            <div style={{ fontWeight: 600, fontSize: "13px", marginBottom: "4px" }}>
              {t(mo.labelKey)}
            </div>
            <div style={{ fontSize: "12px", color: "var(--color-text-secondary)" }}>
              {t(mo.descKey)}
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
              border: "1px solid " +
                (language === l.id ? "var(--color-accent)" : "var(--color-border)"),
              boxShadow: language === l.id ? "inset 0 0 0 1px var(--color-accent)" : "none",
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
