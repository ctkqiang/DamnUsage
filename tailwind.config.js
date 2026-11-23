export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          "SF Pro Display",
          "SF Pro Text",
          "PingFang SC",
          "Hiragino Sans GB",
          "Microsoft YaHei",
          "sans-serif",
        ],
        mono: [
          "SF Mono",
          "JetBrains Mono",
          "Menlo",
          "Monaco",
          "monospace",
        ],
      },
      colors: {
        notch: {
          950: "#000000",
          900: "#070709",
          850: "#0e0f14",
          800: "#161822",
          700: "#232635",
        },
      },
      boxShadow: {
        notch: "0 24px 64px -12px rgba(0, 0, 0, 0.85), 0 0 0 1px rgba(255, 255, 255, 0.08)",
        glow: "0 0 24px -4px rgba(124, 92, 255, 0.45)",
      },
    },
  },
  plugins: [],
};
