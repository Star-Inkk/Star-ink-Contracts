#![no_std]

mod book;
mod errors;
mod events;
mod marketplace;
mod royalty;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use types::{Book, Contributor, Edition, Listing};
use errors::ContractError;

#[contract]
pub struct StarInkContract;

#[contractimpl]
impl StarInkContract {
    // ── Admin ─────────────────────────────────────────────────────────────────

    /// One-time initialisation. Sets admin, payment token, and platform fee.
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        platform_fee_bps: u32,
    ) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        if platform_fee_bps > 10_000 {
            return Err(ContractError::InvalidFeeBps);
        }
        storage::set_admin(&env, &admin);
        storage::set_token(&env, &token);
        storage::set_platform_fee_bps(&env, platform_fee_bps);
        Ok(())
    }

    /// Update platform fee (admin only).
    pub fn update_platform_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        let admin = storage::get_admin(&env);
        admin.require_auth();
        if fee_bps > 10_000 {
            return Err(ContractError::InvalidFeeBps);
        }
        storage::set_platform_fee_bps(&env, fee_bps);
        events::fee_updated(&env, fee_bps);
        Ok(())
    }

    /// Withdraw accumulated platform fees to `to` (admin only).
    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        let admin = storage::get_admin(&env);
        admin.require_auth();
        let amount = storage::get_accumulated_fees(&env);
        if amount == 0 {
            return Err(ContractError::NoFeesToWithdraw);
        }
        let token = soroban_sdk::token::Client::new(&env, &storage::get_token(&env));
        token.transfer(&env.current_contract_address(), &to, &amount);
        storage::clear_accumulated_fees(&env);
        events::fees_withdrawn(&env, &to, amount);
        Ok(())
    }

    // ── Author ────────────────────────────────────────────────────────────────

    pub fn mint_book(
        env: Env,
        author: Address,
        metadata_uri: String,
        edition_limit: u32,
        contributors: Vec<Contributor>,
    ) -> Result<u64, ContractError> {
        book::mint_book(&env, author, metadata_uri, edition_limit, contributors)
    }

    pub fn add_edition(
        env: Env,
        author: Address,
        book_id: u64,
        edition_limit: u32,
    ) -> Result<u32, ContractError> {
        book::add_edition(&env, author, book_id, edition_limit)
    }

    // ── Marketplace ───────────────────────────────────────────────────────────

    pub fn create_listing(
        env: Env,
        seller: Address,
        book_id: u64,
        edition_id: u32,
        price: i128,
    ) -> Result<u64, ContractError> {
        marketplace::create_listing(&env, seller, book_id, edition_id, price)
    }

    pub fn buy_listing(env: Env, buyer: Address, listing_id: u64) -> Result<(), ContractError> {
        marketplace::buy_listing(&env, buyer, listing_id)
    }

    pub fn cancel_listing(
        env: Env,
        seller: Address,
        listing_id: u64,
    ) -> Result<(), ContractError> {
        marketplace::cancel_listing(&env, seller, listing_id)
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    pub fn get_book(env: Env, book_id: u64) -> Result<Book, ContractError> {
        storage::get_book(&env, book_id).ok_or(ContractError::BookNotFound)
    }

    pub fn get_edition(env: Env, book_id: u64, edition_id: u32) -> Result<Edition, ContractError> {
        storage::get_edition(&env, book_id, edition_id).ok_or(ContractError::EditionNotFound)
    }

    pub fn get_listing(env: Env, listing_id: u64) -> Result<Listing, ContractError> {
        storage::get_listing(&env, listing_id).ok_or(ContractError::ListingNotFound)
    }

    pub fn get_platform_fee_bps(env: Env) -> u32 {
        storage::get_platform_fee_bps(&env)
    }

    pub fn get_accumulated_fees(env: Env) -> i128 {
        storage::get_accumulated_fees(&env)
    }

    pub fn get_book_count(env: Env) -> u64 {
        storage::get_book_count(&env)
    }
}
