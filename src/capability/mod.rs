//! Capability-based permission model.
//!
//! Controls which RPC domains and operations a client can access.
//! Loaded from VCLI_CAPABILITY environment variable (comma-separated list).

use std::collections::HashSet;
use std::env;

/// High-level capability categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    Schematic,
    Maestro,
    Window,
    Cell,
    Simulation,
    Transaction,
    /// Allow raw SKILL exec (dangerous — local dev only)
    Admin,
}

impl Capability {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "schematic" => Some(Self::Schematic),
            "maestro" => Some(Self::Maestro),
            "window" => Some(Self::Window),
            "cell" => Some(Self::Cell),
            "simulation" => Some(Self::Simulation),
            "transaction" => Some(Self::Transaction),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    /// Returns the domain prefix used in RPC method names.
    pub fn domain(&self) -> &'static str {
        match self {
            Self::Schematic => "schematic",
            Self::Maestro => "maestro",
            Self::Window => "window",
            Self::Cell => "cell",
            Self::Simulation => "simulation",
            Self::Transaction => "transaction",
            Self::Admin => "*",
        }
    }

    /// Returns true if this capability allows admin/raw skill exec.
    pub fn is_admin(&self) -> bool {
        *self == Self::Admin
    }
}

/// A set of capabilities loaded from environment.
#[derive(Debug, Clone)]
pub struct CapabilitySet(HashSet<Capability>);

impl CapabilitySet {
    /// Load from VCLI_CAPABILITY env var (comma-separated).
    pub fn from_env() -> Self {
        let caps = env::var("VCLI_CAPABILITY")
            .ok()
            .map(|s| {
                s.split(',')
                    .filter_map(|part| Capability::from_str(part.trim()))
                    .collect()
            })
            .unwrap_or_else(|| {
                // Default: allow all capabilities (existing behavior)
                let mut set = HashSet::new();
                set.insert(Capability::Schematic);
                set.insert(Capability::Maestro);
                set.insert(Capability::Window);
                set.insert(Capability::Cell);
                set.insert(Capability::Simulation);
                set.insert(Capability::Transaction);
                set
            });
        Self(caps)
    }

    /// Check if this set includes the given capability.
    pub fn permits(&self, cap: Capability) -> bool {
        self.0.contains(&cap) || self.0.contains(&Capability::Admin)
    }

    /// Check if a specific RPC method name is permitted.
    /// Method names are "domain.operation" (e.g. "schematic.place").
    pub fn permits_method(&self, method: &str) -> bool {
        let domain = method.split('.').next().unwrap_or("");
        match domain {
            "schematic" => self.permits(Capability::Schematic),
            "maestro" => self.permits(Capability::Maestro),
            "window" => self.permits(Capability::Window),
            "cell" => self.permits(Capability::Cell),
            _ => false,
        }
    }

    /// Returns true if raw SKILL exec is allowed (Admin capability).
    pub fn allows_raw_skill(&self) -> bool {
        self.0.contains(&Capability::Admin)
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_domain() {
        assert_eq!(Capability::Schematic.domain(), "schematic");
        assert_eq!(Capability::Maestro.domain(), "maestro");
        assert_eq!(Capability::Window.domain(), "window");
        assert_eq!(Capability::Cell.domain(), "cell");
    }

    #[test]
    fn permits_method() {
        let caps = CapabilitySet(HashSet::from([Capability::Schematic, Capability::Maestro]));
        assert!(caps.permits_method("schematic.place"));
        assert!(caps.permits_method("maestro.open_session"));
        assert!(!caps.permits_method("window.list"));
        assert!(!caps.permits_method("cell.open"));
    }

    #[test]
    fn admin_allows_everything() {
        let caps = CapabilitySet(HashSet::from([Capability::Admin]));
        assert!(caps.permits_method("schematic.place"));
        assert!(caps.permits_method("maestro.run"));
        assert!(caps.permits_method("window.list"));
        assert!(caps.permits_method("cell.open"));
    }

    #[test]
    fn capability_from_str() {
        assert_eq!(
            Capability::from_str("schematic"),
            Some(Capability::Schematic)
        );
        assert_eq!(Capability::from_str("MAESTRO"), Some(Capability::Maestro));
        assert_eq!(Capability::from_str("admin"), Some(Capability::Admin));
        assert_eq!(Capability::from_str("unknown"), None);
    }
}
