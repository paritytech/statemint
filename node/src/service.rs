use cumulus_client_consensus_aura::{
	build_aura_consensus, BuildAuraConsensusParams, SlotProportion,
};
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::{
	ParaId, relay_chain::v1::{Hash as PHash, PersistedValidationData},
};
use cumulus_client_consensus_common::{ParachainConsensus, ParachainCandidate};
use polkadot_primitives::v0::CollatorPair;
use runtime_common::Header;

use sc_client_api::ExecutorProvider;
use sc_executor::native_executor_instance;
use sc_service::{Configuration, PartialComponents, Role, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};
use sp_api::{ConstructRuntimeApi, ApiExt};
use sp_consensus::SlotData;
use sp_consensus_aura::AuraApi;
use sp_runtime::{generic::{self, BlockId}, OpaqueExtrinsic, traits::BlakeTwo256};
use std::sync::Arc;

pub use sc_executor::NativeExecutor;

pub type Block = generic::Block<Header, OpaqueExtrinsic>;

// Native Statemint executor instance.
native_executor_instance!(
	pub StatemintRuntimeExecutor,
	statemint_runtime::api::dispatch,
	statemint_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

// Native Statemine executor instance.
native_executor_instance!(
	pub StatemineRuntimeExecutor,
	statemine_runtime::api::dispatch,
	statemine_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

// Native Westmint executor instance.
native_executor_instance!(
	pub WestmintRuntimeExecutor,
	westmint_runtime::api::dispatch,
	westmint_runtime::native_version,
	frame_benchmarking::benchmarking::HostFunctions,
);

enum BuildOnAccess<R> {
	Uninitialized(Box<dyn FnOnce() -> R + Send + Sync>),
	Initialized(R),
}

impl<R> BuildOnAccess<R> {
	fn get_mut(&mut self) -> &mut R {
		loop {
			match self {
				Self::Uninitialized(f) => {
					*self = Self::Initialized((*f)());
				},
				Self::Initialized(r) => return &mut r,
			}
		}
	}
}

/// Special [`ParachainConsensus`] implementation that waits for the upgrade from
/// shell to a parachain runtime that implements Aura.
struct WaitForAuraConsensus<Client> {
	client: Arc<Client>,
	aura_consensus: BuildOnAccess<Box<dyn ParachainConsensus<Block>>>,
}

impl<Client> Clone for WaitForAuraConsensus<Client> {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			aura_consensus: self.aura_consensus.clone(),
		}
	}
}

