# Architecture

> TODO: expand with detailed module descriptions and data flow diagrams.

See `ai.md` at the repo root for the cross-repo system overview and the `README.md` for
the Mermaid architecture and sequence diagrams.

## Module Responsibilities

| File | Responsibility |
|------|---------------|
| `lib.rs` | Public contract API, auth entry points |
| `book.rs` | `mint_book`, `add_edition` |
| `marketplace.rs` | `create_listing`, `buy_listing`, `cancel_listing` |
| `royalty.rs` | Atomic contributor royalty distribution |
| `storage.rs` | All storage reads/writes with TTL management |
| `types.rs` | Shared data structures |
| `errors.rs` | `ContractError` enum |
| `events.rs` | Event emission |

## Royalty Math

```
for each contributor:
    share = price * contributor.bps / 10_000

platform_fee = price * platform_fee_bps / 10_000

seller_amount = price - sum(contributor_shares) - platform_fee
```

All transfers are atomic within a single `buy_listing` invocation.
