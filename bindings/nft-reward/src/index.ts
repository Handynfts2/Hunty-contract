import { ContractSpec } from "@stellar/stellar-sdk";
import type {
  AssembledTransaction,
  ContractClientOptions,
} from "@stellar/stellar-sdk/contract";
import { ContractClient } from "@stellar/stellar-sdk/contract";

export const networks = {
  testnet: { networkPassphrase: "Test SDF Network ; September 2015" },
  mainnet: { networkPassphrase: "Public Global Stellar Network ; September 2015" },
} as const;

export interface NftMetadata {
  title: string;
  description: string;
  image_uri: string;
  hunt_title: string;
  rarity: number;
  tier: number;
  creator?: string;
  royalty_bps?: number;
}

export interface NftMetadataResponse {
  nft_id: bigint;
  hunt_id: bigint;
  hunt_title: string;
  completion_timestamp: bigint;
  completion_player: string;
  current_owner: string;
  title: string;
  description: string;
  image_uri: string;
  rarity: number;
  tier: number;
  creator?: string;
  royalty_bps?: number;
}

export interface NftData {
  nft_id: bigint;
  hunt_id: bigint;
  owner: string;
  completion_player: string;
  metadata: NftMetadata;
  transferable: boolean;
  minted_at: bigint;
}

export interface AdminImageUrisUpdatedEvent {
  old_prefix: string;
  new_prefix: string;
  updated_count: number;
}

export class Client extends ContractClient {
  constructor(public readonly options: ContractClientOptions) {
    super(new ContractSpec([]), options);
  }

  async initialize({
    admin,
    max_supply,
  }: {
    admin: string;
    max_supply?: bigint;
  }): Promise<AssembledTransaction<void>> {
    return this.call("initialize", admin, max_supply);
  }

  async get_admin(): Promise<AssembledTransaction<string | undefined>> {
    return this.call("get_admin");
  }

  async set_reward_manager({
    admin,
    reward_manager,
  }: {
    admin: string;
    reward_manager: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("set_reward_manager", admin, reward_manager);
  }

  async admin_update_image_uris({
    admin,
    old_prefix,
    new_prefix,
  }: {
    admin: string;
    old_prefix: string;
    new_prefix: string;
  }): Promise<AssembledTransaction<number>> {
    return this.call("admin_update_image_uris", admin, old_prefix, new_prefix);
  }

  async mint_reward_nft({
    hunt_id,
    player_address,
    metadata,
  }: {
    hunt_id: bigint;
    player_address: string;
    metadata: NftMetadata;
  }): Promise<AssembledTransaction<bigint>> {
    return this.call("mint_reward_nft", hunt_id, player_address, metadata);
  }

  async mint_reward_nft_from_map({
    hunt_id,
    player_address,
    metadata,
  }: {
    hunt_id: bigint;
    player_address: string;
    metadata: Map<string, any>;
  }): Promise<AssembledTransaction<bigint>> {
    return this.call("mint_reward_nft_from_map", hunt_id, player_address, metadata);
  }

  async get_nft({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<NftData | undefined>> {
    return this.call("get_nft", nft_id);
  }

  async get_nft_metadata({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<NftMetadataResponse | undefined>> {
    return this.call("get_nft_metadata", nft_id);
  }

  async update_nft_metadata({
    nft_id,
    updater,
    new_description,
    new_image_uri,
  }: {
    nft_id: bigint;
    updater: string;
    new_description: string;
    new_image_uri: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("update_nft_metadata", nft_id, updater, new_description, new_image_uri);
  }

  async total_supply(): Promise<AssembledTransaction<bigint>> {
    return this.call("total_supply");
  }

  async owner_of({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<string | undefined>> {
    return this.call("owner_of", nft_id);
  }

  async get_nft_owner({
    nft_id,
  }: {
    nft_id: bigint;
  }): Promise<AssembledTransaction<string | undefined>> {
    return this.call("get_nft_owner", nft_id);
  }

  async get_player_nfts({
    owner,
    offset,
    limit,
  }: {
    owner: string;
    offset: number;
    limit: number;
  }): Promise<AssembledTransaction<bigint[]>> {
    return this.call("get_player_nfts", owner, offset, limit);
  }

  async burn({
    nft_id,
    owner,
  }: {
    nft_id: bigint;
    owner: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("burn", nft_id, owner);
  }

  async transfer_nft({
    nft_id,
    from_address,
    to_address,
  }: {
    nft_id: bigint;
    from_address: string;
    to_address: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("transfer_nft", nft_id, from_address, to_address);
  }

  async add_minter({
    admin,
    minter,
  }: {
    admin: string;
    minter: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("add_minter", admin, minter);
  }

  async remove_minter({
    admin,
    minter,
  }: {
    admin: string;
    minter: string;
  }): Promise<AssembledTransaction<void>> {
    return this.call("remove_minter", admin, minter);
  }

  async contract_version(): Promise<AssembledTransaction<number>> {
    return this.call("contract_version");
  }
}
