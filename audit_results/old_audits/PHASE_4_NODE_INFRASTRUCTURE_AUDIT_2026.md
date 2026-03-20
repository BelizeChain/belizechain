# Phase 4: Node Binary & Infrastructure Audit

**Audit Date**: 2026-01-XX  
**Auditor**: AI Automated Audit (Phase 4 of 16)  
**Scope**: `node/src/*` (8 files), `Dockerfile`, `.github/workflows/deploy.yml`  
**Previous Phase**: [Phase 3 — Runtime Configuration Audit](./PHASE_3_RUNTIME_CONFIGURATION_AUDIT_2026.md)  
**Prior Findings Cross-Referenced**: Phase 2 (382 findings), Phase 3 (27 findings)

---

## Executive Summary

Phase 4 audits the node binary, service configuration, chain specifications, networking, Docker container, and CI/CD pipeline. **32 findings** identified:

| Severity | Count | Description |
|----------|-------|-------------|
| **Critical** | 3 | Supply violation, YAML corruption, data persistence |
| **High** | 5 | Select chain, sudo leak, RPC exposure, peer config, deployment secret |
| **Medium** | 8 | try-runtime stub, block announce startup, block time mismatch, benchmark gaps, CI coverage, warp sync, resource limits, env guard |
| **Low** | 5 | Missing typed RPCs, sudo dependency, Dockerfile defaults, pallet-sudo unconditional, stale comments |
| **Informational** | 5 | Positive practices and architecture notes |
| **Deployment-Critical** | 6 | AKS-specific findings (subset of above, flagged separately) |
| **TOTAL** | **32** | (6 overlap with Deployment-Critical) |

**Risk Rating**: **HIGH** — Multiple deployment-blocking issues in CI/CD manifest and chain specification economics.

---

## Severity Definitions

| Level | Description |
|-------|-------------|
| **Critical** | Can cause fund loss, chain halt, or security breach; must fix before mainnet |
| **High** | Significant risk to security, integrity, or availability |
| **Medium** | Moderate risk; should fix before production |
| **Low** | Minor issues or improvements |
| **Informational** | Positive practices, notes, or suggestions |

---

## 1. Service Configuration (`node/src/service.rs`)

### ND-H-001: LongestChain Select Chain (CONS-035 Still Pending)
- **Severity**: HIGH
- **File**: `node/src/service.rs`
- **Lines**: Type alias `FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>`
- **Description**: The node uses `LongestChain` for fork selection. In BABE+GRANDPA consensus, the correct approach is `FinalityTrackingSelectChain` which prefers chains containing the latest finalized block. `LongestChain` can cause validators to build on non-finalized forks during network partitions.
- **Impact**: Fork choice divergence during network splits. Validators may waste resources authoring blocks on abandoned forks. In extreme cases, this can stall finality.
- **Status**: Marked as TODO in code (CONS-035) from prior consensus audit
- **Recommendation**: Replace with `sc_consensus::FinalityTrackingSelectChain` or Substrate's `SelectChainWithFinalizedBest`.

### ND-I-001: CONS-034/036 Fixes Applied (Positive)
- **Severity**: INFORMATIONAL
- **Description**: Two critical consensus fixes are properly applied:
  - `GRANDPA_JUSTIFICATION_PERIOD = 32` (reduced from 512, CONS-036)
  - `backoff_authoring_blocks: Some(BackoffAuthoringOnFinalizedHeadLagging::default())` (CONS-034)
  - GRANDPA gossip_duration = 333ms (aggressive but acceptable for small network)
  - BABE SlotProportion = 2/3 (reasonable)

### ND-I-002: P2P Fixes Implemented (Positive)
- **Severity**: INFORMATIONAL
- **Description**: Service properly implements:
  - P2P-FIX-002: `BelizeBlockAnnounceValidator` registered as block announce validator
  - P2P-FIX-005: `enable_http_requests: false` in offchain worker config
  - Network config with `enable_dht_random_walk: true` for peer discovery

