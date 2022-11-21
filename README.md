# rust_erc20_payment


# Sample environment setup

ETH_PRIVATE_KEY=0000000000000000000000000000000000000000000000000000000000000000
PROVIDER_URL=https://rpc-mumbai.matic.today
RUST_LOG=debug,sqlx::query=warn,web=warn

# Sample runs

```
cargo run -- transfer --plain-eth --amounts=1,2,3,4 --receivers=0xA000000000000000000000000000000000050001,0xA000000000000000000000000000000000050002,0xa000000000000000000000000000000000050003,0xa000000000000000000000000000000000050004
cargo run -- transfer --token-addr=0x2036807b0b3aaf5b1858ee822d0e111fddac7018 --amounts=1,2,3,4 --receivers=0xA000000000000000000000000000000000050001,0xA000000000000000000000000000000000050002,0xa000000000000000000000000000000000050003,0xa000000000000000000000000000000000050004
```

prepare test transfers into db (it generates 100 random GLM transfers)

```cargo run --example generate_transfers -- --generate-count 100```

dry run without processing transactions

```cargo run -- process --generate-tx-only=1```


```sql
SELECT id,
       (CAST((julianday(broadcast_date) - 2440587.5)*86400000 AS INTEGER) - CAST((julianday(created_date) - 2440587.5)*86400000 AS INTEGER)) / 1000.0 as broadcast_delay,
       broadcast_count,
       (CAST((julianday(confirm_date) - 2440587.5)*86400000 AS INTEGER) - CAST((julianday(broadcast_date) - 2440587.5)*86400000 AS INTEGER)) / 1000.0 as confirm_delay,
       tx_hash,
       *
FROM tx
order by created_date desc
```

# TODO

- [x] Add error handling in gather_transactions, now SQL will loop forever, when hit error in gather


