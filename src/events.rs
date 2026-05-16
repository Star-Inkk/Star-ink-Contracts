use soroban_sdk::{symbol_short, Address, Env};

pub fn book_minted(env: &Env, book_id: u64, author: &Address) {
    env.events().publish((symbol_short!("book_mint"), book_id), author);
}

pub fn edition_added(env: &Env, book_id: u64, edition_id: u32) {
    env.events().publish((symbol_short!("edn_added"), book_id), edition_id);
}

pub fn listing_created(env: &Env, listing_id: u64, seller: &Address, price: i128) {
    env.events().publish((symbol_short!("lst_creat"), listing_id), (seller, price));
}

pub fn listing_sold(env: &Env, listing_id: u64, buyer: &Address, price: i128) {
    env.events().publish((symbol_short!("lst_sold"), listing_id), (buyer, price));
}

pub fn listing_cancelled(env: &Env, listing_id: u64, seller: &Address) {
    env.events().publish((symbol_short!("lst_cncl"), listing_id), seller);
}

pub fn fee_updated(env: &Env, new_bps: u32) {
    env.events().publish((symbol_short!("fee_upd"),), new_bps);
}

pub fn fees_withdrawn(env: &Env, to: &Address, amount: i128) {
    env.events().publish((symbol_short!("fee_with"),), (to, amount));
}
