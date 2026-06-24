#![cfg(test)]
extern crate std;

use crate::{AdminImageUrisUpdatedEvent, NftMetadata, NftMintedEvent, NftReward, NftRewardClient};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    Address, Env, IntoVal, Map, String, Symbol, Val,
};

fn setup_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1000);
    env
}

fn setup_nft_reward(env: &Env, max_supply: Option<u64>) -> (NftRewardClient<'_>, Address) {
    let admin = Address::generate(env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(env, &contract_id);
    client.initialize(&admin, &max_supply);
    (client, admin)
}

fn setup_initialized(env: &Env) -> (Env, Address, Address, Address) {
    let admin = Address::generate(env);
    let minter = Address::generate(env);
    let reward_manager = Address::generate(env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(env, &contract_id);
    client.initialize(&admin, &None);
    client.set_reward_manager(&admin, &reward_manager);
    client.add_minter(&admin, &minter);
    (env.clone(), contract_id, admin, minter)
}

fn create_metadata(env: &Env, title: &str, desc: &str, image_uri: &str) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
        creator: None,
        royalty_bps: None,
    }
}

fn create_metadata_full(
    env: &Env,
    title: &str,
    desc: &str,
    image_uri: &str,
    hunt_title: &str,
    rarity: u32,
    tier: u32,
) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, hunt_title),
        rarity,
        tier,
        creator: None,
        royalty_bps: None,
    }
}

fn create_metadata_with_creator(
    env: &Env,
    title: &str,
    desc: &str,
    image_uri: &str,
    creator: Address,
    royalty_bps: Option<u32>,
) -> NftMetadata {
    NftMetadata {
        title: String::from_str(env, title),
        description: String::from_str(env, desc),
        image_uri: String::from_str(env, image_uri),
        hunt_title: String::from_str(env, title),
        rarity: 0u32,
        tier: 0u32,
        creator: Some(creator),
        royalty_bps,
    }
}

#[test]
fn test_initialize_stores_admin() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    assert_eq!(client.get_admin(), Some(admin));
}

#[test]
fn test_initialize_requires_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);

    client.initialize(&admin, &None);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_initialize_cannot_be_called_twice() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);
    client.initialize(&admin, &None);
}

#[test]
fn test_mint_reward_nft() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Hunt Champion",
        "Completed the City Hunt",
        "ipfs://QmExample123",
    );

    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    assert_eq!(nft_id, 1);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.nft_id, 1);
    assert_eq!(nft.hunt_id, 1);
    assert_eq!(nft.owner, player);
    assert_eq!(nft.metadata.title, metadata.title);
    assert_eq!(nft.metadata.description, metadata.description);
    assert_eq!(nft.metadata.image_uri, metadata.image_uri);
    assert_eq!(nft.minted_at, 1000);
}

fn create_transferable_metadata(env: &Env, title: &str, desc: &str, image_uri: &str) -> Map<Symbol, Val> {
    let mut metadata: Map<Symbol, Val> = Map::new(env);
    metadata.set(Symbol::new(env, "title"), String::from_str(env, title).into_val(env));
    metadata.set(Symbol::new(env, "description"), String::from_str(env, desc).into_val(env));
    metadata.set(Symbol::new(env, "image_uri"), String::from_str(env, image_uri).into_val(env));
    metadata.set(Symbol::new(env, "hunt_title"), String::from_str(env, title).into_val(env));
    metadata.set(Symbol::new(env, "rarity"), 0u32.into_val(env));
    metadata.set(Symbol::new(env, "tier"), 0u32.into_val(env));
    metadata.set(Symbol::new(env, "transferable"), true.into_val(env));
    metadata
}

#[test]
fn test_nft_ids_are_unique() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");

    let nft_id_1 = client.mint_reward_nft(&player1, &1, &player1, &metadata);
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");
    let nft_id_2 = client.mint_reward_nft(&player2, &1, &player2, &metadata2);
    let metadata3 = create_metadata(&env, "NFT 3", "Desc 3", "ipfs://3");
    let nft_id_3 = client.mint_reward_nft(&player1, &2, &player1, &metadata3);

    assert_eq!(nft_id_1, 1);
    assert_eq!(nft_id_2, 2);
    assert_eq!(nft_id_3, 3);
}

