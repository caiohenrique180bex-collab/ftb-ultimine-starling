# FTB Ultimine — Starling / Horizon native port

Port nativo em Rust, sem Forge, Fabric, NeoForge ou bibliotecas de terceiros em runtime.

> **Estado atual: fundação do projeto.** Este primeiro estágio prepara o bundle nativo, o algoritmo puro de seleção de blocos e a automação de validação. A integração que efetivamente remove blocos em sequência ainda não está implementada; não confunda os testes do algoritmo com uma versão jogável.

## Alvo e formato

Este projeto segue o caminho de *native source-mixin* do Horizon. O launcher Horizon compila o bundle `.hnmod` para a revisão compatível do jogo. Não é um mod Java/Fabric e não deve ser colocado na pasta comum de `.hmod`.

- ID: `ftb_ultimine`
- Linguagem: Rust
- Dependências de runtime do mod: nenhuma
- Revisão Horizon: definida em `compatibility.json`
- Empacotamento: `python3 tools/package.py`

## Plano de implementação

1. **Core** — seleção determinística de blocos conectados, limites de alcance e segurança.
2. **Vein mining** — agrupar blocos adjacentes do mesmo tipo.
3. **Tree felling** — reconhecer troncos e folhas sem atingir construções próximas.
4. **Excavation** — padrões de área configuráveis.
5. **Input e modos** — ativação por controle/toque e seleção de modo.
6. **Integração** — executar quebras pelo caminho nativo do jogo, respeitando ferramenta, durabilidade, drops, proteção e servidor.

Cada etapa precisa ser validada no commit Horizon fixado antes de ser considerada concluída.

## Desenvolvimento

O CI usa Linux, Rust 1.95.0 e Python 3.12. Os testes locais sem o código do Horizon:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python3 tools/test_template.py
```

Para compilar o cliente Horizon modificado e testar o bundle contra o jogo, siga o fluxo em `tools/horizon.py` e as instruções do template nativo compatível. O checkout do Horizon precisa conter o commit fixado em `compatibility.json`; os recursos do jogo são fornecidos separadamente pelo launcher.

## Segurança e integridade

O primeiro protótipo não tenta quebrar blocos adicionais. A integração futura deve ter limites rígidos de blocos por ativação, evitar recursão nos próprios eventos e nunca contornar as regras normais do jogo.
