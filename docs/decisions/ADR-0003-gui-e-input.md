# ADR-0003: GUI desktop egui + camada nativa de input de caneta
- Date: 2026-11-26 | Status: Accepted (substitui o item 5 do ADR-0001)
- Rev. 2026-11-27: acrescentada a seção "Referências complementares"
  (precedentes fora do domínio de desenho). Nenhuma decisão alterada.
- Rev. 2026-11-27 (2): item 2 ALTERADO — mobile deixa de ser "adiado" e vira
  alvo de arquitetura no mesmo shell egui/eframe (ver item 2).

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
2. Mobile (Android/iPad) é ALVO DE ARQUITETURA desde já, no MESMO shell
   egui/eframe: UI densa estilo CSP Studio Mode, sem toolkit separado
   (Flutter/nativo descartados). Viabilidade verificada: eframe roda em
   Android (backends android-game-activity/android-native-activity do
   egui-winit) e iOS (via winit); wgpu cobre Vulkan/GLES no Android e Metal
   no iPad; pressão básica de stylus já chega no Android via `force` do
   evento de toque (egui discussion #8666) — tilt/rotation/eraser ficam para
   o crates/input (backends MotionEvent e UITouch, já no mapa do item 3).
   Regime: **tablet-compatible desde já** (restrições de design: nenhuma
   interação que exija hover ou botão direito sem alternativa; alvos de
   toque generosos), **tablet-shipping depois** — bring-up Android-first
   (Tab S10 Ultra disponível para teste; sideload sem fricção) num spike do
   playground logo após o compositor desktop existir; iPad depois. O core
   segue sem dependência de toolkit (golden rule 1), então o timing mobile
   não bloqueia as fases atuais (engines, curvas, .kpp — tudo independente
   de plataforma).
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
- Flutter deixa de ser candidato: o shell mobile é o próprio egui/eframe,
  sem retrabalho de UI e sem segunda base de código.
- O shell egui carrega restrições de design tablet-compatible desde já (sem
  interações hover-only/right-click-only; alvos de toque generosos), mesmo
  antes do bring-up mobile.
- Bring-up mobile adiciona dois backends ao crates/input (MotionEvent,
  UITouch) e trabalho de app shell por plataforma (teclado virtual/IME, safe
  areas, seletor de arquivos/SAF, ciclo de vida, lojas) — por isso fica para
  depois do compositor desktop, não por dúvida técnica.
- Compatibilidade .kpp inalterada (ADR-0002); o motor de curvas do ADR-0002
  passa a consumir PenSample enriquecido (velocity/direction inclusos).

## Referências complementares (fora do domínio de desenho)

Precedentes de outros domínios (game engines, jogos de ritmo, drivers,
toolkits mobile) que reforçam esta decisão e detalham a implementação do
`crates/input`. Origem: crita-architecture_ideas.md §6.

| Precedente | O que valida |
|---|---|
| Godot DisplayServer | Abstração por plataforma com um backend por SO (a forma do `crates/input`). No Windows mantém DOIS drivers de caneta — `winink` (default) e `wintab` — selecionáveis pelo usuário (docs.godotengine.org/en/stable/classes/class_displayserver.html), porque os drivers de tablet deixam o usuário escolher qual API alimentar (caso real: godotengine/godot#102740). Lição: "WinTab opcional" do item 3 é, na prática, dois backends com toggle — requisito de produto. |
| Unity Input System (`Pen`) | Superfície de API de caneta unificada mais completa em produção: position, tilt, pressure, twist, eraser, `inRange` (hover), botões de barril, radius. Valida o núcleo do `PenSample` e aponta campos futuros: proximidade/hover (cursor outline, preview de stroke) e botões de barril. As lacunas dela (Linux/Web/iOS fracos) confirmam: nem uma engine comercial resolve caneta sem camada nativa. |
| OpenTabletDriver + osu! lazer | Plano B quando o SO não expõe o tablet: leitura HID direta em user-space, precedente de driver embutido no engine (ppy/osu#39207). O banco público de configs do OTD (github.com/OpenTabletDriver/OpenTabletDriver) cataloga quirks por modelo — insumo para o tablet debugger do item 5. Não é o caminho default. |
| compose-stylus (Kotlin MP) | Confirmação independente, de outro ecossistema, do mapa plataforma→API: Cocoa / X11+XInput2 / Windows RTS via JNI no desktop, MotionEvent (Android), UITouch (iOS), PointerEvent (web) — github.com/MohamedRejeb/compose-stylus. Cobre os dois alvos mobile adiados no item 2 (MotionEvent = caminho nativo mais limpo para Android-first). |
