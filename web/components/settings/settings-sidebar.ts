export class SettingsSidebar extends HTMLElement {
  connectedCallback() {
      this.render();
      this.loadTypes();
  }

  async loadTypes() {
      // Mock Data (matches reference image categories)
      const types = [
          { id: 'general', name: 'General Settings', icon: 'settings', display_mode: 'form' },
          { id: 'model_providers', name: 'Model Provider', icon: 'cloud', display_mode: 'master-detail' },
          { id: 'default_model', name: 'Default Model', icon: 'auto_awesome', display_mode: 'form' },
          { id: 'display', name: 'Display Settings', icon: 'monitor', display_mode: 'form' },
          { id: 'data', name: 'Data Settings', icon: 'database', display_mode: 'form' },
          { id: 'mcp', name: 'MCP Servers', icon: 'extension', display_mode: 'master-detail' },
      ];
      
      this.renderList(types);
  }

  render() {
      this.className = "flex flex-col h-full bg-surfaceVariant/30";
      this.innerHTML = `
          <div class="p-4 border-b border-panelBorder flex items-center justify-between">
            <span class="font-semibold text-sm">Settings</span>
          </div>
          <div id="types-list" class="flex-1 overflow-y-auto py-2 space-y-0.5"></div>
      `;
  }

  renderList(types: any[]) {
      const list = this.querySelector('#types-list');
      if (!list) return;

      types.forEach(t => {
          const item = document.createElement('button');
          item.className = `
            w-full flex items-center gap-3 px-4 py-2 text-sm text-textMuted hover:text-textPrimary hover:bg-surfaceVariant/50 transition-colors text-left
          `;
          item.innerHTML = `
            <span class="material-symbols-rounded text-lg opacity-70">${t.icon}</span>
            <span>${t.name}</span>
          `;
          item.onclick = () => {
              // Highlight selection
              list.querySelectorAll('button').forEach(b => b.classList.remove('bg-surfaceVariant', 'text-textPrimary', 'font-medium'));
              item.classList.add('bg-surfaceVariant', 'text-textPrimary', 'font-medium');
              
              // Dispatch event
              this.dispatchEvent(new CustomEvent('settings-type-selected', {
                  bubbles: true,
                  detail: { typeId: t.id, displayMode: t.display_mode }
              }));
          };
          list.appendChild(item);
      });
      
      // Auto-select first
      (list.firstElementChild as HTMLElement)?.click();
  }
}

customElements.define("settings-sidebar", SettingsSidebar);
