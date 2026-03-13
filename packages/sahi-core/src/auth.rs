use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

/// Supported application roles for bearer-token authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    PlatformAdmin,
    TenantAdmin,
    IssuerOperator,
    VerifierOperator,
    GuardOperator,
    ResidentUser,
    GuestUser,
    ServiceInternal,
}

impl Role {
    /// Canonical string form used in JWT claims.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlatformAdmin => "platform_admin",
            Self::TenantAdmin => "tenant_admin",
            Self::IssuerOperator => "issuer_operator",
            Self::VerifierOperator => "verifier_operator",
            Self::GuardOperator => "guard_operator",
            Self::ResidentUser => "resident_user",
            Self::GuestUser => "guest_user",
            Self::ServiceInternal => "service_internal",
        }
    }

    /// Parse a role name from token claims.
    #[must_use]
    pub fn from_claim(value: &str) -> Option<Self> {
        match value {
            "platform_admin" => Some(Self::PlatformAdmin),
            "tenant_admin" => Some(Self::TenantAdmin),
            "issuer_operator" => Some(Self::IssuerOperator),
            "verifier_operator" => Some(Self::VerifierOperator),
            "guard_operator" => Some(Self::GuardOperator),
            "resident_user" => Some(Self::ResidentUser),
            "guest_user" => Some(Self::GuestUser),
            "service_internal" => Some(Self::ServiceInternal),
            _ => None,
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Role {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_claim(s).ok_or("unknown role")
    }
}

/// Auth context extracted from a validated bearer token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthContext {
    pub tenant_id: String,
    pub subject: Option<String>,
    #[serde(default)]
    pub roles: BTreeSet<Role>,
    #[serde(default)]
    pub scopes: BTreeSet<String>,
    pub client_id: Option<String>,
    pub authorized_party: Option<String>,
}

impl AuthContext {
    #[must_use]
    pub fn new(
        tenant_id: impl Into<String>,
        subject: Option<String>,
        roles: BTreeSet<Role>,
        scopes: BTreeSet<String>,
        client_id: Option<String>,
        authorized_party: Option<String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            subject,
            roles,
            scopes,
            client_id,
            authorized_party,
        }
    }

    #[must_use]
    pub fn has_role(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }

    #[must_use]
    pub fn has_any_role(&self, roles: &[Role]) -> bool {
        roles.iter().any(|role| self.has_role(*role))
    }

    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(scope)
    }

    #[must_use]
    pub fn has_any_scope<'a>(&self, scopes: impl IntoIterator<Item = &'a str>) -> bool {
        scopes.into_iter().any(|scope| self.has_scope(scope))
    }

    #[must_use]
    pub fn actor_id(&self) -> Option<&str> {
        self.subject
            .as_deref()
            .or(self.client_id.as_deref())
            .or(self.authorized_party.as_deref())
    }
}
