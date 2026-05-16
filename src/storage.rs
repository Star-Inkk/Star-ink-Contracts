use soroban_sdk::{contracttype, Address, Env};
use crate::{errors::ContractError, types::{Book, Edition, Listing}};

// Persistent entry TTL: ~1 year in ledgers (roughly 6s/ledger)
const PERSISTENT_TTL: u32 = 5_256_000;

#[contracttype]
pub enum DataKey {
    Admin,
    Token,
    PlatformFeeBps,
    AccumulatedFees,
    BookCount,
    ListingCount,
    Book(u64),
    Edition(u64, u32),
    Listing(u64),
}

// ── Guard ─────────────────────────────────────────────────────────────────────

pub fn require_initialized(env: &Env) -> Result<(), ContractError> {
    if !has_admin(env) {
        return Err(ContractError::NotInitialized);
    }
    Ok(())
}

// ── Instance storage ──────────────────────────────────────────────────────────

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}
pub fn has_admin(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}
pub fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

pub fn set_platform_fee_bps(env: &Env, bps: u32) {
    env.storage().instance().set(&DataKey::PlatformFeeBps, &bps);
}
pub fn get_platform_fee_bps(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::PlatformFeeBps).unwrap_or(0)
}

pub fn add_accumulated_fees(env: &Env, amount: i128) {
    let current: i128 = env.storage().instance().get(&DataKey::AccumulatedFees).unwrap_or(0);
    env.storage().instance().set(&DataKey::AccumulatedFees, &(current + amount));
}
pub fn get_accumulated_fees(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::AccumulatedFees).unwrap_or(0)
}
pub fn clear_accumulated_fees(env: &Env) {
    env.storage().instance().set(&DataKey::AccumulatedFees, &0_i128);
}

// ── Counters ──────────────────────────────────────────────────────────────────

pub fn next_book_id(env: &Env) -> u64 {
    let id: u64 = env.storage().instance().get(&DataKey::BookCount).unwrap_or(0);
    env.storage().instance().set(&DataKey::BookCount, &(id + 1));
    id
}
pub fn get_book_count(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::BookCount).unwrap_or(0)
}

pub fn next_listing_id(env: &Env) -> u64 {
    let id: u64 = env.storage().instance().get(&DataKey::ListingCount).unwrap_or(0);
    env.storage().instance().set(&DataKey::ListingCount, &(id + 1));
    id
}

// ── Persistent storage (with TTL bump) ───────────────────────────────────────

pub fn save_book(env: &Env, book: &Book) {
    let key = DataKey::Book(book.id);
    env.storage().persistent().set(&key, book);
    env.storage().persistent().extend_ttl(&key, PERSISTENT_TTL, PERSISTENT_TTL);
}
pub fn get_book(env: &Env, book_id: u64) -> Option<Book> {
    let key = DataKey::Book(book_id);
    let val = env.storage().persistent().get(&key)?;
    env.storage().persistent().extend_ttl(&key, PERSISTENT_TTL, PERSISTENT_TTL);
    Some(val)
}

pub fn save_edition(env: &Env, edition: &Edition) {
    let key = DataKey::Edition(edition.book_id, edition.id);
    env.storage().persistent().set(&key, edition);
    env.storage().persistent().extend_ttl(&key, PERSISTENT_TTL, PERSISTENT_TTL);
}
pub fn get_edition(env: &Env, book_id: u64, edition_id: u32) -> Option<Edition> {
    let key = DataKey::Edition(book_id, edition_id);
    let val = env.storage().persistent().get(&key)?;
    env.storage().persistent().extend_ttl(&key, PERSISTENT_TTL, PERSISTENT_TTL);
    Some(val)
}

pub fn save_listing(env: &Env, listing: &Listing) {
    let key = DataKey::Listing(listing.id);
    env.storage().persistent().set(&key, listing);
    env.storage().persistent().extend_ttl(&key, PERSISTENT_TTL, PERSISTENT_TTL);
}
pub fn get_listing(env: &Env, listing_id: u64) -> Option<Listing> {
    let key = DataKey::Listing(listing_id);
    let val = env.storage().persistent().get(&key)?;
    env.storage().persistent().extend_ttl(&key, PERSISTENT_TTL, PERSISTENT_TTL);
    Some(val)
}
