# StarInk — AI Cross-Repo Context

> This file is the single source of truth for AI assistants working across the StarInk organisation.
> Place a copy of this file (or a symlink) at the root of every repo:
> `Star-ink-Contracts/ai.md`, `Star-ink-Frontend/ai.md`, `Star-ink-Backend/ai.md`

---

## Organisation Overview

| Repo | Purpose | Primary Language |
|------|---------|-----------------|
| `Star-ink-Contracts` | Soroban smart contracts on Stellar | Rust |
| `Star-ink-Frontend` | User-facing dApp | TypeScript / React |
| `Star-ink-Backend` | Indexer, API, event listener | TypeScript / Node.js |

StarInk is a decentralised book publishing platform. Authors mint books as on-chain assets, set contributor royalty splits, and sell editions. Buyers trade on a peer-to-peer marketplace. Royalties are distributed atomically at the point of sale.

---

## Shared Domain Vocabulary

| Term | Definition |
|------|-----------|
| `book_id` | `u64` — auto-incrementing on-chain ID assigned at mint |
| `edition_id` | `u32` — per-book edition index (0 = first edition) |
| `listing_id` | `u64` — auto-incrementing marketplace listing ID |
| `bps` | Basis points. 10000 bps = 100%. Used for fees and royalty splits |
| `contributor` | Address + bps share registered at mint time |
| `platform_fee_bps` | Contract-level fee taken from every sale (default 250 = 2.5%) |
| `metadata_uri` | IPFS URI pointing to book metadata JSON (title, cover, description) |

---

## Contract Interface (Star-ink-Contracts)

### Network
- **Testnet RPC**: `https://soroban-testnet.stellar.org`
- **Mainnet RPC**: `https://soroban-mainnet.stellar.org`
- Contract ID is set per-environment in `.env` files (see below).

### Public Functions

```
initialize(admin, token, platform_fee_bps)
update_platform_fee(fee_bps)          // admin only
withdraw_fees(to)                     // admin only

mint_book(author, metadata_uri, edition_limit, contributors) → book_id
add_edition(author, book_id, edition_limit)                  → edition_id

create_listing(seller, book_id, edition_id, price)           → listing_id
buy_listing(buyer, listing_id)
cancel_listing(seller, listing_id)

get_book(book_id)           → Book
get_edition(book_id, edition_id) → Edition
get_listing(listing_id)     → Listing
get_platform_fee_bps()      → u32
get_accumulated_fees()      → i128
get_book_count()            → u64
```

### Key Types (mirrored in Frontend/Backend)

```typescript
// TypeScript equivalents — keep in sync with src/types.rs

interface Contributor {
  address: string;   // Stellar G-address
  bps: number;       // 0–10000
}

interface Book {
  id: bigint;
  author: string;
  metadata_uri: string;
  contributors: Contributor[];
  edition_count: number;
}

interface Edition {
  id: number;
  book_id: bigint;
  limit: number;   // 0 = open edition
  sold: number;
}

interface Listing {
  id: bigint;
  seller: string;
  book_id: bigint;
  edition_id: number;
  price: bigint;    // in token stroops
  status: 'Active' | 'Sold' | 'Cancelled';
}
```

### Contract Events

| Event topic | Data | Emitted by |
|-------------|------|-----------|
| `book_mint` | `(book_id, author)` | `mint_book` |
| `edn_added` | `(book_id, edition_id)` | `add_edition` |
| `lst_creat` | `(listing_id, seller, price)` | `create_listing` |
| `lst_sold`  | `(listing_id, buyer, price)` | `buy_listing` |
| `lst_cncl`  | `(listing_id, seller)` | `cancel_listing` |
| `fee_upd`   | `new_bps` | `update_platform_fee` |
| `fee_with`  | `(to, amount)` | `withdraw_fees` |

### Error Codes

| Code | Name | Meaning |
|------|------|---------|
| 1 | AlreadyInitialized | Contract already set up |
| 2 | NotInitialized | Contract not yet set up |
| 3 | InvalidAmount | Price ≤ 0 |
| 4 | InvalidFeeBps | Fee > 10000 bps |
| 5 | InvalidSplitBps | Contributor bps sum > 10000 |
| 6 | BookNotFound | Unknown book_id |
| 7 | EditionNotFound | Unknown edition_id for book |
| 8 | ListingNotFound | Unknown listing_id |
| 9 | EditionSoldOut | Edition supply exhausted |
| 10 | ListingNotActive | Listing already sold or cancelled |
| 11 | Unauthorized | Wrong signer |
| 12 | NoFeesToWithdraw | Accumulated fees = 0 |
| 13 | InvalidMetadataUri | Empty metadata URI |
| 14 | Overflow | Arithmetic overflow |