### ND-M-001: Benchmark ExtrinsicBuilder Commented Out
- **Severity**: MEDIUM
- **File**: `node/src/service.rs`
- **Description**: `RemarkBuilder` struct defined but `ExtrinsicBuilder` trait impl is commented out. This means `frame-benchmarking-cli` cannot generate proper extrinsic overhead benchmarks.
- **Impact**: Extrinsic base weight may be inaccurate, affecting fee calculation for all transactions.
- **Recommendation**: Complete the `ExtrinsicBuilder` implementation or use substrate's default `BaselineExtrinsicBuilder`.

---

## 2. RPC Configuration (`node/src/rpc.rs`)

### ND-H-002: No RPC Rate Limiting, Authentication, or CORS Policy
- **Severity**: HIGH
- **File**: `node/src/rpc.rs`
- **Description**: The RPC server registers System and TransactionPayment modules with no middleware for:
  - Rate limiting (DoS amplification via `state_getStorage` or `system_dryRun`)
  - Authentication (any connection can submit extrinsics via `author_submitExtrinsic`)
  - Input validation beyond Substrate defaults
- **Context**: The deploy.yml uses `--rpc-cors all` and `--unsafe-rpc-external` (see ND-DC-003), compounding this issue.
- **Recommendation**:
  1. Use `--rpc-methods Safe` (already done in deploy.yml ✓)
  2. Add reverse proxy (nginx/envoy) with rate limiting in front of RPC
  3. Configure strict CORS in production (`--rpc-cors` with specific origins)
  4. Never use `--unsafe-rpc-external` in production

### ND-L-001: No Custom Typed RPCs for 19 Pallets
- **Severity**: LOW
- **File**: `node/src/rpc.rs`
- **Description**: Only `System`, `TransactionPayment`, and a static `belizechain_getChainInfo` endpoint are registered. None of the 19 custom pallets expose RPC endpoints. TODO comments exist for governance, staking, bridge, and oracle RPCs but none are implemented.
- **Impact**: Clients must use generic `state_getStorage` calls to query pallet state, which is less ergonomic and harder to version/deprecate.
- **Recommendation**: Implement typed RPC traits for high-traffic pallets (BNS, Economy, Governance, Oracle) as a UX improvement. Not blocking for launch.

### ND-L-005: Static getChainInfo Returns Hardcoded Pallet List
- **Severity**: LOW
- **File**: `node/src/rpc.rs`
- **Description**: `belizechain_getChainInfo` returns a hardcoded JSON string with pallet names. If pallets are added/removed, this must be manually updated.
- **Recommendation**: Generate pallet list from runtime metadata instead.

---

## 3. Chain Specification (`node/src/chain_spec.rs`)

