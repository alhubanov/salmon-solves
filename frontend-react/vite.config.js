import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  // "/" suits a custom domain, where the site is served from the root. GitHub's default project
  // URL (user.github.io/salmon-solves/) serves from a sub-path instead, and every asset 404s
  // unless this matches it — the deploy workflow sets BASE_PATH for that case.
  base: process.env.BASE_PATH || "/",
  plugins: [react()],
  server: {
    fs: {
      allow: [
        path.resolve(__dirname, ".."),
      ],
    },
  }
})