Создать простое приложение на Rust, взаимодействующее с Solana. Нужно разработать смарт-контракт (программу), который позволяет пользователям вносить депозиты в SOL, отслеживать их баланс и выводить средства.

## Setup

```shell
cargo update -p bytemuck_derive --precise 1.8.1
```

```shell
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```

## Build

```shell
cargo build-sbf
```