/**
 * File Upload Web Component with Drag-and-Drop Support
 *
 * Provides a modern interface for adding multimodal attachments (images, PDFs, documents)
 * to chat prompts. Handles drag-and-drop, visual feedback, and client-side validation.
 */

import { generateUuid } from "../../utils/uuid";

/**
 * Configuration for file upload constraints.
 */
export interface FileUploadConfig {
  /** Maximum number of files per prompt */
  maxFilesPerPrompt: number;
  /** Maximum size per file in bytes */
  maxFileSize: number;
  /** Maximum total size of all files in bytes */
  maxTotalSize: number;
  /** Allowed MIME types (empty = all) */
  allowedMimeTypes: string[];
}

/**
 * Represents an attached file with preview and status.
 */
export interface AttachedFile {
  id: string;
  file: File;
  /** Data URL for image previews */
  preview?: string;
  /** Upload/processing status */
  status: "pending" | "uploading" | "ready" | "error";
  /** Error message if status is 'error' */
  error?: string;
}

/**
 * Custom event detail for files-changed event.
 */
export interface FilesChangedEventDetail {
  files: AttachedFile[];
}

/**
 * File Upload Web Component
 *
 * @fires files-changed - Fired when files are added or removed
 *
 * @example
 * ```html
 * <file-upload max-files="10" max-file-size="52428800"></file-upload>
 * ```
 */
export class FileUpload extends HTMLElement {
  private config: FileUploadConfig = {
    maxFilesPerPrompt: 10,
    maxFileSize: 50 * 1024 * 1024, // 50MB
    maxTotalSize: 100 * 1024 * 1024, // 100MB
    allowedMimeTypes: [],
  };

  private attachedFiles: AttachedFile[] = [];
  private fileInput: HTMLInputElement | null = null;
  private previewContainer: HTMLElement | null = null;
  private dropZoneOverlay: HTMLElement | null = null;
  private dragCounter = 0;

  // Bound event handlers for cleanup
  private _handleDragEnter: EventListener | null = null;
  private _handleDragOver: EventListener | null = null;
  private _handleDragLeave: EventListener | null = null;
  private _handleDrop: EventListener | null = null;

  static get observedAttributes(): string[] {
    return ["max-files", "max-file-size", "max-total-size", "allowed-types"];
  }

  connectedCallback(): void {
    this.parseAttributes();
    this.render();
    this.setupEventListeners();
  }

  disconnectedCallback(): void {
    this.cleanupEventListeners();
  }

  attributeChangedCallback(
    _name: string,
    oldValue: string | null,
    newValue: string | null
  ): void {
    if (oldValue === newValue) return;
    this.parseAttributes();
  }

  private parseAttributes(): void {
    const maxFiles = this.getAttribute("max-files");
    const maxFileSize = this.getAttribute("max-file-size");
    const maxTotalSize = this.getAttribute("max-total-size");
    const allowedTypes = this.getAttribute("allowed-types");

    if (maxFiles) this.config.maxFilesPerPrompt = parseInt(maxFiles, 10);
    if (maxFileSize) this.config.maxFileSize = parseInt(maxFileSize, 10);
    if (maxTotalSize) this.config.maxTotalSize = parseInt(maxTotalSize, 10);
    if (allowedTypes)
      this.config.allowedMimeTypes = allowedTypes.split(",").map((t) => t.trim());
  }

