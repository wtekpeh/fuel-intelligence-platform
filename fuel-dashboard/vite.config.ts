import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    allowedHosts: [".williamtekpeh.com"],
    // Adding a dot at the start (.your-domain.com) allows all subdomains as well.
  },
});
