# Solana vault

Cмарт-контракт, который позволяет пользователям вносить депозиты в SOL, отслеживать их баланс и выводить средства.

- `src`- смарт контракт.
- `tests` - rust тесты для контракта
- `scripts` - ts скрипты для взаимодействия с контрактом

## Setup

1. https://docs.anza.xyz/cli/install
2. 
```shell
cargo update -p bytemuck_derive --precise 1.8.1
```

## Build

```shell
cargo build-sbf
```

на выходе: `target/deploy/solana_vault.so`
