# ADR-0001: Foundation decisions
- Date: 2026-11-25 | Status: Accepted (GUI provisional, pending spike)

## Context
App de quadrinhos/manga estilo CSP com brush engine estilo Krita, desktop + tablet,
roteiro integrado. Protótipo brush-core validou a matemática dos brushes.

## Decision
1. Rust + GPL-3.0 (herdado da derivação do Krita)
2. Separação núcleo/GUI (modelo PhotoCraft) via command registry tipado
3. Render: wgpu + oráculo CPU; engine de brush na CPU
4. Contrato de performance: &lt;30 ms em telas 120-160 Hz, dirty rect desde o dia 1
5. GUI: Slint candidata — PROVISÓRIA até spike no S10 Ultra; fallback egui; Qt rejeitado
6. Docking custom (dockable/floating/hidden + snap), estado no núcleo
7. Input normalizado em PenSample neutro por plataforma
8. Presets .cbrush nativo + leitura direta de .kpp (sem importação)
9. RGB agora, CMYK/lcms2 na exportação pra impressão

## Consequences
- Spike é caminho crítico; docking é o maior componente custom; licença viral nas duas direções