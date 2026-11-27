# ADR-0005: Mistura de cores — Oklab padrão + modo espectral (Kubelka-Munk)
- Date: 2026-11-26 | Status: Proposed

## Context

O colorsmudge (ADR-0002, stub WIP) precisa de um modelo de mistura de cor. O
estado da arte:

- **RGB/sRGB gama**: mistura "suja" — azul + amarelo → cinza/neutro, não verde.
  É o que o Krita faz hoje; a MR de mistura Kubelka-Munk no smudge brush foi
  abandonada sem merge.
- **Oklab** (modelo atual do Capy Canvas, default dos brushes de mistura deles):
  interpolação perceptualmente uniforme, barata (3 componentes + matriz), sem a
  faixa suja do sRGB. Mas é modelo de luz, não de pigmento — azul + amarelo
  continua dessaturado.
- **Perceptual do CSP 2.0+**: proprietário (CELSYS não publica), imita mídia
  tradicional. Referência de UX: o seletor Standard vs. Perceptual e o knob de
  comportamento de mídia expostos ao artista.
- **Espectral (Kubelka-Munk)**: mistura de pigmento real — azul + amarelo =
  verde, complementares escurecem. A referência open source é o spectral.js
  (https://github.com/rvanwijnen/spectral.js, MIT, Ronald van Wijnen):
  upsampling RGB → refletância de 38 bandas (380–750 nm) via 7 curvas base
  (W, C, M, Y, R, G, B; método de Scott Allen Burns), mistura no espaço K/S
  (single-constant K-M), retorno via CIE 1931 CMF × D65 → XYZ → sRGB.
  Licença MIT e teoria de 1931 — sem o problema de patente/licença do Mixbox.

## Decision

1. **Dois modos de mistura** nas engines que misturam tinta (colorsmudge):
   **Oklab** como default (barato, adequado a blending geral em tempo real) e
   **Espectral** como modo "mídia realista" (espelha o Standard vs. Perceptual
   do CSP). A escolha fica no preset, como no Capy Canvas.
2. **Implementação espectral baseada no modelo do spectral.js**, portada para
   Rust. Constantes a portar: as 7 curvas de refletância base (38 amostras
   cada), as CMFs CIE 1931 ponderadas por D65, e o par de fórmulas
   `K/S = (1−R)² / 2R` e inversa `R = 1 + K/S − sqrt(K/S² + 2·K/S)`.
   A concentração efetiva por cor segue `C = f² · T² · L` (fator, tinting
   strength, luminância) — heurística do spectral.js, ajustável por preset.
3. **Divisão CPU × pixel**: o upsampling RGB → K/S de cada cor do pincel roda
   na CPU, memoizado por cor. Por pixel: soma ponderada dos K/S, inversa de
   K-M e conversão de volta a sRGB (f32 é suficiente, ΔE < 1).
4. **Caminho rápido via LUT 3D** (mesma estratégia do Mixbox): uma textura 3D
   32³–64³ mapeia RGB → coeficientes espectrais (K/S por banda), construída uma
   vez na inicialização (~260 mil entradas para 64³, ≈1 MB em f32). A mistura
   acontece DEPOIS das duas lookups, como soma ponderada linear no espaço
   latente — o hardware faz a interpolação trilinear de graça. A LUT elimina o
   upsampling por pixel, não a mistura. Trade-off: LUT assume modelo fixo;
   tinting strength por pigmento exige dimensões extras ou recálculo.
5. **Escopo do modelo espectral**: opera sobre straight color. Alpha, cobertura
   e composição seguem o pipeline normal da engine — K-M não entra no blending
   de camadas nem no cálculo de cobertura.
6. **Modelo por pixel**: cada pixel guarda 1 cor; um stamp mistura no máximo
   tinta(s) do pincel + cor do pixel (2 cores na maioria dos casos, 4 no teto
   com "pincel sujo" de 3 pigmentos). Mais pigmentos: misturar a paleta antes
   (vira 1 cor no pincel) e depois com a tela. Custo por pixel é constante,
   independente da variedade de cores sob o stamp.

## Consequences

- (+) Diferencial real sobre Krita (sem mistura de pigmento) e Capy Canvas
  (só Oklab): até onde sabemos, seria o primeiro app open source com mistura
  de pigmento integrada ao motor de pintura.
- (+) MIT, teoria K-M é ciência de 1931: sem restrições de licença ou patente
  (diferente do Mixbox).
- (+) Não exige engine GPU: MyPaint 2.0 (2020) já entrega spectral pigment
  blending 100% CPU em produção (libmypaint, OpenMP), e a MR K-M abandonada do
  Krita era CPU/XSIMD. LUT por traço, bandas reduzidas e multithread de dabs
  mantêm o modo espectral viável na CPU; offload para compute shader é
  otimização posterior opcional, nunca requisito (ver ADR-0002).
- (−) Custo por mistura ~20–50× um lerp Oklab; mitigado por memoização de
  cores do pincel e pela LUT, mas o modo espectral será sempre mais pesado —
  por isso Oklab continua default.
- (−) As curvas base são calibradas para sRGB. Wide gamut/HDR (e o futuro
  CMYK via lcms2 para export) exigem regenerar as curvas base pelo método do
  Burns para cada espaço.
- (−) Escopo desta fase: o colorsmudge começa com Oklab (port fiel do Krita,
  ADR-0002). O modo espectral é evolução posterior, não bloqueia o port.

## Referências

- spectral.js: https://github.com/rvanwijnen/spectral.js (demo: https://spectraljs.com/)
- CSP Perceptual mixing: https://tips.clip-studio.com/en-us/articles/8216
- Kubelka & Munk (1931): absorção/espalhamento em camadas de pigmento.
- Scott Allen Burns: "Generating Reflectance Curves from sRGB Triplets".
- Mixbox (referência da estratégia LUT): https://scrtwpns.com/mixbox/
- MyPaint 2.0 (precedente de spectral blending em CPU):
  https://github.com/mypaint/mypaint/blob/master/lib/blending.hpp
