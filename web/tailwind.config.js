/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Сохраняем цветовую палитру из текущих стилей
        primary: {
          DEFAULT: '#3182ce',
          hover: '#2c5aa0',
        },
        danger: {
          DEFAULT: '#e53e3e',
          hover: '#c53030',
        },
        secondary: {
          DEFAULT: '#718096',
          hover: '#4a5568',
        },
      },
    },
  },
  plugins: [],
}