  private render(): void {
    this.innerHTML = `
      <div class="file-upload-container relative">
        <!-- File preview area (shows above the input when files are attached) -->
        <div class="file-preview-area hidden absolute bottom-full left-0 right-0 mb-2 p-3 bg-surface rounded-xl border border-outline/20 shadow-lg max-h-40 overflow-y-auto z-10">
          <div class="file-list flex flex-wrap gap-2"></div>
        </div>
        
        <!-- Drop zone overlay (full-screen when dragging files) -->
        <div class="drop-zone-overlay hidden fixed inset-0 z-50 bg-primary/10 backdrop-blur-sm flex items-center justify-center pointer-events-none">
          <div class="bg-surface p-8 rounded-2xl shadow-xl border-2 border-dashed border-primary">
            <div class="flex flex-col items-center gap-2">
              <svg class="w-12 h-12 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                <polyline points="17 8 12 3 7 8"/>
                <line x1="12" y1="3" x2="12" y2="15"/>
              </svg>
              <p class="text-lg font-medium text-primary">Drop files here</p>
              <p class="text-sm text-textSecondary">Images, PDFs, and documents</p>
            </div>
          </div>
        </div>
        
        <!-- Upload button with attachment icon -->
        <label class="upload-button shrink-0 h-11 w-11 md:h-12 md:w-12 rounded-xl md:rounded-2xl bg-surface hover:bg-surfaceContainer flex items-center justify-center cursor-pointer transition-all border border-outline/20 relative" title="Attach files">
          <input 
            type="file" 
            multiple 
            accept="image/*,.pdf,.doc,.docx,.txt,.md,.xlsx,.pptx,.csv,.json,.xml,.html,.css,.js,.ts,.py,.rs,.go,.java,.c,.cpp,.h,.hpp" 
            class="hidden" 
          />
          <svg class="h-5 w-5 text-textSecondary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48"/>
          </svg>
          <span class="file-count-badge hidden absolute -top-1 -right-1 w-5 h-5 bg-primary text-white text-xs rounded-full flex items-center justify-center font-medium"></span>
        </label>
      </div>
    `;

    this.fileInput = this.querySelector('input[type="file"]');
    this.previewContainer = this.querySelector(".file-list");
    this.dropZoneOverlay = this.querySelector(".drop-zone-overlay");
  }

  private setupEventListeners(): void {
    // File input change
    this.fileInput?.addEventListener("change", (e) => {
      const target = e.target as HTMLInputElement;
      if (target.files) {
        this.handleFiles(Array.from(target.files));
        target.value = ""; // Reset to allow re-selecting same files
      }
    });

    // Drag and drop listeners on document body
    this._handleDragEnter = this.handleDragEnter.bind(this) as EventListener;
    this._handleDragOver = this.handleDragOver.bind(this) as EventListener;
    this._handleDragLeave = this.handleDragLeave.bind(this) as EventListener;
    this._handleDrop = this.handleDrop.bind(this) as EventListener;

    document.body.addEventListener("dragenter", this._handleDragEnter);
    document.body.addEventListener("dragover", this._handleDragOver);
    document.body.addEventListener("dragleave", this._handleDragLeave);
    document.body.addEventListener("drop", this._handleDrop);
  }

  private cleanupEventListeners(): void {
    if (this._handleDragEnter)
      document.body.removeEventListener("dragenter", this._handleDragEnter);
    if (this._handleDragOver)
      document.body.removeEventListener("dragover", this._handleDragOver);
    if (this._handleDragLeave)
      document.body.removeEventListener("dragleave", this._handleDragLeave);
    if (this._handleDrop)
      document.body.removeEventListener("drop", this._handleDrop);
  }

  // ---------------------------------------------------------------------------
  // Drag and Drop Handlers
  // ---------------------------------------------------------------------------

  private handleDragEnter(e: DragEvent): void {
    e.preventDefault();
    e.stopPropagation();
    this.dragCounter++;

    if (e.dataTransfer?.types.includes("Files")) {
      this.dropZoneOverlay?.classList.remove("hidden");
    }
  }

  private handleDragOver(e: DragEvent): void {
    e.preventDefault();
    e.stopPropagation();
  }

  private handleDragLeave(e: DragEvent): void {
    e.preventDefault();
    e.stopPropagation();
    this.dragCounter--;

    if (this.dragCounter === 0) {
      this.dropZoneOverlay?.classList.add("hidden");
    }
  }

