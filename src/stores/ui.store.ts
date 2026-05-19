import { create } from "zustand";
import { useComposeStore } from "./compose.store";
import { useMailStore } from "./mail.store";
import { deferPersist } from "@/lib/deferPersist";

export type ActiveView = "inbox" | "kanban" | "settings" | "search" | "snoozed" | "starred" | "compose" | "login";
export type SettingsTab = "accounts" | "general" | "proxy" | "appearance" | "privacy" | "rules" | "remoteWrites" | "translation" | "shortcuts" | "cloudSync" | "about";

interface UIState {
  sidebarCollapsed: boolean;
  activeView: ActiveView;
  previousView: ActiveView;
  isMobile: boolean;
  drawerOpen: boolean;
  searchQuery: string;
  settingsTab: SettingsTab;
  pendingRuleDraftText: string | null;
  showFolderUnreadCount: boolean;
  setIsMobile: (isMobile: boolean) => void;
  setDrawerOpen: (open: boolean) => void;
  toggleDrawer: () => void;
  toggleSidebar: () => void;
  setActiveView: (view: ActiveView) => void;
  openMessageInInbox: (messageId: string) => void;
  setSearchQuery: (q: string) => void;
  setSettingsTab: (tab: SettingsTab) => void;
  setPendingRuleDraftText: (text: string | null) => void;
  setShowFolderUnreadCount: (show: boolean) => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarCollapsed: false,
  activeView: "inbox",
  previousView: "inbox",
  isMobile: window.innerWidth < 768,
  drawerOpen: false,
  searchQuery: "",
  settingsTab: (sessionStorage.getItem("pebble-settings-tab") as SettingsTab) || "accounts",
  pendingRuleDraftText: null,
  showFolderUnreadCount: localStorage.getItem("pebble-show-unread-count") === "true",
  setIsMobile: (isMobile) => set({ isMobile }),
  setDrawerOpen: (open) => set({ drawerOpen: open }),
  toggleDrawer: () => set((state) => ({ drawerOpen: !state.drawerOpen })),
  toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setActiveView: (view) => {
    const state = useUIStore.getState();
    if (state.activeView === view) {
      if (state.isMobile) set({ drawerOpen: false });
      return;
    }
    if (state.activeView === "compose" && view !== "compose") {
      const composeState = useComposeStore.getState();
      if (composeState.composeDirty) {
        useComposeStore.setState({ showComposeLeaveConfirm: true, pendingView: view });
        return;
      }
      useComposeStore.setState({
        composeMode: null, composeReplyTo: null, composePrefill: null, composeDirty: false,
        showComposeLeaveConfirm: false, pendingView: null,
      });
      set({ activeView: view, drawerOpen: false });
      return;
    }
    set({ activeView: view, drawerOpen: false });
  },
  openMessageInInbox: (messageId) => {
    useMailStore.setState({
      selectedMessageId: messageId, selectedThreadId: null, threadView: false,
      selectedMessageIds: new Set(), batchMode: false,
    });
    set({ activeView: "inbox" });
  },
  setSearchQuery: (q) => set({ searchQuery: q }),
  setSettingsTab: (tab) => {
    deferPersist(() => sessionStorage.setItem("pebble-settings-tab", tab));
    set({ settingsTab: tab });
  },
  setPendingRuleDraftText: (text) => set({ pendingRuleDraftText: text }),
  setShowFolderUnreadCount: (show) => {
    deferPersist(() => localStorage.setItem("pebble-show-unread-count", String(show)));
    set({ showFolderUnreadCount: show });
  },
}));
