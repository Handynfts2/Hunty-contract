use crate::NftData;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Storage layer for NFTs.
pub struct Storage;

impl Storage {
    const NFT_KEY: soroban_sdk::Symbol = symbol_short!("NFT");
    const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CNTR");
    const OWNER_NFTS_KEY: soroban_sdk::Symbol = symbol_short!("ONFT");

    fn nft_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_KEY, nft_id)
    }

    fn owner_nfts_key(owner: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::OWNER_NFTS_KEY, owner.clone())
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

    /// Generates a unique, unpredictable NFT ID using Soroban's ledger-seeded PRNG.
    ///
    /// # RNG Approach
    ///
    /// Uses `env.prng().gen::<u64>()` which draws a full 64-bit random value
    /// from the Soroban PRNG seeded by the ledger's per-transaction randomness.
    /// Each transaction receives a fresh seed derived from the consensus-level
    /// random value, making the output unpredictable to any party before the
    /// transaction is finalized.
    ///
    /// The raw value is ORed with 1 to guarantee non-zero (ID 0 is reserved as
    /// "no NFT"), avoiding `gen_range` which had an overflow vulnerability
    /// (CVE-2026-24889) in soroban-sdk ≤ 25.x.
    ///
    /// A collision check (`Storage::get_nft`) is performed after each draw.
    /// In the astronomically unlikely event of a collision (probability ~1 in
    /// 2^63 per attempt), the PRNG is queried again. The loop is capped at
    /// 10 attempts, after which execution panics — this bound is unreachable
    /// in practice.
    ///
    /// The NFT total-supply counter (`NFTCNT`) is incremented separately via
    /// `increment_nft_counter` so `total_supply` remains accurate.
    pub fn gen_unique_nft_id(env: &Env) -> u64 {
        for _ in 0..10u32 {
            // OR with 1 ensures the value is always non-zero without using gen_range.
            let candidate: u64 = env.prng().gen::<u64>() | 1;
            if Self::get_nft(env, candidate).is_none() {
                return candidate;
            }
        }
        panic!("NftIdGenerationFailed");
    }

    /// Increments the total-supply counter and returns the new count.
    /// Call this once per successful mint, separately from ID generation.
    pub fn increment_nft_counter(env: &Env) -> u64 {
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
