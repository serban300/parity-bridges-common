// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

//! Autogenerated weights for pallet_bridge_messages
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-03-01, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `covid`, CPU: `11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// target/release/millau-bridge-node
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_bridge_messages
// --extrinsic=*
// --execution=wasm
// --wasm-execution=Compiled
// --heap-pages=4096
// --output=./modules/messages/src/weights.rs
// --template=./.maintain/millau-weight-template.hbs

#![allow(clippy::all)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{
	traits::Get,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_bridge_messages.
pub trait WeightInfo {
	fn receive_single_message_proof() -> Weight;
	fn receive_two_messages_proof() -> Weight;
	fn receive_single_message_proof_with_outbound_lane_state() -> Weight;
	fn receive_single_message_proof_1_kb() -> Weight;
	fn receive_single_message_proof_16_kb() -> Weight;
	fn receive_delivery_proof_for_single_message() -> Weight;
	fn receive_delivery_proof_for_two_messages_by_single_relayer() -> Weight;
	fn receive_delivery_proof_for_two_messages_by_two_relayers() -> Weight;
}

/// Weights for `pallet_bridge_messages` that are generated using one of the Bridge testnets.
///
/// Those weights are test only and must never be used in production.
pub struct BridgeWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for BridgeWeight<T> {
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	///
	/// Storage: Balances TotalIssuance (r:1 w:1)
	///
	/// Proof: Balances TotalIssuance (max_values: Some(1), max_size: Some(8), added: 503, mode:
	/// MaxEncodedLen)
	fn receive_single_message_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `693`
		//  Estimated: `54703`
		// Minimum execution time: 48_426 nanoseconds.
		Weight::from_parts(50_113_000, 54703)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	///
	/// Storage: Balances TotalIssuance (r:1 w:1)
	///
	/// Proof: Balances TotalIssuance (max_values: Some(1), max_size: Some(8), added: 503, mode:
	/// MaxEncodedLen)
	fn receive_two_messages_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `693`
		//  Estimated: `54703`
		// Minimum execution time: 59_739 nanoseconds.
		Weight::from_parts(61_704_000, 54703)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	///
	/// Storage: Balances TotalIssuance (r:1 w:1)
	///
	/// Proof: Balances TotalIssuance (max_values: Some(1), max_size: Some(8), added: 503, mode:
	/// MaxEncodedLen)
	fn receive_single_message_proof_with_outbound_lane_state() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `693`
		//  Estimated: `54703`
		// Minimum execution time: 53_760 nanoseconds.
		Weight::from_parts(55_645_000, 54703)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	fn receive_single_message_proof_1_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `618`
		//  Estimated: `54200`
		// Minimum execution time: 49_582 nanoseconds.
		Weight::from_parts(51_250_000, 54200)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	fn receive_single_message_proof_16_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `618`
		//  Estimated: `54200`
		// Minimum execution time: 76_418 nanoseconds.
		Weight::from_parts(77_877_000, 54200)
			.saturating_add(T::DbWeight::get().reads(3_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages OutboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages OutboundLanes (max_values: Some(1), max_size: Some(44), added:
	/// 539, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRelayers RelayerRewards (r:1 w:1)
	///
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(65), added: 2540,
	/// mode: MaxEncodedLen)
	fn receive_delivery_proof_for_single_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `579`
		//  Estimated: `5624`
		// Minimum execution time: 41_795 nanoseconds.
		Weight::from_parts(43_683_000, 5624)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages OutboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages OutboundLanes (max_values: Some(1), max_size: Some(44), added:
	/// 539, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRelayers RelayerRewards (r:1 w:1)
	///
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(65), added: 2540,
	/// mode: MaxEncodedLen)
	fn receive_delivery_proof_for_two_messages_by_single_relayer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `596`
		//  Estimated: `5624`
		// Minimum execution time: 39_946 nanoseconds.
		Weight::from_parts(41_509_000, 5624)
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages OutboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages OutboundLanes (max_values: Some(1), max_size: Some(44), added:
	/// 539, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRelayers RelayerRewards (r:2 w:2)
	///
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(65), added: 2540,
	/// mode: MaxEncodedLen)
	fn receive_delivery_proof_for_two_messages_by_two_relayers() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `596`
		//  Estimated: `8164`
		// Minimum execution time: 42_882 nanoseconds.
		Weight::from_parts(44_367_000, 8164)
			.saturating_add(T::DbWeight::get().reads(5_u64))
			.saturating_add(T::DbWeight::get().writes(3_u64))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	///
	/// Storage: Balances TotalIssuance (r:1 w:1)
	///
	/// Proof: Balances TotalIssuance (max_values: Some(1), max_size: Some(8), added: 503, mode:
	/// MaxEncodedLen)
	fn receive_single_message_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `693`
		//  Estimated: `54703`
		// Minimum execution time: 48_426 nanoseconds.
		Weight::from_parts(50_113_000, 54703)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	///
	/// Storage: Balances TotalIssuance (r:1 w:1)
	///
	/// Proof: Balances TotalIssuance (max_values: Some(1), max_size: Some(8), added: 503, mode:
	/// MaxEncodedLen)
	fn receive_two_messages_proof() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `693`
		//  Estimated: `54703`
		// Minimum execution time: 59_739 nanoseconds.
		Weight::from_parts(61_704_000, 54703)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	///
	/// Storage: Balances TotalIssuance (r:1 w:1)
	///
	/// Proof: Balances TotalIssuance (max_values: Some(1), max_size: Some(8), added: 503, mode:
	/// MaxEncodedLen)
	fn receive_single_message_proof_with_outbound_lane_state() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `693`
		//  Estimated: `54703`
		// Minimum execution time: 53_760 nanoseconds.
		Weight::from_parts(55_645_000, 54703)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	fn receive_single_message_proof_1_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `618`
		//  Estimated: `54200`
		// Minimum execution time: 49_582 nanoseconds.
		Weight::from_parts(51_250_000, 54200)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages InboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages InboundLanes (max_values: None, max_size: Some(49180), added:
	/// 51655, mode: MaxEncodedLen)
	fn receive_single_message_proof_16_kb() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `618`
		//  Estimated: `54200`
		// Minimum execution time: 76_418 nanoseconds.
		Weight::from_parts(77_877_000, 54200)
			.saturating_add(RocksDbWeight::get().reads(3_u64))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages OutboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages OutboundLanes (max_values: Some(1), max_size: Some(44), added:
	/// 539, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRelayers RelayerRewards (r:1 w:1)
	///
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(65), added: 2540,
	/// mode: MaxEncodedLen)
	fn receive_delivery_proof_for_single_message() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `579`
		//  Estimated: `5624`
		// Minimum execution time: 41_795 nanoseconds.
		Weight::from_parts(43_683_000, 5624)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages OutboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages OutboundLanes (max_values: Some(1), max_size: Some(44), added:
	/// 539, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRelayers RelayerRewards (r:1 w:1)
	///
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(65), added: 2540,
	/// mode: MaxEncodedLen)
	fn receive_delivery_proof_for_two_messages_by_single_relayer() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `596`
		//  Estimated: `5624`
		// Minimum execution time: 39_946 nanoseconds.
		Weight::from_parts(41_509_000, 5624)
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().writes(2_u64))
	}
	/// Storage: BridgeRialtoMessages PalletOperatingMode (r:1 w:0)
	///
	/// Proof: BridgeRialtoMessages PalletOperatingMode (max_values: Some(1), max_size: Some(2),
	/// added: 497, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoGrandpa ImportedHeaders (r:1 w:0)
	///
	/// Proof: BridgeRialtoGrandpa ImportedHeaders (max_values: Some(14400), max_size: Some(68),
	/// added: 2048, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRialtoMessages OutboundLanes (r:1 w:1)
	///
	/// Proof: BridgeRialtoMessages OutboundLanes (max_values: Some(1), max_size: Some(44), added:
	/// 539, mode: MaxEncodedLen)
	///
	/// Storage: BridgeRelayers RelayerRewards (r:2 w:2)
	///
	/// Proof: BridgeRelayers RelayerRewards (max_values: None, max_size: Some(65), added: 2540,
	/// mode: MaxEncodedLen)
	fn receive_delivery_proof_for_two_messages_by_two_relayers() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `596`
		//  Estimated: `8164`
		// Minimum execution time: 42_882 nanoseconds.
		Weight::from_parts(44_367_000, 8164)
			.saturating_add(RocksDbWeight::get().reads(5_u64))
			.saturating_add(RocksDbWeight::get().writes(3_u64))
	}
}
