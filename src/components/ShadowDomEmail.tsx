import { useRef, useLayoutEffect } from "react";
import { sanitizeHtml } from "@/lib/sanitizeHtml";

interface ShadowDomEmailProps {
  html: string;
  className?: string;
}

export function ShadowDomEmail({ html, className }: ShadowDomEmailProps) {
  const hostRef = useRef<HTMLDivElement>(null);

  // Shadow DOM 的主体需要在绘制前准备好，避免读信时从 fallback 文本闪到正文。
  useLayoutEffect(() => {
    if (!hostRef.current) return;
    const shadow = hostRef.current.shadowRoot || hostRef.current.attachShadow({ mode: "open" });

    const safeHtml = sanitizeHtml(html);
    const stylesheet = document.createElement("link");
    stylesheet.rel = "stylesheet";
    stylesheet.href = "/shadow-dom-email.css";

    const content = document.createElement("div");
    content.className = "pebble-email-content";
    content.innerHTML = safeHtml;
    shadow.replaceChildren(stylesheet, content);

    const handleClick = (event: Event) => {
      const target = event.target;
      if (!(target instanceof Element)) return;

      const anchor = target.closest<HTMLAnchorElement>("a[href]");
      const href = anchor?.getAttribute("href")?.trim();
      if (!href) return;

      if (/^https?:\/\//i.test(href)) {
        event.preventDefault();
        window.open(href, "_blank", "noopener,noreferrer");
      }
    };

    shadow.addEventListener("click", handleClick);
    return () => {
      shadow.removeEventListener("click", handleClick);
    };
  }, [html]);

  return <div ref={hostRef} className={className} />;
}
