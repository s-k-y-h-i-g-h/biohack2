// biohack2 service worker — static asset cache only.
//
// v4: the app JS/WASM (biohack2_web.js, biohack2_web_bg.wasm) are deliberately
// NOT cached. They change on every build, and caching them creates a PWA update
// deadlock: the old SW serves stale JS, the new JS can't run until the SW updates,
// and the SW won't update until the new JS runs. Static assets (icons, manifest,
// styles) are cached instead — they're stable across builds.
const CACHE_NAME = 'biohack2-v4';
const urlsToCache = [
  '/',
  '/index.html',
  '/manifest.json',
  '/icon-192.png',
  '/icon-512.png',
  '/styles/global.css',
];

self.addEventListener('install', event => {
  self.skipWaiting();
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => cache.addAll(urlsToCache))
  );
});

self.addEventListener('fetch', event => {
  const url = new URL(event.request.url);

  // API calls always hit the network — never serve them from cache
  if (url.pathname.startsWith('/api/')) {
    event.respondWith(fetch(event.request));
    return;
  }

  // The app bundle (JS/WASM) always goes to network so fixes land immediately.
  if (url.pathname.endsWith('.js') || url.pathname.endsWith('.wasm')) {
    event.respondWith(fetch(event.request));
    return;
  }

  // Stale-while-revalidate for everything else (static assets).
  event.respondWith(
    caches.open(CACHE_NAME).then(cache =>
      cache.match(event.request).then(cached => {
        const fetched = fetch(event.request)
          .then(response => {
            if (response && response.status === 200 && response.type === 'basic') {
              cache.put(event.request, response.clone());
            }
            return response;
          })
          .catch(() => cached);
        return cached || fetched;
      })
    )
  );
});

self.addEventListener('activate', event => {
  event.waitUntil(
    Promise.all([
      caches.keys().then(cacheNames =>
        Promise.all(
          cacheNames
            .filter(name => name !== CACHE_NAME)
            .map(name => caches.delete(name))
        )
      ),
      self.clients.claim(),
    ])
  );
});