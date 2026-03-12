//! Business Logic Rules Engine (VP-3c)
//!
//! Access decision rules evaluated at the edge verifier.
//!
//! Per MASTER_PLAN section 8.5:
//! - Rules evaluate conditions: TimeWindow, FloorAccess, Role, Clearance, CredentialType
//! - Actions: Allow, Deny, AllowWithLog
//! - Priority-based evaluation (higher priority wins on conflict)

mod conditions;
mod engine;
mod types;

pub use conditions::{
    ClearanceCondition, CredentialTypeCondition, FloorAccessCondition, RoleCondition,
    TimeWindowCondition,
};
pub use engine::RulesEngine;
pub use types::{
    AccessAction, AccessDecision, AccessRule, Condition, ConditionMatch, EvaluationContext, RuleSet,
};
