import { getCurrentWindow } from "../tauri-mock";
import { logStartupTiming } from "@/lib/startupTiming";

export async function showMainWindow() {
  try {
    await getCurrentWindow().show();
    logStartupTiming("main window shown");
  } catch (err) {
    console.warn("Failed to show main window", err);
  }
}
