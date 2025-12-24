/**
 * Copy Button Web Component
 *
 * A button that copies text content to the clipboard.
 */

import { copyToClipboard } from "../../utils/clipboard";

/**
 * Copy Button component for clipboard operations.
 */
export class CopyButton extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["target", "text"];
  }

  private _target: string = "";
  private _text: string = "";

  get target(): string {
    return this._target;
  }

  set target(value: string) {
    this._target = value;
  }

  get text(): string {
    return this._text;
  }

  set text(value: string) {
    this._text = value;
  }

  connectedCallback(): void {
    this._target = this.getAttribute("target") ?? "";
    this._text = this.getAttribute("text") ?? "";
    this.render();
    this.addEventListener("click", this.handleClick.bind(this));
  }

  disconnectedCallback(): void {
    this.removeEventListener("click", this.handleClick.bind(this));
  }

  attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null,
  ): void {
    if (oldValue === newValue) return;

    if (name === "target") {
      this._target = newValue ?? "";
    } else if (name === "text") {
      this._text = newValue ?? "";
    }
  }

  private render(): void {
    this.innerHTML = `
      <button 
        type="button" 
        class="copy-button inline-flex items-center justify-center p-1.5 rounded-md 
               bg-surfaceContainerHighest hover:bg-primary/20 text-textPrimary hover:text-primary 
               transition-colors text-xs shadow-sm"
        aria-label="Copy to clipboard"
        title="Copy to clipboard"
      >
        <svg class="copy-icon h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
        </svg>
        <svg class="check-icon h-4 w-4 hidden" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
      </button>
    `;
  }

  private async handleClick(event: Event): Promise<void> {
    event.preventDefault();
    event.stopPropagation();

    let textToCopy = this._text;

    // If no direct text, try to get from target element
    if (!textToCopy && this._target) {
      const targetEl = document.getElementById(this._target);
      if (targetEl) {
        // First check for raw content attribute (for markdown)
        const rawContent = targetEl.getAttribute("data-raw-content");
        if (rawContent) {
          textToCopy = rawContent;
        } else {
          textToCopy = targetEl.textContent ?? "";
        }
      }
    }

    if (!textToCopy) {
      console.warn("[copy-button] No text to copy");
      return;
    }

    const success = await copyToClipboard(textToCopy.trim());

    if (success) {
      this.showSuccess();
    } else {
      this.showError();
    }
  }

  private showSuccess(): void {
    const copyIcon = this.querySelector(".copy-icon");
    const checkIcon = this.querySelector(".check-icon");
    const button = this.querySelector("button");

    if (copyIcon && checkIcon && button) {
      copyIcon.classList.add("hidden");
      checkIcon.classList.remove("hidden");
      button.classList.add("text-success");

      setTimeout(() => {
        copyIcon.classList.remove("hidden");
        checkIcon.classList.add("hidden");
        button.classList.remove("text-success");
      }, 2000);
    }
  }

  private showError(): void {
    const button = this.querySelector("button");
    if (button) {
      button.classList.add("text-danger");
      setTimeout(() => {
        button.classList.remove("text-danger");
      }, 2000);
    }
  }
}
