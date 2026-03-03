import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  transpilePackages: ["@rustedgeom/kernel"],
};

export default nextConfig;
