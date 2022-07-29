// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Parity Bridges Common.

// Parity Bridges Common is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Bridges Common is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Bridges Common.  If not, see <http://www.gnu.org/licenses/>.

//! Parachains finality module.
//!
//! This module needs to be deployed with GRANDPA module, which is syncing relay
//! chain blocks. The main entry point of this module is `submit_parachain_heads`, which
//! accepts storage proof of some parachain `Heads` entries from bridged relay chain.
//! It requires corresponding relay headers to be already synced.

#![cfg_attr(not(feature = "std"), no_std)]

pub use weights::WeightInfo;
pub use weights_ext::WeightInfoExt;

use bp_parachains::parachain_head_storage_key_at_source;
use bp_polkadot_core::parachains::{ParaHash, ParaHasher, ParaHead, ParaHeadsProof, ParaId};
use bp_runtime::StorageProofError;
use codec::{Decode, Encode};
use frame_support::{traits::Contains, weights::PostDispatchInfo, RuntimeDebug};
use scale_info::TypeInfo;
use sp_runtime::traits::Header as HeaderT;
use sp_std::vec::Vec;

// Re-export in crate namespace for `construct_runtime!`.
pub use pallet::*;

pub mod weights;
pub mod weights_ext;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

mod extension;
#[cfg(test)]
mod mock;

/// The target that will be used when publishing logs related to this pallet.
pub const LOG_TARGET: &str = "runtime::bridge-parachains";

/// Block hash of the bridged relay chain.
pub type RelayBlockHash = bp_polkadot_core::Hash;
/// Block number of the bridged relay chain.
pub type RelayBlockNumber = bp_polkadot_core::BlockNumber;
/// Hasher of the bridged relay chain.
pub type RelayBlockHasher = bp_polkadot_core::Hasher;

/// Best known parachain head as it is stored in the runtime storage.
#[derive(Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct BestParaHead {
	/// Number of relay block where this head has been updated.
	pub at_relay_block_number: RelayBlockNumber,
	/// Hash of parachain head.
	pub head_hash: ParaHash,
	/// Current ring buffer position for this parachain.
	pub next_imported_hash_position: u32,
}

