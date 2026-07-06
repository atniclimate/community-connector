use cn_model::OpId;
use serde::{Deserialize, Serialize};

use crate::fold::{GroupState, StoreReport};
use crate::op::Operation;

/// Authorization boundary implemented by cn-perm.
pub trait Authorizer {
    fn authorize(&self, state: &GroupState, op: &Operation) -> Result<(), AuthzDenial>;
}

/// Permission denial returned by an authorizer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthzDenial {
    pub code: String,
    pub message: String,
}

/// Submission result for each attempted operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmitOutcome {
    Applied(OpId),
    Denied { op: OpId, denial: AuthzDenial },
    Quarantined(OpId),
}

/// Authorizes and applies operations without embedding permission rules here.
pub fn submit<A: Authorizer>(
    state: &mut GroupState,
    authorizer: &A,
    ops: Vec<Operation>,
    report: &mut StoreReport,
) -> Vec<SubmitOutcome> {
    let mut outcomes = Vec::with_capacity(ops.len());
    for op in ops {
        match authorizer.authorize(state, &op) {
            Ok(()) => {
                let before_quarantine = report.quarantined.len();
                let op_id = op.op_id;
                state.apply(op, report);
                if report.quarantined[before_quarantine..]
                    .iter()
                    .any(|entry| entry.op_id == op_id)
                {
                    outcomes.push(SubmitOutcome::Quarantined(op_id));
                } else {
                    outcomes.push(SubmitOutcome::Applied(op_id));
                }
            }
            Err(denial) => outcomes.push(SubmitOutcome::Denied {
                op: op.op_id,
                denial,
            }),
        }
    }
    outcomes
}
