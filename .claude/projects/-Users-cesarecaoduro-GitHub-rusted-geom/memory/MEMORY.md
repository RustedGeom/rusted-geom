# rusted-geom Project Memory

## Showcase Architecture (post Feb 2026 redesign)

### File Structure
- `showcase/src/components/kernel-viewer.tsx` — coordinator component (~4076 lines)
- `showcase/src/lib/viewer-types.ts` — shared type definitions (LogLevel, GizmoMode, ExampleKey, etc.)
- `showcase/src/lib/examples.ts` — EXAMPLE_OPTIONS, EXAMPLE_SUMMARIES, EXAMPLE_CATEGORIES, helper fns
- `showcase/src/lib/use-theme.ts` — dark/light theme hook (localStorage "rgm-theme", data-theme attr)
- `showcase/src/lib/use-keyboard-shortcut.ts` — keyboard shortcut binding hook (stable ref pattern)
- `showcase/src/components/viewer/toolbar/ViewerToolbar.tsx` — toolbar JSX
- `showcase/src/components/viewer/toolbar/ToolIcon.tsx` — SVG icon wrapper
- `showcase/src/components/ui/ToolButton.tsx` — reusable button wrapper
- `showcase/src/components/viewer/inspector/InspectorPanel.tsx` — inspector shell
- `showcase/src/components/viewer/inspector/ExampleSection.tsx` — example trigger button
- `showcase/src/components/viewer/inspector/GizmoSection.tsx`
- `showcase/src/components/viewer/inspector/ProbeSection.tsx`
- `showcase/src/components/viewer/inspector/SurfaceProbeSection.tsx`
- `showcase/src/components/viewer/inspector/PerformanceSection.tsx`
- `showcase/src/components/viewer/console/KernelConsole.tsx` — console with filter
- `showcase/src/components/viewer/console/ConsoleLogEntry.tsx` — log row with copy-on-click
- `showcase/src/components/viewer/ExampleBrowser.tsx` — command palette modal (Cmd+K)

### Known Pre-existing Build Failures
- `pnpm build` in showcase/ fails with 6 TypeScript errors about KernelSession methods
  (surfacePointAt, surfaceFrameAt, pointAt, curveLengthAt, handle, releaseObject)
- These are pre-existing and related to Rust crate changes not yet reflected in TS types

### Dark Mode
- CSS vars: `:root, [data-theme="light"]` and `[data-theme="dark"]`
- Three.js scene background reads `--viewport-bg` via `getComputedStyle`
- Theme persisted in localStorage("rgm-theme"), falls back to prefers-color-scheme

### Keyboard Shortcuts (showcase)
- Cmd+K: Open example browser
- Escape: Close example browser
- G: Toggle grid, A: Toggle axes, I: Toggle inspector, C: Toggle console
