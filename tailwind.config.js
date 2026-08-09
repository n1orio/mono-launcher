/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./app.vue",
    "./pages/**/*.{vue,ts}",
    "./components/**/*.{vue,ts}",
    "./layouts/**/*.{vue,ts}",
    "./composables/**/*.ts",
    "./lib/**/*.ts",
  ],
  theme: {
    extend: {
      colors: {
        nio: {
          bg: "#0b0f1a",
          panel: "#131a2b",
          panel2: "#1a2338",
          accent: "#6d5cff",
          accent2: "#2ed4ff",
        },
      },
      fontFamily: {
        display: ["'Rubik'", "'Segoe UI'", "system-ui", "sans-serif"],
      },
      keyframes: {
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
      },
    },
  },
  plugins: [],
};