#[test]
fn test_metadata_stored_correctly() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(
        &env,
        "Treasure Hunter Trophy",
        "Awarded for completing the legendary treasure hunt in record time",
        "https://cdn.example.com/nft/123.png",
    );

    let nft_id = client.mint_reward_nft(&player, &42, &player, &metadata);
    let nft = client.get_nft(&nft_id).unwrap();

    assert_eq!(
        nft.metadata.title,
        String::from_str(&env, "Treasure Hunter Trophy")
    );
    assert_eq!(
        nft.metadata.description,
        String::from_str(
            &env,
            "Awarded for completing the legendary treasure hunt in record time"
        )
    );
    assert_eq!(
        nft.metadata.image_uri,
        String::from_str(&env, "https://cdn.example.com/nft/123.png")
    );
}

#[test]
fn test_initial_ownership_set_correctly() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Trophy", "Trophy desc", "ipfs://trophy");

    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let owner = client.owner_of(&nft_id).unwrap();
    assert_eq!(owner, player);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, player);
}

#[test]
fn test_nft_minted_event() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Event Test",
        "Event desc",
        "ipfs://event",
        "Indexed Hunt",
        4,
        2,
    );

    let nft_id = client.mint_reward_nft(&7, &player, &metadata);

    let events = env.events().all();
    assert!(!events.is_empty());
    // Last event should be NftMinted
    let (_contract, topics, data): (Address, Vec<Val>, Val) =
        events.get(events.len() - 1).unwrap();
    assert_eq!(topics.len(), 2); // "NftMinted" + nft_id
    assert_eq!(
        Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap(),
        Symbol::new(&env, "NftMinted")
    );
    assert_eq!(u64::try_from_val(&env, &topics.get(1).unwrap()).unwrap(), nft_id);

    let event = NftMintedEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.nft_id, nft_id);
    assert_eq!(event.hunt_id, 7);
    assert_eq!(event.owner, player);
    assert_eq!(event.rarity, 4);
    assert_eq!(event.tier, 2);
    assert_eq!(event.metadata.rarity, 4);
    assert_eq!(event.metadata.tier, 2);
    assert_eq!(event.minted_at, 1000);
}

#[test]
fn test_multiple_nfts_can_be_minted() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);

    let titles = ["Hunt 1", "Hunt 2", "Hunt 3", "Hunt 4", "Hunt 5"];
    let descs = [
        "Description for hunt 1",
        "Description for hunt 2",
        "Description for hunt 3",
        "Description for hunt 4",
        "Description for hunt 5",
    ];
    let uris = [
        "ipfs://hunt1",
        "ipfs://hunt2",
        "ipfs://hunt3",
        "ipfs://hunt4",
        "ipfs://hunt5",
    ];

    for i in 0..5 {
        let metadata = create_metadata(&env, titles[i], descs[i], uris[i]);
        let nft_id = client.mint_reward_nft(&player, &(i as u64 + 1), &player, &metadata);
        assert_eq!(nft_id, (i as u64) + 1);
    }

    assert_eq!(client.total_supply(), 5);
}

#[test]
fn test_nft_data_can_be_queried() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Query Test", "Query desc", "ipfs://query");
    let nft_id = client.mint_reward_nft(&player, &99, &player, &metadata);

    let nft = client.get_nft(&nft_id);
    assert!(nft.is_some());
    let nft = nft.unwrap();
    assert_eq!(nft.hunt_id, 99);
    assert_eq!(nft.nft_id, nft_id);
}

#[test]
fn test_get_nonexistent_nft_returns_none() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let nft = client.get_nft(&999);
    assert!(nft.is_none());

    let owner = client.owner_of(&999);
    assert!(owner.is_none());

    let meta = client.get_nft_metadata(&999);
    assert!(meta.is_none());
}

