import { SettingsView } from "./settings-view";

export class SettingsPane extends HTMLElement {
  private _displayMode: string = "form";

  // Cache for master-detail selections
  private _selectedItemId: string | null = null;

  loadType(_typeId: string, displayMode: string) {
    this._displayMode = displayMode;
    this.render();
  }

  render() {
    this.innerHTML = "";

    if (this._displayMode === "master-detail") {
      this.renderMasterDetail();
    } else {
      this.renderSimpleForm();
    }
  }

  async renderMasterDetail() {
    this.classList.add("flex", "w-full");

    // Mock List Data (Middle Pane)
    const items = [
      { id: "azure", name: "AzureAnthropic", enabled: true },
      { id: "local", name: "Local", enabled: true },
      { id: "openai", name: "OpenAI", enabled: true },
      { id: "ollama", name: "Ollama", enabled: true },
    ];

    // Pane 2: List
    const listPane = document.createElement("div");
    listPane.className = "w-72 bg-surfaceContainer flex flex-col";

    listPane.innerHTML = `
        <div class="p-4 bg-surfaceContainerHighest">
            <div class="relative">
                <span class="material-symbols-rounded absolute left-2 top-2 text-textMuted text-sm">search</span>
                <input type="text" placeholder="Search Providers..." class="w-full pl-8 pr-3 py-1.5 bg-surfaceContainerHighest rounded-md text-sm outline-none focus:ring-2 focus:ring-primary/30">
            </div>
        </div>
        <div class="flex-1 overflow-y-auto p-2 space-y-1">
            ${items
              .map(
                (i) => `
                <div class="provider-item group flex items-center justify-between p-2 rounded-lg cursor-pointer hover:bg-surfaceVariant/50 ${this._selectedItemId === i.id ? "bg-surfaceVariant" : ""}" data-id="${i.id}">
                    <div class="flex items-center gap-2">
                        <div class="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center text-xs font-bold text-primary">${i.name[0]}</div>
                        <span class="text-sm font-medium ${this._selectedItemId === i.id ? "text-textPrimary" : "text-textMuted group-hover:text-textPrimary"}">${i.name}</span>
                    </div>
                    <div class="w-8 h-4 rounded-full ${i.enabled ? "bg-successContainer" : "bg-surfaceVariant"} relative transition-colors">
                        <div class="absolute top-0.5 left-0.5 w-3 h-3 rounded-full ${i.enabled ? "bg-success translate-x-4" : "bg-surfaceContainerHighest"} transition-transform shadow-sm"></div>
                    </div>
                </div>
            `,
              )
              .join("")}
        </div>
      `;

    // Pane 3: Detail
    const detailPane = document.createElement("div");
    detailPane.className = "flex-1 bg-surface relative";
    detailPane.id = "detail-pane";

    if (this._selectedItemId) {
      // Render SettingsView for selected item
      const view = new SettingsView();
      // Inject context so it knows what to load
      view.setAttribute("context-id", this._selectedItemId);
      detailPane.appendChild(view);
    } else {
      detailPane.innerHTML = `<div class="absolute inset-0 flex items-center justify-center text-textMuted text-sm">Select an item to configure</div>`;
    }

    // Events
    listPane.querySelectorAll(".provider-item").forEach((el: Element) => {
      (el as HTMLElement).onclick = () => {
        this._selectedItemId = (el as HTMLElement).dataset.id || null;
        this.renderMasterDetail(); // Re-render to update selection state
      };
    });

    this.appendChild(listPane);
    this.appendChild(detailPane);
  }

  renderSimpleForm() {
    this.classList.remove("flex", "w-full"); // Reset specific MD classes
    this.className = "flex-1 bg-surface overflow-y-auto"; // Simple view container

    const view = new SettingsView();
    // Inject context (global settings for this type)
    view.setAttribute("context-id", "global");
    this.appendChild(view);
  }
}

customElements.define("settings-pane", SettingsPane);
