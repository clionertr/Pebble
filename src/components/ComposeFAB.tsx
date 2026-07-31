import { PenLine } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useUIStore } from "@/stores/ui.store";
import { useComposeStore } from "@/stores/compose.store";

export default function ComposeFAB() {
  const { t } = useTranslation();
  const activeView = useUIStore((s) => s.activeView);
  const openCompose = useComposeStore((s) => s.openCompose);

  if (activeView === "compose") return null;

  return (
    <button
      className="compose-fab"
      onClick={() => openCompose("new")}
      aria-label={t("sidebar.compose", "Compose")}
      title={t("sidebar.compose", "Compose")}
      style={{
        position: "fixed",
        bottom: "48px",
        right: "24px",
        zIndex: 100,
        width: "48px",
        height: "48px",
        borderRadius: "50%",
        border: "none",
        backgroundColor: "var(--color-accent, #2563eb)",
        color: "#fff",
        cursor: "pointer",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        boxShadow: "0 4px 12px rgba(0,0,0,0.2)",
      }}
    >
      <PenLine size={20} />
    </button>
  );
}
