# LandXML Support Matrix

This matrix summarizes currently supported LandXML capabilities in the kernel.

## Parsing

- LandXML 1.2 parsing: supported
- Strict mode: supported
- Lenient mode with warnings: supported
- Point order options: NEZ / ENZ / EZN supported
- Units policy: normalize-to-meters / preserve-source supported

## Alignment Features

- Horizontal line/arc/spiral segments: supported
- Station equations: supported
- Profile association and profile sampling: supported
- 3D alignment sampling (alignment + profile): supported

## Terrain/TIN Features

- Surface count/name queries: supported
- Raw vertex/index extraction: supported
- Surface extraction to kernel mesh handle: supported

## Warnings and Metadata

- Parse warning count: supported
- Source linear unit retrieval: supported

## Known Limits

- Some parser internals remain monolithic and are scheduled for future splits.
- Additional LandXML feature classes should be added with benchmark + contract tests.
