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

use crate::messages_source::SubstrateMessagesProof;
use crate::messages_target::SubstrateMessagesReceivingProof;
use crate::on_demand_headers::OnDemandHeadersRelay;

use bp_messages::{LaneId, MessageNonce};
use frame_support::weights::Weight;
use messages_relay::message_lane::{MessageLane, SourceHeaderIdOf, TargetHeaderIdOf};
use relay_substrate_client::{
	metrics::{FloatStorageValueMetric, StorageProofOverheadMetric},
	BlockNumberOf, Chain, Client, HashOf,
};
use relay_utils::{
	metrics::{F64SharedRef, MetricsParams},
	BlockNumberBase,
};
use sp_core::{storage::StorageKey, Bytes};
use sp_runtime::FixedU128;
use std::ops::RangeInclusive;

/// Substrate <-> Substrate messages relay parameters.
pub struct MessagesRelayParams<SC: Chain, SS, TC: Chain, TS> {
	/// Messages source client.
	pub source_client: Client<SC>,
	/// Sign parameters for messages source chain.
	pub source_sign: SS,
	/// Messages target client.
	pub target_client: Client<TC>,
	/// Sign parameters for messages target chain.
	pub target_sign: TS,
	/// Optional on-demand source to target headers relay.
	pub source_to_target_headers_relay: Option<OnDemandHeadersRelay<SC>>,
	/// Optional on-demand target to source headers relay.
	pub target_to_source_headers_relay: Option<OnDemandHeadersRelay<TC>>,
	/// Identifier of lane that needs to be served.
	pub lane_id: LaneId,
	/// Metrics parameters.
	pub metrics_params: MetricsParams,
}

/// Message sync pipeline for Substrate <-> Substrate relays.
pub trait SubstrateMessageLane: MessageLane {
	/// Name of the runtime method that returns dispatch weight of outbound messages at the source chain.
	const OUTBOUND_LANE_MESSAGE_DETAILS_METHOD: &'static str;
	/// Name of the runtime method that returns latest generated nonce at the source chain.
	const OUTBOUND_LANE_LATEST_GENERATED_NONCE_METHOD: &'static str;
	/// Name of the runtime method that returns latest received (confirmed) nonce at the the source chain.
	const OUTBOUND_LANE_LATEST_RECEIVED_NONCE_METHOD: &'static str;

	/// Name of the runtime method that returns latest received nonce at the target chain.
	const INBOUND_LANE_LATEST_RECEIVED_NONCE_METHOD: &'static str;
	/// Name of the runtime method that returns latest confirmed (reward-paid) nonce at the target chain.
	const INBOUND_LANE_LATEST_CONFIRMED_NONCE_METHOD: &'static str;
	/// Numebr of the runtime method that returns state of "unrewarded relayers" set at the target chain.
	const INBOUND_LANE_UNREWARDED_RELAYERS_STATE: &'static str;

	/// Name of the runtime method that returns id of best finalized source header at target chain.
	const BEST_FINALIZED_SOURCE_HEADER_ID_AT_TARGET: &'static str;
	/// Name of the runtime method that returns id of best finalized target header at source chain.
	const BEST_FINALIZED_TARGET_HEADER_ID_AT_SOURCE: &'static str;

	/// Source chain.
	type SourceChain: Chain;
	/// Target chain.
	type TargetChain: Chain;

	/// Returns id of account that we're using to sign transactions at target chain (messages proof).
	fn target_transactions_author(&self) -> <Self::TargetChain as Chain>::AccountId;

	/// Make messages delivery transaction.
	fn make_messages_delivery_transaction(
		&self,
		transaction_nonce: <Self::TargetChain as Chain>::Index,
		generated_at_header: SourceHeaderIdOf<Self>,
		nonces: RangeInclusive<MessageNonce>,
		proof: Self::MessagesProof,
	) -> Bytes;

	/// Returns id of account that we're using to sign transactions at source chain (delivery proof).
	fn source_transactions_author(&self) -> <Self::SourceChain as Chain>::AccountId;

	/// Make messages receiving proof transaction.
	fn make_messages_receiving_proof_transaction(
		&self,
		transaction_nonce: <Self::SourceChain as Chain>::Index,
		generated_at_header: TargetHeaderIdOf<Self>,
		proof: Self::MessagesReceivingProof,
	) -> Bytes;
}

