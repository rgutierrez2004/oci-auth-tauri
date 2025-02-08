const { nextui } = require("@nextui-org/react");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './pages/**/*.{js,ts,jsx,tsx,mdx}',
    './components/**/*.{js,ts,jsx,tsx,mdx}',
    './app/**/*.{js,ts,jsx,tsx,mdx}',
    './node_modules/@nextui-org/theme/dist/**/*.{js,ts,jsx,tsx}'
  ],
  theme: {
    extend: {},
  },
  darkMode: "class",
  plugins: [nextui({
    prefix: "nextui", 
    addCommonColors: false, 
    defaultTheme: "light", 
    defaultExtendTheme: "light", 
    layout: {}, 
    themes: {
      light: {
        layout: {}, 
        colors: {
          background: "#FAFAFA",
          foreground: "#18181B",
          primary: {
            DEFAULT: "#00DC82",
            foreground: "#FFFFFF",
          },
          muted: "#71717A",
        },
      },
      dark: {
        layout: {}, 
        colors: {
          background: "#18181B",
          foreground: "#FAFAFA",
          primary: {
            DEFAULT: "#00DC82",
            foreground: "#FFFFFF",
          },
          muted: "#A1A1AA",
        },
      },
    },
  })],
}
