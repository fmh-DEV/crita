# ADR-0004: Layout do repositório (apps/ + crates/)
- Date: 2026-11-26 | Status: Accepted

## Context
A estrutura do PhotoCraft foi avaliada como referência (legível, um crate por
preocupação, binários separados das bibliotecas). Surgiu a dúvida entre copiar
a estrutura deles de uma vez ou deixar os crates nascerem conforme a
implementação pedir. O domínio deles (clone de Photoshop: io de PSD,
adjustment layers, MCP server, web runner) não mapeia 1:1 no nosso (app de
quadrinhos; mapa de crates rascunhado no ADR-0001: brush engine, document,
commands, script, workspace), e crates scaffoldados vazios tendem a divergir
das necessidades reais e virar ruído.

## Decision
1. Adotamos os PRINCÍPIOS da estrutura do PhotoCraft, não os arquivos:
   - `apps/` para binários, `crates/` para bibliotecas;
   - um crate por preocupação limitada;
   - core sem dependência de GUI (golden rule 1, reafirmada);
   - unsafe isolado por crate (ver ADR-0003, crates/input);
   - lints declarados no nível do workspace.
2. Crates NÃO são scaffoldados por antecipação: um crate nasce quando a
   primeira implementação real precisa dele (o layout acima impede renomeações,
   não a criação tardia). Ordem esperada pela roadmap: `crates/input` (spike
   macOS, ADR-0003) → motor de curvas dentro do `crates/brush-engine`
   (ADR-0002) → `crates/document`, `crates/workspace`, `crates/commands`
   quando o app real começar.
3. O playground (hoje binário em `crates/playground`) move-se para
   `apps/playground` já, como diff mecânico próprio, antes da reescrita em
   egui (ADR-0003). O app real virá depois em `apps/crita`.

## Consequences
- Layout estável sem congelar estrutura prematura: convenções fixas, crates
  sob demanda.
- Moves mecânicos ficam separados de mudanças semânticas (reescrita egui),
  preservando diffs pequenos e revisáveis (golden rule 6).
- O workspace passa a ter dois globs de membros (`crates/*`, `apps/*`); paths
  relativos entre crates são ajustados na move.
