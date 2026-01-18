import type { PgliteStatus } from "../../stores/pglite-store";
import { pgliteStore } from "../../stores/pglite-store";

type StorageEstimate = {
  usage?: number;
  quota?: number;
};

export class StorageHealth extends HTMLElement {
  private status: PgliteStatus = pgliteStore.getStatus();
  private estimate: StorageEstimate = {};
  private refreshTimer: number | null = null;
  private readonly handleStatus = (event: Event) => {
    const detail = (event as CustomEvent<PgliteStatus>).detail;
    if (detail) {
      this.status = detail;
      this.render();
    }
  };

  connectedCallback(): void {
    this.classList.add("inline-flex");
    window.addEventListener("pglite-status", this.handleStatus);
    void this.refreshEstimate();
    this.scheduleRefresh();
    this.render();
  }

  disconnectedCallback(): void {
    window.removeEventListener("pglite-status", this.handleStatus);
    if (this.refreshTimer !== null) {
      window.clearInterval(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  private scheduleRefresh(): void {
    this.refreshTimer = window.setInterval(() => {
      void this.refreshEstimate();
    }, 60_000);
  }

  private async refreshEstimate(): Promise<void> {
    if (!navigator.storage?.estimate) {
      return;
    }

    try {
      this.estimate = await navigator.storage.estimate();
      this.render();
    } catch {
      // Ignore estimate failures; status should not be noisy.
    }
  }

  private formatBytes(value: number): string {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let idx = 0;
    let size = value;

    while (size >= 1024 && idx < units.length - 1) {
      size /= 1024;
      idx += 1;
    }

    return `${size.toFixed(size >= 10 || idx === 0 ? 0 : 1)}${units[idx]}`;
  }

  private buildStatus(): {
    label: string;
    containerClass: string;
    dotClass: string;
    detail: string;
  } {
    const usage = this.estimate.usage;
    const quota = this.estimate.quota;
    const ratio = usage && quota ? usage / quota : null;

    const migrationsTotal = this.status.migrationsTotal;
    const migrationsApplied = this.status.migrationsApplied;
    const migrationLabel =
      migrationsTotal > 0
        ? `Migrations ${migrationsApplied}/${migrationsTotal}`
        : "";

    const usageLabel =
      usage !== undefined && quota !== undefined
        ? `Usage ${this.formatBytes(usage)} of ${this.formatBytes(quota)}`
        : "Usage unknown";

    if (this.status.status === "error") {
      return {
        label: "Storage Error",
        containerClass: "bg-dangerContainer text-danger",
        dotClass: "bg-danger",
        detail: [this.status.error, usageLabel, migrationLabel]
          .filter(Boolean)
          .join(" • "),
      };
    }

    if (this.status.status !== "ready") {
      return {
        label: "Storage Starting",
        containerClass: "bg-surfaceContainerHighest text-textSecondary",
        dotClass: "bg-info animate-pulse",
        detail: [usageLabel, migrationLabel].filter(Boolean).join(" • "),
      };
    }

    if (ratio !== null && ratio >= 0.9) {
      return {
        label: "Storage Critical",
        containerClass: "bg-dangerContainer text-danger",
        dotClass: "bg-danger animate-pulse",
        detail: [usageLabel, migrationLabel].filter(Boolean).join(" • "),
      };
    }

    if (ratio !== null && ratio >= 0.8) {
      return {
        label: "Storage Low",
        containerClass: "bg-warningContainer text-warning",
        dotClass: "bg-warning",
        detail: [usageLabel, migrationLabel].filter(Boolean).join(" • "),
      };
    }

    return {
      label: "Storage OK",
      containerClass: "bg-successContainer text-success",
      dotClass: "bg-success",
      detail: [usageLabel, migrationLabel].filter(Boolean).join(" • "),
    };
  }

  private render(): void {
    const status = this.buildStatus();
    const usage = this.estimate.usage;
    const quota = this.estimate.quota;
    const usageText =
      usage !== undefined && quota !== undefined
        ? `${this.formatBytes(usage)}/${this.formatBytes(quota)}`
        : "Usage N/A";

    this.setAttribute("title", status.detail);
    this.setAttribute("aria-label", status.detail || status.label);
    this.setAttribute("role", "status");
    this.setAttribute("aria-live", "polite");

    this.innerHTML = `
      <div class="flex items-center gap-2 rounded-full px-3 py-1.5 text-[11px] font-medium ${status.containerClass}">
        <span class="h-2 w-2 rounded-full ${status.dotClass}" aria-hidden="true"></span>
        <span>${status.label}</span>
        <span class="text-[10px] text-textMuted">${usageText}</span>
      </div>
    `;
  }
}

customElements.define("storage-health", StorageHealth);
