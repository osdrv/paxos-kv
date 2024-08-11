use crate::paxos_message::PaxosMessage;
use crate::paxos_message::PaxosValue;
use crate::paxos_message::ProposalId;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PaxosNode {
    node_id: u64,
    current_proposal_id: Option<ProposalId>,
    promised_id: Option<ProposalId>,
    accepted_id: Option<ProposalId>,
    accepted_value: Option<PaxosValue>,
    learned_values: HashMap<String, String>,
}

impl PaxosNode {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            current_proposal_id: None,
            promised_id: None,
            accepted_id: None,
            accepted_value: None,
            learned_values: HashMap::new(),
        }
    }

    pub fn start_proposal(
        &mut self,
        proposal_id: u64,
        _key: String,
        _value: String,
    ) -> PaxosMessage {
        let proposal_id = ProposalId {
            id: proposal_id,
            node_id: self.node_id,
        };

        self.current_proposal_id = Some(proposal_id.clone());

        PaxosMessage::Prepare { proposal_id }
    }

    pub fn handle_promise(&mut self, message: PaxosMessage) -> Option<PaxosMessage> {
        println!("Node {:?} recv msg: {:?}", self.node_id, message);
        if let PaxosMessage::Promise {
            proposal_id,
            last_accepted_id: _,
            last_accepted_value,
        } = message
        {
            if let Some(ref current_proposal_id) = self.current_proposal_id {
                if proposal_id.eq(current_proposal_id) {
                    let value = last_accepted_value.unwrap_or(PaxosValue {
                        key: "default".to_string(),
                        value: "default".to_string(),
                    });

                    return Some(PaxosMessage::Accept { proposal_id, value });
                }
            }
        }
        None
    }

    pub fn handle_prepare(&mut self, message: PaxosMessage) -> Option<PaxosMessage> {
        println!("Node {:?} recv msg: {:?}", self.node_id, message);
        if let PaxosMessage::Prepare { proposal_id } = message {
            if self.promised_id.is_none() || proposal_id.gt(self.promised_id.as_ref().unwrap()) {
                // proposal_id.gt(self.promised_id.unwrap()) {
                self.promised_id = Some(proposal_id.clone());

                return Some(PaxosMessage::Promise {
                    proposal_id,
                    last_accepted_id: self.accepted_id.clone(),
                    last_accepted_value: self.accepted_value.clone(),
                });
            }
        }
        None
    }

    pub fn handle_accept(&mut self, message: PaxosMessage) -> Option<PaxosMessage> {
        println!("Node {:?} recv msg: {:?}", self.node_id, message);
        if let PaxosMessage::Accept { proposal_id, value } = message {
            if self.promised_id.is_none() || proposal_id.ge(self.promised_id.as_ref().unwrap()) {
                self.accepted_id = Some(proposal_id.clone());
                self.accepted_value = Some(value.clone());

                return Some(PaxosMessage::Accepted { proposal_id, value });
            }
        }
        None
    }

    pub fn handle_accepted(&mut self, message: PaxosMessage) -> Option<PaxosMessage> {
        if let PaxosMessage::Accepted { proposal_id, value } = message {
            self.learned_values
                .insert(value.key.clone(), value.value.clone());

            return Some(PaxosMessage::Learn { proposal_id, value });
        }
        None
    }

    pub fn handle_learn(&mut self, message: PaxosMessage) {
        println!("Node {:?} recv msg: {:?}", self.node_id, message);
        if let PaxosMessage::Learn { value, .. } = message {
            self.learned_values
                .insert(value.key.clone(), value.value.clone());
        }
    }
}
