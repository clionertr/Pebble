# brainstorm: mobile web adaptation

## Goal

Adapt the Webmail frontend (React application in `src/`) for mobile devices to ensure a functional and user-friendly experience on small screens.

## What I already know

* The project has been secondary developed as a Webmail.
* The frontend is a React application located in the `src/` directory.
* The landing page in `site/` also exists but the user specifically highlighted the frontend adaptation.
* The application likely has a multi-pane layout (sidebar, mail list, mail detail).

## Assumptions (temporary)

* The React app in `src/` is the primary target for mobile adaptation.
* A responsive layout (likely using a drawer or stack-based navigation for mobile) is required.
* The landing page in `site/` should also be reasonably mobile-friendly as a secondary goal.

## Technical Approach

### 1. Responsive Layout Strategy
* **Breakpoints:** Use 768px as the primary breakpoint for switching from desktop to mobile layout.
* **Conditional Sidebar:**
    * Desktop: Sidebar is fixed/collapsed as per current behavior.
    * Mobile: Sidebar becomes an overlay drawer (z-index: 50). Toggled via a hamburger menu button.
* **Stack Navigation for Mail:**
    * On mobile, the multi-pane layout (List + Detail) will switch to a single-pane layout.
    * If a message/thread is selected, only the Detail view is shown.
    * A "Back" button will be visible in the Detail view header on mobile to return to the list.

### 2. Component Enhancements
* **TitleBar:** Add a hamburger menu button on the left when on mobile. Hide system buttons (min/max/close) if not in Tauri environment or if they don't make sense on mobile web.
* **MessageList / ThreadList:** Adjust padding and font sizes for better readability on small screens.
* **MessageDetail:** Ensure content (HTML mail) is responsive and doesn't break the layout.

## Decision (ADR-lite)

**Context**: Need to adapt a desktop-first multi-pane email client for mobile web.
**Decision**: Use a drawer-based sidebar and a stack-based view navigation for the mail list/detail.
**Consequences**: Requires state management for the drawer and conditional rendering logic in views. Improved mobile UX without major refactoring of existing business logic.

## Requirements (finalized)

* [ ] Implement a mobile-responsive `Layout` that handles a toggleable drawer.
* [ ] Add a hamburger menu to `TitleBar`.
* [ ] Implement conditional rendering in `InboxView` (and other views) for single-pane vs multi-pane.
* [ ] Add "Back" button functionality in `MessageDetail` for mobile users.
* [ ] Ensure `ComposeView` and `SettingsView` are responsive.
* [ ] Optimize `StatusBar` and other global components for mobile.

## Acceptance Criteria (evolving)

* [ ] Navigation menu is accessible on mobile devices.
* [ ] No horizontal scrolling on mobile screens.
* [ ] Text remains readable and images are appropriately sized.
* [ ] All interactive elements (buttons, language switcher) work correctly on touch devices.

## Definition of Done (team quality bar)

* Tests added/updated (if applicable)
* Lint / typecheck / CI green
* Docs/notes updated if behavior changes
* Verified on simulated mobile devices (Chrome DevTools)

## Out of Scope (explicit)

* Mobile adaptation of the full React application in `src/` (unless confirmed).
* Large-scale redesign of the landing page.

## Technical Notes

* `site/index.html`: Main HTML file.
* `site/style.css`: Main stylesheet with existing `@media` queries.
* `site/main.js`: Handles interactions and scroll reveals.
