# RustedGeom.Bindings

Generated .NET bindings for the RustedGeom CAD kernel.

## Pack

```bash
./scripts/pack_dotnet.sh osx-arm64
```

NuGet artifacts are emitted to `dist/nuget`.

## Native library

The package expects a native library named `rusted_geom` loaded through `LibraryImport`.

Packaged native assets are placed under:

- `runtimes/osx-arm64/native/librusted_geom.dylib`
- `runtimes/win-x64/native/rusted_geom.dll`
- `runtimes/linux-x64/native/librusted_geom.so`

## Usage

```csharp
using RustedGeom.Generated;

using var kernel = KernelHandle.Create();

var curve = kernel.InterpolateNurbsFitPoints(
  points,
  degree: 3,
  closed: false,
  new RgmToleranceContext { AbsTol = 1e-6, RelTol = 1e-6, AngleTol = 1e-6 }
);

var p = curve.PointAt(0.5);
var t = curve.TangentAt(0.5);
```
