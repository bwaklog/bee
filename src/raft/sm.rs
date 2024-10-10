// SM -> State Machine that is to be replicated and
// remain fault tolerant across the nodes
//
// raft works with consensus of a replicated state machine.
// Each server consists of a _state machine_ and _log_,
// goal is for this state machine to act as if, when accessed
// by an external client, it should be appear such that all
// the data is stored only one one node rather than a cluster,
//
// Hence regardless of a node failing, the state machine should
// remain intact per server.

#[allow(dead_code)]
pub struct StateMachine {}
