export default defineNuxtConfig({
  ssr: false,
  compatibilityDate: "2026-08-09",
  devtools: { enabled: false },

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
        ignored: ["**/src-tauri/**"],
      },
    },
  },

  css: [
    "@fontsource/monaspace-neon/400.css",
    "@fontsource/monaspace-neon/500.css",
    "@fontsource/monaspace-neon/600.css",
    "@fontsource/monaspace-neon/700.css",
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
  nitro: {
    preset: "static",
    output: {
      publicDir: "dist",
    },
  },

  typescript: {
    strict: true,
    typeCheck: true,
  },
});