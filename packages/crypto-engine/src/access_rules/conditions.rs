//! Condition evaluators for access rules.

use chrono::Datelike;

use super::types::{parse_time, weekday_number, Condition, ConditionMatch, EvaluationContext};
use crate::credential_types::CredentialType;

/// Trait for evaluating conditions.
pub trait ConditionEvaluator: Send + Sync {
    /// Evaluate the condition against the context.
    fn evaluate(&self, ctx: &EvaluationContext) -> ConditionMatch;
}

/// Time window condition evaluator.
pub struct TimeWindowCondition {
    /// Start time (HH:MM).
    pub start: String,
    /// End time (HH:MM).
    pub end: String,
    /// Days of week (0=Sunday, 6=Saturday).
    pub days: Vec<u8>,
}

impl TimeWindowCondition {
    /// Create a new time window condition.
    #[must_use]
    pub fn new(start: impl Into<String>, end: impl Into<String>, days: Vec<u8>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
            days,
        }
    }

    /// Create a weekday-only condition (Monday-Friday).
    #[must_use]
    pub fn weekdays(start: impl Into<String>, end: impl Into<String>) -> Self {
        Self::new(start, end, vec![1, 2, 3, 4, 5])
    }

    /// Create a weekend-only condition (Saturday-Sunday).
    #[must_use]
    pub fn weekends(start: impl Into<String>, end: impl Into<String>) -> Self {
        Self::new(start, end, vec![0, 6])
    }

    /// Create an all-days condition.
    #[must_use]
    pub fn all_days(start: impl Into<String>, end: impl Into<String>) -> Self {
        Self::new(start, end, vec![0, 1, 2, 3, 4, 5, 6])
    }
}

impl ConditionEvaluator for TimeWindowCondition {
    fn evaluate(&self, ctx: &EvaluationContext) -> ConditionMatch {
        // Parse time bounds
        let Some(start_time) = parse_time(&self.start) else {
            return ConditionMatch::unmatched("time_window", "Invalid start time format");
        };
        let Some(end_time) = parse_time(&self.end) else {
            return ConditionMatch::unmatched("time_window", "Invalid end time format");
        };

        // Get current time components
        let current = ctx.current_time;
        let current_time = current.time();
        let current_weekday = weekday_number(current.weekday());

        // Check day of week
        if !self.days.contains(&current_weekday) {
            return ConditionMatch::unmatched(
                "time_window",
                format!("Day {} not in allowed days {:?}", current_weekday, self.days),
            );
        }

        // Check time window (handle overnight windows like 22:00-06:00)
        let in_window = if start_time <= end_time {
            // Normal window: 09:00-17:00
            current_time >= start_time && current_time <= end_time
        } else {
            // Overnight window: 22:00-06:00
            current_time >= start_time || current_time <= end_time
        };

        if in_window {
            ConditionMatch::matched(
                "time_window",
                format!("Time {} is within {}-{}", current_time, self.start, self.end),
            )
        } else {
            ConditionMatch::unmatched(
                "time_window",
                format!(
                    "Time {} is outside {}-{}",
                    current_time, self.start, self.end
                ),
            )
        }
    }
}

/// Floor access condition evaluator.
pub struct FloorAccessCondition {
    /// Allowed floors.
    pub floors: Vec<String>,
}

impl FloorAccessCondition {
    /// Create a new floor access condition.
    #[must_use]
    pub fn new(floors: Vec<String>) -> Self {
        Self { floors }
    }

    /// Create from a slice of floor strings.
    #[must_use]
    pub fn from_slice(floors: &[&str]) -> Self {
        Self {
            floors: floors.iter().map(|s| (*s).to_string()).collect(),
        }
    }
}

impl ConditionEvaluator for FloorAccessCondition {
    fn evaluate(&self, ctx: &EvaluationContext) -> ConditionMatch {
        // Check if target floor is in allowed floors
        if self.floors.contains(&ctx.target_floor) {
            ConditionMatch::matched(
                "floor_access",
                format!("Floor {} is in allowed floors", ctx.target_floor),
            )
        } else if self.floors.iter().any(|f| f == "*") {
            // Wildcard allows all floors
            ConditionMatch::matched("floor_access", "All floors allowed")
        } else {
            ConditionMatch::unmatched(
                "floor_access",
                format!(
                    "Floor {} not in allowed floors {:?}",
                    ctx.target_floor, self.floors
                ),
            )
        }
    }
}

/// Role condition evaluator.
pub struct RoleCondition {
    /// Allowed roles.
    pub roles: Vec<String>,
}

impl RoleCondition {
    /// Create a new role condition.
    #[must_use]
    pub fn new(roles: Vec<String>) -> Self {
        Self { roles }
    }

