/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // GitHub Light theme colors
        'github-light': {
          bg: '#ffffff',
          text: '#24292e',
          border: '#e1e4e8',
          primary: '#0366d6',
          secondary: '#6a737d',
          accent: '#2188ff',
          input: '#fafbfc',
          error: '#d73a49'
        },
        // GitHub Dark theme colors
        'github-dark': {
          bg: '#0d1117',
          text: '#c9d1d9',
          border: '#30363d',
          primary: '#58a6ff',
          secondary: '#8b949e',
          accent: '#388bfd',
          input: '#0d1117',
          error: '#f85149'
        }
      }
    }
  },
  plugins: []
}