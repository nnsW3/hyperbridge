// Copyright (C) 2023 Polytope Labs.
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
// limitations under the License.

use codec::{Decode, Encode};
use geth_primitives::CodecHeader;
use ismp::{
	consensus::{StateMachineHeight, StateMachineId},
	events::StateMachineUpdated,
	messaging::ConsensusMessage,
};
use tesseract_primitives::{ByzantineHandler, IsmpHost, IsmpProvider};

use crate::BnbPosHost;

#[async_trait::async_trait]
impl ByzantineHandler for BnbPosHost {
	async fn query_consensus_message(
		&self,
		event: StateMachineUpdated,
	) -> Result<ConsensusMessage, anyhow::Error> {
		let header = self.prover.fetch_header(event.latest_height).await?;
		let message = ConsensusMessage {
			consensus_proof: header.encode(),
			consensus_state_id: self.consensus_state_id,
		};

		Ok(message)
	}

	async fn check_for_byzantine_attack<C: IsmpHost + IsmpProvider>(
		&self,
		counterparty: &C,
		consensus_message: ConsensusMessage,
	) -> Result<(), anyhow::Error> {
		let source_header = CodecHeader::decode(&mut &*consensus_message.consensus_proof)?;
		let finalized_state_root = source_header.state_root;
		let height = StateMachineHeight {
			id: StateMachineId {
				state_id: self.state_machine,
				consensus_state_id: self.consensus_state_id,
			},
			height: source_header.number.low_u64(),
		};
		let state_machine_commitment = counterparty.query_state_machine_commitment(height).await?;
		if finalized_state_root != state_machine_commitment.state_root {
			// Submit message
			log::info!(
				"Freezing {:?} on {:?}",
				self.state_machine,
				counterparty.state_machine_id().state_id
			);
			counterparty.freeze_state_machine(height.id).await?;
		}

		Ok(())
	}
}