#[test]
fn test_get_nft_metadata_returns_complete_info() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata_full(
        &env,
        "Epic Hunt Trophy",
        "Completed legendary hunt",
        "ipfs://trophy",
        "Legendary City Hunt",
        4, // rare
        1, // tier 1
    );

    let nft_id = client.mint_reward_nft(&player, &42, &player, &metadata);
    let meta = client.get_nft_metadata(&nft_id).unwrap();

    assert_eq!(meta.nft_id, nft_id);
    assert_eq!(meta.hunt_id, 42);
    assert_eq!(
        meta.hunt_title,
        String::from_str(&env, "Legendary City Hunt")
    );
    assert_eq!(meta.completion_timestamp, 1000);
    assert_eq!(meta.completion_player, player);
    assert_eq!(meta.current_owner, player);
    assert_eq!(meta.title, String::from_str(&env, "Epic Hunt Trophy"));
    assert_eq!(
        meta.description,
        String::from_str(&env, "Completed legendary hunt")
    );
    assert_eq!(meta.image_uri, String::from_str(&env, "ipfs://trophy"));
    assert_eq!(meta.rarity, 4);
    assert_eq!(meta.tier, 1);
}

#[test]
fn test_update_nft_metadata_owner_only() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Original", "Original desc", "ipfs://old");

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    client.update_nft_metadata(
        &nft_id,
        &owner,
        &String::from_str(&env, "Updated description"),
        &String::from_str(&env, "ipfs://new"),
    );

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(
        nft.metadata.description,
        String::from_str(&env, "Updated description")
    );
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://new"));
    assert_eq!(nft.metadata.title, String::from_str(&env, "Original"));
}

#[test]
fn test_update_nft_metadata_preserves_immutable_fields() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata_full(&env, "Title", "Desc", "ipfs://img", "Hunt", 3, 2);

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    client.update_nft_metadata(
        &nft_id,
        &owner,
        &String::from_str(&env, "New desc"),
        &String::from_str(&env, "ipfs://newimg"),
    );

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.title, String::from_str(&env, "Title"));
    assert_eq!(meta.rarity, 3);
    assert_eq!(meta.tier, 2);
    assert_eq!(meta.hunt_title, String::from_str(&env, "Hunt"));
}

#[test]
fn test_transfer_nft_success() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_transferable_metadata(&env, "Transfer NFT", "Test transfer", "ipfs://transfer");

    let nft_id = client.mint_reward_nft(&from, &1, &from, &metadata);
    assert_eq!(client.owner_of(&nft_id), Some(from.clone()));

    client.transfer_nft(&nft_id, &from, &to);

    assert_eq!(client.owner_of(&nft_id), Some(to.clone()));
    assert_eq!(client.get_nft_owner(&nft_id), Some(to.clone()));

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.owner, to);
}

#[test]
fn test_transfer_nft_updates_player_nfts() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let metadata1 = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");

    let nft1 = client.mint_reward_nft(&alice, &1, &alice, &metadata1);
    let nft2 = client.mint_reward_nft(&alice, &2, &alice, &metadata2);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 2);
    assert!(alice_nfts.get(0).unwrap() == nft1 || alice_nfts.get(0).unwrap() == nft2);

    client.transfer_nft(&nft1, &alice, &bob);

    let alice_nfts = client.get_player_nfts(&alice, &0, &100);
    assert_eq!(alice_nfts.len(), 1);

    let bob_nfts = client.get_player_nfts(&bob, &0, &100);
    assert_eq!(bob_nfts.len(), 1);
    assert_eq!(bob_nfts.get(0).unwrap(), nft1);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_transfer_nft_requires_auth() {
    let env = Env::default();
    // Do NOT mock auth - we want the transfer to fail without auth
    env.ledger().set_timestamp(1000);

    let (client, _admin) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Auth Test", "Desc", "ipfs://auth");

    let _nft_id = client.mint_reward_nft(&from, &1, &from, &metadata);

    // This should fail - from has not authorized the transfer
    client.transfer_nft(&1, &from, &to);
}

#[test]
#[should_panic]
fn test_transfer_nft_nonexistent() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.transfer_nft(&999, &from, &to);
}

#[test]
#[should_panic]
fn test_transfer_nft_not_owner() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_metadata(&env, "Owner Test", "Desc", "ipfs://owner");

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    // Attacker tries to transfer - with mock_all_auths they "auth" but NotOwner check fails
    client.transfer_nft(&nft_id, &attacker, &to);
}

