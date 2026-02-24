import { expect, test } from "@playwright/test";

test.describe("viewer shell", () => {
  test("loads, renders viewport, and exposes toolbar and console actions", async ({ page, isMobile }) => {
    test.skip(isMobile, "Desktop-only smoke check");
    await page.goto("/");

    await expect(page.locator(".toolbar")).toBeVisible();
    await expect(page.locator(".viewport canvas")).toBeVisible();
    await expect(page.locator(".inspector-panel.is-open")).toBeVisible();
    await expect(page.locator(".kernel-console.is-open")).toBeVisible();

    await expect(page.getByRole("button", { name: "Load IGES" })).toBeDisabled();
    await expect(page.getByRole("button", { name: "Save IGES" })).toBeDisabled();

    await page.getByRole("button", { name: "Zoom Extents" }).click();
    await page.getByRole("button", { name: "Orbit" }).click();
    await page.getByRole("button", { name: "Orbit" }).click();

    await page.getByRole("button", { name: "Toggle Console" }).click();
    await expect(page.locator(".kernel-console.is-collapsed")).toBeVisible();
    await page.getByRole("button", { name: "Toggle Console" }).click();
    await expect(page.locator(".kernel-console.is-open")).toBeVisible();

    const logDownloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Export Logs" }).click();
    const logDownload = await logDownloadPromise;
    expect(logDownload.suggestedFilename()).toMatch(/^rusted-geom-console-.*\.log$/);

    await expect(page.locator(".kernel-log").first()).toBeVisible();
    await page.getByRole("button", { name: "Clear Logs" }).click();
    await expect(page.locator(".kernel-log")).toHaveCount(0);
    await expect(page.locator(".kernel-console-empty")).toBeVisible();

    const exampleSelect = page.locator(".inspector-field select").first();
    await exampleSelect.selectOption("meshLarge");
    await expect(
      page.locator(".inspector-readout output").filter({ hasText: "Dense Torus Benchmark" }),
    ).toBeVisible();
    await exampleSelect.selectOption("meshBoolean");
    await expect(
      page
        .locator(".inspector-readout output")
        .filter({ hasText: "Boolean Difference (Box - Torus)" }),
    ).toBeVisible();
    await exampleSelect.selectOption("surfaceLarge");
    await expect(
      page
        .locator(".inspector-readout output")
        .filter({ hasText: "Large Trimmed NURBS Surface" }),
    ).toBeVisible();
    await exampleSelect.selectOption("bboxCurveNonTrivial");
    await expect(
      page
        .locator(".inspector-readout output")
        .filter({ hasText: "Bounds Curve: Mixed Polycurve" }),
    ).toBeVisible();
    await exampleSelect.selectOption("bboxSurfaceWarped");
    await expect(
      page
        .locator(".inspector-readout output")
        .filter({ hasText: "Bounds Surface: Warped Rational" }),
    ).toBeVisible();
    await exampleSelect.selectOption("bboxMeshBooleanAssembly");
    await expect(
      page
        .locator(".inspector-readout output")
        .filter({ hasText: "Bounds Mesh: Boolean Assembly" }),
    ).toBeVisible();
    await exampleSelect.selectOption("bboxBrepSolidLifecycle");
    await expect(
      page
        .locator(".inspector-readout output")
        .filter({ hasText: "Bounds BREP: Solid Lifecycle" }),
    ).toBeVisible();
    await expect(page.locator(".inspector-readout span").filter({ hasText: "Bounds" })).toBeVisible();
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

  test("mobile viewport coordinates controls drawer and console dock", async ({ page, isMobile }) => {
    test.skip(!isMobile, "Mobile-only expectation");
    await page.goto("/");

    const controlsButton = page.getByRole("button", { name: "Toggle Controls" });
    const consoleButton = page.getByRole("button", { name: "Toggle Console" });

    await controlsButton.click();
    await expect(page.locator(".inspector-panel.is-open")).toBeVisible();
    await expect(page.locator(".kernel-console.is-collapsed")).toBeVisible();

    await consoleButton.click();
    await expect(page.locator(".kernel-console.is-open")).toBeVisible();
    await expect(page.locator(".inspector-panel.is-collapsed")).toBeVisible();
  });
});
