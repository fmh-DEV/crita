# Crita

A drawing and painting application for comics and manga, written in Rust.

The name says it: **Krita with a C, for comics.** Crita combines the
powerful, open brush engine philosophy of [Krita](https://krita.org) with
the comic-focused production workflow of
[Clip Studio Paint](https://www.clipstudio.net) — panel layouts,
screentones, and page management — built from the ground up as free and
open source software.

## Status

Crita is in early design phase. The architecture is being decided through
ADRs ([Architecture Decision Records](docs/decisions/)), and the roadmap
below reflects the planned phases rather than shipped features.

## Roadmap

Development is organized in phases:

1. **Brush engine** — a CPU-based raster brush engine, Krita-style, with a
   native preset format plus direct reading of Krita `.kpp` presets
2. **Canvas + GUI** — a low-latency drawing surface and application
   interface, targeting pen displays and pen tablets
3. **Panels & screentone** — comic-specific production tools: panel
   creation, screentone layers, and page management
4. **Integrated scripting** — write your script alongside your pages, tied
   to panels and dialogue

## Design principles

- **Core/GUI separation** — the brush engine, document model, and workspace
  logic never depend on GUI crates; the GUI depends on the core, never the
  reverse.
- **Latency is a hard requirement** — pen-to-pixel must stay under ~30 ms on
  high-refresh displays.
- **Free software first** — ported Krita code keeps its copyright headers
  and provenance; everything stays GPL.

## Contributing

Crita is a community-driven project and contributions are welcome — code,
bug reports, feature ideas, or documentation. Feel free to open an issue or
a pull request.

## License

Crita is free software, licensed under the
[GNU General Public License v3.0 or later](LICENSE).
