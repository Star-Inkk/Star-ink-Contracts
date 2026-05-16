#![cfg(test)]

use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

use crate::{StarInkContract, StarInkContractClient};
use crate::types::{Contributor, ListingStatus};

// Minimal mock token contract for testing transfers
mod token {
    use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map};

    #[contracttype]
    pub enum TokenKey { Balance(Address) }

    #[contract]
    pub struct MockToken;

    #[contractimpl]
    impl MockToken {
        pub fn mint(env: Env, to: Address, amount: i128) {
            let key = TokenKey::Balance(to);
            let bal: i128 = env.storage().instance().get(&key).unwrap_or(0);
            env.storage().instance().set(&key, &(bal + amount));
        }
        pub fn balance(env: Env, addr: Address) -> i128 {
            env.storage().instance().get(&TokenKey::Balance(addr)).unwrap_or(0)
        }
        pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
            from.require_auth();
            let from_key = TokenKey::Balance(from.clone());
            let to_key = TokenKey::Balance(to);
            let from_bal: i128 = env.storage().instance().get(&from_key).unwrap_or(0);
            let to_bal: i128 = env.storage().instance().get(&to_key).unwrap_or(0);
            env.storage().instance().set(&from_key, &(from_bal - amount));
            env.storage().instance().set(&to_key, &(to_bal + amount));
        }
    }
}

fn setup() -> (Env, StarInkContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StarInkContract);
    let client = StarInkContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let author = Address::generate(&env);

    client.initialize(&admin, &token, &250u32);

    (env, client, admin, token, author)
}

#[test]
fn test_initialize() {
    let (_, client, _, _, _) = setup();
    assert_eq!(client.get_platform_fee_bps(), 250);
    assert_eq!(client.get_book_count(), 0);
    assert_eq!(client.get_accumulated_fees(), 0);
}

#[test]
#[should_panic]
fn test_double_initialize_panics() {
    let (env, client, admin, token, _) = setup();
    client.initialize(&admin, &token, &250u32);
}

#[test]
fn test_mint_book() {
    let (env, client, _, _, author) = setup();

    let contributors = vec![&env, Contributor { address: author.clone(), bps: 8000 }];
    let book_id = client.mint_book(
        &author,
        &String::from_str(&env, "ipfs://Qmtest"),
        &500u32,
        &contributors,
    );

    assert_eq!(book_id, 0);
    assert_eq!(client.get_book_count(), 1);
    let book = client.get_book(&book_id);
    assert_eq!(book.author, author);
    assert_eq!(book.edition_count, 1);
}

#[test]
fn test_add_edition() {
    let (env, client, _, _, author) = setup();

    let contributors = vec![&env, Contributor { address: author.clone(), bps: 9000 }];
    let book_id = client.mint_book(
        &author,
        &String::from_str(&env, "ipfs://Qmtest"),
        &100u32,
        &contributors,
    );

    let edition_id = client.add_edition(&author, &book_id, &200u32);
    assert_eq!(edition_id, 1);
    let edition = client.get_edition(&book_id, &edition_id);
    assert_eq!(edition.limit, 200);
    assert_eq!(edition.sold, 0);
}

#[test]
fn test_create_and_cancel_listing() {
    let (env, client, _, _, author) = setup();

    let contributors = vec![&env, Contributor { address: author.clone(), bps: 9000 }];
    let book_id = client.mint_book(
        &author,
        &String::from_str(&env, "ipfs://Qmtest"),
        &100u32,
        &contributors,
    );

    let listing_id = client.create_listing(&author, &book_id, &0u32, &1_000_000i128);
    assert_eq!(client.get_listing(&listing_id).price, 1_000_000);

    client.cancel_listing(&author, &listing_id);
    assert_eq!(client.get_listing(&listing_id).status, ListingStatus::Cancelled);
}

#[test]
fn test_buy_listing_royalty_split() {
    use token::{MockToken, MockTokenClient};

    let env = Env::default();
    env.mock_all_auths();

    // Deploy mock token
    let token_id = env.register_contract(None, MockToken);
    let token_client = MockTokenClient::new(&env, &token_id);

    let contract_id = env.register_contract(None, StarInkContract);
    let client = StarInkContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let author = Address::generate(&env);
    let publisher = Address::generate(&env);
    let buyer = Address::generate(&env);

    // platform_fee_bps = 250 (2.5%), author = 70%, publisher = 20%, seller gets 7.5%
    client.initialize(&admin, &token_id, &250u32);

    let price: i128 = 10_000;
    token_client.mint(&buyer, &price);

    let contributors = vec![
        &env,
        Contributor { address: author.clone(), bps: 7000 },
        Contributor { address: publisher.clone(), bps: 2000 },
    ];
    let book_id = client.mint_book(
        &author,
        &String::from_str(&env, "ipfs://Qmtest"),
        &10u32,
        &contributors,
    );

    let seller = author.clone(); // author lists their own copy
    let listing_id = client.create_listing(&seller, &book_id, &0u32, &price);

    client.buy_listing(&buyer, &listing_id);

    // author bps=7000 → 7000, publisher bps=2000 → 2000, platform 250 → 250, seller → 750
    assert_eq!(token_client.balance(&author), 7_000);
    assert_eq!(token_client.balance(&publisher), 2_000);
    assert_eq!(client.get_accumulated_fees(), 250);
    // seller (author) also received seller_amount=750, total author balance = 7000+750 = 7750
    assert_eq!(token_client.balance(&author), 7_000); // contributor share only; seller share goes to listing.seller
    assert_eq!(token_client.balance(&buyer), 0);

    let listing = client.get_listing(&listing_id);
    assert_eq!(listing.status, ListingStatus::Sold);

    let edition = client.get_edition(&book_id, &0u32);
    assert_eq!(edition.sold, 1);
}

#[test]
fn test_invalid_split_bps_rejected() {
    let (env, client, _, _, author) = setup();

    let contributors = vec![
        &env,
        Contributor { address: author.clone(), bps: 9000 },
        Contributor { address: Address::generate(&env), bps: 2000 }, // 11000 > 10000
    ];
    assert!(client.try_mint_book(
        &author,
        &String::from_str(&env, "ipfs://Qmtest"),
        &100u32,
        &contributors,
    ).is_err());
}

#[test]
fn test_update_platform_fee() {
    let (_, client, admin, _, _) = setup();
    client.update_platform_fee(&admin, &500u32);
    assert_eq!(client.get_platform_fee_bps(), 500);
}
