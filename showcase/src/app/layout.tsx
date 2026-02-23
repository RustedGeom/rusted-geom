import type { Metadata } from "next";
import { Sora, Source_Sans_3 } from "next/font/google";

import "./globals.css";

const displayFont = Sora({
  variable: "--font-display",
  subsets: ["latin"],
  weight: ["500", "600", "700"],
});

const bodyFont = Source_Sans_3({
  variable: "--font-body",
  subsets: ["latin"],
  weight: ["400", "500", "600"],
});

export const metadata: Metadata = {
  title: "rusted-geom | Geometry Kernel Showcase",
  description: "Interactive showcase for the rusted-geom WASM geometry kernel — NURBS, meshes, surfaces, intersections, and trim operations.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${displayFont.variable} ${bodyFont.variable}`}>{children}</body>
    </html>
  );
}
