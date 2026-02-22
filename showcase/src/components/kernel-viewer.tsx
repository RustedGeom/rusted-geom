"use client";

import {
  createKernelRuntime,
  type CurvePresetInput,
  type KernelRuntime,
  type KernelSession,
} from "@rusted-geom/bindings-web";
import type { RgmPoint3 } from "@rusted-geom/bindings-web";
import { Pane } from "tweakpane";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import {
  parseCurvePreset,
  parseViewerSession,
  type CurvePreset,
  type ViewerSessionFile,
} from "@/lib/preset-schema";

const DEFAULT_CAMERA_POSITION = new THREE.Vector3(10, 8, 11);
const DEFAULT_CAMERA_TARGET = new THREE.Vector3(0, 0, 0);

interface CameraSnapshot {
  position: RgmPoint3;
  target: RgmPoint3;
  up: RgmPoint3;
  fov: number;
}

interface PaneChangeBinding {
  on(event: "change", handler: (event: { value: unknown }) => void): void;
}

interface PaneButtonBinding {
  on(event: "click", handler: () => void): void;
}

interface PaneFolderBinding {
  addBinding(target: object, key: string, options?: Record<string, unknown>): PaneChangeBinding;
  addButton(options: { title: string }): PaneButtonBinding;
}

interface PaneLike {
  addFolder(options: { title: string }): PaneFolderBinding;
  dispose(): void;
}

function toPoint3(vector: THREE.Vector3): RgmPoint3 {
  return { x: vector.x, y: vector.y, z: vector.z };
}

function fromPoint3(point: RgmPoint3): THREE.Vector3 {
  return new THREE.Vector3(point.x, point.y, point.z);
}