#[async_trait::async_trait]
impl<Client> ParachainConsensus<Block> for WaitForAuraConsensus<Client>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
{
	async fn produce_candidate(
		&mut self,
		parent: &Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<Block>> {
		let block_id = BlockId::hash(parent.hash());
		if self.client
			.runtime_api()
			.has_api::<dyn AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>>(&block_id)
			.unwrap_or(false)
		{
			self.aura_consensus
				.get_mut()
				.produce_candidate(
					parent,
					relay_parent,
					validation_data,
				).await
		} else {
			log::debug!("Waiting for runtime with AuRa api");
			None
		}
	}
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi, Executor>(
	config: &Configuration,
) -> Result<
	PartialComponents<
		TFullClient<Block, RuntimeApi, Executor>,
		TFullBackend<Block>,
		(),
		sp_consensus::DefaultImportQueue<Block, TFullClient<Block, RuntimeApi, Executor>>,
		sc_transaction_pool::FullPool<Block, TFullClient<Block, RuntimeApi, Executor>>,
		(Option<Telemetry>, Option<TelemetryWorkerHandle>),
	>,
	sc_service::Error,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, Executor>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<
			Block,
			StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
		> + sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
		)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", worker.run());
		telemetry
	});

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

	let block_import = cumulus_client_consensus_aura::AuraBlockImport::<
		_,
		_,
		_,
		sp_consensus_aura::sr25519::AuthorityPair,
	>::new(client.clone(), client.clone());

	let import_queue = cumulus_client_consensus_aura::import_queue::<
		sp_consensus_aura::sr25519::AuthorityPair,
		_,
		_,
		_,
		_,
		_,
		_,
	>(cumulus_client_consensus_aura::ImportQueueParams {
		block_import,
		client: client.clone(),
		create_inherent_data_providers: move |_, _| async move {
			let time = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
					*time,
					slot_duration.slot_duration(),
				);

			Ok((time, slot))
		},
		registry: config.prometheus_registry().clone(),
		can_author_with: sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
		spawner: &task_manager.spawn_essential_handle(),
		telemetry: telemetry.as_ref().map(|telemetry| telemetry.handle()),
	})?;

	let params = PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: (),
		other: (telemetry, telemetry_worker_handle),
	};

	Ok(params)
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
pub async fn start_node<RuntimeApi, Executor, RB>(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	polkadot_config: Configuration,
	id: ParaId,
	rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, TFullClient<Block, RuntimeApi, Executor>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<
			Block,
			StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
		> + sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>: sp_api::StateBackend<BlakeTwo256>,
	Executor: sc_executor::NativeExecutionDispatch + 'static,
	RB: Fn(
			Arc<TFullClient<Block, RuntimeApi, Executor>>,
		) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
		+ Send
		+ 'static,
{
	if matches!(parachain_config.role, Role::Light) {
		return Err("Light client not supported!".into());
	}

	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial::<RuntimeApi, Executor>(&parachain_config)?;
	let (mut telemetry, telemetry_worker_handle) = params.other;

	let relay_chain_full_node = cumulus_client_service::build_polkadot_full_node(
		polkadot_config,
		collator_key.clone(),
		telemetry_worker_handle,
	)
	.map_err(|e| match e {
		polkadot_service::Error::Sub(x) => x,
		s => format!("{}", s).into(),
	})?;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let block_announce_validator = build_block_announce_validator(
		relay_chain_full_node.client.clone(),
		id,
		Box::new(relay_chain_full_node.network.clone()),
		relay_chain_full_node.backend.clone(),
	);

	let force_authoring = parachain_config.force_authoring;
	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let mut task_manager = params.task_manager;
	let import_queue = params.import_queue;
	let (network, network_status_sinks, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
		})?;

	let rpc_client = client.clone();
	let rpc_extensions_builder = Box::new(move |_, _| rpc_ext_builder(rpc_client.clone()));

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		on_demand: None,
		remote_blockchain: None,
		rpc_extensions_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		network_status_sinks,
		system_rpc_tx,
		telemetry: telemetry.as_mut(),
	})?;

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, data))
	};

	if validator {
		let client2 = client.clone();
		let relay_chain_full_node2 = relay_chain_full_node.clone();

		let parachain_consensus = Box::new(WaitForAuraConsensus {
			client: client.clone(),
			aura_consensus: BuildOnAccess::Uninitialized(Box::new(move || {
				let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

				let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
					task_manager.spawn_handle(),
					client.clone(),
					transaction_pool,
					prometheus_registry.as_ref(),
					telemetry.as_ref().map(|t| t.handle()),
				);

				let relay_chain_backend = relay_chain_full_node.backend.clone();
				let relay_chain_client = relay_chain_full_node.client.clone();
				let aura_consensus = build_aura_consensus::<
						sp_consensus_aura::sr25519::AuthorityPair,
					_,
					_,
					_,
					_,
					_,
					_,
					_,
					_,
					_,
					>(BuildAuraConsensusParams {
						proposer_factory,
						create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
							let parachain_inherent =
								cumulus_primitives_parachain_inherent::ParachainInherentData::create_at_with_client(
									relay_parent,
									&relay_chain_client,
									&*relay_chain_backend,
									&validation_data,
									id,
								);
							async move {
								let time = sp_timestamp::InherentDataProvider::from_system_time();

								let slot =
									sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
										*time,
										slot_duration.slot_duration(),
									);

								let parachain_inherent = parachain_inherent.ok_or_else(|| {
									Box::<dyn std::error::Error + Send + Sync>::from(
										"Failed to create parachain inherent",
									)
								})?;
								Ok((time, slot, parachain_inherent))
							}
						},
						block_import: client.clone(),
						relay_chain_client: relay_chain_full_node2.client.clone(),
						relay_chain_backend: relay_chain_full_node2.backend.clone(),
						para_client: client.clone(),
						backoff_authoring_blocks: Option::<()>::None,
						sync_oracle: network.clone(),
						keystore: params.keystore_container.sync_keystore(),
						force_authoring,
						slot_duration,
						// We got around 500ms for proposing
						block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
						telemetry: telemetry.map(|t| t.handle()),
					});
			})),
		});

		let spawner = task_manager.spawn_handle();

		let params = StartCollatorParams {
			para_id: id,
			block_status: client2.clone(),
			announce_block,
			client: client2.clone(),
			task_manager: &mut task_manager,
			collator_key,
			relay_chain_full_node,
			spawner,
			parachain_consensus,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			polkadot_full_node: relay_chain_full_node,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}
