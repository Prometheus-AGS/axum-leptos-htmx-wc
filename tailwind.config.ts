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
      // Colors - Material 3 Flat 2.0 (Dark & Light modes)
      // ─────────────────────────────────────────────────────────────────────
      colors: {
        // Light mode colors (applied when .light class is on html)
        light: {
          background: "hsl(210 20% 98%)",
          foreground: "hsl(220 20% 10%)",
          surface: "hsl(210 15% 95%)",
          surfaceVariant: "hsl(210 12% 92%)",
          surfaceContainer: "hsl(210 18% 96%)",
          bubbleUser: "hsl(262 60% 92%)",
          bubbleAssistant: "hsl(210 15% 94%)",
          bubbleTool: "hsl(210 12% 96%)",
          panel: "hsl(210 15% 95%)",
          panelBorder: "hsl(210 12% 85%)",
          primary: "hsl(262 70% 45%)",
          primaryMuted: "hsl(262 70% 55%)",
          primaryForeground: "hsl(0 0% 100%)",
          primaryContainer: "hsl(262 60% 92%)",
          secondary: "hsl(210 12% 85%)",
          secondaryForeground: "hsl(220 20% 10%)",
          muted: "hsl(210 12% 88%)",
          mutedForeground: "hsl(215 10% 45%)",
          textPrimary: "hsl(220 20% 10%)",
          textSecondary: "hsl(215 15% 25%)",
          textMuted: "hsl(215 10% 45%)",
          codeBg: "hsl(210 20% 96%)",
          codeFg: "hsl(220 15% 20%)",
          success: "hsl(142 70% 35%)",
          successContainer: "hsl(142 60% 90%)",
          warning: "hsl(48 96% 45%)",
          warningContainer: "hsl(48 80% 90%)",
          danger: "hsl(0 72% 45%)",
          dangerContainer: "hsl(0 60% 92%)",
          info: "hsl(214 94% 55%)",
          infoContainer: "hsl(214 70% 92%)",
          border: "transparent",
          input: "hsl(210 12% 85%)",
          ring: "hsl(262 70% 45%)",
        },
        // Dark mode colors (default)
        // Material 3 Flat 2.0 - Dark Mode
        // Base backgrounds - deeper contrast
        background: "hsl(220 25% 8%)",        // Main app background
        foreground: "hsl(210 40% 98%)",       // Main text

        // Surface colors - distinct layers
        surface: "hsl(220 20% 12%)",          // Primary surface
        surfaceVariant: "hsl(220 18% 16%)",   // Secondary surface
        surfaceContainer: "hsl(220 22% 10%)", // Container background
        
        // Chat bubble backgrounds - significant contrast
        bubbleUser: "hsl(262 60% 18%)",       // User message bubble (purple tint)
        bubbleAssistant: "hsl(220 20% 15%)",  // Assistant message bubble
        bubbleTool: "hsl(220 18% 13%)",       // Tool call/result bubble
        
        // Legacy panel colors (for compatibility)
        panel: "hsl(220 20% 12%)",
        panelBorder: "hsl(220 18% 20%)",      // Only for focus/ring, not visible borders

        // Primary accent - Material 3 purple
        primary: "hsl(262 83% 58%)",
        primaryMuted: "hsl(262 70% 45%)",
        primaryForeground: "hsl(210 40% 98%)",
        primaryContainer: "hsl(262 60% 22%)", // For user bubbles

        // Secondary
        secondary: "hsl(220 18% 20%)",
        secondaryForeground: "hsl(210 40% 98%)",

        // Muted elements
        muted: "hsl(220 15% 18%)",
        mutedForeground: "hsl(215 15% 65%)",

        // Text colors - WCAG AAA compliant
        textPrimary: "hsl(210 40% 98%)",      // 17.5:1 contrast
        textSecondary: "hsl(215 20% 85%)",    // 12:1 contrast
        textMuted: "hsl(215 15% 65%)",        // 7:1 contrast

        // Code blocks - distinct from bubbles
        codeBg: "hsl(220 25% 10%)",
        codeFg: "hsl(210 30% 90%)",

        // Status colors - Material 3 tones
        success: "hsl(142 70% 45%)",
        successContainer: "hsl(142 60% 18%)",
        warning: "hsl(48 96% 53%)",
        warningContainer: "hsl(48 80% 20%)",
        danger: "hsl(0 72% 51%)",
        dangerContainer: "hsl(0 60% 20%)",
        info: "hsl(214 94% 68%)",
        infoContainer: "hsl(214 70% 22%)",

        // Border colors (only for focus states, not visible borders)
        border: "transparent",
        input: "hsl(220 18% 20%)",
        ring: "hsl(262 83% 58%)",
      },

      // ─────────────────────────────────────────────────────────────────────
      // Typography
      // ─────────────────────────────────────────────────────────────────────
      fontFamily: {
        sans: [
          "Inter",
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
          "Source Code Pro",
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
