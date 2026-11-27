# ADR-0006: Engines de brush plugáveis — registry estático multi-ecossistema
- Date: 2026-11-26 | Status: Accepted

## Context
O ADR-0002 decidiu um port fiel por paint op do Krita, com dispatch por enums
centrais (`PaintOpId` em paintops, `BrushParams` em preset). Dois fatos novos:

1. O roadmap passa a incluir engines de OUTROS ecossistemas. O caso motivador
   é o brush engine do PhotoCraft (Apache-2.0), que caminha para ser
   totalmente compatível com Photoshop: um usuário de Photoshop usaria seus
   `.abr` sem conversão. O mesmo vale para uma eventual engine CSP, se algum
   dia surgir uma implementação open source.
2. O dispatch por enums exige tocar pontos centrais (dois enums + vários
   `match`) para cada engine nova — acoplamento que escala mal com o roadmap
   de ports (mypaint, sketch, spray...) e com engines externas.

Plugins dinâmicos (.so/.dylib, modelo dos plugins Qt do Krita) foram
reavaliados e seguem rejeitados: Rust não tem ABI estável — exigiria cdylib
com FFI em C ou crates como abi_stable/stabby — e todas as engines previstas
são first-party ou crates Rust linkadas estaticamente (PhotoCraft é
Apache-2.0, compatível com GPL-3.0-or-later nessa direção). O custo e a
insegurança de ABI não compram nada aqui.

## Decision
1. Registry estático de factories: `PaintOpFactory { id, name, create }` em
   `paintops`. Cada módulo de engine exporta sua `FACTORY`; o slice
   `registry()` é o único ponto de registro — adicionar engine = 1 módulo
   novo + 1 entrada no slice. `id` é o identificador nativo do ecossistema
   (nos presets .kpp, o `paintopid`: "defaultbrush", "colorsmudge"...).
   Engine desconhecida falha explicitamente (`factory_for` → None), nunca
   carrega como outra engine (mantém o ADR-0002).
2. Fim dos enums centrais de dispatch (`PaintOpId`, `BrushParams`): engines
   se identificam por string e os parâmetros ficam tipados POR engine,
   expostos no módulo da engine. O editor de brush, como no Krita, terá
   widgets específicos por engine — configuração tipada é responsabilidade
   da camada que conhece a engine, não do core.
3. Engines externas entram como crates ADAPTADORES (ex.: futuro
   `crates/paintops-photocraft` embrulhando o engine do PhotoCraft) que
   implementam `PaintOp`/`Surface` do core, isolando a instabilidade da API
   externa. Um crate por família, criado com sua primeira implementação
   (ADR-0004) — não antes. A dependência do PhotoCraft só entra quando a API
   deles estabilizar.
4. Cada família de engine lê seu formato de preset nativo: engines Krita →
   `.kpp`; engine Photoshop → `.abr`. O preset do core vira `paintop_id` +
   settings, e o roteamento arquivo→engine é por formato. O usuário nunca
   "converte" preset: carrega o arquivo original na engine certa.
5. O trait `PaintOp` recebe `&PenSample` (ADR-0003) em vez de
   `(x, y, pressure)`: engines de outros ecossistemas usam tilt/rotation
   (ex.: dual brush e angle jitter do Photoshop). Mudar a assinatura agora
   (2 engines, 1 stub) é barato; depois da integração externa, seria breaking
   change ampla.
6. A UI lista engines a partir do registry (`id` + `name`), sem conhecer
   engines concretas. Indicação de compatibilidade parcial (ex.: .abr com
   features não implementadas) fica para quando houver engine externa real.

## Consequences
- Adicionar engine = 1 módulo + 1 entrada no registry; nenhum enum central.
- Fidelidade cross-ecossistema não vem de graça: um `.abr` com features não
  implementadas no engine PhotoCraft renderiza diferente do Photoshop — a UI
  deve ser honesta quanto a compatibilidade parcial.
- A assinatura de `create` (hoje sem parâmetros) passa a receber as settings
  do preset quando a leitura de .kpp/.cbrush chegar; é a próxima evolução
  prevista do seam, junto com o motor de curvas do ADR-0002.
- O seam (registry + PenSample) fica pronto e testado com as engines Krita
  antes de qualquer integração externa — o adaptador PhotoCraft nasce sobre
  uma interface já exercitada.
