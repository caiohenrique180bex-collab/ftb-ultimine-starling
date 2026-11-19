# FTB Ultimine — Starling / Horizon native port

Port nativo em Rust, sem Forge, Fabric, NeoForge ou bibliotecas de terceiros em runtime.

> **Estado atual:** núcleo de seleção implementado e validado por CI. O hook do jogo ainda não executa mineração múltipla; a integração de input, classificação de blocos e chamada segura à rotina nativa continua pendente. Portanto, esta ainda não é uma versão jogável.

## Alvo e formato

Este projeto segue o caminho de *native source-mixin* do Horizon. O launcher Horizon compila o bundle `.hnmod` para a revisão compatível do jogo. Não é um mod Java/Fabric e não deve ser colocado na pasta comum de `.hmod`.

- ID: `ftb_ultimine`
- Linguagem: Rust
- Dependências de runtime do mod: nenhuma
- Revisão Horizon: definida em `compatibility.json`
- Empacotamento: `python3 tools/package.py`

## Núcleo implementado

- **Vein mining:** busca BFS de blocos conectados por seis faces, com chave de família, raio e limite.
- **Tree felling:** mesma busca conectada, preparada para receber somente troncos classificados pelo adaptador do mundo; folhas ficam fora.
- **Excavation:** seleção determinística de área cúbica, com filtro de elegibilidade.
- **Segurança:** limite rígido de 256 posições por seleção, dimensões de área limitadas, proteção contra overflow de coordenadas e nenhuma mutação do mundo no algoritmo puro.
- **Testes:** conectividade, filtro de família, limite de alcance, teto rígido, escavação e overflow.

Esses seletores são funções puras: eles não decidem se a ferramenta pode minerar, não geram drops e não removem blocos. Isso precisa acontecer pelo caminho normal do Horizon.

## Próxima integração no jogo

1. Ler o estado real do bloco e classificar minério/tronco/área elegível.
2. Detectar a tecla/gesto de ativação e permitir selecionar o modo.
3. Validar ferramenta, alcance, permissões e regras do servidor.
4. Executar cada posição pela rotina nativa de quebra, sem recursão no hook.
5. Testar durabilidade, drops, XP, eventos, proteção e multiplayer no jogo.

## Desenvolvimento

O CI usa Linux, Rust 1.95.0 e Python 3.12:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python3 tools/test_template.py
```

O workflow também empacota o `.hnmod` e publica-o como artifact quando todas as verificações passam.

## Segurança e integridade

O mod não deve apagar blocos diretamente nem contornar regras do jogo. A implementação final deve usar a rotina normal de quebra, respeitar ferramentas e drops, limitar a quantidade por ativação e evitar recursão nos próprios eventos.
