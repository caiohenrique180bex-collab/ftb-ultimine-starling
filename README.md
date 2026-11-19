# FTB Ultimine — Starling / Horizon native port

Port nativo em Rust pelo caminho de *native source-mixin* do Horizon, sem Forge, Fabric, NeoForge ou dependências de runtime do mod.

> **Estado atual:** o hook de gameplay está integrado. O núcleo de seleção e a integração contra a revisão fixada do Horizon passaram pelo GitHub Actions. Ainda falta testar o comportamento dentro do launcher, em um mundo real e em multiplayer; portanto, esta versão ainda não deve ser tratada como estável.

## Alvo e formato

O projeto gera um bundle de código-fonte `.hnmod`. O launcher Horizon compila esse bundle para a revisão compatível do jogo. Não é um mod Java/Fabric e não deve ser colocado na pasta comum de `.hmod`.

- ID: `ftb_ultimine`
- Versão: `0.1.0`
- Linguagem: Rust
- Jogo-alvo do manifesto: `26.3`
- Revisão Horizon: definida em `compatibility.json`
- Empacotamento local: `python3 tools/package.py`

## Controles implementados

- **Shift + quebrar bloco:** seleciona blocos conectados por seis faces que tenham o mesmo identificador do bloco inicial.
- **Árvores:** quando o bloco inicial é um tronco/madeira/haste reconhecido, seleciona os blocos conectados da mesma família. As folhas ficam de fora para decaírem normalmente.
- **Shift + Ctrl + quebrar bloco:** seleciona uma área cúbica de `3×3×3`, incluindo apenas blocos não vazios que estejam ao alcance.

A busca conectada usa raio máximo de 4 blocos e limite de 64 posições por seleção. As posições extras também passam pela verificação de alcance do jogador.

**Limitação de entrada:** os controles atuais usam teclado (Shift e Ctrl). Ainda não existe botão de toque, menu de configuração ou personalização de teclas.

## Integração com o jogo

Cada bloco extra é enviado pela rotina nativa `Game::break_block`, em vez de remover o bloco diretamente. Isso mantém o Horizon responsável pelas regras normais de quebra, ferramenta, durabilidade, drops, experiência e eventos. Um guard de reentrância impede que as chamadas extras disparem outra seleção em cascata.

## Verificação

O workflow do GitHub Actions passou em:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`
- testes unitários do núcleo Rust
- compilação e testes do hook dentro do checkout real do Horizon fixado
- empacotamento e upload do bundle `.hnmod`

[Ver execução aprovada no GitHub Actions](https://github.com/caiohenrique180bex-collab/ftb-ultimine-starling/actions/runs/44630033925).

Essas verificações confirmam a compilação e os testes automatizados, mas não substituem o teste dentro do launcher, nem validam controles touch, servidores ou multiplayer.

## Desenvolvimento

O CI usa Linux, Rust 1.95.0 e Python 3.12:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python3 tools/horizon.py test --horizon ../horizon
```

## Segurança e integridade

A implementação não deve apagar blocos diretamente nem contornar regras do jogo. Cada quebra adicional deve continuar usando o caminho nativo, respeitando ferramenta, alcance, limites por ativação e proteção contra recursão.
