import daisyui from 'daisyui'

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {},
  },
  plugins: [daisyui],
  daisyui: {
    themes: ['light', 'dark', 'cupcake'],
    base: true,
    styled: true,
    utils: true,
  },
}