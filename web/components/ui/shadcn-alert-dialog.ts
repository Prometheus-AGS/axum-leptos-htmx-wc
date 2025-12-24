/**
 * shadcn-alert-dialog.ts
 * 
 * A web component implementation of the shadcn-ui Alert Dialog pattern.
 * Supports:
 * - <shadcn-alert-dialog> (Root)
 * - <shadcn-alert-dialog-trigger>
 * - <shadcn-alert-dialog-content>
 * - <shadcn-alert-dialog-header>
 * - <shadcn-alert-dialog-title>
 * - <shadcn-alert-dialog-description>
 * - <shadcn-alert-dialog-footer>
 * - <shadcn-alert-dialog-action>
 * - <shadcn-alert-dialog-cancel>
 */

// ==========================================
// Base Styles (Shared)
// ==========================================
const styles = `
  :host {
    display: inline-block;
  }

  * { box-sizing: border-box; }

  /* Overlay */
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 18px;
    z-index: 50;
    opacity: 0;
    visibility: hidden;
    transition: opacity 150ms ease-out, visibility 150ms ease-out;
  }

  .overlay[data-state="open"] {
    opacity: 1;
    visibility: visible;
  }

  /* Content */
  .content {
    background: var(--surface-low, #1e1e1e);
    border-radius: var(--r-xl, 20px);
    box-shadow: var(--shadow-2, 0 18px 44px rgba(0,0,0,0.58));
    padding: var(--sp-4, 24px);
    width: min(92vw, 520px);
    position: relative;
    color: var(--on-surface, #ececec);
    transform: translateY(8px) scale(0.96);
    transition: transform 150ms ease-out;
  }

  .overlay[data-state="open"] .content {
    transform: translateY(0) scale(1);
  }
`;

// ==========================================
// <shadcn-alert-dialog> (Root)
// ==========================================
export class ShadcnAlertDialog extends HTMLElement {
  isOpen = false;

  connectedCallback() {
    this.addEventListener('shadcn-alert-trigger-click', this.open);
    this.addEventListener('shadcn-alert-close-click', this.close);
    this.addEventListener('shadcn-alert-action-click', this.handleAction);
    this.addEventListener('keydown', this.handleKeydown);
  }

  open = () => {
    this.isOpen = true;
    this.updateState();
    this.dispatchEvent(new CustomEvent('shadcn-alert-open'));
  };

  close = () => {
    this.isOpen = false;
    this.updateState();
    this.dispatchEvent(new CustomEvent('shadcn-alert-close'));
  };
  
  private handleAction = () => {
      // Logic for action is handled by the consumer listening to the event, 
      // but we might want to auto-close?
      // Usually Alert Dialogs wait for async op, so consumer calls close().
      // But for simple cases, we bubble up.
  }

  private updateState() {
    const content = this.querySelector('shadcn-alert-dialog-content');
    if (content) {
      if (this.isOpen) {
        content.setAttribute('open', '');
      } else {
        content.removeAttribute('open');
      }
    }
  }

  private handleKeydown = (e: KeyboardEvent) => {
    if (this.isOpen && e.key === 'Escape') {
      this.close();
    }
  };
}

// ==========================================
// <shadcn-alert-dialog-trigger>
// ==========================================
export class ShadcnAlertDialogTrigger extends HTMLElement {
  connectedCallback() {
    this.addEventListener('click', (e) => {
      e.stopPropagation();
      this.dispatchEvent(new CustomEvent('shadcn-alert-trigger-click', { bubbles: true }));
    });
    this.style.display = 'inline-block';
  }
}

// ==========================================
// <shadcn-alert-dialog-content>
// ==========================================
export class ShadcnAlertDialogContent extends HTMLElement {
  private _shadow: ShadowRoot;
  private _previouslyFocused: Element | null = null;

  static get observedAttributes() { return ['open']; }

  constructor() {
    super();
    this._shadow = this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
    // Alert Dialog: usually clicking overlay DOES NOT close it (modal). 
    // shadcn behavior: usually does not close on outside click for alerts? 
    // Actually standard Dialog does, Alert maybe not.
    // Let's allow it for UX, or check spec. 
    // Shadcn Alert Dialog docs: "A modal dialog that interrupts...".
    // Usually they are stubborn. I'll make it stubborn (no overlay click close).
  }

  attributeChangedCallback(name: string, _oldValue: string, newValue: string) {

    if (name === 'open') {
      const isOpen = newValue !== null;
      const overlay = this._shadow.querySelector('.overlay');
      if (overlay) {
        overlay.setAttribute('data-state', isOpen ? 'open' : 'closed');
        if (isOpen) {
            this._previouslyFocused = document.activeElement;
            this.trapFocus();
        } else {
            this.releaseFocus();
        }
      }
    }
  }

