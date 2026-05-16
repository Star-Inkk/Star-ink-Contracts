use soroban_sdk::{Address, Env, String, Vec};
use crate::{
    errors::ContractError,
    events,
    storage,
    types::{Book, Contributor, Edition},
};

/// Mint a new book. Author must authorise the call.
/// `contributors` bps must sum to ≤ 10000.
pub fn mint_book(
    env: &Env,
    author: Address,
    metadata_uri: String,
    edition_limit: u32,
    contributors: Vec<Contributor>,
) -> Result<u64, ContractError> {
    author.require_auth();

    if metadata_uri.len() == 0 {
        return Err(ContractError::InvalidMetadataUri);
    }

    // Validate contributor bps sum
    let mut total_bps: u32 = 0;
    for c in contributors.iter() {
        total_bps = total_bps.checked_add(c.bps).ok_or(ContractError::Overflow)?;
    }
    if total_bps > 10_000 {
        return Err(ContractError::InvalidSplitBps);
    }

    let book_id = storage::next_book_id(env);

    let book = Book {
        id: book_id,
        author: author.clone(),
        metadata_uri,
        contributors,
        edition_count: 1,
    };
    storage::save_book(env, &book);

    // Create the first edition
    let edition = Edition { id: 0, book_id, limit: edition_limit, sold: 0 };
    storage::save_edition(env, &edition);

    events::book_minted(env, book_id, &author);
    Ok(book_id)
}

/// Add a new edition to an existing book. Only the original author may call this.
pub fn add_edition(
    env: &Env,
    author: Address,
    book_id: u64,
    edition_limit: u32,
) -> Result<u32, ContractError> {
    author.require_auth();

    let mut book = storage::get_book(env, book_id).ok_or(ContractError::BookNotFound)?;
    if book.author != author {
        return Err(ContractError::Unauthorized);
    }

    let edition_id = book.edition_count;
    book.edition_count += 1;
    storage::save_book(env, &book);

    let edition = Edition { id: edition_id, book_id, limit: edition_limit, sold: 0 };
    storage::save_edition(env, &edition);

    events::edition_added(env, book_id, edition_id);
    Ok(edition_id)
}
