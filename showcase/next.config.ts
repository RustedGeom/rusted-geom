import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: ["@rusted-geom/bindings-web"],
};

export default nextConfig;
