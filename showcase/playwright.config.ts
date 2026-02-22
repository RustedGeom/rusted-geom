import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 60_000,
  workers: 1,
  expect: {
    timeout: 30_000,
  },
  use: {
    baseURL: "http://localhost:3000",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "pnpm build && PORT=3000 pnpm start",
    url: "http://localhost:3000",
    reuseExistingServer: false,
    timeout: 300_000,
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    {
      name: "mobile-chromium",
      use: {
        ...devices["Pixel 7"],
        browserName: "chromium",
      },
    },
  ],
});