  private handleDrop(e: DragEvent): void {
    e.preventDefault();
    e.stopPropagation();
    this.dragCounter = 0;
    this.dropZoneOverlay?.classList.add("hidden");

    const files = e.dataTransfer?.files;
    if (files) {
      this.handleFiles(Array.from(files));
    }
  }

  // ---------------------------------------------------------------------------
  // File Handling
  // ---------------------------------------------------------------------------

  private handleFiles(files: File[]): void {
    let currentTotalSize = this.attachedFiles.reduce(
      (sum, f) => sum + f.file.size,
      0
    );

    for (const file of files) {
      // Check file count limit
      if (this.attachedFiles.length >= this.config.maxFilesPerPrompt) {
        console.warn(
          `[file-upload] Max files limit (${this.config.maxFilesPerPrompt}) reached`
        );
        break;
      }

      // Check individual file size
      if (file.size > this.config.maxFileSize) {
        console.warn(
          `[file-upload] File "${file.name}" exceeds max size (${this.formatSize(file.size)} > ${this.formatSize(this.config.maxFileSize)})`
        );
        continue;
      }

      // Check total size
      if (currentTotalSize + file.size > this.config.maxTotalSize) {
        console.warn(
          `[file-upload] Total size would exceed limit (${this.formatSize(this.config.maxTotalSize)})`
        );
        break;
      }

      // Check MIME type if restrictions are set
      if (
        this.config.allowedMimeTypes.length > 0 &&
        !this.config.allowedMimeTypes.some((allowed) =>
          file.type.match(new RegExp(allowed.replace("*", ".*")))
        )
      ) {
        console.warn(
          `[file-upload] File "${file.name}" has unsupported type: ${file.type}`
        );
        continue;
      }

      const attachedFile: AttachedFile = {
        id: generateUuid(),
        file,
        status: "ready",
      };

      // Generate preview for images
      if (file.type.startsWith("image/")) {
        const reader = new FileReader();
        reader.onload = (e) => {
          attachedFile.preview = e.target?.result as string;
          this.renderPreviews();
        };
        reader.readAsDataURL(file);
      }

      this.attachedFiles.push(attachedFile);
      currentTotalSize += file.size;
    }

    this.renderPreviews();
    this.updateBadge();
    this.dispatchFilesChanged();
  }

  private removeFile(id: string): void {
    this.attachedFiles = this.attachedFiles.filter((f) => f.id !== id);
    this.renderPreviews();
    this.updateBadge();
    this.dispatchFilesChanged();
  }

  // ---------------------------------------------------------------------------
  // Rendering
  // ---------------------------------------------------------------------------

