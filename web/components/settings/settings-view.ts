import { debugLog } from "../../utils/logging";

interface SchemaProperty {
  type: string;
  enum?: string[];
  title?: string;
  minimum?: number;
  maximum?: number;
}

interface SchemaSection {
  type: string;
  properties: Record<string, SchemaProperty>;
}

export class SettingsView extends HTMLElement {
  private _schema: Record<string, SchemaSection> | null = null;
  private _data: Record<string, Record<string, unknown>> | null = null;

  connectedCallback() {
    void this.loadSettings();
  }

  async loadSettings() {
    // Fetch schema types (simulated for now, would be /api/settings/types)
    // and data
    this.renderLoading();

    try {
      // Mock fetching for UI dev
      this._schema = {
        llm: {
          type: "object",
          properties: {
            provider: {
              type: "string",
              enum: ["openai", "anthropic", "google"],
              title: "Default Provider",
            },
            model: {
              type: "string",
              title: "Default Model",
            },
            temperature: {
              type: "number",
              minimum: 0,
              maximum: 2,
              title: "Temperature",
            },
          },
        },
      };

      this._data = {
        llm: {
          provider: "openai",
          model: "gpt-4o",
          temperature: 0.7,
        },
      };

      this.renderForm();
    } catch (e) {
      this.renderError(e);
    }
  }

  renderLoading() {
    this.innerHTML = `<div class="p-8 text-center text-textMuted animate-pulse">Loading settings...</div>`;
  }

  renderError(e: unknown) {
    const message = e instanceof Error ? e.message : String(e);
    this.innerHTML = `<div class="p-8 text-center text-danger">Failed to load settings: ${message}</div>`;
  }

  renderForm() {
    if (!this._schema || !this._data) return;

    // Simple schema-to-form generator
    const sections = Object.entries(this._schema)
      .map(([key, sectionSchema]) => {
        const sectionData = this._data ? this._data[key] || {} : {};

        return `
            <div class="mb-8 p-6 bg-surfaceContainer rounded-xl">
                <h3 class="text-lg font-semibold mb-4 capitalize">${key} Settings</h3>
                <div class="space-y-4">
                    ${this.renderFields(key, sectionSchema.properties, sectionData)}
                </div>
            </div>
          `;
      })
      .join("");

    this.innerHTML = `
        <div class="max-w-3xl mx-auto py-8 px-4">
            <h2 class="text-2xl font-bold mb-6">Settings</h2>
            <form id="settings-form" onsubmit="return false;">
                ${sections}
                <div class="sticky bottom-4 flex justify-end gap-3 mt-8 p-4 bg-surfaceContainerHighest/80 backdrop-blur-md rounded-lg shadow-lg">
                    <button type="button" class="px-4 py-2 text-sm font-medium text-textMuted hover:text-textPrimary transition-colors" onclick="this.closest('settings-view').reset()">Reset</button>
                    <button type="button" class="px-4 py-2 text-sm font-medium bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors shadow-sm" onclick="this.closest('settings-view').save()">Save Changes</button>
                </div>
            </form>
        </div>
      `;
  }

  renderFields(
    sectionKey: string,
    properties: Record<string, SchemaProperty>,
    data: Record<string, unknown>,
  ): string {
    return Object.entries(properties)
      .map(([propKey, prop]) => {
        const value = data[propKey];
        const fullKey = `${sectionKey}.${propKey}`;

        if (prop.enum) {
          const options = prop.enum
            .map(
              (opt: string) =>
                `<option value="${opt}" ${value === opt ? "selected" : ""}>${opt}</option>`,
            )
            .join("");
          return `
                <div class="flex flex-col gap-1.5">
                    <label class="text-sm font-medium text-textPrimary">${prop.title || propKey}</label>
                    <select name="${fullKey}" class="px-3 py-2 bg-surfaceContainerHighest rounded-lg text-sm text-textPrimary focus:ring-2 focus:ring-primary/20 outline-none transition-all">
                        ${options}
                    </select>
                </div>
              `;
        }

        if (prop.type === "number") {
          return `
                <div class="flex flex-col gap-1.5">
                    <label class="text-sm font-medium text-textPrimary">${prop.title || propKey}</label>
                    <input type="number" name="${fullKey}" value="${value}" step="0.1" min="${prop.minimum}" max="${prop.maximum}" class="px-3 py-2 bg-surfaceContainerHighest rounded-lg text-sm text-textPrimary focus:ring-2 focus:ring-primary/20 outline-none transition-all" />
                </div>
              `;
        }

        return `
            <div class="flex flex-col gap-1.5">
                <label class="text-sm font-medium text-textPrimary">${prop.title || propKey}</label>
                <input type="text" name="${fullKey}" value="${value || ""}" class="px-3 py-2 bg-surfaceContainerHighest rounded-lg text-sm text-textPrimary focus:ring-2 focus:ring-primary/20 outline-none transition-all" />
            </div>
          `;
      })
      .join("");
  }

  save() {
    // Collect form data and PUT to API
    debugLog("Saving settings...");
    // Implementation pending API integration
  }

  reset() {
    void this.loadSettings();
  }
}

customElements.define("settings-view", SettingsView);
