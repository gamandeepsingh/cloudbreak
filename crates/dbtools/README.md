```sh
cargo run -p cloudbreak-dbtools -- analytics get-biggest-programs
cargo run -p cloudbreak-dbtools -- analytics table-size
cargo run -p cloudbreak-dbtools -- analytics indexes-sizes
cargo run -p cloudbreak-dbtools -- analytics partition-sizes
cargo run -p cloudbreak-dbtools -- analytics distinct-owners-count
cargo run -p cloudbreak-dbtools -- analytics mint-accounts-count
cargo run -p cloudbreak-dbtools -- analytics indexes-count
cargo run -p cloudbreak-dbtools -- analytics slow-queries
cargo run -p cloudbreak-dbtools -- analytics get-delegates
cargo run -p cloudbreak-dbtools -- analytics accounts-count
```

# Queries to modify db config:

```sh
# Sets the level of detail for pg_tracing spans
cargo run -p cloudbreak-dbtools -- config pg-tracing-high-detail --false
```
