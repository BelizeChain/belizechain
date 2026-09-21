//! Source-controlled network bootstrap helpers.
//!
//! Live-network bootstrap multiaddrs are operator-owned data: they belong in the
//! generated chain spec or the deployment inventory, never in source control.
//! Every helper here is therefore intentionally empty so the built-in config
//! cannot advertise unpublished hosts.
//!
//! The previous "production-grade validator and collator configuration" presets
//! (peer limits, reserved nodes, keystore/prometheus/database/transport
//! factories, `node_roles`) were never wired into the service — the running node
//! is configured by CLI flags and the deployment inventory. They were removed
//! rather than left in place claiming behaviour the node does not have. If those
//! presets are wanted later, they should be re-added together with the service
//! wiring and a rollout that applies them.

/// Network bootstrap nodes for peer discovery.
///
/// `mainnet()` and `testnet()` are deliberately empty and stay that way: live
/// bootnodes come from the operator-supplied chain spec, and shipping fake DNS
/// bootnodes in source control would be worse than shipping none.
pub struct BootstrapNodes;

impl BootstrapNodes {
    /// Mainnet bootnodes.
    ///
    /// Live-network bootstrap multiaddrs must be published in the operator-owned
    /// chain spec or deployment inventory. Source control intentionally keeps
    /// this empty so the built-in config cannot ship fake DNS bootnodes.
    pub fn mainnet() -> Vec<String> {
        vec![]
    }

    /// Testnet bootnodes.
    ///
    /// The active public-testnet flow exports an explicit raw spec with
    /// operator-provided bootnodes. Keep the source-controlled helper empty so
    /// it cannot advertise unpublished hosts.
    pub fn testnet() -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_network_bootnodes_default_empty() {
        assert!(
            BootstrapNodes::mainnet().is_empty(),
            "mainnet bootnodes must come from the operator chain spec"
        );
        assert!(
            BootstrapNodes::testnet().is_empty(),
            "testnet bootnodes must come from the operator chain spec"
        );
    }
}