function downloadJson(filename: string, payload: unknown): void {
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function downloadDataUrl(filename: string, dataUrl: string): void {
  const anchor = document.createElement("a");
  anchor.href = dataUrl;
  anchor.download = filename;
  anchor.click();
}

export function KernelViewer() {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const paneHostRef = useRef<HTMLDivElement | null>(null);
  const sessionFileInputRef = useRef<HTMLInputElement | null>(null);

  const runtimeRef = useRef<KernelRuntime | null>(null);
  const sessionRef = useRef<KernelSession | null>(null);
  const curveHandleRef = useRef<bigint | null>(null);

  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const lineRef = useRef<THREE.Line<THREE.BufferGeometry, THREE.LineBasicMaterial> | null>(null);
  const gridRef = useRef<THREE.GridHelper | null>(null);
  const axesRef = useRef<THREE.AxesHelper | null>(null);

  const [preset, setPreset] = useState<CurvePreset | null>(null);
  const [sampledPoints, setSampledPoints] = useState<RgmPoint3[]>([]);
  const [statusMessage, setStatusMessage] = useState("Booting kernel runtime...");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [capabilities, setCapabilities] = useState({ igesImport: false, igesExport: false });
  const [showGrid, setShowGrid] = useState(true);
  const [showAxes, setShowAxes] = useState(false);
  const [orbitEnabled, setOrbitEnabled] = useState(true);
  const [mobilePaneOpen, setMobilePaneOpen] = useState(false);

  const updateCurve = useCallback(
    (nextPreset: CurvePreset, successMessage: string): void => {
      const session = sessionRef.current;
      if (!session) {
        throw new Error("Kernel session is not ready");
      }

      if (curveHandleRef.current !== null) {
        session.releaseObject(curveHandleRef.current);
      }

      const handle = session.buildCurveFromPreset(nextPreset as CurvePresetInput);
      curveHandleRef.current = handle;
      const curveSamples = session.sampleCurvePolyline(handle, nextPreset.sampleCount);

      setPreset(nextPreset);
      setSampledPoints(curveSamples);
      setStatusMessage(successMessage);
      setErrorMessage(null);
    },
    [],
  );

  const loadDefaultPreset = useCallback(async (): Promise<CurvePreset> => {
    const response = await fetch("/showcases/default.json");
    if (!response.ok) {
      throw new Error(`Failed to load default preset (${response.status})`);
    }

    const data = await response.json();
    return parseCurvePreset(data);
  }, []);

  const cameraSnapshot = useCallback((): CameraSnapshot | null => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls) {
      return null;
    }

    return {
      position: toPoint3(camera.position),
      target: toPoint3(controls.target),
      up: toPoint3(camera.up),
      fov: camera.fov,
    };
  }, []);

  const zoomExtents = useCallback((): void => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls || sampledPoints.length === 0) {
      return;
    }

    const bounds = new THREE.Box3();
    sampledPoints.forEach((point) => {
      bounds.expandByPoint(new THREE.Vector3(point.x, point.y, point.z));
    });

    const sphere = bounds.getBoundingSphere(new THREE.Sphere());
    const distance = Math.max(4, sphere.radius * 2.8);
    camera.position.set(
      sphere.center.x + distance,
      sphere.center.y + distance * 0.55,
      sphere.center.z + distance,
    );
    controls.target.copy(sphere.center);
    controls.update();
  }, [sampledPoints]);

  const resetCamera = useCallback((): void => {
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!camera || !controls) {
      return;
    }

    camera.position.copy(DEFAULT_CAMERA_POSITION);
    controls.target.copy(DEFAULT_CAMERA_TARGET);
    camera.up.set(0, 1, 0);
    controls.update();
  }, []);

  const applySession = useCallback(
    (sessionFile: ViewerSessionFile): void => {
      updateCurve(sessionFile.preset, "Session loaded");
      setShowGrid(sessionFile.view.showGrid);
      setShowAxes(sessionFile.view.showAxes);
      setOrbitEnabled(sessionFile.view.orbitEnabled);

      const camera = cameraRef.current;
      const controls = controlsRef.current;
      if (camera && controls) {
        camera.position.copy(fromPoint3(sessionFile.view.camera.position));
        camera.up.copy(fromPoint3(sessionFile.view.camera.up));
        camera.fov = sessionFile.view.camera.fov;
        camera.updateProjectionMatrix();
        controls.target.copy(fromPoint3(sessionFile.view.camera.target));
        controls.update();
      }
    },
    [updateCurve],
  );

  useEffect(() => {
    let disposed = false;

    (async () => {
      try {
        const runtime = await createKernelRuntime("/wasm/kernel_ffi.wasm");
        const session = runtime.createSession();
        const loadedPreset = await loadDefaultPreset();
        if (disposed) {
          session.destroy();
          runtime.destroy();
          return;
        }

        runtimeRef.current = runtime;
        sessionRef.current = session;
        setCapabilities({
          igesImport: runtime.capabilities.igesImport,
          igesExport: runtime.capabilities.igesExport,
        });
        updateCurve(loadedPreset, "Default preset loaded");
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : String(error));
      }
    })();

    return () => {
      disposed = true;
      if (curveHandleRef.current !== null && sessionRef.current) {
        sessionRef.current.releaseObject(curveHandleRef.current);
      }
      sessionRef.current?.destroy();
      runtimeRef.current?.destroy();
      sessionRef.current = null;
      runtimeRef.current = null;
      curveHandleRef.current = null;
    };
  }, [loadDefaultPreset, updateCurve]);

  useEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) {
      return;
    }

    const scene = new THREE.Scene();
    scene.background = new THREE.Color("#0b1220");
    scene.fog = new THREE.Fog("#0b1220", 24, 118);

    const camera = new THREE.PerspectiveCamera(
      46,
      viewport.clientWidth / Math.max(1, viewport.clientHeight),
      0.01,
      1200,
    );
    camera.position.copy(DEFAULT_CAMERA_POSITION);

    let renderer: THREE.WebGLRenderer | null = null;
    let renderCanvas: HTMLCanvasElement | null = null;
    let fallbackContext: CanvasRenderingContext2D | null = null;
    const forceFallback = /HeadlessChrome/i.test(window.navigator.userAgent);
    if (!forceFallback) {
      try {
        renderer = new THREE.WebGLRenderer({
          antialias: true,
          alpha: true,
          preserveDrawingBuffer: true,
        });
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.setSize(viewport.clientWidth, Math.max(1, viewport.clientHeight));
        renderer.outputColorSpace = THREE.SRGBColorSpace;
        renderCanvas = renderer.domElement;
      } catch {
        renderer = null;
      }
    }

    if (!renderCanvas) {
      // Headless CI can lack a usable WebGL context. Keep the UI operational.
      renderCanvas = document.createElement("canvas");
      renderCanvas.className = "viewport-fallback-canvas";
      fallbackContext = renderCanvas.getContext("2d");
    }

    viewport.appendChild(renderCanvas);

    const controls = new OrbitControls(camera, renderCanvas);
    controls.enableDamping = true;
    controls.target.copy(DEFAULT_CAMERA_TARGET);
    controls.update();

    const grid = new THREE.GridHelper(30, 30, "#33415f", "#1d2740");
    grid.material.opacity = 0.5;
    grid.material.transparent = true;
    scene.add(grid);

    const axes = new THREE.AxesHelper(3.5);
    axes.visible = false;
    scene.add(axes);

    const key = new THREE.DirectionalLight("#cfdbff", 0.62);
    key.position.set(3, 10, 7);
    scene.add(key);
    scene.add(new THREE.AmbientLight("#6078ac", 0.45));

    const onResize = (): void => {
      const width = viewport.clientWidth;
      const height = Math.max(1, viewport.clientHeight);
      camera.aspect = width / height;
      camera.updateProjectionMatrix();
      if (renderer) {
        renderer.setSize(width, height);
      } else {
        renderCanvas.width = Math.floor(width * window.devicePixelRatio);
        renderCanvas.height = Math.floor(height * window.devicePixelRatio);
        renderCanvas.style.width = `${width}px`;
        renderCanvas.style.height = `${height}px`;
        if (fallbackContext) {
          fallbackContext.save();
          fallbackContext.scale(window.devicePixelRatio, window.devicePixelRatio);
          fallbackContext.clearRect(0, 0, width, height);
          fallbackContext.fillStyle = "#0b1220";
          fallbackContext.fillRect(0, 0, width, height);
          fallbackContext.fillStyle = "#a7b6d8";
          fallbackContext.font = "600 13px sans-serif";
          fallbackContext.fillText("WebGL unavailable in this environment", 14, 28);
          fallbackContext.restore();
        }
      }
    };

    const resizeObserver = new ResizeObserver(onResize);
    resizeObserver.observe(viewport);

    let frame = 0;
    const animate = (): void => {
      frame = window.requestAnimationFrame(animate);
      controls.update();
      if (renderer) {
        renderer.render(scene, camera);
      }
    };
    animate();
    onResize();

    sceneRef.current = scene;
    cameraRef.current = camera;
    controlsRef.current = controls;
    rendererRef.current = renderer;
    gridRef.current = grid;
    axesRef.current = axes;

    return () => {
      window.cancelAnimationFrame(frame);
      resizeObserver.disconnect();
      controls.dispose();
      renderer?.dispose();
      if (lineRef.current) {
        lineRef.current.geometry.dispose();
        lineRef.current.material.dispose();
      }
      if (renderCanvas.parentElement === viewport) {
        viewport.removeChild(renderCanvas);
      }
      scene.clear();
      sceneRef.current = null;
      controlsRef.current = null;
      cameraRef.current = null;
      rendererRef.current = null;
      gridRef.current = null;
      axesRef.current = null;
      lineRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!sceneRef.current) {
      return;
    }

    if (lineRef.current) {
      lineRef.current.geometry.dispose();
      lineRef.current.material.dispose();
      sceneRef.current.remove(lineRef.current);
      lineRef.current = null;
    }

    if (!sampledPoints.length) {
      return;
    }

    const points = sampledPoints.map((point) => new THREE.Vector3(point.x, point.y, point.z));
    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    const material = new THREE.LineBasicMaterial({
      color: "#74a1ff",
      transparent: true,
      opacity: 0.96,
    });
    const line = new THREE.Line(geometry, material);
    lineRef.current = line;
    sceneRef.current.add(line);
  }, [sampledPoints]);

  useEffect(() => {
    if (gridRef.current) {
      gridRef.current.visible = showGrid;
    }
  }, [showGrid]);

  useEffect(() => {
    if (axesRef.current) {
      axesRef.current.visible = showAxes;
    }
  }, [showAxes]);

  useEffect(() => {
    if (controlsRef.current) {
      controlsRef.current.enabled = orbitEnabled;
    }
  }, [orbitEnabled]);

  useEffect(() => {
    if (sampledPoints.length > 0) {
      zoomExtents();
    }
  }, [sampledPoints, zoomExtents]);

  useEffect(() => {
    const paneHost = paneHostRef.current;
    if (!paneHost || !preset) {
      return;
    }

    paneHost.innerHTML = "";
    const pane = new Pane({
      container: paneHost,
      title: "Kernel Lab",
    }) as unknown as PaneLike;

    const kernelState = {
      degree: preset.degree,
      closed: preset.closed,
      sampleCount: preset.sampleCount,
      absTol: preset.tolerance.abs_tol,
      relTol: preset.tolerance.rel_tol,
      angleTol: preset.tolerance.angle_tol,
      points: preset.points.length,
      sampled: sampledPoints.length,
    };

    const kernelFolder = pane.addFolder({ title: "Kernel" });
    kernelFolder.addBinding(kernelState, "degree", { min: 1, max: 7, step: 1 });
    kernelFolder.addBinding(kernelState, "closed");
    kernelFolder.addBinding(kernelState, "sampleCount", { min: 2, max: 512, step: 1 });
    kernelFolder.addBinding(kernelState, "absTol", { min: 1e-12, max: 1e-4 });
    kernelFolder.addBinding(kernelState, "relTol", { min: 1e-12, max: 1e-4 });
    kernelFolder.addBinding(kernelState, "angleTol", { min: 1e-12, max: 1e-4 });
    kernelFolder
      .addButton({ title: "Apply Kernel Params" })
      .on("click", () => {
        const nextPreset: CurvePreset = {
          ...preset,
          degree: Math.max(1, Math.floor(kernelState.degree)),
          closed: kernelState.closed,
          sampleCount: Math.max(2, Math.floor(kernelState.sampleCount)),
          tolerance: {
            abs_tol: kernelState.absTol,
            rel_tol: kernelState.relTol,
            angle_tol: kernelState.angleTol,
          },
        };

        try {
          updateCurve(nextPreset, "Kernel params applied");
          zoomExtents();
        } catch (error) {
          setErrorMessage(error instanceof Error ? error.message : String(error));
        }
      });

    const renderState = {
      showGrid,
      showAxes,
      orbitEnabled,
    };

    const renderFolder = pane.addFolder({ title: "Render" });
    renderFolder
      .addBinding(renderState, "showGrid")
      .on("change", (event: { value: unknown }) => {
      setShowGrid(Boolean(event.value));
      });
    renderFolder
      .addBinding(renderState, "showAxes")
      .on("change", (event: { value: unknown }) => {
      setShowAxes(Boolean(event.value));
      });
    renderFolder
      .addBinding(renderState, "orbitEnabled")
      .on("change", (event: { value: unknown }) => {
      setOrbitEnabled(Boolean(event.value));
      });

    return () => {
      pane.dispose();
    };
  }, [orbitEnabled, preset, sampledPoints.length, showAxes, showGrid, updateCurve, zoomExtents]);

  const canExportIges = useMemo(() => capabilities.igesExport, [capabilities.igesExport]);
  const canImportIges = useMemo(() => capabilities.igesImport, [capabilities.igesImport]);

  const onSaveSession = useCallback(() => {
    if (!preset) {
      return;
    }

    const snapshot = cameraSnapshot();
    if (!snapshot) {
      return;
    }

    const payload: ViewerSessionFile = {
      version: 1,
      preset,
      view: {
        camera: snapshot,
        showGrid,
        showAxes,
        orbitEnabled,
      },
    };

    downloadJson("rusted-geom-session.json", payload);
    setStatusMessage("Session saved");
  }, [cameraSnapshot, orbitEnabled, preset, showAxes, showGrid]);

  const onSaveScreenshot = useCallback(() => {
    const renderer = rendererRef.current;
    if (!renderer) {
      return;
    }

    downloadDataUrl("rusted-geom-view.png", renderer.domElement.toDataURL("image/png"));
    setStatusMessage("PNG snapshot saved");
  }, []);

  const onLoadSessionFile = useCallback(
    async (file: File): Promise<void> => {
      try {
        const text = await file.text();
        const parsed = parseViewerSession(JSON.parse(text));
        applySession(parsed);
      } catch (error) {
        setErrorMessage(error instanceof Error ? error.message : String(error));
      }
    },
    [applySession],
  );

  const onLoadSessionClick = useCallback(() => {
    sessionFileInputRef.current?.click();
  }, []);

  return (
    <div className="viewer-shell">
      <input
        ref={sessionFileInputRef}
        type="file"
        accept="application/json"
        className="hidden-input"
        onChange={(event) => {
          const file = event.target.files?.[0];
          if (file) {
            void onLoadSessionFile(file);
          }
          event.currentTarget.value = "";
        }}
      />

      <header className="toolbar" role="toolbar" aria-label="Viewer actions">
        <div className="toolbar-left">
          <button type="button" className="tool-btn" onClick={onLoadSessionClick}>
            Load Session
          </button>
          <button type="button" className="tool-btn" onClick={onSaveSession}>
            Save Session
          </button>
          <button
            type="button"
            className="tool-btn"
            disabled={!canImportIges}
            title="Kernel IGES import API pending"
          >
            Load IGES
          </button>
          <button
            type="button"
            className="tool-btn"
            disabled={!canExportIges}
            title="Kernel IGES export API pending"
          >
            Save IGES
          </button>
        </div>

        <div className="toolbar-center">
          <button type="button" className="tool-btn" onClick={zoomExtents}>
            Zoom Extents
          </button>
          <button type="button" className="tool-btn" onClick={resetCamera}>
            Reset View
          </button>
          <button
            type="button"
            className={`tool-btn ${orbitEnabled ? "is-active" : ""}`}
            onClick={() => setOrbitEnabled((value) => !value)}
          >
            Orbit
          </button>
          <button
            type="button"
            className={`tool-btn ${showGrid ? "is-active" : ""}`}
            onClick={() => setShowGrid((value) => !value)}
          >
            Grid
          </button>
          <button
            type="button"
            className={`tool-btn ${showAxes ? "is-active" : ""}`}
            onClick={() => setShowAxes((value) => !value)}
          >
            Axes
          </button>
          <button type="button" className="tool-btn" onClick={onSaveScreenshot}>
            Save PNG
          </button>
        </div>

        <div className="toolbar-right">
          <button
            type="button"
            className="tool-btn mobile-pane-toggle"
            onClick={() => setMobilePaneOpen((open) => !open)}
          >
            Params
          </button>
          <div className="status-pill" aria-live="polite">
            {errorMessage ? `Error: ${errorMessage}` : statusMessage}
          </div>
        </div>
      </header>

      <main className="viewer-main">
        <section className="viewport-wrap">
          <div ref={viewportRef} className="viewport" aria-label="Three.js viewport" />
        </section>

        <aside className={`pane-dock ${mobilePaneOpen ? "mobile-open" : ""}`}>
          <div className="pane-header">
            <h2>Kernel Controls</h2>
            <p>Runtime-bound parameter testing with no UI-side geometry generation.</p>
          </div>
          <div ref={paneHostRef} className="pane-host" />
        </aside>
      </main>
    </div>
  );
}
