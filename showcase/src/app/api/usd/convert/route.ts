import { execFile } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

export const runtime = "nodejs";

export async function POST(request: Request): Promise<Response> {
  const usda = await request.text();
  if (!usda.trim()) {
    return new Response("Missing USDA payload", { status: 400 });
  }

  const tempDir = await mkdtemp(join(tmpdir(), "rusted-geom-usdc-"));
  const inputPath = join(tempDir, "stage.usda");
  const outputPath = join(tempDir, "stage.usdc");

  try {
    await writeFile(inputPath, usda, "utf8");
    await execFileAsync("usdcat", [inputPath, "-o", outputPath, "--usdFormat", "usdc"]);
    const bytes = await readFile(outputPath);
    return new Response(bytes, {
      status: 200,
      headers: {
        "content-type": "application/octet-stream",
        "content-disposition": 'attachment; filename="export.usdc"',
      },
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return new Response(`USDC conversion failed: ${message}`, { status: 500 });
  } finally {
    await rm(tempDir, { recursive: true, force: true });
  }
}
