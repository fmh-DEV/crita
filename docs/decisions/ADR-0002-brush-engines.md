# ADR-0002: Brush engines — um port fiel por paint op do Krita
- Date: 2026-11-26 | Status: Accepted

## Context
A primeira versão do brush engine misturava Pixel Brush e Color Smudge numa
`PixelBrush` híbrida. Isso quebra a compatibilidade com .kpp (ADR-0001, item 8):
presets do Krita carregam um `paintopid` (`defaultbrush`, `colorsmudge`,
`mypaintbrush`...) e cada paint op tem pipeline e parâmetros próprios — uma
engine híbrida não reproduz fielmente nenhum dos dois. Além disso, o editor de
brush do Krita mostra que só o Pixel Engine tem ~20 opções, e cada uma é uma
"curve option" (KisPaintOpOption/KisCurveOption): valor base modulado por
sensores (pressão, tilt, velocidade...) via curva editável.

## Decision
1. Uma engine por paint op do Krita, nunca misturadas. Cada uma é identificada
   pelo MESMO `paintopid` do .kpp e registrada em `paintops/mod.rs`
   (`PaintOpId`). `from_krita_id` devolve `None` para paint ops ainda não
   portadas — preset de engine ausente falha explicitamente, nunca carrega
   como engine errada.
2. Ordem de ports (um por vez, cada um com golden test próprio):
   pixel (portado, núcleo) → colorsmudge (stub WIP; falta modos
   dulling/smearing, SmudgeLength/ColorRate, smudge radius) → mypaint →
   sketch → spray...
3. Roadmap de parâmetros do Pixel Engine, em etapas:
   a. `PenSample` + motor de curvas de sensor (o backbone; até o input real
      chegar — pendente do spike de GUI, ADR-0001 — as curvas avaliam com
      pressure constante 1.0)
   b. Size/Opacity/Flow com curva (trio que todo preset usa)
   c. Geometria do dab: Ratio, Softness (fade H/V), Rotation/Angle, Mirror
   d. Scatter, Sharpness, Airbrush/Rate
   e. Color dynamics (Source/Darken/Mix/HSV), Texture, Masked Brush
4. Parâmetros do painel "Brush Tip" do Krita pertencem ao KisBrush/KisAutoBrush,
   não ao paint op: evoluem o `AutoBrushMask` em separado (Mask Type
   Default/Soft/Gaussian/predefinida, Shape círculo/polígono com spikes,
   randomness, density, anti-alias).
5. Nomes de campos dos params espelham os nomes dos campos do .kpp, para o
   mapeamento preset→engine ser direto e auditável.
6. Os ports rodam na CPU (como no Krita), nunca na GPU. Ver a evidência abaixo.

## Por que CPU e não GPU (evidência, 2026-11-26)

- Todo o ecossistema open source é CPU: Krita (todas as paint ops, multithread
  + SIMD/Vc; OpenGL só no canvas display), GIMP/GEGL e MyPaint. GPU brush no
  Krita é feature request aberta desde 2024, adiada pelos devs (a prioridade da
  comunidade ficou em Transform Masks e Liquify), nunca implementada.
- O único contra-exemplo open source é o Capy Canvas (engine brush 100% GPU em
  wgpu, sem fallback CPU). Análise do clone em fa0fe07 (v1.1.0): brushes
  molhados caem atrás da caneta até nos tablets de referência deles
  (1.0–2.6 updates/s no low tier; 7.7–33 updates/s para wet/blenders no top
  tier), e o port macOS segue "unqualified", com engasgos de 33–44 ms num
  M2 Pro — gargalos de compilação de shader, scheduling e apresentação, não do
  modelo CPU vs. GPU. A GPU move o limite (estado úmido residente), não o
  elimina.
- Custo e fidelidade: engine GPU exigiria redesign de cada paintop como
  shaders (o Capy tem ~170 WGSL) e produziria pixels diferentes do Krita,
  quebrando golden tests e a promessa .kpp deste ADR.
- Decisão mantida: ports CPU fiéis. A abstração de acesso a pixels
  (tiles/dirty-rect) deve permitir, no futuro, um colorsmudge GPU nativo
  (.cbrush) como engine NOVA — com o port CPU servindo de oracle —, nunca como
  substituta do port.
- Mistura espectral (ADR-0005) NÃO exige engine GPU: MyPaint 2.0 (2020) entrega
  spectral pigment blending 100% CPU em produção, e a MR K-M abandonada do
  Krita também era CPU/XSIMD. O caminho é LUT 3D por traço + bandas reduzidas +
  multithread; offload para compute shader é otimização posterior opcional.

## Consequences
- Compatibilidade com .kpp preservada por engine; o trade-off aceito é cobrir
  menos features no início (fidelidade > amplitude).
- A infra de curvas/sensores é o caminho crítico: sem ela, parâmetros fixos
  não reproduzem preset real nenhum (size→pressure é o caso dominante).
- Curvas dependem de PenSample; input de pressão real continua bloqueado na
  decisão de GUI (Slint não expõe pressão — ver spike do ADR-0001).
- O stub colorsmudge NÃO é referência de compatibilidade até o port fiel.