    /// Create from a slice of role strings.
    #[must_use]
    pub fn from_slice(roles: &[&str]) -> Self {
        Self {
            roles: roles.iter().map(|s| (*s).to_string()).collect(),
        }
    }
}

impl ConditionEvaluator for RoleCondition {
    fn evaluate(&self, ctx: &EvaluationContext) -> ConditionMatch {
        if self.roles.contains(&ctx.role) {
            ConditionMatch::matched("role", format!("Role {} is allowed", ctx.role))
        } else if self.roles.iter().any(|r| r == "*") {
            ConditionMatch::matched("role", "All roles allowed")
        } else {
            ConditionMatch::unmatched(
                "role",
                format!("Role {} not in allowed roles {:?}", ctx.role, self.roles),
            )
        }
    }
}

/// Clearance level condition evaluator.
pub struct ClearanceCondition {
    /// Minimum required clearance (0-3).
    pub min_level: u8,
}

impl ClearanceCondition {
    /// Create a new clearance condition.
    #[must_use]
    pub fn new(min_level: u8) -> Self {
        Self { min_level }
    }
}

impl ConditionEvaluator for ClearanceCondition {
    fn evaluate(&self, ctx: &EvaluationContext) -> ConditionMatch {
        if ctx.clearance >= self.min_level {
            ConditionMatch::matched(
                "clearance",
                format!(
                    "Clearance {} meets minimum {}",
                    ctx.clearance, self.min_level
                ),
            )
        } else {
            ConditionMatch::unmatched(
                "clearance",
                format!(
                    "Clearance {} below minimum {}",
                    ctx.clearance, self.min_level
                ),
            )
        }
    }
}

/// Credential type condition evaluator.
pub struct CredentialTypeCondition {
    /// Allowed credential types.
    pub types: Vec<CredentialType>,
}

impl CredentialTypeCondition {
    /// Create a new credential type condition.
    #[must_use]
    pub fn new(types: Vec<CredentialType>) -> Self {
        Self { types }
    }

    /// Create for residents only.
    #[must_use]
    pub fn residents_only() -> Self {
        Self::new(vec![CredentialType::ResidentBadge])
    }

    /// Create for visitors only.
    #[must_use]
    pub fn visitors_only() -> Self {
        Self::new(vec![CredentialType::VisitorPass])
    }

    /// Create for all credential types.
    #[must_use]
    pub fn all_types() -> Self {
        Self::new(vec![
            CredentialType::ResidentBadge,
            CredentialType::VisitorPass,
            CredentialType::ContractorBadge,
            CredentialType::EmergencyAccess,
        ])
    }
}

impl ConditionEvaluator for CredentialTypeCondition {
    fn evaluate(&self, ctx: &EvaluationContext) -> ConditionMatch {
        if self.types.contains(&ctx.credential_type) {
            ConditionMatch::matched(
                "credential_type",
                format!("Credential type {:?} is allowed", ctx.credential_type),
            )
        } else {
            ConditionMatch::unmatched(
                "credential_type",
                format!(
                    "Credential type {:?} not in allowed types {:?}",
                    ctx.credential_type, self.types
                ),
            )
        }
    }
}

