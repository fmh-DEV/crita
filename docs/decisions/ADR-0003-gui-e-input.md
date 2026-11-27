# ADR-0003: GUI desktop egui + camada nativa de input de caneta
- Date: 2026-11-26 | Status: Accepted (substitui o item 5 do ADR-0001)

## Context
Verificado no código do Slint 1.19.1: `PointerEvent` não tem pressão e o
backend winit não trata eventos de tablet — o spike no S10 Ultra (ADR-0001)
já nascia contornando uma limitação estrutural. A causa é mais funda: o winit
não tem eventos de tablet em nenhuma plataforma, então NENHUM toolkit Rust
(egui incluído) entrega input de caneta — a camada nativa é necessária de
qualquer jeito, e a escolha do toolkit passa a ser por produtividade do shell.

Referências analisadas:
- **Krita**: o Qt faz o trabalho nativo (WinTab/Windows Ink, XInput2,
  zwp_tablet_v2, AppKit) e o Krita mantém patches próprios NO Qt
  (libs/ui/input/on-input-handling-in-qt.md). Abstrações: KoPointerEvent →
  KisPaintInformation (tilt -60..60°). Compressão de move events por
  ferramenta + tabletLatencyTracker (latência medida explicitamente) e
  KisTabletDebugger como infra de depuração.
- **GIMP**: o GDK faz o nativo; o GIMP só escolhe o backend no Windows
  (env GDK_WIN32_TABLET_INPUT_API=wintab|winpointer). GimpCoords tem eixos
  sintéticos (velocity, direction) computados na camada de input e tilt
  normalizado -1..1; dynamics matrix + curva de pressão por dispositivo.
- **PhotoCraft** (modelo já citado no ADR-0001): egui + crate nativo próprio
  (photocraft-tablet, Apache-2.0) AO LADO do winit: macOS AppKit monitor
  (objc2), Linux X11 XInput2 (x11rb, sem unsafe), Windows hooks WM_POINTER +
  WinTab opcional; StylusFeed desacoplado; doc hardware-parity mostra que
  input de caneta é desigual por plataforma (estimativa deles: 80-150 h).
- **Flutter**: resolve de verdade em Android/iPad, mas desktop não (macOS
  #146387, Windows #65248 sem pressão); mesmo o ecossistema Flutter precisou
  de camada nativa lateral (pacote stylet, MIT — com referências úteis:
  Wayland nativo zwp_tablet_v2 e predição de ponteiro no Windows).

## Decision
1. GUI desktop: **egui** (eframe + wgpu), desktop-first. Spike do Slint
   cancelado (sem suporte a pressão de stylus). Qt segue rejeitado.
2. Front-end mobile (Android/iPad) fica ADIADO; candidatos: Flutter, nativo
   (Kotlin/Swift) ou egui-mobile. O core continua sem dependência de toolkit
   (golden rule 1), então a decisão mobile não bloqueia nada.
3. Input de caneta: camada nativa própria num crate `crates/input` (fora do
   core), modelada no photocraft-tablet (Apache-2.0 — reuso/port com headers
   de proveniência, golden rule 5): macOS AppKit monitor (objc2), Linux X11
   XInput2 (x11rb, sem unsafe), Windows hooks WM_POINTER + WinTab opcional.
   Wayland nativo (zwp_tablet_v2) fica para depois (referência: stylet, MIT);
   até lá, fallback Xwayland. Predição de ponteiro no Windows é trabalho
   futuro de latência (referência: stylet). unsafe isolado no crate, com
   comentários SAFETY. SEM fork do winit (o modelo Krita de patchar o
   toolkit foi rejeitado pelo custo de manutenção).
4. `PenSample` no core: posição vem da camada de janelas; o feed nativo
   fornece pressure/tilt/rotation/tip_type (eraser). Convenções: tilt
   normalizado -1..1 (modelo GIMP), convertendo nas fronteiras (.kpp/Krita
   -60..60°; W3C -90..90°); eixos sintéticos (velocity, direction) computados
   na camada de normalização de input, NÃO nas engines (ADR-0002); mouse =
   pressure 1.0; curva global de pressão como preferência, depois.
5. Compressão de eventos de move vira feature explícita por ferramenta
   (modelo Krita), e um logger/debugger de tablet entra cedo como infra.

## Consequences
- Sem fork de winit; todo unsafe de input fica isolado em crates/input.
- A realidade de pressão é por plataforma (Windows Ink sem tilt/rotation;
  Wayland via Xwayland): bring-up plataforma a plataforma, com debugger.
- egui/wgpu no mesmo processo preserva o controle de latência (dirty-rect
  upload) exigido pela golden rule 3.
- Flutter permanece candidato mobile sem retrabalho no core.
- Compatibilidade .kpp inalterada (ADR-0002); o motor de curvas do ADR-0002
  passa a consumir PenSample enriquecido (velocity/direction inclusos).