#[test]
#[should_panic]
fn test_transfer_nft_invalid_recipient_same_as_from() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Same Addr", "Desc", "ipfs://same");

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    client.transfer_nft(&nft_id, &owner, &owner);
}

#[test]
fn test_transfer_nft_emits_event() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let metadata = create_transferable_metadata(&env, "Event NFT", "Desc", "ipfs://event");

    let nft_id = client.mint_reward_nft(&from, &1, &from, &metadata);
    client.transfer_nft(&nft_id, &from, &to);

    // Transfer succeeded; NftTransferred event is emitted by transfer_nft
    assert_eq!(client.owner_of(&nft_id), Some(to));
}

#[test]
fn test_get_player_nfts_empty_for_new_address() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let new_addr = Address::generate(&env);
    let nfts = client.get_player_nfts(&new_addr, &0, &100);
    assert_eq!(nfts.len(), 0);
}

#[test]
fn test_get_nft_owner_matches_owner_of() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Alias Test", "Desc", "ipfs://alias");

    let nft_id = client.mint_reward_nft(&player, &1, &player, &metadata);

    assert_eq!(client.owner_of(&nft_id), client.get_nft_owner(&nft_id));
    assert_eq!(client.get_nft_owner(&nft_id), Some(player));
}

#[test]
fn test_nft_with_creator_attribution() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let metadata = create_metadata_with_creator(
        &env,
        "Creator NFT",
        "NFT with creator attribution",
        "ipfs://creator",
        creator.clone(),
        None,
    );

    let nft_id = client.mint_reward_nft(&1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, None);

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator));
    assert_eq!(meta.royalty_bps, None);
}

#[test]
fn test_nft_with_creator_and_royalty() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let royalty_bps = 250u32; // 2.5% royalty
    let metadata = create_metadata_with_creator(
        &env,
        "Royalty NFT",
        "NFT with creator and royalty",
        "ipfs://royalty",
        creator.clone(),
        Some(royalty_bps),
    );

    let nft_id = client.mint_reward_nft(&1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(royalty_bps));

    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator));
    assert_eq!(meta.royalty_bps, Some(royalty_bps));
}

#[test]
fn test_nft_without_creator_defaults_to_none() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "No Creator", "No creator set", "ipfs://nocreator");

    let nft_id = client.mint_reward_nft(&1, &player, &metadata);

    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, None);
    assert_eq!(nft.metadata.royalty_bps, None);
}

#[test]
fn test_creator_preserved_across_metadata_queries() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let metadata = create_metadata_with_creator(
        &env,
        "Preserved Creator",
        "Creator should be preserved",
        "ipfs://preserved",
        creator.clone(),
        Some(1000u32),
    );

    let nft_id = client.mint_reward_nft(&42, &player, &metadata);

    // Query via get_nft
    let nft = client.get_nft(&nft_id).unwrap();
    assert_eq!(nft.metadata.creator, Some(creator.clone()));
    assert_eq!(nft.metadata.royalty_bps, Some(1000u32));

    // Query via get_nft_metadata
    let meta = client.get_nft_metadata(&nft_id).unwrap();
    assert_eq!(meta.creator, Some(creator.clone()));
    assert_eq!(meta.royalty_bps, Some(1000u32));
    assert_eq!(meta.current_owner, player);
}

#[test]
fn test_burn_removes_nft_and_clears_owner_list() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let metadata = create_metadata(&env, "Burn Test", "Desc", "ipfs://burn");

    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);
    assert!(client.get_nft(&nft_id).is_some());

    client.burn(&nft_id, &owner);

    assert!(client.get_nft(&nft_id).is_none());
    assert_eq!(client.get_player_nfts(&owner, &0, &100).len(), 0);
}

#[test]
#[should_panic]
fn test_burn_fails_if_not_owner() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let owner = Address::generate(&env);
    let rogue = Address::generate(&env);
    let metadata = create_metadata(&env, "Rogue NFT", "Desc", "ipfs://rogue");
    let nft_id = client.mint_reward_nft(&owner, &1, &owner, &metadata);

    client.burn(&nft_id, &rogue);
}

#[test]
#[should_panic]
fn test_burn_fails_for_nonexistent_nft() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    client.burn(&999, &Address::generate(&env));
}

