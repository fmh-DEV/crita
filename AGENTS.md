# AGENTS.md — Crita

Comics/manga drawing & painting app (Krita-style brush engine + Clip Studio
Paint-style production workflow). Rust, GPL-3.0-or-later. Name: Crita
(Krita with a C for comics).

## Golden rules

1. Core crates (brush engine, document, commands, script, workspace) MUST NOT
   depend on GUI/toolkit crates (egui, slint, winit, wgpu). GUI depends on
   core, never the reverse. Enforced by crate structure, not discipline.
2. Brush engine runs on CPU by nature (sequential read-modify-write per dab).
   GPU (wgpu) is for the compositor only. Two compositors are planned:
   CPU reference (oracle) + wgpu, tested against each other.
3. Latency is a hard requirement: pen-to-pixel must stay under ~30 ms on
   120-160 Hz displays. Every render/input change is evaluated against this.
   Dirty-rect upload is mandatory; no full-frame copies per frame.
4. Never run `cargo` with project, CARGO_HOME or RUSTUP_HOME on different
   volumes (external SSD is exFAT: no symlinks, no cross-volume hardlinks).
5. Code ported from Krita keeps KDE/Krita copyright headers and provenance
   comments. Everything stays GPL-3.0-or-later.
6. One task per request. Small diffs, reviewed by the human before applying.

## Status: phase 0 (design)

Do NOT scaffold crates, do NOT add dependencies, do NOT restructure the repo
without explicit instruction. Architecture is still being decided. Open
questions live in docs/decisions/ (ADRs).

## Environment (dev machine)

- macOS (Apple Silicon), bash 5, Ghostty.
- Repo and toolchain on external SSD /Volumes/PortableSSD (exFAT):
  repo: /Volumes/PortableSSD/code/crita
  CARGO_HOME=/Volumes/PortableSSD/rust/cargo
  RUSTUP_HOME=/Volumes/PortableSSD/rust/rustup
- Global gitignore already excludes ._* and .DS_Store (macOS metadata on
  exFAT). Never commit those.

## Decided (do not relitigate without new data)

- License: GPL-3.0-or-later (imports/port from Krita).
- Compositor: wgpu (confirmed), with CPU oracle for testing.
- GUI: Slint is the candidate — PENDING hardware spike on Android
  (Samsung S10 Ultra): stylus pressure via init_with_event_listener probe,
  input latency, require_wgpu_30 support. Fallback: egui/egui_stack
  (desktop-first). Qt rejected.
- Docking: custom dock manager (3 states per panel: docked/floating/hidden,
  edge snap), modeled after CSP Studio Mode. Dock state lives in the core
  (workspace crate) as data + typed commands, not in GUI code.
- Brush presets: native format (.cbrush) + direct reading of Krita .kpp
  (no import step).
- Colors: RGB pipeline now; CMYK via lcms2 planned for print export.
- Input abstraction: all pen input normalizes to a platform-neutral
  PenSample { pos, pressure, tilt, rotation, tip_type } in the core.

## Conventions

- Rust edition 2021, rustfmt default, clippy clean.
- Comments and docs in English (OSS project).
- ADRs in docs/decisions/ADR-NNNN-title.md (short: context, decision,
  consequences).
- Commit style: conventional commits (feat:, fix:, docs:, chore:).