### ND-H-003: Mainnet Genesis Pre-configures Sudo Key
- **Severity**: HIGH
- **File**: `node/src/chain_spec.rs`
- **Lines**: `mainnet_genesis()` function
- **Description**: Mainnet genesis JSON includes `"sudo": { "key": Some(root_key) }`. Even though `MAINNET_KEYS_CONFIGURED = false` currently prevents mainnet launch, when keys are eventually configured, the Sudo pallet will be active on mainnet with a pre-configured superuser key.
- **Impact**: Single key can bypass all governance and execute arbitrary privileged calls on mainnet.
- **Recommendation**:
  1. Exclude Sudo from mainnet genesis entirely (don't include the `"sudo"` key in genesis JSON)
  2. Or configure Sudo to `None` in mainnet genesis
  3. The runtime should also feature-gate `pallet_sudo` behind `#[cfg(feature = "dev")]` (cross-ref Phase 3)

### ND-I-003: MAINNET_KEYS_CONFIGURED Guard (Positive)
- **Severity**: INFORMATIONAL
- **File**: `node/src/chain_spec.rs`
- **Description**: `const MAINNET_KEYS_CONFIGURED: bool = false` with compile-time `assert!` in `belizechain_mainnet_config()`. This is an excellent safety mechanism preventing accidental mainnet launch with placeholder keys.
- **Recommendation**: Keep this pattern. Consider adding a similar guard for `ValidatorPeerIds` in `validator_config.rs`.

### ND-M-008: Mainnet Genesis Environment-Only Safety
- **Severity**: MEDIUM
- **File**: `node/src/chain_spec.rs`
- **Description**: The `MAINNET_KEYS_CONFIGURED` guard is a compile-time constant. If someone accidentally changes it to `true` without replacing placeholder keys, no runtime check prevents launch with insecure keys.
- **Recommendation**: Add a key-content validation: verify that authority session keys don't match well-known dev keys (Alice, Bob, etc.) as a secondary safety net.

---

## 4. Chain Specification Configs (`node/src/chain_spec_configs.rs`)

### ND-C-001: Mainnet Initial Allocation Exceeds Max Supply
- **Severity**: CRITICAL
- **File**: `node/src/chain_spec_configs.rs`
- **Lines**: `NetworkConfig::mainnet()`
- **Description**: Mainnet initial allocation totals:
  ```
  central_treasury:   500_000_000 * DALLA = 500M DALLA
  staking_rewards:    300_000_000 * DALLA = 300M DALLA
  development_fund:   100_000_000 * DALLA = 100M DALLA
  public_distribution: 100_000_000 * DALLA = 100M DALLA
  ────────────────────────────────────────────────────
  TOTAL:              1,000,000,000 DALLA = 1 BILLION DALLA
  ```
  The runtime defines `MaxSupply = 501_000_000_000 * DOLLARS` (501 billion DALLA). While the allocation technically doesn't exceed max supply (1B < 501B), the economic whitepaper and tokenomics documentation should be verified for consistency. 
  
  **UPDATE after re-analysis**: The allocation of 1B DALLA is within the 501B max supply. However, the total supply at genesis (1B) should be documented and validated against the economic model. If `DOLLARS = 10^12`, then 500M × 10^12 = 5×10^20 units, which is correct for the 12-decimal system. The total genesis supply of 1B DALLA is reasonable.
  
  **Revised Finding**: The initial allocation arithmetic is correct. However, the genesis balances in `chain_spec.rs` (`testnet_genesis`) endow each account with `1_000_000 * DOLLARS` = 1M DALLA, and there are typically 4+ endowed accounts. Cross-validate that endowments + allocation don't exceed max supply.
- **Revised Severity**: MEDIUM → renamed to ND-M-009

### ND-M-009: Cross-Validate Genesis Total Supply
- **Severity**: MEDIUM
- **File**: `node/src/chain_spec_configs.rs` + `node/src/chain_spec.rs`
- **Description**: Multiple genesis config functions allocate tokens (endowed accounts, treasury, staking reserves). No compile-time or genesis-time assertion validates that total allocated ≤ `MaxSupply`.
- **Recommendation**: Add a `debug_assert!` or test that sums all genesis allocations and asserts `total <= MaxSupply`.

### ND-M-003: Dev Config Block Time Mismatch
- **Severity**: MEDIUM
- **File**: `node/src/chain_spec_configs.rs`
- **Lines**: `NetworkConfig::development()` — `block_time_ms: 3000`
- **Description**: Dev config sets 3-second blocks, but the runtime hardcodes `MILLISECS_PER_BLOCK = 6000` for BABE slot duration. BABE epoch configuration, session length, and time-based constants all assume 6-second blocks.
- **Impact**: In dev mode, blocks may be produced every 3 seconds but BABE's slot timing still expects 6 seconds. This can cause:
  - Slots being skipped or double-authored
  - Time-based governance deadlines being halved
  - Session durations being incorrect
- **Recommendation**: Either change dev block_time_ms to 6000, or make `MILLISECS_PER_BLOCK` configurable per chain spec (currently hardcoded in runtime).

---

## 5. Block Announce Validator (`node/src/block_announce_validator.rs`)

### ND-M-002: best_number Starts at Zero on Startup
- **Severity**: MEDIUM
- **File**: `node/src/block_announce_validator.rs`
- **Description**: `BelizeBlockAnnounceValidator` initializes `best_number: AtomicU32::new(0)`. When a node starts (especially after syncing), the first block announcement could be at height 1,000,000+. Since `MAX_BLOCKS_AHEAD = 64`, any announcement > 64 will be rejected until the validator's internal `best_number` catches up.
- **Impact**: Brief period after startup where legitimate block announcements may be incorrectly rejected or delayed. Nodes recovering from crash may take extra time to resync.
- **Root Cause**: `best_number` is only updated when a `Validation::Success` is returned. The validator has no reference to the client's actual best block.
- **Recommendation**: Initialize `best_number` from the client's `info().best_number` at construction time:
  ```rust
  impl<B: BlockT> BelizeBlockAnnounceValidator<B> {
      pub fn new(client: Arc<dyn HeaderBackend<B>>) -> Self {
          let best = client.info().best_number;
          Self {
              best_number: AtomicU32::new(best.saturated_into()),
              _phantom: PhantomData,
          }
      }
  }
  ```

### ND-I-004: Block Announce Validator Unit Tests (Positive)
- **Severity**: INFORMATIONAL
- **Description**: 4 unit tests cover valid, invalid (non-empty data), far-future, and near-future block announcements. Good coverage of the P2P-FIX-002 implementation.

---

## 6. Validator Config (`node/src/validator_config.rs`)

### ND-H-004: Reserved Peers Mode Set to Accept
- **Severity**: HIGH
- **File**: `node/src/validator_config.rs`
- **Lines**: `reserved_peers_config()` function
- **Description**: `non_reserved_mode: NonReservedPeerMode::Accept` allows any peer to connect to validator nodes, not just reserved/trusted peers. For a validator, this expands the attack surface for eclipse attacks.
- **Impact**: Validators can be overwhelmed by connections from unknown peers, potentially leading to eclipse attacks where an attacker controls all peers a validator sees.
- **Recommendation**: Set `non_reserved_mode: NonReservedPeerMode::Deny` for validator presets. The `Accept` mode is appropriate for RPC/archive nodes but not for block-producing validators.
- **Note**: The AKS deployment does use `--reserved-only` flag (ND-DC-005 positive), but the code-level default should also be secure.

### ND-M-006: 29 Placeholder Peer IDs in Mainnet Config
- **Severity**: MEDIUM (blocked by MAINNET_KEYS_CONFIGURED guard)
- **File**: `node/src/validator_config.rs`
- **Lines**: `ValidatorPeerIds` array
- **Description**: Only 3 of 32 `ValidatorPeerIds` entries contain real PeerIds. The remaining 29 are explicit `"Placeholder_XX"` strings. If `MAINNET_KEYS_CONFIGURED` is set to `true` without replacing these, the node will attempt to connect to invalid peer IDs.
- **Impact**: Mitigated by `MAINNET_KEYS_CONFIGURED` guard, but no secondary validation exists.
- **Recommendation**: Add a startup assertion that validates all `ValidatorPeerIds` entries are valid base58-encoded PeerIds (not containing "Placeholder").

### ND-L-002: Reserved Nodes Use Documentation IP Range (Correct)
- **Severity**: LOW (Informational)
- **File**: `node/src/validator_config.rs`
- **Description**: `ReservedNodes` uses 192.0.2.x addresses (RFC 5737 TEST-NET-1). These are correctly non-routable and serve as obvious placeholders. Good practice.

---

## 7. CLI & Main (`node/src/main.rs`, `node/src/cli.rs`)

### ND-M-004: try-runtime Subcommand Returns Hardcoded Error
- **Severity**: MEDIUM
- **File**: `node/src/main.rs`
- **Lines**: `Some(Subcommand::TryRuntime) => ...`
- **Description**: The `TryRuntime` subcommand is compiled (behind `#[cfg(feature = "try-runtime")]`) but returns a hardcoded error: `Err("TryRuntime is not implemented yet".into())`. This feature is critical for testing runtime upgrades against live state.
- **Impact**: No ability to test runtime migrations against live state before deployment. This is a significant gap for a chain preparing for mainnet.
- **Recommendation**: Implement the `TryRuntime` subcommand using `try-runtime-cli` crate. This is essential for safe runtime upgrades.

### ND-L-003: No Default Chain Spec in Load Spec
- **Severity**: LOW
- **File**: `node/src/main.rs`
- **Lines**: `load_spec()` match: `"" | "belize" => mainnet_config()`
- **Description**: Empty string maps to mainnet config, which is appropriate. However, the `"belize"` variant is undocumented and could cause confusion.
- **Recommendation**: Document supported chain spec IDs in `--help` output and README.

---

## 8. Dockerfile

### ND-L-004: No Default --chain in ENTRYPOINT
- **Severity**: LOW
- **File**: `Dockerfile`
- **Description**: The ENTRYPOINT is `["/usr/local/bin/belizechain-node"]` with no default arguments. Operators must always pass `--chain`, `--name`, etc. While this is flexible, a missing `--chain` will default to mainnet (due to `"" => mainnet_config()`), which could be dangerous for test deployments.
- **Recommendation**: Consider setting a safe default like `--chain testnet` in CMD or document required arguments prominently.

### ND-I-005: Docker Security Practices (Positive)
- **Severity**: INFORMATIONAL
- **Description**: Good Docker security practices observed:
  - Multi-stage build (compile → minimal runtime image)
  - Non-root user (`belizechain:1000`)
  - Binary stripping for smaller image
  - Health check (`curl` against `system_health` RPC)
  - Minimal installed packages (`ca-certificates`, `libssl3`, `curl`)
  - Testnet chain spec raw generated at build time

---

## 9. CI/CD Pipeline (`.github/workflows/deploy.yml`)

### ND-C-002: YAML Manifest Corruption — Volume Mount Syntax Error
- **Severity**: CRITICAL
- **File**: `.github/workflows/deploy.yml`
- **Lines**: ~296-304 (inline Kubernetes manifest)
- **Description**: The inline Kubernetes Deployment YAML has a **catastrophic syntax error** in the volume mount section:
  ```yaml
                    resources:
                      requests:
                        cpu: "500m"
                        memory: 
                  - name: reserved-nodes
                    configMap:
                      name: reserved-nodes"1Gi"
                      limits:
                        cpu: "1500m"
                        memory: "3Gi"
  ```
  The `memory:` field under `requests` is empty, and the `volumes` section is interleaved with `resources`. The string `reserved-nodes"1Gi"` is clearly a copy-paste error merging the volume configMap name with the memory limit.
- **Impact**: **This manifest will fail to apply.** `kubectl apply` will reject the malformed YAML. Deployments will break silently as the deploy job will fail.
- **Correct Structure**:
  ```yaml
                    resources:
                      requests:
                        cpu: "500m"
                        memory: "1Gi"
                      limits:
                        cpu: "1500m"
                        memory: "3Gi"
                volumes:
                  - name: blockchain-data
                    emptyDir: {}
                  - name: reserved-nodes
                    configMap:
                      name: reserved-nodes
  ```
- **Recommendation**: Fix immediately. This is a deployment-blocking bug.

### ND-C-003: Blockchain Data on emptyDir — Data Loss on Pod Restart
- **Severity**: CRITICAL
- **File**: `.github/workflows/deploy.yml`
- **Lines**: Volume definition: `emptyDir: {}`
- **Description**: Blockchain data is stored on an `emptyDir` volume, which is ephemeral. When the pod restarts, moves, or is evicted, **all synced blockchain data is permanently lost**. The node must resync from genesis on every restart.
- **Impact**:
  - Hours/days of sync time lost on every pod restart
  - For validators: missed blocks and slashing risk during resync
  - For archive nodes: complete data loss
- **Recommendation**: Use a PersistentVolumeClaim (PVC):
  ```yaml
          volumes:
            - name: blockchain-data
              persistentVolumeClaim:
                claimName: belizechain-data
  ---
  apiVersion: v1
  kind: PersistentVolumeClaim
  metadata:
    name: belizechain-data
    namespace: belizechain
  spec:
    accessModes: [ReadWriteOnce]
    resources:
      requests:
        storage: 50Gi
    storageClassName: managed-csi
  ```
  Note: For validators, consider using a StatefulSet instead of a Deployment for stable network identity and persistent storage.

### ND-H-005: --unsafe-rpc-external Exposes RPC to Internet
- **Severity**: HIGH
- **File**: `.github/workflows/deploy.yml`
- **Lines**: Deployment args include `--unsafe-rpc-external`
- **Description**: The `--unsafe-rpc-external` flag explicitly disables Substrate's RPC safety checks and binds the RPC server to `0.0.0.0`. Combined with `--rpc-cors all`, this allows any origin to submit RPC calls including extrinsic submission.
- **Context**: `--rpc-methods Safe` is set (limits to read-only methods), which partially mitigates this. However:
  - `Safe` mode still exposes `state_getStorage`, `chain_getBlock`, etc. which can leak validator state
  - No reverse proxy or WAF is configured
  - The `LoadBalancer` Service type (line ~330) exposes this directly to the internet
- **Recommendation**:
  1. Remove `--unsafe-rpc-external`; use `--rpc-external` instead
  2. Add an nginx/envoy sidecar with rate limiting
  3. Use `ClusterIP` Service type + Ingress controller for RPC
  4. Keep `--rpc-methods Safe` for public-facing nodes

### ND-DC-001: NODE_KEY Passed as Command Argument
- **Severity**: HIGH  
- **File**: `.github/workflows/deploy.yml`
- **Lines**: `--node-key` / `${{ secrets.NODE_KEY }}`
- **Description**: The node key (private key for libp2p identity) is passed as a command-line argument. This means:
  - The key is visible in `kubectl describe pod` output under `Args`
  - The key is visible in process listing (`ps aux`)
  - The key is logged in Kubernetes audit logs
- **Impact**: If any cluster user or monitoring tool captures the args, the node's P2P identity is compromised. An attacker could impersonate this node.
- **Recommendation**: Use a Kubernetes Secret mounted as a file:
  ```yaml
  args:
    - "--node-key-file"
    - "/secrets/node-key"
  volumeMounts:
    - name: node-key
      mountPath: /secrets
      readOnly: true
  volumes:
    - name: node-key
      secret:
        secretName: belizechain-node-key
  ```

### ND-M-005: CI Benchmark Only Smoke-Tests pallet_balances
- **Severity**: MEDIUM
- **File**: `.github/workflows/deploy.yml`
- **Lines**: Benchmark job `--pallet pallet_balances --steps 2 --repeat 1`
- **Description**: The CI benchmark job only tests `pallet_balances` with minimal iterations (2 steps, 1 repeat). This means:
  - None of the 19 custom pallets are benchmarked in CI
  - Weight values for custom pallets are never validated in CI
  - Regressions in custom pallet performance go undetected
- **Recommendation**: Add at least a smoke benchmark for critical custom pallets: `pallet_belize_economy`, `pallet_staking`, `pallet_governance`, `pallet_bns`.

### ND-M-007: Coverage Gate Excludes Critical Files
- **Severity**: MEDIUM
- **File**: `.github/workflows/deploy.yml` + `.tarpaulin.toml` (implied)
- **Description**: CI enforces 100% code coverage but excludes `service.rs` and `rpc.rs` via `.tarpaulin.toml`. These files contain critical networking and consensus initialization code.
- **Impact**: Bugs in service initialization, consensus setup, or RPC registration will not be caught by coverage checks.
- **Recommendation**: While 100% coverage of service.rs is impractical (integration tests needed), aim for at least a compilation check or basic instantiation test.

### ND-DC-002: --alice Flag on Testnet Deployment
- **Severity**: MEDIUM (testnet only)
- **File**: `.github/workflows/deploy.yml`
- **Lines**: Deployment args include `--alice`
- **Description**: The deployment uses `--alice` which inserts well-known development keys. This is acceptable for testnet but MUST be removed for mainnet deployment.
- **Impact**: On testnet: acceptable. If accidentally used on mainnet: anyone with knowledge of Alice's keys can produce blocks and finalize.
- **Recommendation**: Add a CI check that errors if `--alice` appears in any mainnet deployment manifest.

### ND-DC-003: --force-authoring on Shared Deployment
- **Severity**: MEDIUM
- **File**: `.github/workflows/deploy.yml`
- **Description**: `--force-authoring` allows the node to author blocks even when it's not fully synced or when it hasn't heard from other validators. Safe for single-node testnets but dangerous for multi-validator networks.
- **Recommendation**: Remove for multi-validator testnet or mainnet deployments.

### ND-DC-004: LoadBalancer Service Exposes All Ports
- **Severity**: MEDIUM
- **File**: `.github/workflows/deploy.yml`
- **Lines**: Service specification
- **Description**: The Kubernetes Service uses `type: LoadBalancer` which creates a public Azure Load Balancer exposing ports 30333 (P2P), 9944 (RPC), and 9615 (Prometheus). Prometheus metrics port should not be publicly accessible.
- **Recommendation**:
  1. Use separate Services: `LoadBalancer` for P2P (30333), `ClusterIP` for RPC and metrics
  2. Or use an Ingress controller with path-based routing
  3. At minimum, remove port 9615 from the LoadBalancer Service

### ND-DC-005: --reserved-only Flag (Positive)
- **Severity**: INFORMATIONAL
- **Description**: The deployment correctly uses `--reserved-only` and `--no-mdns`, restricting peer connections to the reserved nodes file. This is good security practice for validators.

---

## 10. Node Dependencies (`node/Cargo.toml`)

### ND-L-006: pallet-sudo Dependency is Unconditional
- **Severity**: LOW
- **File**: `node/Cargo.toml`
- **Description**: `pallet-sudo` is listed as a dependency without any feature gate. The runtime should be the only consumer of Sudo, and the node binary doesn't directly use it.
- **Recommendation**: Verify this dependency is actually needed in the node crate. If only used transitively through the runtime, remove it from node/Cargo.toml.

---

## 11. Cross-Phase References

| Phase 4 Finding | Related Phase 2/3 Finding | Notes |
|-----------------|---------------------------|-------|
| ND-H-003 (Sudo in mainnet genesis) | RTM-C-004 (Sudo not feature-gated) | Same root issue: Sudo available on mainnet |
| ND-M-003 (Dev 3s blocks) | RTM-M-003 (BABE slot timing) | Runtime hardcodes 6s, dev config uses 3s |
| ND-H-001 (LongestChain) | CONS-035 (Phase 2) | Previously identified, still unresolved |
| ND-M-005 (CI benchmark gap) | Multiple pallets missing weights | Weights unvalidated in CI |
| ND-H-002 (No RPC protection) | — | New finding |
| ND-C-002 (YAML corruption) | — | New finding |
| ND-C-003 (emptyDir data loss) | — | New finding |
| ND-H-005 (unsafe-rpc-external) | — | New finding |
| ND-DC-001 (NODE_KEY exposure) | — | New finding |

---

## 12. Remediation Priority Matrix

### P0 — Fix Immediately (Deployment Blockers)
| ID | Finding | Effort |
|----|---------|--------|
| ND-C-002 | YAML manifest corruption | 15min |
| ND-C-003 | emptyDir → PersistentVolumeClaim | 30min |
| ND-DC-001 | Node key as file, not arg | 20min |
| ND-H-005 | Remove --unsafe-rpc-external | 5min |

### P1 — Fix Before Public Testnet
| ID | Finding | Effort |
|----|---------|--------|
| ND-H-001 | LongestChain → FinalityTrackingSelectChain | 2h |
| ND-H-003 | Remove Sudo from mainnet genesis | 1h |
| ND-H-004 | Reserved peers Deny mode for validators | 30min |
| ND-H-002 | RPC rate limiting + reverse proxy | 4h |
| ND-M-002 | Block announce validator init from client | 1h |
| ND-DC-004 | Separate services for P2P/RPC/metrics | 1h |

### P2 — Fix Before Mainnet
| ID | Finding | Effort |
|----|---------|--------|
| ND-M-004 | Implement try-runtime subcommand | 4h |
| ND-M-003 | Dev block time alignment | 30min |
| ND-M-005 | CI benchmarks for custom pallets | 2h |
| ND-M-006 | Validate placeholder peer IDs at startup | 1h |
| ND-M-007 | Extend coverage to critical node files | 2h |
| ND-M-008 | Secondary key validation guard | 1h |
| ND-M-009 | Cross-validate genesis total supply | 1h |
| ND-M-001 | Complete ExtrinsicBuilder impl | 2h |

### P3 — Nice to Have
| ID | Finding | Effort |
|----|---------|--------|
| ND-L-001 | Custom typed RPCs for pallets | 8h+ |
| ND-L-003 | Document chain spec IDs | 30min |
| ND-L-004 | Dockerfile safe defaults | 15min |
| ND-L-005 | Dynamic getChainInfo from metadata | 2h |
| ND-L-006 | Remove unnecessary pallet-sudo dep | 15min |

---

## 13. Warp Sync Configuration

### ND-M-010: Empty Hard Forks Vector
- **Severity**: MEDIUM
- **File**: `node/src/service.rs`
- **Description**: Warp sync provider initialized with `Vec::new()` for hard forks. For any chain that has undergone or plans to undergo hard forks, this must be populated to ensure correct warp sync behavior.
- **Impact**: If a hard fork occurs, nodes using warp sync may sync to the wrong fork.
- **Recommendation**: Document warp sync limitations. Before any hard fork, update this vector and ship a node update.

---

## Summary Statistics

| Category | Critical | High | Medium | Low | Info | Total |
|----------|----------|------|--------|-----|------|-------|
| Service (service.rs) | 0 | 1 | 1 | 0 | 2 | 4 |
| RPC (rpc.rs) | 0 | 1 | 0 | 2 | 0 | 3 |
| Chain Spec (chain_spec.rs) | 0 | 1 | 1 | 0 | 1 | 3 |
| Chain Spec Configs | 0 | 0 | 2 | 0 | 0 | 2 |
| Block Announce Validator | 0 | 0 | 1 | 0 | 1 | 2 |
| Validator Config | 0 | 1 | 1 | 1 | 0 | 3 |
| CLI & Main | 0 | 0 | 1 | 1 | 0 | 2 |
| Dockerfile | 0 | 0 | 0 | 1 | 1 | 2 |
| CI/CD Pipeline | 2 | 2 | 4 | 0 | 1 | 9 |
| Dependencies | 0 | 0 | 0 | 1 | 0 | 1 |
| Warp Sync | 0 | 0 | 1 | 0 | 0 | 1 |
| **TOTAL** | **2** | **6** | **12** | **6** | **6** | **32** |

---

## Files Audited

| File | Lines | Description |
|------|-------|-------------|
| `node/src/main.rs` | ~200 | CLI entry point, subcommand routing |
| `node/src/cli.rs` | ~60 | CLI subcommand definitions |
| `node/src/service.rs` | ~500 | Consensus setup, network, OCW |
| `node/src/rpc.rs` | ~97 | RPC endpoint registration |
| `node/src/chain_spec.rs` | ~500 | Genesis configs (dev/local/mainnet) |
| `node/src/chain_spec_configs.rs` | ~200 | NetworkConfig presets |
| `node/src/block_announce_validator.rs` | ~200 | P2P block validation |
| `node/src/validator_config.rs` | ~350 | Network/DB/role presets |
| `Dockerfile` | ~60 | Container build |
| `.github/workflows/deploy.yml` | ~340 | CI/CD pipeline |
| **TOTAL** | **~2,507** | |

---

## Next Phase

**Phase 5: Consensus Mechanism Deep Dive** — Cross-pallet analysis of BABE+GRANDPA+Session+Staking consensus interactions, epoch transitions, authority rotation, and fork choice safety.

---

*Phase 4 audit complete. 32 findings (2C, 6H, 12M, 6L, 6I). Deployment-critical YAML corruption and data persistence issues require immediate attention.*