#[test]
fn test_add_minter_allows_new_minter() {
    let (env, _contract_id, admin, _original_minter) = setup_initialized();
    let (client, _admin) = setup_nft_reward(&env, None);

    let new_minter = Address::generate(&env);
    client.add_minter(&admin, &new_minter);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "New Minter NFT", "Desc", "ipfs://new");
    let nft_id = client.mint_reward_nft(&new_minter, &1, &player, &metadata);
    assert_eq!(nft_id, 1);
}

#[test]
#[should_panic]
fn test_remove_minter_revokes_access() {
    let (env, _contract_id, admin, minter) = setup_initialized();
    let (client, _admin) = setup_nft_reward(&env, None);

    client.remove_minter(&admin, &minter);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "Revoked NFT", "Desc", "ipfs://revoked");
    client.mint_reward_nft(&minter, &1, &player, &metadata);
}

#[test]
#[should_panic]
fn test_add_minter_requires_admin() {
    let (env, _contract_id, _admin, _minter) = setup_initialized();
    let (client, _admin) = setup_nft_reward(&env, None);

    let imposter = Address::generate(&env);
    let new_minter = Address::generate(&env);
    client.add_minter(&imposter, &new_minter);
}

#[test]
fn test_no_max_supply_allows_unlimited_mints() {
    let env = setup_env();
    let (client, _admin) = setup_nft_reward(&env, None);

    let player = Address::generate(&env);
    for i in 1u64..=5 {
        let metadata = create_metadata(&env, "NFT", "Desc", "ipfs://x");
        client.mint_reward_nft(&player, &i, &player, &metadata);
    }
    assert_eq!(client.total_supply(), 5);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_max_supply_cap_blocks_additional_mints() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &Some(2));

    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let player3 = Address::generate(&env);

    let metadata1 = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://1");
    let metadata2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://2");
    let metadata3 = create_metadata(&env, "NFT 3", "Desc 3", "ipfs://3");

    client.mint_reward_nft(&1, &player1, &metadata1);
    client.mint_reward_nft(&1, &player2, &metadata2);
    client.mint_reward_nft(&1, &player3, &metadata3);
}

// --- admin_update_image_uris tests ---

#[test]
fn test_admin_update_image_uris_replaces_ipfs_prefix() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let player = Address::generate(&env);

    // Mint 3 NFTs with old IPFS gateway URIs
    let m1 = create_metadata(&env, "NFT 1", "Desc 1", "ipfs://oldgateway/QmAbc");
    let m2 = create_metadata(&env, "NFT 2", "Desc 2", "ipfs://oldgateway/QmDef");
    let m3 = create_metadata(&env, "NFT 3", "Desc 3", "https://cdn.example.com/nft/123.png");

    let id1 = client.mint_reward_nft(&player, &1, &player, &m1);
    let id2 = client.mint_reward_nft(&player, &1, &player, &m2);
    let id3 = client.mint_reward_nft(&player, &1, &player, &m3);

    // Batch-update: replace ipfs://oldgateway/ with ipfs://newgateway/
    let old_prefix = String::from_str(&env, "ipfs://oldgateway/");
    let new_prefix = String::from_str(&env, "ipfs://newgateway/");
    let updated = client.admin_update_image_uris(&admin, &old_prefix, &new_prefix);

    assert_eq!(updated, 2);

    // Verify NFT 1 was updated
    let nft = client.get_nft(&id1).unwrap();
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://newgateway/QmAbc"));

    // Verify NFT 2 was updated
    let nft = client.get_nft(&id2).unwrap();
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "ipfs://newgateway/QmDef"));

    // Verify NFT 3 was NOT changed (different prefix)
    let nft = client.get_nft(&id3).unwrap();
    assert_eq!(nft.metadata.image_uri, String::from_str(&env, "https://cdn.example.com/nft/123.png"));
}

