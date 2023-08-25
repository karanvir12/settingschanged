// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of peer.

// peer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// peer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with peer.  If not, see <http://www.gnu.org/licenses/>.

//! peer chain configurations.

use beefy_primitives::crypto::AuthorityId as BeefyId;
use frame_support::weights::Weight;
use grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_staking::Forcing;
use peer_primitives::v2::{AccountId, AccountPublic, AssignmentId, ValidatorId};
#[cfg(feature = "peer-native")]
use Peer_Runtime as peer;
#[cfg(feature = "peer-native")]
use Peer_Runtime_constants::currency::UNITS as peer;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;

use Peer_Runtime::EVMChainIdConfig;
use Peer_Runtime::EvmConfig;
use Peer_Runtime::EthereumConfig;
use std::collections::BTreeMap;
use sp_core::{H160, U256};
use std::str::FromStr;

use sc_chain_spec::{ChainSpecExtension, ChainType};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{traits::IdentifyAccount, Perbill};
use telemetry::TelemetryEndpoints;


#[cfg(feature = "peer-native")]
const PEER_STAGING_TELEMETRY_URL: &str = "wss://telemetry.peer.io/submit/";

const DEFAULT_PROTOCOL_ID: &str = "Peer";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<peer_primitives::v2::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<peer_primitives::v2::Block>,
	/// The light sync state.
	///
	/// This value will be set by the `sync-state rpc` implementation.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// The `ChainSpec` parameterized for the peer runtime.
#[cfg(feature = "peer-native")]
pub type peerChainSpec = service::GenericChainSpec<peer::GenesisConfig, Extensions>;

// Dummy chain spec, in case when we don't have the native runtime.
pub type DummyChainSpec = service::GenericChainSpec<(), Extensions>;

// Dummy chain spec, but that is fine when we don't have the native runtime.
#[cfg(not(feature = "peer-native"))]
pub type peerChainSpec = DummyChainSpec;
/// The `ChainSpec` parameterized for the `versi` runtime.
///
//

pub fn peer_config() -> Result<peerChainSpec, String> {
	peerChainSpec::from_json_bytes(&include_bytes!("../chain-specs/peer.json")[..])
}


/// The default parachains host configuration.
#[cfg(any(
	feature = "peer-native"
))]
fn default_parachains_host_configuration(
) -> Peer_Runtime_parachains::configuration::HostConfiguration<
	peer_primitives::v2::BlockNumber,
> {
	use peer_primitives::v2::{MAX_CODE_SIZE, MAX_POV_SIZE};

	Peer_Runtime_parachains::configuration::HostConfiguration {
		validation_upgrade_cooldown: 2u32,
		validation_upgrade_delay: 2,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		chain_availability_period: 4,
		thread_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024 * 1024,
		ump_service_total_weight: Weight::from_ref_time(100_000_000_000)
			.set_proof_size(MAX_POV_SIZE as u64),
		max_upward_message_size: 50 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_max_parathread_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_parathread_outbound_channels: 4,
		hrmp_max_message_num_per_candidate: 5,
		dispute_period: 6,
		no_show_slots: 2,
		n_delay_tranches: 25,
		needed_approvals: 2,
		relay_vrf_modulo_samples: 2,
		zeroth_delay_tranche_width: 0,
		minimum_validation_upgrade_delay: 5,
		..Default::default()
	}
}

pub fn peer_chain_spec_properties() -> serde_json::map::Map<String, serde_json::Value> {
	serde_json::json!({
		"tokenDecimals": 18,
		"tokenSymbol":"PEER",
	})
	.as_object()
	.expect("Map given; qed")
	.clone()
}

// peer staging testnet config.
#[cfg(feature = "peer-native")]
pub fn peer_staging_testnet_config() -> Result<peerChainSpec, String> {
	let wasm_binary = peer::WASM_BINARY.ok_or("Peer development wasm not available")?;
	let boot_nodes = vec![];

	Ok(peerChainSpec::from_genesis(
		"PEER Staging Testnet",
		"Peer_Stage",
		ChainType::Live,
		move || peer_staging_testnet_config_genesis(wasm_binary, 100),
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(PEER_STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("peer Staging telemetry url is valid; qed"),
		),
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Some(peer_chain_spec_properties()),
		Default::default(),
	))
}

#[cfg(any(

	feature = "peer-native"
))]
#[test]
fn default_parachains_host_configuration_is_consistent() {
	default_parachains_host_configuration().panic_if_not_consistent();
}

#[cfg(feature = "peer-native")]
fn peer_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
) -> peer::SessionKeys {
	peer::SessionKeys {
		babe,
		grandpa,
		im_online,
		para_validator,
		para_assignment,
		authority_discovery,
	}
}

