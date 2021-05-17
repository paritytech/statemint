// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.use cumulus_primitives_core::ParaId;

use cumulus_primitives_core::ParaId;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, crypto::UncheckedInto, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use runtime_common::{AccountId, Signature, AuraId, Balance};
use hex_literal::hex;
use serde_json;

const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<statemint_runtime::GenesisConfig, Extensions>;
pub type StatemineChainSpec = sc_service::GenericChainSpec<statemine_runtime::GenesisConfig, Extensions>;

const STATEMINT_ED: Balance = statemint_runtime::constants::currency::EXISTENTIAL_DEPOSIT;
const STATEMINE_ED: Balance = statemine_runtime::constants::currency::EXISTENTIAL_DEPOSIT;

/// Helper function to generate a crypto pair from seed
pub fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_pair_from_seed::<AuraId>(seed)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn statemint_session_keys(keys: AuraId) -> statemint_runtime::opaque::SessionKeys {
	statemint_runtime::opaque::SessionKeys { aura: keys }
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn statemine_session_keys(keys: AuraId) -> statemine_runtime::opaque::SessionKeys {
	statemine_runtime::opaque::SessionKeys { aura: keys }
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;
/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_pair_from_seed::<TPublic>(seed)).into_account()
}

pub fn statemint_development_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Statemint Development",
		// ID
		"statemint_dev",
		ChainType::Local,
		move || {
			statemint_testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					)
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn statemint_local_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			statemint_testnet_genesis(
				vec![
					(
						hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").into(),
						hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").unchecked_into()
					)
				],
				vec![
					hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").into(),
					hex!("c8f226d8a15b8d23241596862ce10d2db8359f816d45efb01c65524725543219").into(),
					hex!("dee1e2a19c2f7ddee43e66373d58768c6dc9ba4424af6101a5497b2e4a945371").into(),
					hex!("6a9099150aa91fd6cb5ec1a497e0d6b0e14cca7a863ed5608f6aa6a4970c6169").into(),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: id.into(),
		},
	)
}

fn statemint_testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> statemint_runtime::GenesisConfig {
	statemint_runtime::GenesisConfig {
		frame_system: statemint_runtime::SystemConfig {
			code: statemint_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: statemint_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, STATEMINT_ED * 4096))
				.collect(),
		},
		parachain_info: statemint_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_collator_selection: statemint_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: STATEMINT_ED * 16,
			..Default::default()
		},
		pallet_session: statemint_runtime::SessionConfig {
			keys: invulnerables.iter().cloned().map(|(acc, aura)| (
				acc.clone(), // account id
				acc.clone(), // validator id
				statemint_session_keys(aura), // session keys
			)).collect()
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		pallet_aura: Default::default(),
		cumulus_pallet_aura_ext: Default::default(),
	}
}

pub fn statemine_development_config(id: ParaId) -> StatemineChainSpec {
	StatemineChainSpec::from_genesis(
		// Name
		"Statemine Development",
		// ID
		"statemine_dev",
		ChainType::Live,
		move || {
			statemine_testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					)
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-dev".into(),
			para_id: id.into(),
		},
	)
}

pub fn statemine_local_config(id: ParaId) -> StatemineChainSpec {
	StatemineChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			statemine_testnet_genesis(
				vec![
					(
						hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").into(),
						hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").unchecked_into()
					)
				],
				vec![
					hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").into(),
					hex!("c8f226d8a15b8d23241596862ce10d2db8359f816d45efb01c65524725543219").into(),
					hex!("dee1e2a19c2f7ddee43e66373d58768c6dc9ba4424af6101a5497b2e4a945371").into(),
					hex!("6a9099150aa91fd6cb5ec1a497e0d6b0e14cca7a863ed5608f6aa6a4970c6169").into(),
				],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: id.into(),
		},
	)
}

pub fn statemine_mainnet_config(id: ParaId) -> StatemineChainSpec {
	//TODO check if correct
	let data = r#"
		{
			"ss58Format": 1,
			"tokenDecimals": 12,
			"tokenSymbol": "ksm"
		}"#;

	let properties = serde_json::from_str(data).unwrap();
	StatemineChainSpec::from_genesis(
		// Name
		"statemine",
		// ID
		"statemine",
		ChainType::Live,
		move || {
			statemine_mainnet_genesis(
				vec![
					//TODO replace with proper keys
					(
						hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").into(),
						hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").unchecked_into()
					)
				],
				// TODO replace with proper keys
				vec![
					hex!("2241c74de78435b5f21fb95e40b919c30a73cb4a32776dffce87a062a05ff665").into(),
				],
				id,
			)
		},
		//TODO bootnodes
		vec![],
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		Some("statemine"),
		properties,
		Extensions {
			relay_chain: "kusama".into(),
			para_id: id.into(),
		},
	)
}

fn statemine_testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> statemine_runtime::GenesisConfig {
	statemine_runtime::GenesisConfig {
		frame_system: statemine_runtime::SystemConfig {
			code: statemine_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: statemine_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, STATEMINE_ED * 4096))
				.collect(),
		},
		parachain_info: statemine_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_collator_selection: statemine_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: STATEMINE_ED * 16,
			..Default::default()
		},
		pallet_session: statemine_runtime::SessionConfig {
			keys: invulnerables.iter().cloned().map(|(acc, aura)| (
				acc.clone(), // account id
				acc.clone(), // validator id
				statemine_session_keys(aura), // session keys
			)).collect()
		},
		pallet_aura: Default::default(),
		cumulus_pallet_aura_ext: Default::default(),
	}
}

fn statemine_mainnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> statemine_runtime::GenesisConfig {
	statemine_runtime::GenesisConfig {
		frame_system: statemine_runtime::SystemConfig {
			code: statemine_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: statemine_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, STATEMINE_ED))
				.collect(),
		},
		parachain_info: statemine_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_collator_selection: statemine_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: STATEMINE_ED * 16,
			..Default::default()
		},
		pallet_session: statemine_runtime::SessionConfig {
			keys: invulnerables.iter().cloned().map(|(acc, aura)| (
				acc.clone(), // account id
				acc.clone(), // validator id
				statemine_session_keys(aura), // session keys
			)).collect()
		},
		pallet_aura: Default::default(),
		cumulus_pallet_aura_ext: Default::default(),
	}
}