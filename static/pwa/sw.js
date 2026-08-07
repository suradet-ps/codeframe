const CACHE_NAME = "codeframe-v1";
const PRECACHE_URLS = [
  "/",
  "/index.html",
  "/style.css",
  "/favicon.svg",
  "/manifest.json",
  "/icon-192.png",
  "/icon-512.png"
];

// Content-hashed assets (JS/WASM) are added to the cache on first fetch and
// never referenced again after a redeploy, so the cache would grow forever.
// Cap the total number of entries; the oldest extra entries are evicted.
const MAX_CACHE_ENTRIES = 80;

async function pruneCache() {
  const cache = await caches.open(CACHE_NAME);
  const keys = await cache.keys();
  if (keys.length <= MAX_CACHE_ENTRIES) return;
  const keep = new Set(
    PRECACHE_URLS.map((url) => new URL(url, self.location.origin).href)
  );
  // keys() returns insertion order, so filtering keeps the newest extras.
  const extras = keys.filter((request) => !keep.has(request.url));
  const excess = Math.min(keys.length - MAX_CACHE_ENTRIES, extras.length);
  for (let i = 0; i < excess; i++) {
    await cache.delete(extras[i]);
  }
}

// --- Install: precache the app shell ---
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(PRECACHE_URLS))
  );
  self.skipWaiting();
});

// --- Activate: clean up old caches ---
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      )
    )
  );
  self.clients.claim();
});

// --- Fetch: cache-first for static assets, network-first for navigation ---
self.addEventListener("fetch", (event) => {
  const { request } = event;

  // Navigation requests: network-first, fallback to cached index.html
  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const clone = response.clone();
          caches.open(CACHE_NAME).then((cache) => cache.put(request, clone));
          return response;
        })
        .catch(() => caches.match("/index.html"))
    );
    return;
  }

  // All other requests (CSS, WASM, JS, fonts, images): stale-while-revalidate
  event.respondWith(
    caches.match(request).then((cached) => {
      const networkFetch = fetch(request)
        .then((response) => {
          if (response.ok && request.url.startsWith(self.location.origin)) {
            const clone = response.clone();
            caches.open(CACHE_NAME).then((cache) =>
              cache.put(request, clone).then(pruneCache)
            );
          }
          return response;
        })
        .catch(() => {
          // Network failed and no cache — return a proper error response
          // instead of letting the promise reject with a TypeError.
          return new Response("Offline", {
            status: 503,
            statusText: "Service Unavailable",
          });
        });

      return cached || networkFetch;
    })
  );
});
