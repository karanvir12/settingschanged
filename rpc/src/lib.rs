// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

//! peer-specific RPCs implementation.

#![warn(missing_docs)]

use std::sync::Arc;

use fc_mapping_sync::MappingSyncWorker;
use jsonrpsee::RpcModule;
use peer_primitives::v2::{AccountId, Balance, Block, BlockNumber, Hash, Nonce};
use sc_client_api::AuxStore;
use sc_consensus_babe::{BabeConfiguration, Epoch};
use sc_finality_grandpa::FinalityProofProvider;
pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sp_api::{ProvideRuntimeApi, BlockT, HeaderT};
use std::time::Duration;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_keystore::SyncCryptoStorePtr;
use txpool_api::TransactionPool;
use sc_client_api::BlockOf;
use std::collections::BTreeMap;
use fc_rpc::EthTask;
use sc_client_api::{	
	backend::{ Backend, StateBackend, StorageProvider},	
	client::BlockchainEvents,	
};	
//=============================================	
use sp_runtime::{traits::BlakeTwo256, testing::H256};	
use sc_transaction_pool::{ChainApi, Pool};	
use sc_network::NetworkService;	
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};	
// Frontier	
use fc_rpc::{	
	EthBlockDataCacheTask, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,	
	SchemaV2Override, SchemaV3Override, StorageOverride,	
};	
use fp_storage::EthereumStorageSchema;

/// A type representing all RPC extensions.
pub type RpcExtension = RpcModule<()>;

/// Extra dependencies for BABE.
pub struct BabeDeps {
	/// BABE protocol config.
	pub babe_config: BabeConfiguration,
	/// BABE pending epoch changes.
	pub shared_epoch_changes: sc_consensus_epochs::SharedEpochChanges<Block, Epoch>,
	/// The keystore that manages the keys of the node.
	pub keystore: SyncCryptoStorePtr,
}

/// Dependencies for GRANDPA
pub struct GrandpaDeps<B> {
	/// Voting round info.
	pub shared_voter_state: sc_finality_grandpa::SharedVoterState,
	/// Authority set info.
	pub shared_authority_set: sc_finality_grandpa::SharedAuthoritySet<Hash, BlockNumber>,
	/// Receives notifications about justification events from Grandpa.
	pub justification_stream: sc_finality_grandpa::GrandpaJustificationStream<Block>,
	/// Executor to drive the subscription manager in the Grandpa RPC handler.
	pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
	/// Finality proof provider.
	pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

use beefy_gadget::communication::notification::{
	BeefyBestBlockStream, BeefyVersionedFinalityProofStream,
};
/// Dependencies for BEEFY
pub struct BeefyDeps {
	/// Receives notifications about finality proof events from BEEFY.
	pub beefy_finality_proof_stream: BeefyVersionedFinalityProofStream<Block>,
	/// Receives notifications about best block events from BEEFY.
	pub beefy_best_block_stream: BeefyBestBlockStream<Block>,
	/// Executor to drive the subscription manager in the BEEFY RPC handler.
	pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
}

/// Full client dependencies
pub struct FullDeps<C, P, SC, B,A:ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The [`SelectChain`] Strategy
	pub select_chain: SC,
	/// A copy of the chain spec.
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<B>,
	/// BEEFY specific dependencies.
	pub beefy: BeefyDeps,


	/// Graph pool instance.	
	pub graph: Arc<Pool<A>>,	
	/// The Node authority flag	
	pub is_authority: bool,	
	/// Whether to enable dev signer	
	pub enable_dev_signer: bool,	
	/// Network service	
	pub network: Arc<NetworkService<Block, Hash>>,	
	/// EthFilterApi pool.	
	pub filter_pool: Option<FilterPool>,	
	/// Backend.	
	pub backend: Arc<fc_db::Backend<Block>>,	
	/// Maximum number of logs in a query.	
	pub max_past_logs: u32,	
	/// Fee history cache.	
	pub fee_history_cache: FeeHistoryCache,	
	/// Maximum fee history cache size.	
	pub fee_history_cache_limit: FeeHistoryCacheLimit,	
	/// Ethereum data access overrides.	
	pub overrides: Arc<OverrideHandle<Block>>,	
	/// Cache for Ethereum block data.	
	pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,	
	pub execute_gas_limit_multiplier: u64,
}

