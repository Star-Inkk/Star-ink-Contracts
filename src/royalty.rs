use soroban_sdk::{Address, Env, token};
use crate::{errors::ContractError, storage, types::Book};

/// Distribute royalties to all contributors.
/// Returns `(contributor_total, platform_fee)` — both already transferred from `buyer`.
pub fn distribute(
    env: &Env,
    book: &Book,
    buyer: &Address,
    price: i128,
) -> Result<(i128, i128), ContractError> {
    let token_addr = storage::get_token(env);
    let token = token::Client::new(env, &token_addr);
    let platform_fee_bps = storage::get_platform_fee_bps(env) as i128;

    let mut contributor_total: i128 = 0;

    for contributor in book.contributors.iter() {
        let share = price
            .checked_mul(contributor.bps as i128)
            .ok_or(ContractError::Overflow)?
            / 10_000;

        if share > 0 {
            token.transfer(buyer, &contributor.address, &share);
            contributor_total = contributor_total
                .checked_add(share)
                .ok_or(ContractError::Overflow)?;
        }
    }

    let platform_fee = price
        .checked_mul(platform_fee_bps)
        .ok_or(ContractError::Overflow)?
        / 10_000;

    Ok((contributor_total, platform_fee))
}
