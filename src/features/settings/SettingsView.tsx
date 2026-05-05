import { useTranslation } from "react-i18next";
import { useState } from "react";
import { useUIStore, type SettingsTab } from "@/stores/ui.store";
import AccountsTab from "./AccountsTab";
import GeneralTab from "./GeneralTab";
import ProxyTab from "./ProxyTab";
import AppearanceTab from "./AppearanceTab";
import CloudSyncTab from "./CloudSyncTab";
import RulesTab from "./RulesTab";
import PendingOpsTab from "./PendingOpsTab";
import ShortcutsTab from "./ShortcutsTab";
import TranslateTab from "./TranslateTab";
import PrivacyTab from "./PrivacyTab";
import AboutTab from "./AboutTab";

const TAB_IDS = ["accounts", "general", "proxy", "appearance", "privacy", "rules", "remoteWrites", "translation", "shortcuts", "cloudSync", "about"] as const;

const TAB_LABEL_KEYS: Record<string, string> = {
  accounts: "settings.accounts",
  general: "settings.general",
  proxy: "settings.proxy",
  appearance: "settings.appearance",
  privacy: "settings.privacy",
  rules: "settings.rules",
  remoteWrites: "settings.remoteWrites",
  translation: "settings.translation",
  shortcuts: "settings.shortcuts",
  cloudSync: "settings.cloudSync",
  about: "settings.about",
};

export default function SettingsView() {
  const { t } = useTranslation();
  const activeTab = useUIStore((s) => s.settingsTab);
  const setSettingsTab = useUIStore((s) => s.setSettingsTab);
  const isMobile = useUIStore((s) => s.isMobile);

  const [mobileTabActive, setMobileTabActive] = useState(false);

  function handleTabChange(id: SettingsTab) {
    setSettingsTab(id);
    if (isMobile) {
      setMobileTabActive(true);
    }
  }

  const showTabList = !isMobile || !mobileTabActive;
  const showTabContent = !isMobile || mobileTabActive;

  return (
    <div style={{ display: "flex", height: "100%", flexDirection: isMobile ? "column" : "row" }}>
      {/* Tab sidebar */}
      {showTabList && (
        <div
          role="tablist"
          aria-orientation={isMobile ? "horizontal" : "vertical"}
          aria-label={t("settings.tabs", "Settings tabs")}
          style={{
            width: isMobile ? "100%" : "180px",
            borderRight: isMobile ? "none" : "1px solid var(--color-border)",
            borderBottom: isMobile ? "1px solid var(--color-border)" : "none",
            padding: isMobile ? "8px 0" : "16px 0",
            flexShrink: 0,
            overflowX: isMobile ? "auto" : "visible",
            display: isMobile ? "flex" : "block",
          }}
        >
          {TAB_IDS.map((id, index) => (
            <button
              key={id}
              id={`settings-tab-${id}`}
              role="tab"
              aria-selected={activeTab === id}
              aria-controls={`settings-tabpanel-${id}`}
              tabIndex={activeTab === id ? 0 : -1}
              onClick={() => handleTabChange(id)}
              onKeyDown={(e) => {
                if (isMobile) return;
                let nextIndex = index;
                if (e.key === "ArrowDown") { nextIndex = (index + 1) % TAB_IDS.length; }
                else if (e.key === "ArrowUp") { nextIndex = (index - 1 + TAB_IDS.length) % TAB_IDS.length; }
                else if (e.key === "Home") { nextIndex = 0; }
                else if (e.key === "End") { nextIndex = TAB_IDS.length - 1; }
                else { return; }
                e.preventDefault();
                handleTabChange(TAB_IDS[nextIndex]);
                document.getElementById(`settings-tab-${TAB_IDS[nextIndex]}`)?.focus();
              }}
              style={{
                display: isMobile ? "inline-block" : "block",
                width: isMobile ? "auto" : "100%",
                whiteSpace: "nowrap",
                textAlign: "left",
                padding: isMobile ? "8px 16px" : "8px 20px",
                border: "none",
                background: activeTab === id ? "var(--color-bg-hover)" : "none",
                color: activeTab === id ? "var(--color-text-primary)" : "var(--color-text-secondary)",
                fontWeight: activeTab === id ? 600 : 400,
                fontSize: "13px",
                cursor: "pointer",
                borderRight: !isMobile && activeTab === id ? "2px solid var(--color-accent)" : "2px solid transparent",
                borderBottom: isMobile && activeTab === id ? "2px solid var(--color-accent)" : "none",
                transition: "background-color 0.15s ease, color 0.15s ease, border-color 0.15s ease",
              }}
            >
              {t(TAB_LABEL_KEYS[id])}
            </button>
          ))}
        </div>
      )}
      {/* Tab content */}
      {showTabContent && (
        <div
          id={`settings-tabpanel-${activeTab}`}
          className="scroll-region settings-panel-scroll"
          role="tabpanel"
          aria-labelledby={`settings-tab-${activeTab}`}
          style={{
            flex: 1,
            minWidth: 0,
            padding: isMobile ? "16px" : "32px",
            maxWidth: !isMobile && activeTab === "remoteWrites" ? "980px" : !isMobile ? "640px" : "100%",
            boxSizing: "border-box",
            overflowY: "auto",
            overflowX: "hidden",
          }}
        >
          {isMobile && (
            <button
              onClick={() => setMobileTabActive(false)}
              style={{
                marginBottom: "20px",
                padding: "6px 12px",
                fontSize: "13px",
                color: "var(--color-text-secondary)",
                background: "var(--color-bg-secondary)",
                border: "1px solid var(--color-border)",
                borderRadius: "6px",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: "4px",
              }}
            >
              ← {t("common.back", "Back")}
            </button>
          )}
          {activeTab === "accounts" && <AccountsTab />}
        {activeTab === "general" && <GeneralTab />}
        {activeTab === "proxy" && <ProxyTab />}
        {activeTab === "appearance" && <AppearanceTab />}
        {activeTab === "rules" && <RulesTab />}
        {activeTab === "remoteWrites" && <PendingOpsTab />}
        {activeTab === "translation" && <TranslateTab />}
        {activeTab === "shortcuts" && <ShortcutsTab />}
        {activeTab === "privacy" && <PrivacyTab />}
        {activeTab === "cloudSync" && <CloudSyncTab />}
        {activeTab === "about" && <AboutTab />}
      </div>
      )}
    </div>
  );
}
