use soroban_sdk::{token, Address, Env};
use crate::{
    errors::ContractError,
    events,
    royalty,
    storage,
    types::{Listing, ListingStatus},
};

/// List a book edition for sale.
pub fn create_listing(
    env: &Env,
    seller: Address,
    book_id: u64,
    edition_id: u32,
    price: i128,
) -> Result<u64, ContractError> {
    seller.require_auth();

    if price <= 0 {
        return Err(ContractError::InvalidAmount);
    }
    storage::get_book(env, book_id).ok_or(ContractError::BookNotFound)?;
    storage::get_edition(env, book_id, edition_id).ok_or(ContractError::EditionNotFound)?;

    // TODO: verify seller holds a token for this edition (requires NFT ownership tracking)

    let listing_id = storage::next_listing_id(env);
    let listing = Listing {
        id: listing_id,
        seller: seller.clone(),
        book_id,
        edition_id,
        price,
        status: ListingStatus::Active,
    };
    storage::save_listing(env, &listing);

    events::listing_created(env, listing_id, &seller, price);
    Ok(listing_id)
}

/// Purchase a listing. Royalties are distributed atomically.
/// Payment flow: buyer → contributors, buyer → platform (contract), buyer → seller.
/// Total deducted from buyer = price exactly.
pub fn buy_listing(
    env: &Env,
    buyer: Address,
    listing_id: u64,
) -> Result<(), ContractError> {
    buyer.require_auth();

    let mut listing = storage::get_listing(env, listing_id).ok_or(ContractError::ListingNotFound)?;
    if listing.status != ListingStatus::Active {
        return Err(ContractError::ListingNotActive);
    }

    let mut edition = storage::get_edition(env, listing.book_id, listing.edition_id)
        .ok_or(ContractError::EditionNotFound)?;

    if edition.limit > 0 && edition.sold >= edition.limit {
        return Err(ContractError::EditionSoldOut);
    }

    let book = storage::get_book(env, listing.book_id).ok_or(ContractError::BookNotFound)?;
    let token_addr = storage::get_token(env);
    let token = token::Client::new(env, &token_addr);

    // Distribute contributor royalties; transfers happen inside distribute()
    let (contributor_total, platform_fee) = royalty::distribute(env, &book, &buyer, listing.price)?;

    // Seller receives whatever is left after contributors and platform take their cut
    let seller_amount = listing.price
        .checked_sub(contributor_total)
        .and_then(|v| v.checked_sub(platform_fee))
        .ok_or(ContractError::Overflow)?;

    if seller_amount > 0 {
        token.transfer(&buyer, &listing.seller, &seller_amount);
    }
    if platform_fee > 0 {
        token.transfer(&buyer, &env.current_contract_address(), &platform_fee);
        storage::add_accumulated_fees(env, platform_fee);
    }

    edition.sold += 1;
    storage::save_edition(env, &edition);

    listing.status = ListingStatus::Sold;
    storage::save_listing(env, &listing);

    events::listing_sold(env, listing_id, &buyer, listing.price);
    Ok(())
}

/// Cancel an active listing. Only the original seller may cancel.
pub fn cancel_listing(
    env: &Env,
    seller: Address,
    listing_id: u64,
) -> Result<(), ContractError> {
    seller.require_auth();

    let mut listing = storage::get_listing(env, listing_id).ok_or(ContractError::ListingNotFound)?;
    if listing.status != ListingStatus::Active {
        return Err(ContractError::ListingNotActive);
    }
    if listing.seller != seller {
        return Err(ContractError::Unauthorized);
    }

    listing.status = ListingStatus::Cancelled;
    storage::save_listing(env, &listing);

    events::listing_cancelled(env, listing_id, &seller);
    Ok(())
}
