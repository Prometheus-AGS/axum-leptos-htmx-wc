/**
 * shadcn-dialog.ts
 * 
 * A web component implementation of the shadcn-ui Dialog pattern.
 * Supports:
 * - <shadcn-dialog> (Root)
 * - <shadcn-dialog-trigger>
 * - <shadcn-dialog-content>
 * - <shadcn-dialog-header>
 * - <shadcn-dialog-title>
 * - <shadcn-dialog-description>
 * - <shadcn-dialog-footer>
 * - <shadcn-dialog-close>
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
    padding: var(--sp-4, 18px);
    width: min(92vw, 520px);
    position: relative;
    color: var(--on-surface, #ececec);
    transform: translateY(8px) scale(0.96);
    transition: transform 150ms ease-out;
  }

  .overlay[data-state="open"] .content {
    transform: translateY(0) scale(1);
  }

  /* Sizes */
  :host([size="sm"]) .content { width: min(92vw, 420px); }
  :host([size="lg"]) .content { width: min(92vw, 680px); }

  /* Close X Button */
  .close-x {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 32px;
    height: 32px;
    border-radius: 8px;
    border: 0;
    background: transparent;
    color: var(--on-surface-variant, #999);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.2rem;
    line-height: 1;
    transition: background 0.2s, color 0.2s;
  }
  .close-x:hover {
    background: rgba(255,255,255,0.1);
    color: var(--on-surface, #fff);
  }
  .close-x:focus-visible {
    outline: 2px solid var(--ring, #a8caff);
    outline-offset: 2px;
  }
`;

// ==========================================
// <shadcn-dialog> (Root)
// ==========================================
export class ShadcnDialog extends HTMLElement {
  isOpen = false;

  connectedCallback() {
    this.addEventListener('shadcn-dialog-trigger-click', this.open);
    this.addEventListener('shadcn-dialog-close-click', this.close);
    this.addEventListener('keydown', this.handleKeydown);
  }

  open = () => {
    this.isOpen = true;
    this.updateState();
    this.dispatchEvent(new CustomEvent('shadcn-dialog-open'));
  };

  close = () => {
    this.isOpen = false;
    this.updateState();
    this.dispatchEvent(new CustomEvent('shadcn-dialog-close'));
  };

  private updateState() {
    const content = this.querySelector('shadcn-dialog-content');
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
// <shadcn-dialog-trigger>
// ==========================================
export class ShadcnDialogTrigger extends HTMLElement {
  connectedCallback() {
    this.addEventListener('click', (e) => {
      e.stopPropagation(); // prevent bubbling to document
      this.dispatchEvent(new CustomEvent('shadcn-dialog-trigger-click', { bubbles: true }));
    });
    this.style.display = 'inline-block';
  }
}

// ==========================================
// <shadcn-dialog-content>
// ==========================================
export class ShadcnDialogContent extends HTMLElement {
  private _shadow: ShadowRoot;
  private _previouslyFocused: Element | null = null;

  static get observedAttributes() { return ['open', 'size']; }

  constructor() {
    super();
    this._shadow = this.attachShadow({ mode: 'open' });
  }

  connectedCallback() {
    this.render();
    this._shadow.querySelector('.overlay')?.addEventListener('click', (e) => {
      if (e.target === e.currentTarget) {
        this.dispatchEvent(new CustomEvent('shadcn-dialog-close-click', { bubbles: true }));
      }
    });

    this._shadow.querySelector('.close-x')?.addEventListener('click', () => {
      this.dispatchEvent(new CustomEvent('shadcn-dialog-close-click', { bubbles: true }));
    });
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
    // Wait for transition/render
    requestAnimationFrame(() => {
        // Let's just focus the first input inside the slot if we can find it, otherwise the close button.
        // Try to focus the first logical element.
        (this._shadow.querySelector('.close-x') as HTMLElement)?.focus();
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
        <div class="content" role="dialog" aria-modal="true">
          <button class="close-x" aria-label="Close">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
          </button>
          <slot></slot>
        </div>
      </div>
    `;
  }
}

// ==========================================
// <shadcn-dialog-header>
// ==========================================
export class ShadcnDialogHeader extends HTMLElement {
  connectedCallback() {
      this.style.display = 'grid';
      this.style.gap = '8px';
      this.style.marginBottom = 'var(--sp-4, 16px)';
      this.style.textAlign = 'left';
  }
}

// ==========================================
// <shadcn-dialog-title>
// ==========================================
export class ShadcnDialogTitle extends HTMLElement {
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
// <shadcn-dialog-description>
// ==========================================
export class ShadcnDialogDescription extends HTMLElement {
  connectedCallback() {
      this.style.margin = '0';
      this.style.fontSize = '0.875rem';
      this.style.color = 'var(--on-surface-variant, #a1a1aa)';
      this.style.lineHeight = '1.5';
      this.style.display = 'block';
  }
}

// ==========================================
// <shadcn-dialog-footer>
// ==========================================
export class ShadcnDialogFooter extends HTMLElement {
  connectedCallback() {
      this.style.display = 'flex';
      this.style.flexDirection = 'row';
      this.style.justifyContent = 'flex-end';
      this.style.gap = '8px';
      this.style.marginTop = 'var(--sp-5, 24px)';
  }
}

// ==========================================
// <shadcn-dialog-close>
// ==========================================
export class ShadcnDialogClose extends HTMLElement {
  connectedCallback() {
    this.addEventListener('click', (e) => {
      e.stopPropagation();
      this.dispatchEvent(new CustomEvent('shadcn-dialog-close-click', { bubbles: true }));
    });
    this.style.display = 'inline-block';
  }
}

// ==========================================
// Registration
// ==========================================
customElements.define('shadcn-dialog', ShadcnDialog);
customElements.define('shadcn-dialog-trigger', ShadcnDialogTrigger);
customElements.define('shadcn-dialog-content', ShadcnDialogContent);
customElements.define('shadcn-dialog-header', ShadcnDialogHeader);
customElements.define('shadcn-dialog-title', ShadcnDialogTitle);
customElements.define('shadcn-dialog-description', ShadcnDialogDescription);
customElements.define('shadcn-dialog-footer', ShadcnDialogFooter);
customElements.define('shadcn-dialog-close', ShadcnDialogClose);
