# Fase 1 do Crita — brush engine + playground

Conteúdo deste pacote (a ser descompactado NA RAIZ do repo `crita/`):

```
Cargo.toml                        # workspace
crates/
  brush-engine/                   # núcleo: mask, spacing, smudge por pixel, preset
    tests/core.rs                 # testes unitários
    tests/golden.rs + golden/     # golden test (gerar com UPDATE_GOLDEN=1)
  render/                         # compositor CPU (oráculo) com testes
  playground/                     # app Slint desktop: canvas + sliders + undo
```

## Como instalar

```bash
cd /Volumes/PortableSSD/code/crita
# descompacta o conteúdo do zip AQUI (a pasta crates/ e o Cargo.toml na raiz)
unzip ~/Downloads/crita-fase1.zip -d /tmp/crita-fase1
cp -R /tmp/crita-fase1/crates . && cp /tmp/crita-fase1/Cargo.toml .
# mescla o README-FASE1.md com o README.md existente se quiser
```

## Como rodar

```bash
source ~/.bashrc
cargo test                 # engine + golden (gera golden na 1a vez? NÃO: rode primeiro)
UPDATE_GOLDEN=1 cargo test -p crita-brush-engine --test golden   # 1a vez: gera e commite
cargo test                 # agora compara contra o golden
cargo run -p crita-playground                                  # janela de pintura
```

## Ressalvas honestas

- Código **não compilado** neste ambiente (sem toolchain). APIs estáveis de Rust/Slint,
  mas espere 1-2 erros bobos no primeiro build — cole no Kimi Code do VS Code que ele resolve.
- O golden test FALHA de propósito na primeira execução (golden ainda não existe):
  é o `UPDATE_GOLDEN=1` que gera a referência. Commite o `stroke.ppm` gerado.
- Sliders usam binding de duas vias (`value <=> root.brush-size`). Se a versão do
  Slint reclamar, troca por `value: root.brush-size; value-changed => { root.brush-size = self.value; }`.
