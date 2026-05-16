use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ContractError {
    AlreadyInitialized  = 1,
    NotInitialized      = 2,
    InvalidAmount       = 3,
    InvalidFeeBps       = 4,
    InvalidSplitBps     = 5,
    BookNotFound        = 6,
    EditionNotFound     = 7,
    ListingNotFound     = 8,
    EditionSoldOut      = 9,
    ListingNotActive    = 10,
    Unauthorized        = 11,
    NoFeesToWithdraw    = 12,
    InvalidMetadataUri  = 13,
    Overflow            = 14,
}
