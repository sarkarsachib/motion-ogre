/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: '#121212',
        surface: '#1E1E1E',
        primary: '#3B82F6',
        secondary: '#10B981',
        accent: '#F59E0B',
        timeline: {
          bg: '#181818',
          header: '#252525',
          layer: '#2A2A2A',
          border: '#333333',
        }
      },
    },
  },
  plugins: [],
}
