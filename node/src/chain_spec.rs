use sp_core::{Pair, Public, sr25519};
use std::{str::FromStr};
use node_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, DeipConfig, DeipProposalConfig,
    DeipOrgConfig
};
use pallet_deip::{ DomainId, Domain };
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::ChainType;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
			vec![
				DomainId::from_str("6c4bb3bcf1a88e3b51de88576d592f1f980c5bbb").unwrap(), // Common
				DomainId::from_str("7c3d37cbfea2513a7e03e674448bbeee8ae3d862").unwrap(), // Biology
				DomainId::from_str("9f0224709d86e02b9625b5ebf2786b80ba6bed17").unwrap(), // Physics
				DomainId::from_str("6a8b20f002a7dedf7b873dbc86e0b0051d4fa898").unwrap(), // Chemistry
				DomainId::from_str("a47bf84ac30d0843accb737d5924434ef3ed0517").unwrap(), // Earth sciences
				DomainId::from_str("8e2a3711649993a87848337b9b401dcf64425e2d").unwrap(), // Space sciences
				DomainId::from_str("721e75eb0535e152669b0c3fbbb9e21675483553").unwrap(), // Medicine and health
				DomainId::from_str("2519ef55e1b69f1a7e13275e3273950cce7e26a8").unwrap() // Aquaculture
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
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
			],
			vec![
				DomainId::from_str("6c4bb3bcf1a88e3b51de88576d592f1f980c5bbb").unwrap(), // Common
				DomainId::from_str("7c3d37cbfea2513a7e03e674448bbeee8ae3d862").unwrap(), // Biology
				DomainId::from_str("9f0224709d86e02b9625b5ebf2786b80ba6bed17").unwrap(), // Physics
				DomainId::from_str("6a8b20f002a7dedf7b873dbc86e0b0051d4fa898").unwrap(), // Chemistry
				DomainId::from_str("a47bf84ac30d0843accb737d5924434ef3ed0517").unwrap(), // Earth sciences
				DomainId::from_str("8e2a3711649993a87848337b9b401dcf64425e2d").unwrap(), // Space sciences
				DomainId::from_str("721e75eb0535e152669b0c3fbbb9e21675483553").unwrap(), // Medicine and health
				DomainId::from_str("2519ef55e1b69f1a7e13275e3273950cce7e26a8").unwrap() // Aquaculture
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	domains: Vec<DomainId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
		}),
		pallet_deip: Some(DeipConfig {
			domains: domains.iter().cloned().map(|k|(k, Domain { external_id: k })).collect(),
			domain_count: domains.len() as u32,
		}),
        pallet_deip_proposal: Some(DeipProposalConfig {
            // _myfield: Default::default()
        }),
        pallet_deip_org: Some(DeipOrgConfig {
        }),
		pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		}),
		pallet_sudo: Some(SudoConfig {
			// Assign network admin rights.
			key: root_key,
		}),
	}
}
