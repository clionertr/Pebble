import { useTranslation } from "react-i18next";
import iconUrl from "@/assets/app-icon.png";
import { useUIStore } from "@/stores/ui.store";
import { Menu } from "lucide-react";

export default function TitleBar() {
  const { t } = useTranslation();
  const isMobile = useUIStore((s) => s.isMobile);
  const toggleDrawer = useUIStore((s) => s.toggleDrawer);

  return (
    <div
      className="flex items-center justify-between h-9 select-none"
      style={{ backgroundColor: "var(--color-titlebar-bg)" }}
    >
      <div className="flex items-center gap-2 px-3">
        {isMobile && (
          <button
            onClick={toggleDrawer}
            className="p-1 -ml-1 rounded-md hover:bg-black/5"
            aria-label={t("sidebar.toggle", "Toggle sidebar")}
          >
            <Menu size={20} />
          </button>
        )}
        <img
          src={iconUrl}
          alt=""
          aria-hidden="true"
          draggable={false}
          className="h-5 w-5 shrink-0 bg-transparent object-contain"
        />
        <span
          className="text-sm font-semibold"
          style={{ color: "var(--color-text-primary)" }}
        >
          Pebble
        </span>
      </div>
    </div>
  );
}
