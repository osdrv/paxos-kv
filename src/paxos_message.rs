use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct ProposalId {
    pub id: u64,
    pub node_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PaxosValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaxosMessage {
    Prepare {
        proposal_id: ProposalId,
    },

    Promise {
        proposal_id: ProposalId,
        last_accepted_id: Option<ProposalId>,
        last_accepted_value: Option<PaxosValue>,
    },

    Accept {
        proposal_id: ProposalId,
        value: PaxosValue,
    },

    Accepted {
        proposal_id: ProposalId,
        value: PaxosValue,
    },

    Learn {
        proposal_id: ProposalId,
        value: PaxosValue,
    },
}
