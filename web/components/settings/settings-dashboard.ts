import { SettingsPane } from "./settings-pane.ts";
import { SettingsSidebar } from "./settings-sidebar.ts";

export class SettingsDashboard extends HTMLElement {
  private sidebar: SettingsSidebar;
  private pane: SettingsPane;

  constructor() {
    super();
    this.sidebar = new SettingsSidebar();
    this.pane = new SettingsPane();

    // Wire up events
    this.addEventListener("settings-type-selected", (e: Event) => {
      const detail = (e as CustomEvent).detail;
      this.pane.loadType(detail.typeId, detail.displayMode);
    });
  }

  connectedCallback() {
    this.render();
  }

  render() {
    this.classList.add("flex", "h-full", "bg-surface");
    this.innerHTML = ``;

    // Sidebar Container (Pane 1)
    const sidebarContainer = document.createElement("div");
    sidebarContainer.className = "w-64 bg-surfaceContainer flex flex-col";
    sidebarContainer.appendChild(this.sidebar);
    this.appendChild(sidebarContainer);

    // Main Content (Pane 2 & 3 handled by SettingsPane)
    const contentContainer = document.createElement("div");
    contentContainer.className = "flex-1 flex overflow-hidden";
    contentContainer.appendChild(this.pane);
    this.appendChild(contentContainer);
  }
}

customElements.define("settings-dashboard", SettingsDashboard);
