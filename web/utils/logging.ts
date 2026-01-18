const DEBUG_ENABLED = (() => {
  if (typeof window === "undefined") {
    return false;
  }

  try {
    const params = new URLSearchParams(window.location.search);
    return (
      params.has("debug") || window.localStorage.getItem("debug") === "true"
    );
  } catch {
    return false;
  }
})();

export const isDebugEnabled = (): boolean => DEBUG_ENABLED;

export const debugLog = (...args: unknown[]): void => {
  if (DEBUG_ENABLED) {
    // eslint-disable-next-line no-console
    console.log(...args);
  }
};

export const debugWarn = (...args: unknown[]): void => {
  if (DEBUG_ENABLED) {
    console.warn(...args);
  }
};
