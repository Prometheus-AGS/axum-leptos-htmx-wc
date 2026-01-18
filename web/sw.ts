/// <reference lib="webworker" />

/**
 * Service Worker for Prometheus
 *
 * Provides offline support by caching static assets and PGlite WASM files.
 */

const CACHE_NAME = "prometheus-v1";
const ASSETS_TO_CACHE = [
  "/",
  "/static/app.css",
  "/static/main.js",
  "/static/pglite.wasm",
  "/static/pglite.data",
  "/static/fonts/Inter-Variable.woff2",
  "/static/vendor/alpine.min.js",
];

// Use a self-executing function to scope the service worker logic
((sw: ServiceWorkerGlobalScope) => {
  sw.addEventListener("install", (event: ExtendableEvent) => {
    event.waitUntil(
      caches.open(CACHE_NAME).then((cache) => {
        return cache.addAll(ASSETS_TO_CACHE);
      }),
    );
    void sw.skipWaiting();
  });

  sw.addEventListener("activate", (event: ExtendableEvent) => {
    event.waitUntil(
      caches.keys().then((cacheNames) => {
        return Promise.all(
          cacheNames
            .filter((name) => name !== CACHE_NAME)
            .map((name) => caches.delete(name)),
        );
      }),
    );
    void sw.clients.claim();
  });

  sw.addEventListener("fetch", (event: FetchEvent) => {
    // Skip non-GET requests
    if (event.request.method !== "GET") return;

    // Skip API requests (should always go to network)
    if (event.request.url.includes("/api/")) return;

    // Handle navigation requests (HTML) - Offline-first for the app shell
    if (event.request.mode === "navigate") {
      event.respondWith(
        fetch(event.request).catch(() => {
          return caches.match("/").then((response) => {
            return response || Response.error();
          });
        }),
      );
      return;
    }

    event.respondWith(
      caches.match(event.request).then((cachedResponse) => {
        if (cachedResponse) {
          return cachedResponse;
        }

        return fetch(event.request).then((response) => {
          // Don't cache non-ok responses or non-static assets
          if (
            !response ||
            response.status !== 200 ||
            response.type !== "basic"
          ) {
            return response;
          }

          const responseToCache = response.clone();
          void caches.open(CACHE_NAME).then((cache) => {
            void cache.put(event.request, responseToCache);
          });

          return response;
        });
      }),
    );
  });
})(self as unknown as ServiceWorkerGlobalScope);