/// Evaluate a condition from the enum.
pub fn evaluate_condition(condition: &Condition, ctx: &EvaluationContext) -> ConditionMatch {
    match condition {
        Condition::TimeWindow { start, end, days } => {
            TimeWindowCondition::new(start, end, days.clone()).evaluate(ctx)
        }
        Condition::FloorAccess { floors } => FloorAccessCondition::new(floors.clone()).evaluate(ctx),
        Condition::Role { roles } => RoleCondition::new(roles.clone()).evaluate(ctx),
        Condition::Clearance { min_level } => ClearanceCondition::new(*min_level).evaluate(ctx),
        Condition::CredentialType { types } => {
            CredentialTypeCondition::new(types.clone()).evaluate(ctx)
        }
        Condition::Zone { zones } => {
            // Zone condition
            let Some(target_zone) = &ctx.target_zone else {
                return ConditionMatch::unmatched("zone", "No target zone specified");
            };
            if zones.contains(target_zone) || zones.iter().any(|z| z == "*") {
                ConditionMatch::matched("zone", format!("Zone {target_zone} is allowed"))
            } else {
                ConditionMatch::unmatched(
                    "zone",
                    format!("Zone {target_zone} not in allowed zones {zones:?}"),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn test_context() -> EvaluationContext {
        EvaluationContext::new(
            "TNT_01HXK",
            "PRY_01HXK",
            CredentialType::ResidentBadge,
            "owner",
            "L12",
        )
        .with_clearance(2)
        .with_floors(vec!["L12".to_string(), "L13".to_string()])
        .with_zone("residential")
    }

    #[test]
    fn time_window_within_window() {
        // Tuesday at 10:30
        let time = Utc.with_ymd_and_hms(2026, 3, 10, 10, 30, 0).unwrap();
        let ctx = test_context().at_time(time);

        let condition = TimeWindowCondition::weekdays("09:00", "17:00");
        let result = condition.evaluate(&ctx);

        assert!(result.matched);
    }

    #[test]
    fn time_window_outside_window() {
        // Tuesday at 19:00
        let time = Utc.with_ymd_and_hms(2026, 3, 10, 19, 0, 0).unwrap();
        let ctx = test_context().at_time(time);

        let condition = TimeWindowCondition::weekdays("09:00", "17:00");
        let result = condition.evaluate(&ctx);

        assert!(!result.matched);
    }

    #[test]
    fn time_window_wrong_day() {
        // Sunday at 10:30 (weekday condition)
        let time = Utc.with_ymd_and_hms(2026, 3, 15, 10, 30, 0).unwrap();
        let ctx = test_context().at_time(time);

        let condition = TimeWindowCondition::weekdays("09:00", "17:00");
        let result = condition.evaluate(&ctx);

        assert!(!result.matched);
    }

    #[test]
    fn time_window_overnight() {
        // Night shift: 22:00-06:00
        let condition = TimeWindowCondition::all_days("22:00", "06:00");

        // 23:00 should match
        let time1 = Utc.with_ymd_and_hms(2026, 3, 10, 23, 0, 0).unwrap();
        let ctx1 = test_context().at_time(time1);
        assert!(condition.evaluate(&ctx1).matched);

        // 03:00 should match
        let time2 = Utc.with_ymd_and_hms(2026, 3, 10, 3, 0, 0).unwrap();
        let ctx2 = test_context().at_time(time2);
        assert!(condition.evaluate(&ctx2).matched);

        // 12:00 should not match
        let time3 = Utc.with_ymd_and_hms(2026, 3, 10, 12, 0, 0).unwrap();
        let ctx3 = test_context().at_time(time3);
        assert!(!condition.evaluate(&ctx3).matched);
    }

    #[test]
    fn floor_access_allowed() {
        let ctx = test_context();
        let condition = FloorAccessCondition::from_slice(&["L12", "L13", "L14"]);

        let result = condition.evaluate(&ctx);
        assert!(result.matched);
    }

    #[test]
    fn floor_access_denied() {
        let ctx = test_context();
        let condition = FloorAccessCondition::from_slice(&["L1", "L2"]);

        let result = condition.evaluate(&ctx);
        assert!(!result.matched);
    }

    #[test]
    fn floor_access_wildcard() {
        let ctx = test_context();
        let condition = FloorAccessCondition::from_slice(&["*"]);

        let result = condition.evaluate(&ctx);
        assert!(result.matched);
    }

    #[test]
    fn role_allowed() {
        let ctx = test_context();
        let condition = RoleCondition::from_slice(&["owner", "tenant"]);

        let result = condition.evaluate(&ctx);
        assert!(result.matched);
    }

    #[test]
    fn role_denied() {
        let ctx = test_context();
        let condition = RoleCondition::from_slice(&["visitor", "contractor"]);

        let result = condition.evaluate(&ctx);
        assert!(!result.matched);
    }

    #[test]
    fn clearance_sufficient() {
        let ctx = test_context(); // clearance = 2
        let condition = ClearanceCondition::new(2);

        let result = condition.evaluate(&ctx);
        assert!(result.matched);
    }

    #[test]
    fn clearance_insufficient() {
        let ctx = test_context(); // clearance = 2
        let condition = ClearanceCondition::new(3);

        let result = condition.evaluate(&ctx);
        assert!(!result.matched);
    }

    #[test]
    fn credential_type_allowed() {
        let ctx = test_context(); // ResidentBadge
        let condition = CredentialTypeCondition::residents_only();

        let result = condition.evaluate(&ctx);
        assert!(result.matched);
    }

    #[test]
    fn credential_type_denied() {
        let ctx = test_context(); // ResidentBadge
        let condition = CredentialTypeCondition::visitors_only();

        let result = condition.evaluate(&ctx);
        assert!(!result.matched);
    }

    #[test]
    fn evaluate_condition_enum() {
        let ctx = test_context();

        let condition = Condition::Role {
            roles: vec!["owner".to_string()],
        };
        let result = evaluate_condition(&condition, &ctx);
        assert!(result.matched);

        let condition = Condition::Clearance { min_level: 1 };
        let result = evaluate_condition(&condition, &ctx);
        assert!(result.matched);

        let condition = Condition::Zone {
            zones: vec!["residential".to_string()],
        };
        let result = evaluate_condition(&condition, &ctx);
        assert!(result.matched);
    }
}
