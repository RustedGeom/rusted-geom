import { expect, test } from "@playwright/test";

test.describe("viewer shell", () => {
  test("loads, renders viewport, and shows toolbar controls", async ({ page, isMobile }) => {
    test.skip(isMobile, "Desktop-only smoke check");
    await page.goto("/");

    await expect(page.locator(".toolbar")).toBeVisible();
    await expect(page.locator(".viewport canvas")).toBeVisible();

    await expect(page.getByRole("button", { name: "Load IGES" })).toBeDisabled();
    await expect(page.getByRole("button", { name: "Save IGES" })).toBeDisabled();

    await page.getByRole("button", { name: "Zoom Extents" }).click();
    await page.getByRole("button", { name: "Orbit" }).click();
    await page.getByRole("button", { name: "Orbit" }).click();
  });

  test("saves and loads a session json", async ({ page, isMobile }) => {
    test.skip(isMobile, "Desktop-only smoke check");
    await page.goto("/");

    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Save Session" }).click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toContain("rusted-geom-session");

    const sessionPayload = {
      version: 1,
      preset: {
        name: "Reload",
        degree: 3,
        closed: false,
        sampleCount: 140,
        tolerance: {
          abs_tol: 1e-9,
          rel_tol: 1e-9,
          angle_tol: 1e-9,
        },
        points: [
          { x: -2, y: 0, z: 0 },
          { x: -1, y: 1, z: 0 },
          { x: 1, y: 1, z: 0 },
          { x: 2, y: 0, z: 0 },
        ],
      },
      view: {
        camera: {
          position: { x: 8, y: 5, z: 9 },
          target: { x: 0, y: 0, z: 0 },
          up: { x: 0, y: 1, z: 0 },
          fov: 46,
        },
        showGrid: true,
        showAxes: false,
        orbitEnabled: true,
      },
    };

    await page
      .locator('input[type="file"]')
      .setInputFiles({
        name: "session.json",
        mimeType: "application/json",
        buffer: Buffer.from(JSON.stringify(sessionPayload), "utf8"),
      });

    await expect(page.getByText("Session loaded")).toBeVisible();
  });

  test("mobile viewport exposes params toggle", async ({ page, isMobile }) => {
    test.skip(!isMobile, "Mobile-only expectation");
    await page.goto("/");

    const paramsButton = page.locator(".mobile-pane-toggle");
    await expect(paramsButton).toBeVisible();
    await paramsButton.click();
    await expect(page.locator(".pane-host.mobile-open")).toBeVisible();
  });
});