/// Artifacts of the parachains head update.
struct UpdateParachainHeadArtifacts {
	/// New best head of the parachain.
	pub best_head: BestParaHead,
	/// If `true`, some old parachain head has been pruned during update.
	pub prune_happened: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use bp_parachains::ImportedParaHeadsKeyProvider;
	use bp_runtime::{BasicOperatingMode, OwnedBridgeModule, StorageDoubleMapKeyProvider};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Weight info of the given parachains pallet.
	pub type WeightInfoOf<T, I> = <T as Config<I>>::WeightInfo;

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Relay chain block hash is unknown to us.
		UnknownRelayChainBlock,
		/// The number of stored relay block is different from what the relayer has provided.
		InvalidRelayChainBlockNumber,
		/// Invalid storage proof has been passed.
		InvalidStorageProof,
		/// Given parachain head is unknown.
		UnknownParaHead,
		/// The storage proof doesn't contains storage root. So it is invalid for given header.
		StorageRootMismatch,
		/// Failed to extract state root from given parachain head.
		FailedToExtractStateRoot,
		/// Error generated by the `OwnedBridgeModule` trait.
		BridgeModule(bp_runtime::OwnedBridgeModuleError),
	}

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config<I: 'static = ()>:
		pallet_bridge_grandpa::Config<Self::BridgesGrandpaPalletInstance>
	{
		/// Benchmarks results from runtime we're plugged into.
		type WeightInfo: WeightInfoExt;

		/// Instance of bridges GRANDPA pallet (within this runtime) that this pallet is linked to.
		///
		/// The GRANDPA pallet instance must be configured to import headers of relay chain that
		/// we're interested in.
		type BridgesGrandpaPalletInstance: 'static;

		/// Name of the `paras` pallet in the `construct_runtime!()` call at the bridged chain.
		#[pallet::constant]
		type ParasPalletName: Get<&'static str>;

		/// Set of parachains that are tracked by this pallet.
		///
		/// The set may be extended easily, without requiring any runtime upgrades. Removing tracked
		/// parachain requires special handling - pruning existing heads and cleaning related data
		/// structures.
		type TrackedParachains: Contains<ParaId>;

		/// Maximal number of single parachain heads to keep in the storage.
		///
		/// The setting is there to prevent growing the on-chain state indefinitely. Note
		/// the setting does not relate to parachain block numbers - we will simply keep as much
		/// items in the storage, so it doesn't guarantee any fixed timeframe for heads.
		///
		/// Incautious change of this constant may lead to orphan entries in the runtime storage.
		#[pallet::constant]
		type HeadsToKeep: Get<u32>;
	}

	/// Optional pallet owner.
	///
	/// Pallet owner has a right to halt all pallet operations and then resume them. If it is
	/// `None`, then there are no direct ways to halt/resume pallet operations, but other
	/// runtime methods may still be used to do that (i.e. democracy::referendum to update halt
	/// flag directly or call the `halt_operations`).
	#[pallet::storage]
	pub type PalletOwner<T: Config<I>, I: 'static = ()> =
		StorageValue<_, T::AccountId, OptionQuery>;

	/// The current operating mode of the pallet.
	///
	/// Depending on the mode either all, or no transactions will be allowed.
	#[pallet::storage]
	pub type PalletOperatingMode<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BasicOperatingMode, ValueQuery>;

	/// Best parachain heads.
	#[pallet::storage]
	pub type BestParaHeads<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, ParaId, BestParaHead>;

	/// Parachain heads which have been imported into the pallet.
	#[pallet::storage]
	pub type ImportedParaHeads<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		<ImportedParaHeadsKeyProvider as StorageDoubleMapKeyProvider>::Hasher1,
		<ImportedParaHeadsKeyProvider as StorageDoubleMapKeyProvider>::Key1,
		<ImportedParaHeadsKeyProvider as StorageDoubleMapKeyProvider>::Hasher2,
		<ImportedParaHeadsKeyProvider as StorageDoubleMapKeyProvider>::Key2,
		<ImportedParaHeadsKeyProvider as StorageDoubleMapKeyProvider>::Value,
	>;

	/// A ring buffer of imported parachain head hashes. Ordered by the insertion time.
	#[pallet::storage]
	pub(super) type ImportedParaHashes<T: Config<I>, I: 'static = ()> =
		StorageDoubleMap<_, Blake2_128Concat, ParaId, Twox64Concat, u32, ParaHash>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	impl<T: Config<I>, I: 'static> OwnedBridgeModule<T> for Pallet<T, I> {
		const LOG_TARGET: &'static str = LOG_TARGET;
		type OwnerStorage = PalletOwner<T, I>;
		type OperatingMode = BasicOperatingMode;
		type OperatingModeStorage = PalletOperatingMode<T, I>;
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I>
	where
		<T as pallet_bridge_grandpa::Config<T::BridgesGrandpaPalletInstance>>::BridgedChain:
			bp_runtime::Chain<
				BlockNumber = RelayBlockNumber,
				Hash = RelayBlockHash,
				Hasher = RelayBlockHasher,
			>,
	{
		/// Submit proof of one or several parachain heads.
		///
		/// The proof is supposed to be proof of some `Heads` entries from the
		/// `polkadot-runtime-parachains::paras` pallet instance, deployed at the bridged chain.
		/// The proof is supposed to be crafted at the `relay_header_hash` that must already be
		/// imported by corresponding GRANDPA pallet at this chain.
		#[pallet::weight(WeightInfoOf::<T, I>::submit_parachain_heads_weight(
			T::DbWeight::get(),
			parachain_heads_proof,
			parachains.len() as _,
		))]
		pub fn submit_parachain_heads(
			_origin: OriginFor<T>,
			at_relay_block: (RelayBlockNumber, RelayBlockHash),
			parachains: Vec<(ParaId, ParaHash)>,
			parachain_heads_proof: ParaHeadsProof,
		) -> DispatchResultWithPostInfo {
			Self::ensure_not_halted().map_err(Error::<T, I>::BridgeModule)?;
			// we'll need relay chain header to verify that parachains heads are always increasing.
			let (relay_block_number, relay_block_hash) = at_relay_block;
			let relay_block = pallet_bridge_grandpa::ImportedHeaders::<
				T,
				T::BridgesGrandpaPalletInstance,
			>::get(relay_block_hash)
			.ok_or(Error::<T, I>::UnknownRelayChainBlock)?;
			ensure!(
				*relay_block.number() == relay_block_number,
				Error::<T, I>::InvalidRelayChainBlockNumber,
			);

			// now parse storage proof and read parachain heads
			let mut actual_weight = WeightInfoOf::<T, I>::submit_parachain_heads_weight(
				T::DbWeight::get(),
				&parachain_heads_proof,
				parachains.len() as _,
			);
			pallet_bridge_grandpa::Pallet::<T, T::BridgesGrandpaPalletInstance>::parse_finalized_storage_proof(
				relay_block_hash,
				sp_trie::StorageProof::new(parachain_heads_proof.0),
				move |storage| {
					for (parachain, parachain_head_hash) in parachains {
						// if we're not tracking this parachain, we'll just ignore its head proof here
						if !T::TrackedParachains::contains(&parachain) {
							log::trace!(
								target: LOG_TARGET,
								"The head of parachain {:?} has been provided, but it is not tracked by the pallet",
								parachain,
							);
							continue;
						}

						let parachain_head = match Pallet::<T, I>::read_parachain_head(&storage, parachain) {
							Ok(Some(parachain_head)) => parachain_head,
							Ok(None) => {
								log::trace!(
									target: LOG_TARGET,
									"The head of parachain {:?} is None. {}",
									parachain,
									if BestParaHeads::<T, I>::contains_key(&parachain) {
										"Looks like it is not yet registered at the source relay chain"
									} else {
										"Looks like it has been deregistered from the source relay chain"
									},
								);
								continue;
							},
							Err(e) => {
								log::trace!(
									target: LOG_TARGET,
									"The read of head of parachain {:?} has failed: {:?}",
									parachain,
									e,
								);
								continue;
							},
						};

						// if relayer has specified invalid parachain head hash, ignore the head
						// (this isn't strictly necessary, but better safe than sorry)
						let actual_parachain_head_hash = parachain_head.hash();
						if parachain_head_hash != actual_parachain_head_hash {
							log::trace!(
								target: LOG_TARGET,
								"The submitter has specified invalid parachain {:?} head hash: {:?} vs {:?}",
								parachain,
								parachain_head_hash,
								actual_parachain_head_hash,
							);
							continue;
						}

						let prune_happened: Result<_, ()> = BestParaHeads::<T, I>::try_mutate(parachain, |stored_best_head| {
							let artifacts = Pallet::<T, I>::update_parachain_head(
								parachain,
								stored_best_head.take(),
								relay_block_number,
								parachain_head,
								parachain_head_hash,
							)?;
							*stored_best_head = Some(artifacts.best_head);
							Ok(artifacts.prune_happened)
						});

						if matches!(prune_happened, Err(_) | Ok(false)) {
							actual_weight = actual_weight
								.saturating_sub(WeightInfoOf::<T, I>::parachain_head_pruning_weight(T::DbWeight::get()));
						}
					}
				},
			)
			.map_err(|_| Error::<T, I>::InvalidStorageProof)?;

			Ok(PostDispatchInfo { actual_weight: Some(actual_weight), pays_fee: Pays::Yes })
		}

		/// Change `PalletOwner`.
		///
		/// May only be called either by root, or by `PalletOwner`.
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_owner(origin: OriginFor<T>, new_owner: Option<T::AccountId>) -> DispatchResult {
			<Self as OwnedBridgeModule<_>>::set_owner(origin, new_owner)
		}

		/// Halt or resume all pallet operations.
		///
		/// May only be called either by root, or by `PalletOwner`.
		#[pallet::weight((T::DbWeight::get().reads_writes(1, 1), DispatchClass::Operational))]
		pub fn set_operating_mode(
			origin: OriginFor<T>,
			operating_mode: BasicOperatingMode,
		) -> DispatchResult {
			<Self as OwnedBridgeModule<_>>::set_operating_mode(origin, operating_mode)
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Get best finalized header of the given parachain.
		pub fn best_parachain_head(parachain: ParaId) -> Option<ParaHead> {
			let best_para_head_hash = BestParaHeads::<T, I>::get(parachain)?.head_hash;
			ImportedParaHeads::<T, I>::get(parachain, best_para_head_hash)
		}

		/// Get parachain head with given hash.
		pub fn parachain_head(parachain: ParaId, hash: ParaHash) -> Option<ParaHead> {
			ImportedParaHeads::<T, I>::get(parachain, hash)
		}

		/// Verify that the passed storage proof is valid, given it is crafted using
		/// known finalized header. If the proof is valid, then the `parse` callback
		/// is called and the function returns its result.
		pub fn parse_finalized_storage_proof<R>(
			parachain: ParaId,
			hash: ParaHash,
			storage_proof: sp_trie::StorageProof,
			decode_state_root: impl FnOnce(ParaHead) -> Option<ParaHash>,
			parse: impl FnOnce(bp_runtime::StorageProofChecker<ParaHasher>) -> R,
		) -> Result<R, sp_runtime::DispatchError> {
			let para_head =
				Self::parachain_head(parachain, hash).ok_or(Error::<T, I>::UnknownParaHead)?;
			let state_root =
				decode_state_root(para_head).ok_or(Error::<T, I>::FailedToExtractStateRoot)?;
			let storage_proof_checker =
				bp_runtime::StorageProofChecker::new(state_root, storage_proof)
					.map_err(|_| Error::<T, I>::StorageRootMismatch)?;

			Ok(parse(storage_proof_checker))
		}

		/// Read parachain head from storage proof.
		fn read_parachain_head(
			storage: &bp_runtime::StorageProofChecker<RelayBlockHasher>,
			parachain: ParaId,
		) -> Result<Option<ParaHead>, StorageProofError> {
			let parachain_head_key =
				parachain_head_storage_key_at_source(T::ParasPalletName::get(), parachain);
			storage.read_and_decode_value(parachain_head_key.0.as_ref())
		}

		/// Check if para head has been already updated at better relay chain block.
		/// Without this check, we may import heads in random order.
		pub fn validate_updated_parachain_head(
			parachain: ParaId,
			maybe_stored_best_head: &Option<BestParaHead>,
			updated_at_relay_block_number: RelayBlockNumber,
			updated_head_hash: ParaHash,
			err_log_prefix: &str,
		) -> TransactionValidity {
			let stored_best_head = match maybe_stored_best_head {
				Some(stored_best_head) => stored_best_head,
				None => return Ok(ValidTransaction::default()),
			};

			if stored_best_head.at_relay_block_number >= updated_at_relay_block_number {
				log::trace!(
					target: LOG_TARGET,
					"{}. The parachain head for {:?} was already updated at better relay chain block {} >= {}.",
					err_log_prefix,
					parachain,
					stored_best_head.at_relay_block_number,
					updated_at_relay_block_number
				);
				return InvalidTransaction::Stale.into()
			}

			if stored_best_head.head_hash == updated_head_hash {
				log::trace!(
					target: LOG_TARGET,
					"{}. The parachain head hash for {:?} was already updated to {} at block {} < {}.",
					err_log_prefix,
					parachain,
					updated_head_hash,
					stored_best_head.at_relay_block_number,
					updated_at_relay_block_number
				);
				return InvalidTransaction::Stale.into()
			}

			Ok(ValidTransaction::default())
		}

		/// Try to update parachain head.
		pub(super) fn update_parachain_head(
			parachain: ParaId,
			stored_best_head: Option<BestParaHead>,
			updated_at_relay_block_number: RelayBlockNumber,
			updated_head: ParaHead,
			updated_head_hash: ParaHash,
		) -> Result<UpdateParachainHeadArtifacts, ()> {
			// check if head has been already updated at better relay chain block. Without this
			// check, we may import heads in random order
			Self::validate_updated_parachain_head(
				parachain,
				&stored_best_head,
				updated_at_relay_block_number,
				updated_head_hash,
				"The parachain head can't be updated",
			)
			.map_err(|_| ())?;
			let next_imported_hash_position = stored_best_head
				.map_or(0, |stored_best_head| stored_best_head.next_imported_hash_position);

			// insert updated best parachain head
			let head_hash_to_prune =
				ImportedParaHashes::<T, I>::try_get(parachain, next_imported_hash_position);
			let updated_best_para_head = BestParaHead {
				at_relay_block_number: updated_at_relay_block_number,
				head_hash: updated_head_hash,
				next_imported_hash_position: (next_imported_hash_position + 1) %
					T::HeadsToKeep::get(),
			};
			ImportedParaHashes::<T, I>::insert(
				parachain,
				next_imported_hash_position,
				updated_head_hash,
			);
			ImportedParaHeads::<T, I>::insert(parachain, updated_head_hash, updated_head);
			log::trace!(
				target: LOG_TARGET,
				"Updated head of parachain {:?} to {}",
				parachain,
				updated_head_hash,
			);

			// remove old head
			let prune_happened = head_hash_to_prune.is_ok();
			if let Ok(head_hash_to_prune) = head_hash_to_prune {
				log::trace!(
					target: LOG_TARGET,
					"Pruning old head of parachain {:?}: {}",
					parachain,
					head_hash_to_prune,
				);
				ImportedParaHeads::<T, I>::remove(parachain, head_hash_to_prune);
			}

			Ok(UpdateParachainHeadArtifacts { best_head: updated_best_para_head, prune_happened })
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Initial pallet operating mode.
		pub operating_mode: BasicOperatingMode,
		/// Initial pallet owner.
		pub owner: Option<T::AccountId>,
		/// Dummy marker.
		pub phantom: sp_std::marker::PhantomData<I>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self {
				operating_mode: Default::default(),
				owner: Default::default(),
				phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			PalletOperatingMode::<T, I>::put(&self.operating_mode);
			if let Some(ref owner) = self.owner {
				PalletOwner::<T, I>::put(owner);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{
		run_test, test_relay_header, Origin, TestRuntime, PARAS_PALLET_NAME, UNTRACKED_PARACHAIN_ID,
	};

	use bp_parachains::ImportedParaHeadsKeyProvider;
	use bp_runtime::{BasicOperatingMode, OwnedBridgeModuleError, StorageDoubleMapKeyProvider};
	use bp_test_utils::{
		authority_list, generate_owned_bridge_module_tests, make_default_justification,
	};
	use frame_support::{
		assert_noop, assert_ok,
		dispatch::DispatchResultWithPostInfo,
		storage::generator::{StorageDoubleMap, StorageMap},
		traits::{Get, OnInitialize},
		weights::Weight,
	};
	use sp_runtime::DispatchError;
	use sp_trie::{
		record_all_keys, trie_types::TrieDBMutV1, LayoutV1, MemoryDB, Recorder, TrieMut,
	};

	type BridgesGrandpaPalletInstance = pallet_bridge_grandpa::Instance1;

	fn initialize(state_root: RelayBlockHash) {
		pallet_bridge_grandpa::Pallet::<TestRuntime, BridgesGrandpaPalletInstance>::initialize(
			Origin::root(),
			bp_header_chain::InitializationData {
				header: Box::new(test_relay_header(0, state_root)),
				authority_list: authority_list(),
				set_id: 1,
				operating_mode: BasicOperatingMode::Normal,
			},
		)
		.unwrap();
	}

	fn proceed(num: RelayBlockNumber, state_root: RelayBlockHash) {
		pallet_bridge_grandpa::Pallet::<TestRuntime, BridgesGrandpaPalletInstance>::on_initialize(
			0,
		);

		let header = test_relay_header(num, state_root);
		let justification = make_default_justification(&header);
		assert_ok!(
			pallet_bridge_grandpa::Pallet::<TestRuntime, BridgesGrandpaPalletInstance>::submit_finality_proof(
				Origin::signed(1),
				Box::new(header),
				justification,
			)
		);
	}

	fn prepare_parachain_heads_proof(
		heads: Vec<(u32, ParaHead)>,
	) -> (RelayBlockHash, ParaHeadsProof, Vec<(ParaId, ParaHash)>) {
		let mut parachains = Vec::with_capacity(heads.len());
		let mut root = Default::default();
		let mut mdb = MemoryDB::default();
		{
			let mut trie = TrieDBMutV1::<RelayBlockHasher>::new(&mut mdb, &mut root);
			for (parachain, head) in heads {
				let storage_key =
					parachain_head_storage_key_at_source(PARAS_PALLET_NAME, ParaId(parachain));
				trie.insert(&storage_key.0, &head.encode())
					.map_err(|_| "TrieMut::insert has failed")
					.expect("TrieMut::insert should not fail in tests");
				parachains.push((ParaId(parachain), head.hash()));
			}
		}

		// generate storage proof to be delivered to This chain
		let mut proof_recorder = Recorder::<RelayBlockHash>::new();
		record_all_keys::<LayoutV1<RelayBlockHasher>, _>(&mdb, &root, &mut proof_recorder)
			.map_err(|_| "record_all_keys has failed")
			.expect("record_all_keys should not fail in benchmarks");
		let storage_proof = proof_recorder.drain().into_iter().map(|n| n.data.to_vec()).collect();

		(root, ParaHeadsProof(storage_proof), parachains)
	}

	fn initial_best_head(parachain: u32) -> BestParaHead {
		BestParaHead {
			at_relay_block_number: 0,
			head_hash: head_data(parachain, 0).hash(),
			next_imported_hash_position: 1,
		}
	}

	fn head_data(parachain: u32, head_number: u32) -> ParaHead {
		ParaHead((parachain, head_number).encode())
	}

	fn head_hash(parachain: u32, head_number: u32) -> ParaHash {
		head_data(parachain, head_number).hash()
	}

	fn import_parachain_1_head(
		relay_chain_block: RelayBlockNumber,
		relay_state_root: RelayBlockHash,
		parachains: Vec<(ParaId, ParaHash)>,
		proof: ParaHeadsProof,
	) -> DispatchResultWithPostInfo {
		Pallet::<TestRuntime>::submit_parachain_heads(
			Origin::signed(1),
			(relay_chain_block, test_relay_header(relay_chain_block, relay_state_root).hash()),
			parachains,
			proof,
		)
	}

	fn weight_of_import_parachain_1_head(proof: &ParaHeadsProof, prune_expected: bool) -> Weight {
		let db_weight = <TestRuntime as frame_system::Config>::DbWeight::get();
		WeightInfoOf::<TestRuntime, ()>::submit_parachain_heads_weight(db_weight, proof, 1)
			.saturating_sub(if prune_expected {
				0
			} else {
				WeightInfoOf::<TestRuntime, ()>::parachain_head_pruning_weight(db_weight)
			})
	}

	#[test]
	fn submit_parachain_heads_checks_operating_mode() {
		let (state_root, proof, parachains) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 0))]);

		run_test(|| {
			initialize(state_root);

			// `submit_parachain_heads()` should fail when the pallet is halted.
			PalletOperatingMode::<TestRuntime>::put(BasicOperatingMode::Halted);
			assert_noop!(
				Pallet::<TestRuntime>::submit_parachain_heads(
					Origin::signed(1),
					(0, test_relay_header(0, state_root).hash()),
					parachains.clone(),
					proof.clone(),
				),
				Error::<TestRuntime>::BridgeModule(OwnedBridgeModuleError::Halted)
			);

			// `submit_parachain_heads()` should succeed now that the pallet is resumed.
			PalletOperatingMode::<TestRuntime>::put(BasicOperatingMode::Normal);
			assert_ok!(Pallet::<TestRuntime>::submit_parachain_heads(
				Origin::signed(1),
				(0, test_relay_header(0, state_root).hash()),
				parachains,
				proof,
			),);
		});
	}

	#[test]
	fn imports_initial_parachain_heads() {
		let (state_root, proof, parachains) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 0)), (3, head_data(3, 10))]);
		run_test(|| {
			initialize(state_root);

			// we're trying to update heads of parachains 1, 2 and 3
			assert_ok!(Pallet::<TestRuntime>::submit_parachain_heads(
				Origin::signed(1),
				(0, test_relay_header(0, state_root).hash()),
				parachains,
				proof,
			),);

			// but only 1 and 2 are updated, because proof is missing head of parachain#2
			assert_eq!(BestParaHeads::<TestRuntime>::get(ParaId(1)), Some(initial_best_head(1)));
			assert_eq!(BestParaHeads::<TestRuntime>::get(ParaId(2)), None);
			assert_eq!(
				BestParaHeads::<TestRuntime>::get(ParaId(3)),
				Some(BestParaHead {
					at_relay_block_number: 0,
					head_hash: head_data(3, 10).hash(),
					next_imported_hash_position: 1,
				})
			);

			assert_eq!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(1), initial_best_head(1).head_hash),
				Some(head_data(1, 0))
			);
			assert_eq!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(2), initial_best_head(2).head_hash),
				None
			);
			assert_eq!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(3), head_hash(3, 10)),
				Some(head_data(3, 10))
			);
		});
	}

	#[test]
	fn imports_parachain_heads_is_able_to_progress() {
		let (state_root_5, proof_5, parachains_5) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 5))]);
		let (state_root_10, proof_10, parachains_10) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 10))]);
		run_test(|| {
			// start with relay block #0 and import head#5 of parachain#1
			initialize(state_root_5);
			assert_ok!(import_parachain_1_head(0, state_root_5, parachains_5, proof_5));
			assert_eq!(
				BestParaHeads::<TestRuntime>::get(ParaId(1)),
				Some(BestParaHead {
					at_relay_block_number: 0,
					head_hash: head_data(1, 5).hash(),
					next_imported_hash_position: 1,
				})
			);
			assert_eq!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(1), head_data(1, 5).hash()),
				Some(head_data(1, 5))
			);
			assert_eq!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(1), head_data(1, 10).hash()),
				None
			);

			// import head#10 of parachain#1 at relay block #1
			proceed(1, state_root_10);
			assert_ok!(import_parachain_1_head(1, state_root_10, parachains_10, proof_10));
			assert_eq!(
				BestParaHeads::<TestRuntime>::get(ParaId(1)),
				Some(BestParaHead {
					at_relay_block_number: 1,
					head_hash: head_data(1, 10).hash(),
					next_imported_hash_position: 2,
				})
			);
			assert_eq!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(1), head_data(1, 5).hash()),
				Some(head_data(1, 5))
			);
			assert_eq!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(1), head_data(1, 10).hash()),
				Some(head_data(1, 10))
			);
		});
	}

	#[test]
	fn ignores_untracked_parachain() {
		let (state_root, proof, parachains) = prepare_parachain_heads_proof(vec![
			(1, head_data(1, 5)),
			(UNTRACKED_PARACHAIN_ID, head_data(1, 5)),
			(2, head_data(1, 5)),
		]);
		run_test(|| {
			// start with relay block #0 and try to import head#5 of parachain#1 and untracked
			// parachain
			initialize(state_root);
			assert_ok!(Pallet::<TestRuntime>::submit_parachain_heads(
				Origin::signed(1),
				(0, test_relay_header(0, state_root).hash()),
				parachains,
				proof,
			));
			assert_eq!(
				BestParaHeads::<TestRuntime>::get(ParaId(1)),
				Some(BestParaHead {
					at_relay_block_number: 0,
					head_hash: head_data(1, 5).hash(),
					next_imported_hash_position: 1,
				})
			);
			assert_eq!(BestParaHeads::<TestRuntime>::get(ParaId(UNTRACKED_PARACHAIN_ID)), None,);
			assert_eq!(
				BestParaHeads::<TestRuntime>::get(ParaId(2)),
				Some(BestParaHead {
					at_relay_block_number: 0,
					head_hash: head_data(1, 5).hash(),
					next_imported_hash_position: 1,
				})
			);
		});
	}

	#[test]
	fn does_nothing_when_already_imported_this_head_at_previous_relay_header() {
		let (state_root, proof, parachains) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 0))]);
		run_test(|| {
			// import head#0 of parachain#1 at relay block#0
			initialize(state_root);
			assert_ok!(import_parachain_1_head(0, state_root, parachains.clone(), proof.clone()));
			assert_eq!(BestParaHeads::<TestRuntime>::get(ParaId(1)), Some(initial_best_head(1)));

			// try to import head#0 of parachain#1 at relay block#1
			// => call succeeds, but nothing is changed
			proceed(1, state_root);
			assert_ok!(import_parachain_1_head(1, state_root, parachains, proof));
			assert_eq!(BestParaHeads::<TestRuntime>::get(ParaId(1)), Some(initial_best_head(1)));
		});
	}

	#[test]
	fn does_nothing_when_already_imported_head_at_better_relay_header() {
		let (state_root_5, proof_5, parachains_5) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 5))]);
		let (state_root_10, proof_10, parachains_10) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 10))]);
		run_test(|| {
			// start with relay block #0
			initialize(state_root_5);

			// head#10 of parachain#1 at relay block#1
			proceed(1, state_root_10);
			assert_ok!(import_parachain_1_head(1, state_root_10, parachains_10, proof_10));
			assert_eq!(
				BestParaHeads::<TestRuntime>::get(ParaId(1)),
				Some(BestParaHead {
					at_relay_block_number: 1,
					head_hash: head_data(1, 10).hash(),
					next_imported_hash_position: 1,
				})
			);

			// now try to import head#5 at relay block#0
			// => nothing is changed, because better head has already been imported
			assert_ok!(import_parachain_1_head(0, state_root_5, parachains_5, proof_5));
			assert_eq!(
				BestParaHeads::<TestRuntime>::get(ParaId(1)),
				Some(BestParaHead {
					at_relay_block_number: 1,
					head_hash: head_data(1, 10).hash(),
					next_imported_hash_position: 1,
				})
			);
		});
	}

	#[test]
	fn prunes_old_heads() {
		run_test(|| {
			let heads_to_keep = crate::mock::HeadsToKeep::get();

			// import exactly `HeadsToKeep` headers
			for i in 0..heads_to_keep {
				let (state_root, proof, parachains) =
					prepare_parachain_heads_proof(vec![(1, head_data(1, i))]);
				if i == 0 {
					initialize(state_root);
				} else {
					proceed(i, state_root);
				}

				let expected_weight = weight_of_import_parachain_1_head(&proof, false);
				let result = import_parachain_1_head(i, state_root, parachains, proof);
				assert_ok!(result);
				assert_eq!(result.expect("checked above").actual_weight, Some(expected_weight));
			}

			// nothing is pruned yet
			for i in 0..heads_to_keep {
				assert!(ImportedParaHeads::<TestRuntime>::get(ParaId(1), head_data(1, i).hash())
					.is_some());
			}

			// import next relay chain header and next parachain head
			let (state_root, proof, parachains) =
				prepare_parachain_heads_proof(vec![(1, head_data(1, heads_to_keep))]);
			proceed(heads_to_keep, state_root);
			let expected_weight = weight_of_import_parachain_1_head(&proof, true);
			let result = import_parachain_1_head(heads_to_keep, state_root, parachains, proof);
			assert_ok!(result);
			assert_eq!(result.expect("checked above").actual_weight, Some(expected_weight));

			// and the head#0 is pruned
			assert!(
				ImportedParaHeads::<TestRuntime>::get(ParaId(1), head_data(1, 0).hash()).is_none()
			);
			for i in 1..=heads_to_keep {
				assert!(ImportedParaHeads::<TestRuntime>::get(ParaId(1), head_data(1, i).hash())
					.is_some());
			}
		});
	}

	#[test]
	fn fails_on_unknown_relay_chain_block() {
		let (state_root, proof, parachains) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 5))]);
		run_test(|| {
			// start with relay block #0
			initialize(state_root);

			// try to import head#5 of parachain#1 at unknown relay chain block #1
			assert_noop!(
				import_parachain_1_head(1, state_root, parachains, proof),
				Error::<TestRuntime>::UnknownRelayChainBlock
			);
		});
	}

	#[test]
	fn fails_on_invalid_storage_proof() {
		let (_state_root, proof, parachains) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 5))]);
		run_test(|| {
			// start with relay block #0
			initialize(Default::default());

			// try to import head#5 of parachain#1 at relay chain block #0
			assert_noop!(
				import_parachain_1_head(0, Default::default(), parachains, proof),
				Error::<TestRuntime>::InvalidStorageProof
			);
		});
	}

	#[test]
	fn is_not_rewriting_existing_head_if_failed_to_read_updated_head() {
		let (state_root_5, proof_5, parachains_5) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 5))]);
		let (state_root_10_at_20, proof_10_at_20, parachains_10_at_20) =
			prepare_parachain_heads_proof(vec![(2, head_data(2, 10))]);
		let (state_root_10_at_30, proof_10_at_30, parachains_10_at_30) =
			prepare_parachain_heads_proof(vec![(1, head_data(1, 10))]);
		run_test(|| {
			// we've already imported head#5 of parachain#1 at relay block#10
			initialize(state_root_5);
			import_parachain_1_head(0, state_root_5, parachains_5, proof_5).expect("ok");
			assert_eq!(
				Pallet::<TestRuntime>::best_parachain_head(ParaId(1)),
				Some(head_data(1, 5))
			);

			// then if someone is pretending to provide updated head#10 of parachain#1 at relay
			// block#20, but fails to do that
			//
			// => we'll leave previous value
			proceed(20, state_root_10_at_20);
			assert_ok!(Pallet::<TestRuntime>::submit_parachain_heads(
				Origin::signed(1),
				(20, test_relay_header(20, state_root_10_at_20).hash()),
				parachains_10_at_20,
				proof_10_at_20,
			),);
			assert_eq!(
				Pallet::<TestRuntime>::best_parachain_head(ParaId(1)),
				Some(head_data(1, 5))
			);

			// then if someone is pretending to provide updated head#10 of parachain#1 at relay
			// block#30, and actualy provides it
			//
			// => we'll update value
			proceed(30, state_root_10_at_30);
			assert_ok!(Pallet::<TestRuntime>::submit_parachain_heads(
				Origin::signed(1),
				(30, test_relay_header(30, state_root_10_at_30).hash()),
				parachains_10_at_30,
				proof_10_at_30,
			),);
			assert_eq!(
				Pallet::<TestRuntime>::best_parachain_head(ParaId(1)),
				Some(head_data(1, 10))
			);
		});
	}

	#[test]
	fn storage_keys_computed_properly() {
		assert_eq!(
			BestParaHeads::<TestRuntime>::storage_map_final_key(ParaId(42)).to_vec(),
			bp_parachains::best_parachain_head_hash_storage_key_at_target("Parachains", ParaId(42))
				.0,
		);

		assert_eq!(
			ImportedParaHeads::<TestRuntime>::storage_double_map_final_key(
				ParaId(42),
				ParaHash::from([21u8; 32])
			)
			.to_vec(),
			ImportedParaHeadsKeyProvider::final_key(
				"Parachains",
				&ParaId(42),
				&ParaHash::from([21u8; 32])
			)
			.0,
		);
	}

	generate_owned_bridge_module_tests!(BasicOperatingMode::Normal, BasicOperatingMode::Halted);
}