  private trapFocus() {
    requestAnimationFrame(() => {
        // Focus the cancel button by default for safety
        const cancelBtn = this.querySelector('shadcn-alert-dialog-cancel');
        if (cancelBtn) {
            // Find the button inside the web component
            // We can't access Shadow DOM of light DOM children easily if they are also web components
            // But shadcn-alert-dialog-cancel is a wrapper.
            // Assumption: Consumer puts a <button> inside <shadcn-alert-dialog-cancel> or the component itself is clickable
            // If the wrapper has a button inside, focus it.
            const btn = cancelBtn.querySelector('button');
            if (btn) (btn as HTMLElement).focus();
            else (cancelBtn as HTMLElement).focus();
        }
    });
  }

  private releaseFocus() {
    if (this._previouslyFocused && (this._previouslyFocused as HTMLElement).focus) {
        (this._previouslyFocused as HTMLElement).focus();
    }
  }

  render() {
    this._shadow.innerHTML = `
      <style>${styles}</style>
      <div class="overlay" data-state="closed" role="presentation">
        <div class="content" role="alertdialog" aria-modal="true">
          <slot></slot>
        </div>
      </div>
    `;
  }
}

// ==========================================
// <shadcn-alert-dialog-header>
// ==========================================
export class ShadcnAlertDialogHeader extends HTMLElement {
  connectedCallback() {
      this.style.display = 'grid';
      this.style.gap = '8px';
      this.style.marginBottom = 'var(--sp-4, 16px)';
      this.style.textAlign = 'left';
  }
}

// ==========================================
// <shadcn-alert-dialog-title>
// ==========================================
export class ShadcnAlertDialogTitle extends HTMLElement {
  connectedCallback() {
      this.style.margin = '0';
      this.style.fontSize = '1.125rem';
      this.style.fontWeight = '600';
      this.style.lineHeight = '1';
      this.style.color = 'var(--on-surface, #ececec)';
      this.style.display = 'block';
  }
}

// ==========================================
// <shadcn-alert-dialog-description>
// ==========================================
export class ShadcnAlertDialogDescription extends HTMLElement {
  connectedCallback() {
      this.style.margin = '0';
      this.style.fontSize = '0.875rem';
      this.style.color = 'var(--on-surface-variant, #a1a1aa)';
      this.style.lineHeight = '1.5';
      this.style.display = 'block';
  }
}

// ==========================================
// <shadcn-alert-dialog-footer>
// ==========================================
export class ShadcnAlertDialogFooter extends HTMLElement {
  connectedCallback() {
      this.style.display = 'flex';
      this.style.flexDirection = 'row';
      this.style.justifyContent = 'flex-end';
      this.style.gap = '8px';
      this.style.marginTop = 'var(--sp-5, 24px)';
  }
}

// ==========================================
// <shadcn-alert-dialog-action>
// ==========================================
export class ShadcnAlertDialogAction extends HTMLElement {
  connectedCallback() {
    this.addEventListener('click', (e) => {
      e.stopPropagation();
      this.dispatchEvent(new CustomEvent('shadcn-alert-action-click', { bubbles: true }));
    });
    this.style.display = 'inline-block';
  }
}

// ==========================================
// <shadcn-alert-dialog-cancel>
// ==========================================
export class ShadcnAlertDialogCancel extends HTMLElement {
  connectedCallback() {
    this.addEventListener('click', (e) => {
      e.stopPropagation();
      this.dispatchEvent(new CustomEvent('shadcn-alert-close-click', { bubbles: true }));
    });
    this.style.display = 'inline-block';
  }
}

// ==========================================
// Registration
// ==========================================
customElements.define('shadcn-alert-dialog', ShadcnAlertDialog);
customElements.define('shadcn-alert-dialog-trigger', ShadcnAlertDialogTrigger);
customElements.define('shadcn-alert-dialog-content', ShadcnAlertDialogContent);
customElements.define('shadcn-alert-dialog-header', ShadcnAlertDialogHeader);
customElements.define('shadcn-alert-dialog-title', ShadcnAlertDialogTitle);
customElements.define('shadcn-alert-dialog-description', ShadcnAlertDialogDescription);
customElements.define('shadcn-alert-dialog-footer', ShadcnAlertDialogFooter);
customElements.define('shadcn-alert-dialog-action', ShadcnAlertDialogAction);
customElements.define('shadcn-alert-dialog-cancel', ShadcnAlertDialogCancel);