#[cfg(feature = "peer-native")]
fn peer_staging_testnet_config_genesis(wasm_binary: &[u8], chain_id: u64) -> peer::GenesisConfig {
	// subkey inspect "$SECRET"
	let endowed_accounts = vec![];

	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	) > = vec![];

	const ENDOWMENT: u128 = 1_000_000_000 * peer;
	const STASH: u128 = 100 * peer;

	peer::GenesisConfig {
		sudo: peer::SudoConfig {
			key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		},
		system: peer::SystemConfig { code: wasm_binary.to_vec() },
		balances: peer::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.map(|k: &AccountId| (k.clone(), ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		},
		indices: peer::IndicesConfig { indices: vec![] },
		session: peer::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						peer_session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: peer::StakingConfig {
			validator_count: 50,
			minimum_validator_count: 4,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, peer::StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: Forcing::ForceNone,
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		phragmen_election: Default::default(),
		democracy: Default::default(),
		council: peer::CouncilConfig { members: vec![], phantom: Default::default() },
		technical_committee: peer::TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		babe: peer::BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(peer::BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: Default::default(),
		im_online: Default::default(),
		authority_discovery: peer::AuthorityDiscoveryConfig { keys: vec![] },
		claims: peer::ClaimsConfig { claims: vec![], vesting: vec![] },
		vesting: peer::VestingConfig { vesting: vec![] },
		treasury: Default::default(),
		hrmp: Default::default(),
		configuration: peer::ConfigurationConfig {
			config: default_parachains_host_configuration(),
		},
		paras: Default::default(),
		xcm_pallet: Default::default(),
		nomination_pools: Default::default(),

		// EVM compatibility

		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EvmConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0x0")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					// H160 address of CI test runner account
					H160::from_str("cb90cAD4fafD23Eb939c028263520156AA078831")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0x0").expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
		},
		ethereum: EthereumConfig {},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
	}
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	ValidatorId,
	AssignmentId,
	AuthorityDiscoveryId,
	BeefyId,
) {
	let keys = get_authority_keys_from_seed_no_beefy(seed);
	(keys.0, keys.1, keys.2, keys.3, keys.4, keys.5, keys.6, keys.7, get_from_seed::<BeefyId>(seed))
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed_no_beefy(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	ValidatorId,
	AssignmentId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<ValidatorId>(seed),
		get_from_seed::<AssignmentId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn testnet_accounts() -> Vec<AccountId> {
	vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
		get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
	]
}

/// Helper function to create peer `GenesisConfig` for testing
#[cfg(feature = "peer-native")]
pub fn peer_testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	)>,
	_root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	chain_id: u64,
) -> peer::GenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	const ENDOWMENT: u128 = 1_000_000 * peer;
	const STASH: u128 = 100 * peer;

	peer::GenesisConfig {
		system: peer::SystemConfig { code: wasm_binary.to_vec() },
		indices: peer::IndicesConfig { indices: vec![] },
		balances: peer::BalancesConfig {
			balances: endowed_accounts.iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		},
		session: peer::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						peer_session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: peer::StakingConfig {
			minimum_validator_count: 1,
			validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, peer::StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			force_era: Forcing::NotForcing,
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		phragmen_election: Default::default(),
		democracy: peer::DemocracyConfig::default(),
		council: peer::CouncilConfig { members: vec![], phantom: Default::default() },
		//council: peer::CouncilConfig ::default(),

		technical_committee: peer::TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		babe: peer::BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(peer::BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: Default::default(),
		im_online: Default::default(),
		authority_discovery: peer::AuthorityDiscoveryConfig { keys: vec![] },
		claims: peer::ClaimsConfig { claims: vec![], vesting: vec![] },
		vesting: peer::VestingConfig { vesting: vec![] },
		treasury: Default::default(),
		hrmp: Default::default(),
		configuration: peer::ConfigurationConfig {
			config: default_parachains_host_configuration(),
		},
		paras: Default::default(),
		xcm_pallet: Default::default(),
		nomination_pools: Default::default(),
		sudo: peer::SudoConfig {
			key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		},

		// EVM compatibility

		evm_chain_id: EVMChainIdConfig { chain_id },
		evm: EvmConfig {
			accounts: {
				let mut map = BTreeMap::new();
				map.insert(
					// H160 address of Alice dev account
					// Derived from SS58 (42 prefix) address
					// SS58: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
					// hex: 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d
					// Using the full hex key, truncating to the first 20 bytes (the first 40 hex chars)
					H160::from_str("d43593c715fdd31c61141abd04a99fd6822c8558")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0x0")
							.expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map.insert(
					// H160 address of CI test runner account
					H160::from_str("cb90cAD4fafD23Eb939c028263520156AA078831")
						.expect("internal H160 is valid; qed"),
					fp_evm::GenesisAccount {
						balance: U256::from_str("0x0").expect("internal U256 is valid; qed"),
						code: Default::default(),
						nonce: Default::default(),
						storage: Default::default(),
					},
				);
				map
			},
		},
		ethereum: EthereumConfig {},
		dynamic_fee: Default::default(),
		base_fee: Default::default(),
	}
}

#[cfg(feature = "peer-native")]
fn peer_development_config_genesis(wasm_binary: &[u8]) -> peer::GenesisConfig {
	peer_testnet_genesis(
		wasm_binary,
		vec![get_authority_keys_from_seed_no_beefy("Alice")],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		100,
	)
}

/// peer development config (single validator Alice)
#[cfg(feature = "peer-native")]
pub fn peer_development_config() -> Result<peerChainSpec, String> {
	let wasm_binary = peer::WASM_BINARY.ok_or("peer development wasm not available")?;

	Ok(peerChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		move || peer_development_config_genesis(wasm_binary),
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Some(peer_chain_spec_properties()),
		Default::default(),
	))
}


#[cfg(feature = "peer-native")]
fn peer_local_testnet_genesis(wasm_binary: &[u8]) -> peer::GenesisConfig {
	peer_testnet_genesis(
		wasm_binary,
		vec![
			get_authority_keys_from_seed_no_beefy("Alice"),
			get_authority_keys_from_seed_no_beefy("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		100,
	)
}

/// peer local testnet config (multivalidator Alice + Bob)
#[cfg(feature = "peer-native")]
pub fn peer_local_testnet_config() -> Result<peerChainSpec, String> {
	let wasm_binary = peer::WASM_BINARY.ok_or("peer development wasm not available")?;

	Ok(peerChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		move || peer_local_testnet_genesis(wasm_binary),
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Some(peer_chain_spec_properties()),
		Default::default(),
	))
}