/// Substrate-to-Substrate message lane.
#[derive(Debug)]
pub struct SubstrateMessageLaneToSubstrate<Source: Chain, SourceSignParams, Target: Chain, TargetSignParams> {
	/// Client for the source Substrate chain.
	pub(crate) source_client: Client<Source>,
	/// Parameters required to sign transactions for source chain.
	pub(crate) source_sign: SourceSignParams,
	/// Client for the target Substrate chain.
	pub(crate) target_client: Client<Target>,
	/// Parameters required to sign transactions for target chain.
	pub(crate) target_sign: TargetSignParams,
	/// Account id of relayer at the source chain.
	pub(crate) relayer_id_at_source: Source::AccountId,
}

impl<Source: Chain, SourceSignParams: Clone, Target: Chain, TargetSignParams: Clone> Clone
	for SubstrateMessageLaneToSubstrate<Source, SourceSignParams, Target, TargetSignParams>
{
	fn clone(&self) -> Self {
		Self {
			source_client: self.source_client.clone(),
			source_sign: self.source_sign.clone(),
			target_client: self.target_client.clone(),
			target_sign: self.target_sign.clone(),
			relayer_id_at_source: self.relayer_id_at_source.clone(),
		}
	}
}

impl<Source: Chain, SourceSignParams, Target: Chain, TargetSignParams> MessageLane
	for SubstrateMessageLaneToSubstrate<Source, SourceSignParams, Target, TargetSignParams>
where
	SourceSignParams: Clone + Send + Sync + 'static,
	TargetSignParams: Clone + Send + Sync + 'static,
	BlockNumberOf<Source>: BlockNumberBase,
	BlockNumberOf<Target>: BlockNumberBase,
{
	const SOURCE_NAME: &'static str = Source::NAME;
	const TARGET_NAME: &'static str = Target::NAME;

	type MessagesProof = SubstrateMessagesProof<Source>;
	type MessagesReceivingProof = SubstrateMessagesReceivingProof<Target>;

	type SourceChainBalance = Source::Balance;
	type SourceHeaderNumber = BlockNumberOf<Source>;
	type SourceHeaderHash = HashOf<Source>;

	type TargetHeaderNumber = BlockNumberOf<Target>;
	type TargetHeaderHash = HashOf<Target>;
}

/// Returns maximal number of messages and their maximal cumulative dispatch weight, based
/// on given chain parameters.
pub fn select_delivery_transaction_limits<W: pallet_bridge_messages::WeightInfoExt>(
	max_extrinsic_weight: Weight,
	max_unconfirmed_messages_at_inbound_lane: MessageNonce,
) -> (MessageNonce, Weight) {
	// We may try to guess accurate value, based on maximal number of messages and per-message
	// weight overhead, but the relay loop isn't using this info in a super-accurate way anyway.
	// So just a rough guess: let's say 1/3 of max tx weight is for tx itself and the rest is
	// for messages dispatch.

	// Another thing to keep in mind is that our runtimes (when this code was written) accept
	// messages with dispatch weight <= max_extrinsic_weight/2. So we can't reserve less than
	// that for dispatch.

	let weight_for_delivery_tx = max_extrinsic_weight / 3;
	let weight_for_messages_dispatch = max_extrinsic_weight - weight_for_delivery_tx;

	let delivery_tx_base_weight =
		W::receive_messages_proof_overhead() + W::receive_messages_proof_outbound_lane_state_overhead();
	let delivery_tx_weight_rest = weight_for_delivery_tx - delivery_tx_base_weight;
	let max_number_of_messages = std::cmp::min(
		delivery_tx_weight_rest / W::receive_messages_proof_messages_overhead(1),
		max_unconfirmed_messages_at_inbound_lane,
	);

	assert!(
		max_number_of_messages > 0,
		"Relay should fit at least one message in every delivery transaction",
	);
	assert!(
		weight_for_messages_dispatch >= max_extrinsic_weight / 2,
		"Relay shall be able to deliver messages with dispatch weight = max_extrinsic_weight / 2",
	);

	(max_number_of_messages, weight_for_messages_dispatch)
}

