import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { QueryClientProvider } from "@tanstack/react-query";
import { queryClient } from "@/lib/query-client";
import { logStartupTiming } from "@/lib/startupTiming";
import "@/lib/i18n";
import App from "./App";
import "./styles/index.css";

logStartupTiming("frontend entry loaded");

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </StrictMode>,
);
