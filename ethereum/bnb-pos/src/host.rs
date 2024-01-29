// Copyright (C) 2023 Polytope Labs.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
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
use codec::Encode;
use futures::StreamExt;
use ismp::messaging::{ConsensusMessage, CreateConsensusState};
use jsonrpsee::rpc_params;

use crate::{notification::consensus_notification, BnbPosHost, KeccakHasher};
use tesseract_primitives::{BoxStream, IsmpHost, IsmpProvider};

#[async_trait::async_trait]
impl IsmpHost for BnbPosHost {
	async fn consensus_notification<C>(
		&self,
		counterparty: C,
	) -> Result<BoxStream<ConsensusMessage>, anyhow::Error>
	where
		C: IsmpHost + IsmpProvider + 'static,
	{
		let client = BnbPosHost::clone(&self);

		let sub = self
			.rpc_client
			.subscribe(
				"eth_subscribe".to_string(),
				rpc_params!("newHeads"),
				"eth_unsubscribe".to_string(),
			)
			.await?;
		let stream = sub.filter_map(move |res| {
			let client = client.clone();
			let counterparty = counterparty.clone();
			async move {
				match res {
					Ok(raw) => {
						let header = serde_json::from_str(raw.get()).ok()?;
						consensus_notification(&client, counterparty, header)
							.await
							.ok()
							.flatten()
							.map(|update| {
								Ok(ConsensusMessage {
									consensus_proof: update.encode(),
									consensus_state_id: client.consensus_state_id,
								})
							})
					},
					_ => None,
				}
			}
		});

		Ok(Box::pin(stream))
	}

	async fn get_initial_consensus_state(&self) -> Result<Option<CreateConsensusState>, Error> {
		let initial_consensus_state = self
			.get_consensus_state::<KeccakHasher>(self.config.evm_config.ismp_host)
			.await?;
		Ok(Some(CreateConsensusState {
			consensus_state: initial_consensus_state.encode(),
			consensus_client_id: ismp_bnb_pos::BNB_CONSENSUS_ID,
			consensus_state_id: self.consensus_state_id,
			unbonding_period: 60 * 60 * 60 * 27,
			challenge_period: 5 * 60,
			state_machine_commitments: vec![],
		}))
	}
}
