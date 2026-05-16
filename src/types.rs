use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Contributor {
    pub address: Address,
    pub bps: u32, // basis points, e.g. 8000 = 80%
}

#[contracttype]
#[derive(Clone)]
pub struct Book {
    pub id: u64,
    pub author: Address,
    pub metadata_uri: String,
    pub contributors: Vec<Contributor>,
    pub edition_count: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct Edition {
    pub id: u32,
    pub book_id: u64,
    pub limit: u32,      // 0 = open edition
    pub sold: u32,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct Listing {
    pub id: u64,
    pub seller: Address,
    pub book_id: u64,
    pub edition_id: u32,
    pub price: i128,
    pub status: ListingStatus,
}