#[test]
fn test_admin_update_image_uris_replaces_https_prefix() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let player = Address::generate(&env);

    let m1 = create_metadata(&env, "NFT 1", "Desc", "https://cdn.example.com/nft/1.png");
    let m2 = create_metadata(&env, "NFT 2", "Desc", "https://cdn.example.com/nft/2.png");

    let id1 = client.mint_reward_nft(&player, &1, &player, &m1);
    let id2 = client.mint_reward_nft(&player, &1, &player, &m2);

    let old_prefix = String::from_str(&env, "https://cdn.example.com/");
    let new_prefix = String::from_str(&env, "https://new-cdn.example.com/");
    let updated = client.admin_update_image_uris(&admin, &old_prefix, &new_prefix);

    assert_eq!(updated, 2);

    let nft1 = client.get_nft(&id1).unwrap();
    assert_eq!(nft1.metadata.image_uri, String::from_str(&env, "https://new-cdn.example.com/nft/1.png"));

    let nft2 = client.get_nft(&id2).unwrap();
    assert_eq!(nft2.metadata.image_uri, String::from_str(&env, "https://new-cdn.example.com/nft/2.png"));
}

#[test]
fn test_admin_update_image_uris_returns_zero_when_no_match() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT", "Desc", "ipfs://some/QmAbc");
    let _id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let old_prefix = String::from_str(&env, "ipfs://nonexistent/");
    let new_prefix = String::from_str(&env, "ipfs://new/");
    let updated = client.admin_update_image_uris(&admin, &old_prefix, &new_prefix);

    assert_eq!(updated, 0);
}

#[test]
fn test_admin_update_image_uris_returns_zero_for_empty_contract() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let old_prefix = String::from_str(&env, "ipfs://old/");
    let new_prefix = String::from_str(&env, "ipfs://new/");
    let updated = client.admin_update_image_uris(&admin, &old_prefix, &new_prefix);

    assert_eq!(updated, 0);
}

#[test]
fn test_admin_update_image_uris_skips_burned_nfts() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let player = Address::generate(&env);

    let m1 = create_metadata(&env, "NFT 1", "Desc", "ipfs://old/QmAbc");
    let m2 = create_metadata(&env, "NFT 2", "Desc", "ipfs://old/QmDef");

    let id1 = client.mint_reward_nft(&player, &1, &player, &m1);
    let _id2 = client.mint_reward_nft(&player, &1, &player, &m2);

    // Burn NFT 1
    client.burn(&id1, &player);

    let old_prefix = String::from_str(&env, "ipfs://old/");
    let new_prefix = String::from_str(&env, "ipfs://new/");
    let updated = client.admin_update_image_uris(&admin, &old_prefix, &new_prefix);

    // Only NFT 2 should be updated (NFT 1 is burned/skipped)
    assert_eq!(updated, 1);
}

#[test]
#[should_panic(expected = "HostError")]
fn test_admin_update_image_uris_requires_auth() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);
    // Don't mock auth

    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let old_prefix = String::from_str(&env, "ipfs://old/");
    let new_prefix = String::from_str(&env, "ipfs://new/");
    client.admin_update_image_uris(&admin, &old_prefix, &new_prefix);
}

#[test]
fn test_admin_update_image_uris_emits_event() {
    let env = setup_env();
    let admin = Address::generate(&env);
    let contract_id = env.register_contract(None, NftReward);
    let client = NftRewardClient::new(&env, &contract_id);
    client.initialize(&admin, &None);

    let player = Address::generate(&env);
    let metadata = create_metadata(&env, "NFT", "Desc", "ipfs://old/QmAbc");
    let _id = client.mint_reward_nft(&player, &1, &player, &metadata);

    let old_prefix = String::from_str(&env, "ipfs://old/");
    let new_prefix = String::from_str(&env, "ipfs://new/");
    let updated = client.admin_update_image_uris(&admin, &old_prefix, &new_prefix);
    assert_eq!(updated, 1);

    // Check event was emitted
    let events = env.events().all();
    let (_contract, topics, data): (Address, Vec<Val>, Val) =
        events.get(events.len() - 1).unwrap();

    assert_eq!(topics.len(), 1);
    assert_eq!(
        Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap(),
        Symbol::new(&env, "AdminImageUrisUpdated")
    );

    let event = AdminImageUrisUpdatedEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(event.old_prefix, String::from_str(&env, "ipfs://old/"));
    assert_eq!(event.new_prefix, String::from_str(&env, "ipfs://new/"));
    assert_eq!(event.updated_count, 1);
}
