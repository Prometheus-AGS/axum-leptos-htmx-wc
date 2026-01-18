/**
 * Error Boundary Web Component
 *
 * Catches unhandled errors in child components and displays a fallback UI.
 */

export class ErrorBoundary extends HTMLElement {
  private hasError = false;
  private errorMessage = "";

  constructor() {
    super();
    this.attachShadow({ mode: "open" });
  }

  connectedCallback(): void {
    window.addEventListener("error", this.handleError);
    window.addEventListener("unhandledrejection", this.handleRejection);
    this.render();
  }

  disconnectedCallback(): void {
    window.removeEventListener("error", this.handleError);
    window.removeEventListener("unhandledrejection", this.handleRejection);
  }

  private handleError = (event: ErrorEvent): void => {
    // Check if the error happened inside this boundary
    if (this.contains(event.target as Node) || event.target === window) {
      this.catchError(event.error?.message || "An unexpected error occurred");
    }
  };

  private handleRejection = (event: PromiseRejectionEvent): void => {
    this.catchError(event.reason?.message || "A promise was rejected");
  };

  private catchError(message: string): void {
    console.error("[ErrorBoundary] Caught error:", message);
    this.hasError = true;
    this.errorMessage = message;
    this.render();
  }

  private render(): void {
    if (!this.shadowRoot) return;

    if (this.hasError) {
      this.shadowRoot.innerHTML = `
        <style>
          :host { display: block; }
          .error-container {
            padding: 1.5rem;
            margin: 1rem;
            background-color: var(--color-danger-container, #f9dedc);
            color: var(--color-danger, #b3261e);
            border-radius: 0.75rem;
            border: 1px solid rgba(179, 38, 30, 0.2);
            display: flex;
            flex-direction: column;
            items-center;
            gap: 1rem;
            text-align: center;
            font-family: system-ui, -apple-system, sans-serif;
          }
          .error-icon { width: 3rem; height: 3rem; opacity: 0.5; margin: 0 auto; }
          .error-title { font-size: 1.125rem; font-weight: bold; margin: 0 0 0.25rem 0; }
          .error-message { font-size: 0.875rem; opacity: 0.8; margin: 0; }
          .reload-btn {
            padding: 0.5rem 1rem;
            background-color: var(--color-danger, #b3261e);
            color: white;
            border: none;
            border-radius: 0.5rem;
            font-size: 0.875rem;
            font-weight: 500;
            cursor: pointer;
            transition: background-color 200ms;
            margin: 0 auto;
          }
          .reload-btn:hover { background-color: rgba(179, 38, 30, 0.9); }
        </style>
        <div class="error-container" role="alert">
          <svg class="error-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <div>
            <h3 class="error-title">Something went wrong</h3>
            <p class="error-message">${this.errorMessage}</p>
          </div>
          <button class="reload-btn" onclick="window.location.reload()">
            Reload Application
          </button>
        </div>
      `;
    } else {
      this.shadowRoot.innerHTML = `<slot></slot>`;
    }
  }
}

customElements.define("error-boundary", ErrorBoundary);
