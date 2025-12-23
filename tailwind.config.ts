import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./src/**/*.rs",
    "./web/**/*.ts",
    "./static/**/*.html",
  ],
  darkMode: "class",
  theme: {
    extend: {
      // ─────────────────────────────────────────────────────────────────────
      // Colors - ShadCN-inspired dark theme
      // ─────────────────────────────────────────────────────────────────────
      colors: {
        // Base backgrounds
        background: "hsl(222.2 84% 4.9%)",
        foreground: "hsl(210 40% 98%)",

        // Card/Panel surfaces
        panel: "hsl(222.2 84% 7%)",
        panelBorder: "hsl(217.2 32.6% 17.5%)",

        // Primary accent
        primary: "hsl(262.1 83.3% 57.8%)",
        primaryMuted: "hsl(262.1 83.3% 45%)",
        primaryForeground: "hsl(210 40% 98%)",

        // Secondary
        secondary: "hsl(217.2 32.6% 17.5%)",
        secondaryForeground: "hsl(210 40% 98%)",

        // Muted elements
        muted: "hsl(217.2 32.6% 17.5%)",
        mutedForeground: "hsl(215 20.2% 65.1%)",

        // Text colors
        textPrimary: "hsl(210 40% 98%)",
        textSecondary: "hsl(215 20.2% 85%)",
        textMuted: "hsl(215 20.2% 65.1%)",

        // Code blocks
        codeBg: "hsl(222.2 84% 6%)",
        codeFg: "hsl(210 40% 90%)",

        // Status colors
        success: "hsl(142.1 76.2% 36.3%)",
        warning: "hsl(47.9 95.8% 53.1%)",
        danger: "hsl(0 62.8% 50.6%)",
        info: "hsl(214.3 93.9% 67.8%)",

        // Border colors
        border: "hsl(217.2 32.6% 17.5%)",
        input: "hsl(217.2 32.6% 17.5%)",
        ring: "hsl(262.1 83.3% 57.8%)",
      },

      // ─────────────────────────────────────────────────────────────────────
      // Typography
      // ─────────────────────────────────────────────────────────────────────
      fontFamily: {
        sans: [
          "Geist",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Roboto",
          "Helvetica Neue",
          "Arial",
          "sans-serif",
        ],
        mono: [
          "Geist Mono",
          "ui-monospace",
          "SFMono-Regular",
          "SF Mono",
          "Menlo",
          "Consolas",
          "Liberation Mono",
          "monospace",
        ],
      },

      fontSize: {
        xs: ["0.75rem", { lineHeight: "1rem" }],
        sm: ["0.875rem", { lineHeight: "1.25rem" }],
        base: ["1rem", { lineHeight: "1.5rem" }],
        lg: ["1.125rem", { lineHeight: "1.75rem" }],
        xl: ["1.25rem", { lineHeight: "1.75rem" }],
        "2xl": ["1.5rem", { lineHeight: "2rem" }],
        "3xl": ["1.875rem", { lineHeight: "2.25rem" }],
        "4xl": ["2.25rem", { lineHeight: "2.5rem" }],
      },

      // ─────────────────────────────────────────────────────────────────────
      // Spacing
      // ─────────────────────────────────────────────────────────────────────
      spacing: {
        "4.5": "1.125rem",
        "13": "3.25rem",
        "15": "3.75rem",
        "17": "4.25rem",
        "18": "4.5rem",
        "22": "5.5rem",
        "26": "6.5rem",
        "30": "7.5rem",
      },

      // ─────────────────────────────────────────────────────────────────────
      // Border Radius
      // ─────────────────────────────────────────────────────────────────────
      borderRadius: {
        lg: "0.75rem",
        xl: "1rem",
        "2xl": "1.25rem",
        "3xl": "1.5rem",
      },

      // ─────────────────────────────────────────────────────────────────────
      // Animations
      // ─────────────────────────────────────────────────────────────────────
      animation: {
        "fade-in": "fadeIn 0.3s ease-out",
        "fade-up": "fadeUp 0.3s ease-out",
        "slide-in": "slideIn 0.3s ease-out",
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
      },

      keyframes: {
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        fadeUp: {
          "0%": { opacity: "0", transform: "translateY(10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        slideIn: {
          "0%": { opacity: "0", transform: "translateX(-10px)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
      },

      // ─────────────────────────────────────────────────────────────────────
      // Shadows
      // ─────────────────────────────────────────────────────────────────────
      boxShadow: {
        glow: "0 0 20px hsl(262.1 83.3% 57.8% / 0.3)",
        "glow-lg": "0 0 40px hsl(262.1 83.3% 57.8% / 0.4)",
      },

      // ─────────────────────────────────────────────────────────────────────
      // Prose (Typography plugin styles)
      // ─────────────────────────────────────────────────────────────────────
      typography: {
        DEFAULT: {
          css: {
            "--tw-prose-body": "hsl(210 40% 98%)",
            "--tw-prose-headings": "hsl(210 40% 98%)",
            "--tw-prose-links": "hsl(262.1 83.3% 57.8%)",
            "--tw-prose-bold": "hsl(210 40% 98%)",
            "--tw-prose-code": "hsl(210 40% 90%)",
            "--tw-prose-pre-code": "hsl(210 40% 90%)",
            "--tw-prose-pre-bg": "hsl(222.2 84% 6%)",
            "--tw-prose-quotes": "hsl(215 20.2% 65.1%)",
            "--tw-prose-quote-borders": "hsl(262.1 83.3% 57.8%)",
            "--tw-prose-hr": "hsl(217.2 32.6% 17.5%)",
            "--tw-prose-th-borders": "hsl(217.2 32.6% 17.5%)",
            "--tw-prose-td-borders": "hsl(217.2 32.6% 17.5%)",

            maxWidth: "none",
            code: {
              backgroundColor: "hsl(222.2 84% 6%)",
              padding: "0.2em 0.4em",
              borderRadius: "0.375rem",
              fontWeight: "400",
            },
            "code::before": { content: '""' },
            "code::after": { content: '""' },
            pre: {
              backgroundColor: "hsl(222.2 84% 6%)",
              border: "1px solid hsl(217.2 32.6% 17.5%)",
              borderRadius: "0.75rem",
            },
          },
        },
        invert: {
          css: {
            "--tw-prose-body": "hsl(210 40% 98%)",
            "--tw-prose-headings": "hsl(210 40% 98%)",
            "--tw-prose-links": "hsl(262.1 83.3% 57.8%)",
          },
        },
      },
    },
  },
  // Plugins are loaded via @plugin in CSS for Tailwind v4
  plugins: [],
};

export default config;