/// Shared references to the values of standalone metrics of the message lane relay loop.
#[derive(Debug, Clone)]
pub struct StandaloneMessagesMetrics {
	/// Shared reference to the actual target -> <base> chain token conversion rate.
	pub target_to_base_conversion_rate: Option<F64SharedRef>,
	/// Shared reference to the actual source -> <base> chain token conversion rate.
	pub source_to_base_conversion_rate: Option<F64SharedRef>,
}

/// Add general standalone metrics for the message lane relay loop.
pub fn add_standalone_metrics<P: SubstrateMessageLane>(
	metrics_params: MetricsParams,
	source_client: Client<P::SourceChain>,
	source_chain_token_id: Option<&str>,
	target_chain_token_id: Option<&str>,
	target_to_source_conversion_rate_params: Option<(StorageKey, FixedU128)>,
) -> anyhow::Result<(MetricsParams, StandaloneMessagesMetrics)> {
	let mut source_to_base_conversion_rate = None;
	let mut target_to_base_conversion_rate = None;
	let mut metrics_params =
		relay_utils::relay_metrics(None, metrics_params).standalone_metric(|registry, prefix| {
			StorageProofOverheadMetric::new(
				registry,
				prefix,
				source_client.clone(),
				format!("{}_storage_proof_overhead", P::SourceChain::NAME.to_lowercase()),
				format!("{} storage proof overhead", P::SourceChain::NAME),
			)
		})?;
	if let Some((target_to_source_conversion_rate_storage_key, initial_target_to_source_conversion_rate)) =
		target_to_source_conversion_rate_params
	{
		metrics_params = metrics_params.standalone_metric(|registry, prefix| {
			let metric = FloatStorageValueMetric::<_, sp_runtime::FixedU128>::new(
				registry,
				prefix,
				source_client,
				target_to_source_conversion_rate_storage_key,
				Some(initial_target_to_source_conversion_rate),
				format!(
					"{}_{}_to_{}_conversion_rate",
					P::SourceChain::NAME,
					P::TargetChain::NAME,
					P::SourceChain::NAME
				),
				format!(
					"{} to {} tokens conversion rate (used by {})",
					P::TargetChain::NAME,
					P::SourceChain::NAME,
					P::SourceChain::NAME
				),
			)?;
			Ok(metric)
		})?;
	}
	if let Some(source_chain_token_id) = source_chain_token_id {
		metrics_params = metrics_params.standalone_metric(|registry, prefix| {
			let metric = crate::chains::token_price_metric(registry, prefix, source_chain_token_id)?;
			source_to_base_conversion_rate = Some(metric.shared_value_ref());
			Ok(metric)
		})?;
	}
	if let Some(target_chain_token_id) = target_chain_token_id {
		metrics_params = metrics_params.standalone_metric(|registry, prefix| {
			let metric = crate::chains::token_price_metric(registry, prefix, target_chain_token_id)?;
			target_to_base_conversion_rate = Some(metric.shared_value_ref());
			Ok(metric)
		})?;
	}
	Ok((
		metrics_params.into_params(),
		StandaloneMessagesMetrics {
			source_to_base_conversion_rate,
			target_to_base_conversion_rate,
		},
	))
}

#[cfg(test)]
mod tests {
	use super::*;

	type RialtoToMillauMessagesWeights = pallet_bridge_messages::weights::RialtoWeight<rialto_runtime::Runtime>;

	#[test]
	fn select_delivery_transaction_limits_works() {
		let (max_count, max_weight) = select_delivery_transaction_limits::<RialtoToMillauMessagesWeights>(
			bp_millau::max_extrinsic_weight(),
			bp_millau::MAX_UNREWARDED_RELAYER_ENTRIES_AT_INBOUND_LANE,
		);
		assert_eq!(
			(max_count, max_weight),
			// We don't actually care about these values, so feel free to update them whenever test
			// fails. The only thing to do before that is to ensure that new values looks sane: i.e. weight
			// reserved for messages dispatch allows dispatch of non-trivial messages.
			//
			// Any significant change in this values should attract additional attention.
			(782, 216_583_333_334),
		);
	}
}
