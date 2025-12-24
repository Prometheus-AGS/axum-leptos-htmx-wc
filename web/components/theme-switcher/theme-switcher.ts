/**
 * Theme Switcher Web Component
 *
 * Toggles between light and dark mode with localStorage persistence.
 */

const THEME_STORAGE_KEY = "prometheus-theme";
const THEME_CLASS_DARK = "dark";
const THEME_CLASS_LIGHT = "light";

type Theme = "light" | "dark";

/**
 * Theme Switcher component for toggling between light and dark modes.
 */
export class ThemeSwitcher extends HTMLElement {
  private currentTheme: Theme = "dark";

  connectedCallback(): void {
    // Load theme from localStorage or system preference
    this.currentTheme = this.loadTheme();
    this.applyTheme(this.currentTheme);
    this.render();
    this.attachEventListeners();
  }

  /**
   * Load theme from localStorage or system preference.
   */
  private loadTheme(): Theme {
    // Check localStorage first
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark") {
      return stored;
    }

    // Fall back to system preference
    if (window.matchMedia("(prefers-color-scheme: light)").matches) {
      return "light";
    }

    return "dark";
  }

  /**
   * Apply theme to document.
   */
  private applyTheme(theme: Theme): void {
    const html = document.documentElement;

    if (theme === "light") {
      html.classList.remove(THEME_CLASS_DARK);
      html.classList.add(THEME_CLASS_LIGHT);
    } else {
      html.classList.remove(THEME_CLASS_LIGHT);
      html.classList.add(THEME_CLASS_DARK);
    }

    // Save to localStorage
    localStorage.setItem(THEME_STORAGE_KEY, theme);
    this.currentTheme = theme;
  }

  /**
   * Toggle between light and dark themes.
   */
  private toggleTheme(): void {
    const newTheme: Theme = this.currentTheme === "dark" ? "light" : "dark";
    this.applyTheme(newTheme);
    this.render();
    this.attachEventListeners(); // Re-attach after render
  }

  /**
   * Render the component.
   */
  private render(): void {
    const isDark = this.currentTheme === "dark";
    const label = isDark ? "Switch to light mode" : "Switch to dark mode";

    this.innerHTML = `
      <button
        type="button"
        class="theme-toggle-btn p-2 rounded-xl hover:bg-surface transition-colors"
        aria-label="${label}"
        title="${label}"
      >
        ${isDark ? this.getSunIcon() : this.getMoonIcon()}
      </button>
    `;
  }

  /**
   * Attach event listeners.
   */
  private attachEventListeners(): void {
    const btn = this.querySelector(".theme-toggle-btn");
    if (btn) {
      btn.addEventListener("click", () => this.toggleTheme());
    }
  }

  /**
   * Sun icon (for light mode).
   */
  private getSunIcon(): string {
    return `
      <svg class="h-5 w-5 text-textPrimary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="4"/>
        <path d="M12 2v2"/>
        <path d="M12 20v2"/>
        <path d="m4.93 4.93 1.41 1.41"/>
        <path d="m17.66 17.66 1.41 1.41"/>
        <path d="M2 12h2"/>
        <path d="M20 12h2"/>
        <path d="m6.34 17.66-1.41 1.41"/>
        <path d="m19.07 4.93-1.41 1.41"/>
      </svg>
    `;
  }

  /**
   * Moon icon (for dark mode).
   */
  private getMoonIcon(): string {
    return `
      <svg class="h-5 w-5 text-textPrimary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>
      </svg>
    `;
  }
}
