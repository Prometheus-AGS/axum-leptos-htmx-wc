/**
 * Token Counter Web Component
 * 
 * Displays token usage with progress indicator and cost estimate
 */

import {
  formatTokenCount,
  calculateContextPercentage,
  getUsageColorClass,
  getUsageBackgroundClass,
  formatCost,
} from "../../utils/token-counter";

export class TokenCounter extends HTMLElement {
  static get observedAttributes(): string[] {
    return ["input-tokens", "output-tokens", "context-limit", "model-id", "cost", "is-estimate"];
  }

  private _inputTokens: number = 0;
  private _outputTokens: number = 0;
  private _contextLimit: number = 128000;
  private _modelId: string = "";
  private _cost: number = 0;
  private _isEstimate: boolean = true;

  connectedCallback(): void {
    this.updateFromAttributes();
    this.render();
  }

  attributeChangedCallback(): void {
    this.updateFromAttributes();
    this.render();
  }

  private updateFromAttributes(): void {
    this._inputTokens = parseInt(this.getAttribute("input-tokens") || "0", 10);
    this._outputTokens = parseInt(this.getAttribute("output-tokens") || "0", 10);
    this._contextLimit = parseInt(this.getAttribute("context-limit") || "128000", 10);
    this._modelId = this.getAttribute("model-id") || "";
    this._cost = parseFloat(this.getAttribute("cost") || "0");
    this._isEstimate = this.getAttribute("is-estimate") !== "false";
  }

  private render(): void {
    const totalTokens = this._inputTokens + this._outputTokens;
    const percentage = calculateContextPercentage(totalTokens, this._contextLimit);
    const colorClass = getUsageColorClass(percentage);
    const bgColorClass = getUsageBackgroundClass(percentage);
    const remaining = Math.max(0, this._contextLimit - totalTokens);

    const estimateLabel = this._isEstimate ? " (est.)" : "";

    this.innerHTML = `
      <div class="token-counter relative inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-surfaceContainer text-xs">
        <!-- Token Display -->
        <div class="flex items-center gap-1">
          <svg class="w-3.5 h-3.5 text-textMuted" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/>
          </svg>
          <span class="font-medium ${colorClass}">
            ${formatTokenCount(totalTokens)}${estimateLabel}
          </span>
          <span class="text-textMuted">/</span>
          <span class="text-textMuted">${formatTokenCount(this._contextLimit)}</span>
          <span class="text-textMuted">(${percentage.toFixed(1)}%)</span>
        </div>

        <!-- Progress Bar -->
        <div class="relative w-16 h-1.5 bg-surfaceContainerHighest rounded-full overflow-hidden">
          <div class="${bgColorClass} h-full transition-all duration-300" style="width: ${Math.min(100, percentage)}%"></div>
        </div>

        <!-- Cost (if available) -->
        ${this._cost > 0 ? `
          <div class="flex items-center gap-1 text-textMuted border-l border-surfaceContainerHighest pl-2">
            <svg class="w-3 h-3" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M16 8h-6a2 2 0 1 0 0 4h4a2 2 0 1 1 0 4H8"/>
              <path d="M12 18V6"/>
            </svg>
            <span>${formatCost(this._cost)}</span>
          </div>
        ` : ''}

        <!-- Tooltip (hidden by default, shown on hover) -->
        <div class="token-tooltip hidden absolute bottom-full right-0 mb-2 px-3 py-2 bg-surfaceContainerHighest rounded-lg shadow-lg text-xs whitespace-nowrap z-50">
          <div class="space-y-1">
            <div class="flex justify-between gap-4">
              <span class="text-textMuted">Input:</span>
              <span class="text-textPrimary font-medium">${formatTokenCount(this._inputTokens)}</span>
            </div>
            <div class="flex justify-between gap-4">
              <span class="text-textMuted">Output:</span>
              <span class="text-textPrimary font-medium">${formatTokenCount(this._outputTokens)}</span>
            </div>
            <div class="flex justify-between gap-4">
              <span class="text-textMuted">Remaining:</span>
              <span class="text-textPrimary font-medium">${formatTokenCount(remaining)}</span>
            </div>
            ${this._modelId ? `
              <div class="flex justify-between gap-4 pt-1 border-t border-surfaceContainer">
                <span class="text-textMuted">Model:</span>
                <span class="text-textPrimary font-mono text-xs">${this._modelId}</span>
              </div>
            ` : ''}
          </div>
          <!-- Tooltip arrow -->
          <div class="absolute top-full left-1/2 transform -translate-x-1/2 -mt-px">
            <div class="w-2 h-2 bg-surfaceContainerHighest transform rotate-45"></div>
          </div>
        </div>
      </div>
    `;

    // Add hover listeners for tooltip
    const container = this.querySelector(".token-counter");
    const tooltip = this.querySelector(".token-tooltip");
    
    if (container && tooltip) {
      container.addEventListener("mouseenter", () => {
        tooltip.classList.remove("hidden");
      });
      container.addEventListener("mouseleave", () => {
        tooltip.classList.add("hidden");
      });
    }

    // Listen for global usage updates
    window.addEventListener('token-usage-update', (e: Event) => {
        const detail = (e as CustomEvent).detail;
        if (detail) {
            this.updateTokens(detail.input, detail.output);
            if (detail.cost) this.updateCost(detail.cost);
            if (detail.model) {
                this._modelId = detail.model;
                this.render();
            }
        }
    });
  }

  /**
   * Update token counts programmatically
   */
  updateTokens(inputTokens: number, outputTokens: number, isEstimate: boolean = true): void {
    this._inputTokens = inputTokens;
    this._outputTokens = outputTokens;
    this._isEstimate = isEstimate;
    this.render();
  }

  /**
   * Update cost programmatically
   */
  updateCost(cost: number): void {
    this._cost = cost;
    this.render();
  }
}