  private renderPreviews(): void {
    const container = this.previewContainer;
    const previewArea = this.querySelector(".file-preview-area");
    if (!container || !previewArea) return;

    if (this.attachedFiles.length === 0) {
      previewArea.classList.add("hidden");
      return;
    }

    previewArea.classList.remove("hidden");
    container.innerHTML = this.attachedFiles
      .map(
        (f) => `
      <div class="file-item flex items-center gap-2 p-2 bg-surfaceContainer rounded-lg group" data-file-id="${f.id}">
        ${this.renderFilePreview(f)}
        <div class="flex flex-col min-w-0 flex-1">
          <span class="text-xs font-medium truncate max-w-[120px]" title="${f.file.name}">${f.file.name}</span>
          <span class="text-xs text-textMuted">${this.formatSize(f.file.size)}</span>
        </div>
        <button 
          type="button" 
          class="remove-file p-1 rounded-full hover:bg-danger/20 text-textMuted hover:text-danger transition-colors opacity-0 group-hover:opacity-100"
          data-file-id="${f.id}"
          title="Remove file"
          aria-label="Remove ${f.file.name}"
        >
          <svg class="w-4 h-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    `
      )
      .join("");

    // Setup remove button listeners
    container.querySelectorAll(".remove-file").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        const fileId = (e.currentTarget as HTMLElement).dataset.fileId;
        if (fileId) this.removeFile(fileId);
      });
    });
  }

  private renderFilePreview(file: AttachedFile): string {
    if (file.preview) {
      return `<img src="${file.preview}" class="w-10 h-10 rounded object-cover shrink-0" alt="${file.file.name}" />`;
    }

    // Icon for non-image files based on type
    const icon = this.getFileIcon(file.file.type, file.file.name);
    return `
      <div class="w-10 h-10 rounded bg-surface flex items-center justify-center shrink-0">
        ${icon}
      </div>
    `;
  }

  private getFileIcon(mimeType: string, fileName: string): string {
    const ext = fileName.split(".").pop()?.toLowerCase() || "";

    // PDF
    if (mimeType === "application/pdf" || ext === "pdf") {
      return `<svg class="w-5 h-5 text-danger" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="9" y1="15" x2="15" y2="15"/></svg>`;
    }

    // Word documents
    if (
      mimeType.includes("word") ||
      ext === "doc" ||
      ext === "docx"
    ) {
      return `<svg class="w-5 h-5 text-blue-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg>`;
    }

    // Spreadsheets
    if (
      mimeType.includes("excel") ||
      mimeType.includes("spreadsheet") ||
      ext === "xlsx" ||
      ext === "csv"
    ) {
      return `<svg class="w-5 h-5 text-green-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><rect x="8" y="13" width="8" height="6"/></svg>`;
    }

    // Code files
    if (
      mimeType.includes("javascript") ||
      mimeType.includes("typescript") ||
      mimeType.includes("json") ||
      mimeType.includes("xml") ||
      mimeType.includes("html") ||
      mimeType.includes("css") ||
      ["js", "ts", "py", "rs", "go", "java", "c", "cpp", "h", "hpp"].includes(ext)
    ) {
      return `<svg class="w-5 h-5 text-yellow-500" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>`;
    }

    // Text/Markdown
    if (mimeType.includes("text") || ext === "md" || ext === "txt") {
      return `<svg class="w-5 h-5 text-textSecondary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><line x1="10" y1="9" x2="8" y2="9"/></svg>`;
    }

    // Default file icon
    return `<svg class="w-5 h-5 text-textMuted" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>`;
  }

  private updateBadge(): void {
    const badge = this.querySelector(".file-count-badge");
    if (!badge) return;

    if (this.attachedFiles.length > 0) {
      badge.textContent = this.attachedFiles.length.toString();
      badge.classList.remove("hidden");
    } else {
      badge.classList.add("hidden");
    }
  }

  // ---------------------------------------------------------------------------
  // Utilities
  // ---------------------------------------------------------------------------

  private formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  private dispatchFilesChanged(): void {
    this.dispatchEvent(
      new CustomEvent<FilesChangedEventDetail>("files-changed", {
        detail: { files: [...this.attachedFiles] },
        bubbles: true,
        composed: true,
      })
    );
  }

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  /**
   * Get all attached files.
   */
  public getFiles(): File[] {
    return this.attachedFiles.map((f) => f.file);
  }

  /**
   * Get attached files with metadata.
   */
  public getAttachedFiles(): AttachedFile[] {
    return [...this.attachedFiles];
  }

  /**
   * Clear all attached files.
   */
  public clearFiles(): void {
    this.attachedFiles = [];
    this.renderPreviews();
    this.updateBadge();
    this.dispatchFilesChanged();
  }

  /**
   * Check if any files are attached.
   */
  public hasFiles(): boolean {
    return this.attachedFiles.length > 0;
  }

  /**
   * Get the total size of all attached files.
   */
  public getTotalSize(): number {
    return this.attachedFiles.reduce((sum, f) => sum + f.file.size, 0);
  }
}

// Register the custom element
customElements.define("file-upload", FileUpload);
