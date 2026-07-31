import { create } from "zustand";
import i18n from "@/lib/i18n";
import { getInitialLanguage, LANGUAGE_STORAGE_KEY, type Language } from "@/lib/language";
import { deferPersist } from "@/lib/deferPersist";

export type Theme = "light" | "dark" | "system";
export type MotionPref = "system" | "on" | "off";
export type { Language } from "@/lib/language";

export function resolveTheme(theme: Theme): "dark" | "light" {
  if (theme === "system") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return theme;
}

export function applyThemeToDom(theme: Theme) {
  document.documentElement.setAttribute("data-theme", resolveTheme(theme));
}

export function resolveMotion(pref: MotionPref): "on" | "off" {
  if (pref === "system") {
    return window.matchMedia?.("(prefers-reduced-motion: reduce)")?.matches ? "off" : "on";
  }
  return pref;
}

export function applyMotionToDom(pref: MotionPref) {
  document.documentElement.setAttribute("data-motion", resolveMotion(pref));
}

interface ThemeState {
  theme: Theme;
  motion: MotionPref;
  language: Language;
  setTheme: (theme: Theme) => void;
  setMotion: (motion: MotionPref) => void;
  setLanguage: (lang: Language) => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  theme: (localStorage.getItem("pebble-theme") as Theme) || "light",
  motion: (localStorage.getItem("pebble-motion") as MotionPref) || "system",
  language: getInitialLanguage(),
  setTheme: (theme) => {
    deferPersist(() => localStorage.setItem("pebble-theme", theme));
    applyThemeToDom(theme);
    set({ theme });
  },
  setMotion: (motion) => {
    deferPersist(() => localStorage.setItem("pebble-motion", motion));
    applyMotionToDom(motion);
    set({ motion });
  },
  setLanguage: (lang) => {
    i18n.changeLanguage(lang);
    deferPersist(() => localStorage.setItem(LANGUAGE_STORAGE_KEY, lang));
    set({ language: lang });
  },
}));
