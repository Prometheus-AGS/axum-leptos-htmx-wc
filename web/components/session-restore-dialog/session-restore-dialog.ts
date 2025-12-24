/**
 * Session Restore Dialog Component
 * 
 * Shown when server session has expired, offering to restore from client history
 * or start fresh.
 */

export class SessionRestoreDialog extends HTMLElement {
  private conversationTitle = 'this conversation';
  private messageCount = 0;

  connectedCallback(): void {
    this.render();
    this.attachEventListeners();
  }

  private render(): void {
    this.innerHTML = `
      <div class="dialog-overlay" role="dialog" aria-modal="true" aria-labelledby="dialog-title">
        <div class="dialog-content">
          <div class="dialog-header">
            <svg class="dialog-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 6v6l4 2"/>
            </svg>
            <h2 id="dialog-title" class="dialog-title">Session Expired</h2>
          </div>
          
          <div class="dialog-body">
            <p class="dialog-message">
              The server session for <strong>${this.escapeHtml(this.conversationTitle)}</strong> has expired.
            </p>
            <p class="dialog-submessage">
              Your conversation history (${this.messageCount} messages) is safely stored locally.
              Would you like to continue with your history or start fresh?
            </p>
          </div>
          
          <div class="dialog-actions">
            <button class="dialog-btn dialog-btn-secondary" data-action="fresh">
              <svg class="btn-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 5v14m-7-7h14"/>
              </svg>
              Start Fresh
            </button>
            <button class="dialog-btn dialog-btn-primary" data-action="restore">
              <svg class="btn-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
                <path d="M21 3v5h-5"/>
                <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
                <path d="M3 21v-5h5"/>
              </svg>
              Continue with History
            </button>
          </div>
        </div>
      </div>
    `;
  }

  private attachEventListeners(): void {
    // Close on overlay click
    this.querySelector('.dialog-overlay')?.addEventListener('click', (e) => {
      if (e.target === e.currentTarget) {
        this.handleClose();
      }
    });

    // Start fresh button
    this.querySelector('[data-action="fresh"]')?.addEventListener('click', () => {
      this.dispatchEvent(new CustomEvent('session-fresh'));
      this.remove();
    });

    // Restore button
    this.querySelector('[data-action="restore"]')?.addEventListener('click', () => {
      this.dispatchEvent(new CustomEvent('session-restore'));
      this.remove();
    });

    // Escape key to close
    document.addEventListener('keydown', this.handleEscape);
  }

  private handleEscape = (e: KeyboardEvent): void => {
    if (e.key === 'Escape') {
      this.handleClose();
    }
  };

  private handleClose(): void {
    this.dispatchEvent(new CustomEvent('session-cancel'));
    this.remove();
  }

  disconnectedCallback(): void {
    document.removeEventListener('keydown', this.handleEscape);
  }

  private escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  /**
   * Public API: Set conversation details
   */
  setConversation(title: string, messageCount: number): void {
    this.conversationTitle = title;
    this.messageCount = messageCount;
    this.render();
    this.attachEventListeners();
  }

  /**
   * Public API: Show dialog
   */
  static show(title: string, messageCount: number): SessionRestoreDialog {
    const dialog = document.createElement('session-restore-dialog') as SessionRestoreDialog;
    dialog.setConversation(title, messageCount);
    document.body.appendChild(dialog);
    return dialog;
  }
}

// Register the custom element
customElements.define('session-restore-dialog', SessionRestoreDialog);
