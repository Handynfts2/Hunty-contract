use crate::NftData;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Storage layer for NFTs.
pub struct Storage;

impl Storage {
    const NFT_KEY: soroban_sdk::Symbol = symbol_short!("NFT");
    const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CNTR");
    const OWNER_NFTS_KEY: soroban_sdk::Symbol = symbol_short!("ONFT");
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADMIN");
    const MAX_SUPPLY_KEY: soroban_sdk::Symbol = symbol_short!("MXSUP");
    const MINTER_KEY: soroban_sdk::Symbol = symbol_short!("MINTER");

    fn nft_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_KEY, nft_id)
    }

    fn owner_nfts_key(owner: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::OWNER_NFTS_KEY, owner.clone())
    }

    fn minter_key(minter: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::MINTER_KEY, minter.clone())
    }

    // --- Admin / initialization ---

    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&Self::ADMIN_KEY)
    }

    pub fn save_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&Self::ADMIN_KEY, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::ADMIN_KEY)
    }

    // --- Max supply ---

    /// Stores max supply. Passing None is a no-op (absence of the key means unlimited).
    pub fn save_max_supply(env: &Env, max_supply: Option<u64>) {
        if let Some(supply) = max_supply {
            env.storage().instance().set(&Self::MAX_SUPPLY_KEY, &supply);
        }
    }

    pub fn get_max_supply(env: &Env) -> Option<u64> {
        env.storage().instance().get(&Self::MAX_SUPPLY_KEY)
    }

    // --- Minter whitelist ---

    pub fn add_minter(env: &Env, minter: &Address) {
        let key = Self::minter_key(minter);
        env.storage().persistent().set(&key, &true);
    }

    pub fn remove_minter(env: &Env, minter: &Address) {
        let key = Self::minter_key(minter);
        env.storage().persistent().remove(&key);
    }

    pub fn is_minter(env: &Env, minter: &Address) -> bool {
        let key = Self::minter_key(minter);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    /// Saves an NFT to persistent storage.
    pub fn save_nft(env: &Env, nft: &NftData) {
        let key = Self::nft_key(nft.nft_id);
        env.storage().persistent().set(&key, nft);
    }

    /// Retrieves an NFT by ID.
    pub fn get_nft(env: &Env, nft_id: u64) -> Option<NftData> {
        let key = Self::nft_key(nft_id);
        env.storage().persistent().get(&key)
    }

    /// Increments and returns the next NFT ID.
    pub fn next_nft_id(env: &Env) -> u64 {
        let current: u64 = env.storage().persistent().get(&Self::NFT_COUNTER_KEY).unwrap_or(0);
        let next = current + 1;
        env.storage().persistent().set(&Self::NFT_COUNTER_KEY, &next);
        next
    }

    /// Gets the current NFT counter (total minted).
    pub fn get_nft_counter(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&Self::NFT_COUNTER_KEY)
            .unwrap_or(0)
    }

    /// Adds an NFT ID to the owner's list.
    pub fn add_nft_to_owner(env: &Env, owner: &Address, nft_id: u64) {
        let key = Self::owner_nfts_key(owner);
        let mut nft_ids = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        nft_ids.push_back(nft_id);
        env.storage().persistent().set(&key, &nft_ids);
    }

    /// Removes an NFT ID from the owner's list.
    pub fn remove_nft_from_owner(env: &Env, owner: &Address, nft_id: u64) {
        let key = Self::owner_nfts_key(owner);
        let mut nft_ids = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        if let Some(idx) = nft_ids.first_index_of(nft_id) {
            nft_ids.remove(idx);
        }
        env.storage().persistent().set(&key, &nft_ids);
    }

    /// Gets all NFT IDs owned by an address.
    pub fn get_owner_nfts(env: &Env, owner: &Address) -> Vec<u64> {
        let key = Self::owner_nfts_key(owner);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }
}