---

## Frontend (Star-ink-Frontend)

### Responsibilities
- Connect Stellar wallets (Freighter, xBull, Lobstr)
- Call contract functions via `@stellar/stellar-sdk` + Soroban RPC
- Display books, editions, listings fetched from Backend API
- Emit user-initiated transactions (mint, list, buy, cancel)

### Environment Variables
```
VITE_CONTRACT_ID=          # deployed StarInk contract address
VITE_TOKEN_ADDRESS=        # payment token contract address
VITE_NETWORK=testnet       # testnet | mainnet
VITE_RPC_URL=              # Soroban RPC endpoint
VITE_BACKEND_URL=          # Backend API base URL
```

### Key Integration Points
- **Contract calls**: use `@stellar/stellar-sdk` `SorobanRpc.Server` + `Contract.call()`
- **Event polling**: subscribe to Backend SSE/WebSocket for real-time listing updates
- **Metadata**: fetch book metadata JSON from IPFS using `metadata_uri` from `get_book()`
- **Price display**: divide token amount by `10^7` (XLM stroops) for human-readable values

---

## Backend (Star-ink-Backend)

### Responsibilities
- Listen to Stellar ledger events and index contract state
- Expose REST API consumed by Frontend
- Cache book/edition/listing data in a database
- Serve IPFS metadata proxied/cached for performance

### Environment Variables
```
CONTRACT_ID=               # deployed StarInk contract address
TOKEN_ADDRESS=             # payment token contract address
NETWORK=testnet            # testnet | mainnet
RPC_URL=                   # Soroban RPC endpoint
DATABASE_URL=              # PostgreSQL connection string
PORT=3001
```

### REST API Shape (suggested)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/books` | Paginated list of all books |
| GET | `/books/:id` | Single book + contributors |
| GET | `/books/:id/editions` | All editions for a book |
| GET | `/listings` | Active listings (filterable by book_id) |
| GET | `/listings/:id` | Single listing |
| GET | `/stats` | Platform stats (book count, volume, fees) |

### Event Indexing
- Poll `getEvents` on Soroban RPC for the contract address
- Map event topics to DB writes using the event table above
- Replay from ledger 0 on first boot; store cursor in DB

---

## Royalty Flow (shared understanding)

```
sale_price = listing.price

for each contributor:
    payout = sale_price * contributor.bps / 10000
    transfer(buyer → contributor, payout)

platform_fee = sale_price * platform_fee_bps / 10000
transfer(buyer → contract, platform_fee)

seller_receives = sale_price - sum(contributor_payouts) - platform_fee
transfer(buyer → seller, seller_receives)
```

All transfers happen atomically inside `buy_listing`. There is no separate claim step.

---

## Development Workflow

```
# Contracts
cd Star-ink-Contracts
cargo test
cargo build --target wasm32-unknown-unknown --release
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/star_ink.wasm

# Deploy (testnet)
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/star_ink.optimized.wasm \
  --source deployer --network testnet

# After deploy — update CONTRACT_ID in Frontend and Backend .env files
```

---

## File Map

```
Star-ink-Contracts/
├── src/
│   ├── lib.rs          # contract entry point, all public functions
│   ├── book.rs         # mint_book, add_edition
│   ├── marketplace.rs  # create_listing, buy_listing, cancel_listing
│   ├── royalty.rs      # atomic royalty distribution
│   ├── types.rs        # Book, Edition, Listing, Contributor
│   ├── storage.rs      # storage key helpers
│   ├── errors.rs       # ContractError enum
│   ├── events.rs       # event emission
│   └── test.rs         # test suite
├── .github/workflows/contract-ci.yml
├── Cargo.toml
└── ai.md               ← this file

Star-ink-Frontend/
├── src/
│   ├── lib/contract.ts # Soroban contract client wrapper
│   ├── lib/api.ts      # Backend REST client
│   └── ...
└── ai.md               ← copy of this file

Star-ink-Backend/
├── src/
│   ├── indexer/        # event listener + DB writes
│   ├── api/            # Express/Fastify routes
│   └── ...
└── ai.md               ← copy of this file
```

---

*Last updated: 2026-05-16 — Star-ink-Contracts scaffold at 60%*