/// Override  extensions.	
pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>	
where	
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,	
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,	
	C: Send + Sync + 'static,	
	C::Api: sp_api::ApiExt<Block>	
		+ fp_rpc::EthereumRuntimeRPCApi<Block>	
		+ fp_rpc::ConvertTransactionRuntimeApi<Block>,	
	BE: Backend<Block> + 'static,	
	BE::State: StateBackend<BlakeTwo256>,	
{	
	let mut overrides_map = BTreeMap::new();	
//	let mut overrides_map = BTreeMap::new();

	overrides_map.insert(	
		EthereumStorageSchema::V1,	
		Box::new(SchemaV1Override::new(client.clone()))	
			as Box<dyn StorageOverride<_> + Send + Sync>,	
	);	
	overrides_map.insert(	
		EthereumStorageSchema::V2,	
		Box::new(SchemaV2Override::new(client.clone()))	
			as Box<dyn StorageOverride<_> + Send + Sync>,	
	);	
	overrides_map.insert(	
		EthereumStorageSchema::V3,	
		Box::new(SchemaV3Override::new(client.clone()))	
			as Box<dyn StorageOverride<_> + Send + Sync>,	
	);	
	Arc::new(OverrideHandle {	
		schemas: overrides_map,	
		fallback: Box::new(RuntimeApiStorageOverride::new(client)),	
	})
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, SC, B,BE,A>(
	deps: FullDeps<C, P, SC, B,A>,
	subscription_task_executor: SubscriptionTaskExecutor,
	_backend: Arc<B>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where

BE: Backend<Block> + 'static,	
BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Sync
		+ Send
		+ StorageProvider<Block, BE>	
		+ 'static,
		C: BlockchainEvents<Block>,	
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,	
	C: Send + Sync + 'static,
	C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	// C::Api: mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash, BlockNumber>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BabeApi<Block>,
	C::Api: BlockBuilder<Block>,
	// P: TransactionPool + Sync + Send + 'static,
	P: TransactionPool<Block=Block> + 'static,
	SC: SelectChain<Block> + 'static,
	C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,	
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,

	B: sc_client_api::Backend<Block> + Send + Sync + 'static,
	B::State: sc_client_api::StateBackend<sp_runtime::traits::HashFor<Block>>,
	A: ChainApi<Block = Block> + 'static,	
{
	use beefy_gadget_rpc::{Beefy, BeefyApiServer};
	use frame_rpc_system::{System, SystemApiServer};
	// use mmr_rpc::{Mmr, MmrApiServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use sc_consensus_babe_rpc::{Babe, BabeApiServer};
	use sc_finality_grandpa_rpc::{Grandpa, GrandpaApiServer};
	use sc_sync_state_rpc::{SyncState, SyncStateApiServer};
	//use substrate_state_trie_migration_rpc::{StateMigration, StateMigrationApiServer};
	pub use substrate_state_trie_migration_rpc::{StateMigration, StateMigrationApiServer};	
	use fc_rpc::{	
		Eth,  EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub,EthPubSubApiServer,	
		 EthSigner, Net, NetApiServer, Web3, Web3ApiServer,EthApiServer	
	};

	let mut io = RpcModule::new(());
	// let FullDeps { client, pool, select_chain, chain_spec, deny_unsafe, babe, grandpa, beefy } =
	// 	deps;

	let FullDeps { client, pool, select_chain, chain_spec, deny_unsafe, babe, grandpa,graph,	
		is_authority,	
		enable_dev_signer,	
		network,	
		filter_pool,	
		backend,	
		max_past_logs,	
		fee_history_cache,	
		fee_history_cache_limit,	
		overrides,	
		block_data_cache,	
		execute_gas_limit_multiplier,
		beefy } = deps;

	let BabeDeps { keystore, babe_config, shared_epoch_changes } = babe;
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;

		
	let  pp=pool.clone();	
	let  pbp=pool.clone();

	// io.merge(StateMigration::new(client.clone(), backend, deny_unsafe).into_rpc())?;
	io.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
	io.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	// io.merge(Mmr::new(client.clone()).into_rpc())?;
	let mut signers = Vec::new();	
	if enable_dev_signer {	
		signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);	
	}
	io.merge(
		Babe::new(
			client.clone(),
			shared_epoch_changes.clone(),
			keystore,
			babe_config,
			select_chain,
			deny_unsafe,
		)
		.into_rpc(),
	)?;
	io.merge(
		Grandpa::new(
			subscription_executor,
			shared_authority_set.clone(),
			shared_voter_state,
			justification_stream,
			finality_provider,
		)
		.into_rpc(),
	)?;
	io.merge(
		SyncState::new(chain_spec, client.clone(), shared_authority_set, shared_epoch_changes)?.into_rpc(),
	)?;

	io.merge(
		Beefy::<Block>::new(
			beefy.beefy_finality_proof_stream,
			beefy.beefy_best_block_stream,
			beefy.subscription_executor,
		)?
		.into_rpc(),
	)?;
	io.merge(	
		Eth::new(	
			client.clone(),	
			pp,	
			graph,	
			Some(Peer_Runtime::TransactionConverter),	
			network.clone(),	
			signers,	
			overrides.clone(),	
			backend.clone(),	
			// Is authority.	
			is_authority,	
			block_data_cache.clone(),	
			fee_history_cache,	
			fee_history_cache_limit,	
			execute_gas_limit_multiplier	
		)	
		.into_rpc(),	
	)?;	
    if let Some(filter_pool) = filter_pool {	
			
		io.merge(	
			EthFilter::new(	
				client.clone(),	
				backend,	
				filter_pool,	
				500_usize, // max stored filters	
				max_past_logs,	
				block_data_cache,	
			)	
			.into_rpc(),	
		)?;	
	}	
	io.merge(	
		EthPubSub::new(	
			pbp,	
			client.clone(),	
			network.clone(),	
			subscription_task_executor,	
			overrides,	
		)	
		.into_rpc(),	
	)?;	
	// io.merge(Contracts::new(client.clone()).into_rpc())?;	
	io.merge(	
		Net::new(	
			client.clone(),	
			network,	
			// Whether to format the `peer_count` response as Hex (default) or not.	
			true,	
		)	
		.into_rpc(),	
	)?;	
	io.merge(Web3::new(client).into_rpc())?;	
	// io.merge(Dev::new(client, deny_unsafe).into_rpc())?;
	Ok(io)
}

// pub struct SpawnTasksParams<'a, B: BlockT, C, BE> {
// 	pub task_manager: &'a sc_service::TaskManager,
// 	pub client: Arc<C>,
// 	pub substrate_backend: Arc<BE>,
// 	pub frontier_backend: Arc<fc_db::Backend<B>>,
// 	pub filter_pool: Option<FilterPool>,
// 	pub overrides: Arc<OverrideHandle<B>>,
// 	pub fee_history_limit: u64,
// 	pub fee_history_cache: FeeHistoryCache,
// }

// /// Spawn the tasks that are required to run Moonbeam.
// pub fn spawn_frontier_tasks<B, C, BE>(params: SpawnTasksParams<B, C, BE>)
// where
// 	C: ProvideRuntimeApi<B> + BlockOf,
// 	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
// 	C: BlockchainEvents<B> + StorageProvider<B, BE>,
// 	C: Send + Sync + 'static,
// 	C::Api: fp_rpc::EthereumRuntimeRPCApi<B>,
// 	C::Api: BlockBuilder<B>,
// 	B: BlockT<Hash = H256> + Send + Sync + 'static,
// 	B::Header: HeaderT<Number = u32>,
// 	BE: Backend<B> + 'static,
// 	BE::State: StateBackend<BlakeTwo256>,
// {
// 	// Frontier offchain DB task. Essential.
// 	// Maps emulated ethereum data to substrate native data.
// 	params.task_manager.spawn_essential_handle().spawn(
// 		"frontier-mapping-sync-worker",
// 		Some("frontier"),
// 		MappingSyncWorker::new(
// 			client.import_notification_stream(),
// 				Duration::new(6, 0),
// 				client.clone(),
// 				backend,
// 				frontier_backend,
// 				3,
// 				0,
// 				SyncStrategy::Normal,
// 		)
// 		.for_each(|()| futures::future::ready(())),
// 	);

// 	// Frontier `EthFilterApi` maintenance.
// 	// Manages the pool of user-created Filters.
// 	if let Some(filter_pool) = params.filter_pool {
// 		// Each filter is allowed to stay in the pool for 100 blocks.
// 		const FILTER_RETAIN_THRESHOLD: u64 = 100;
// 		params.task_manager.spawn_essential_handle().spawn(
// 			"frontier-filter-pool",
// 			Some("frontier"),
// 			EthTask::filter_pool_task(
// 				Arc::clone(&params.client),
// 				filter_pool,
// 				FILTER_RETAIN_THRESHOLD,
// 			),
// 		);
// 	}

// 	// Spawn Frontier FeeHistory cache maintenance task.
// 	params.task_manager.spawn_essential_handle().spawn(
// 		"frontier-fee-history",
// 		Some("frontier"),
// 		fc_rpc::EthTask::fee_history_task(
// 			Arc::clone(&params.client),
// 			Arc::clone(&params.overrides),
// 			params.fee_history_cache,
// 			params.fee_history_limit,
// 		),
// 	);
// }