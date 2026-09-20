export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: "2026-08-09",
  devtools: { enabled: false },
  ignore: ["**/.flatpak-builder", "**/flatpak-build", "**/src-tauri"],

  components: {
    dirs: [
      { path: "~/components", pathPrefix: false },
    ],
  },

  postcss: {
    plugins: {
      tailwindcss: {},
      autoprefixer: {},
    },
  },

  devServer: {
    port: 1420,
  },

  vite: {
    server: {
      strictPort: true,
      watch: {
        ignored: [/(^|[\\/])(?:\.flatpak-builder|flatpak-build|src-tauri)(?:[\\/]|$)/],
      },
    },
  },

  css: [
    "@fontsource/rubik/400.css",
    "@fontsource/rubik/500.css",
    "@fontsource/rubik/600.css",
    "@fontsource/rubik/700.css",
    "~/assets/css/main.css",
  ],

  app: {
    head: {
      title: "Mono Launcher",
      htmlAttrs: { lang: "ru" },
      meta: [
        { charset: "utf-8" },
        { name: "viewport", content: "width=device-width, initial-scale=1.0" },
      ],
      link: [
        {
          rel: "icon",
          type: "image/svg+xml",
          href: "/vite.svg",
        },
      ],
    },
  },

  // SPA-приложение внутри Tauri: генерируем чистый статический вывод в dist/.
  // Отключаем пререндеринг: при статической генерации Nuxt пытается выполнять
  // browser-код в Node.js, что зависает на browser-only API (window, navigator и т.д.).
  routeRules: {
    '/': { prerender: false },
    '/**': { prerender: false },
  },

  nitro: {
    preset: "static",
    output: {
      publicDir: "dist",
    },
    prerender: {
      autoSubfolderIndex: false,
      crawlLinks: false,
      routes: [],
    },
  },

  typescript: {
    strict: true,
    typeCheck: false,
  },
});