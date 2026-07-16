import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Minimal `.next/standalone` server output for the production Docker
  // image (frontend/Dockerfile) — avoids shipping the full node_modules tree.
  output: "standalone",
};

export default nextConfig;
