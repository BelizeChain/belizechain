# Phase 12: CI/CD, Docker & Deployment Security Audit

**Date**: 2026-03-14
**Auditor**: AI Security Audit (Phase 12)
**Scope**: `.github/workflows/deploy.yml`, `.github/workflows/security-audit.yml`, `Dockerfile`, `deny.toml`, `rust-toolchain.toml`, K8s manifests, deployment scripts
**Severity Scale**: CRITICAL / HIGH / MEDIUM / LOW / INFO

---

## Executive Summary

The BelizeChain CI/CD pipeline has a functional test→build→deploy flow but contains **multiple critical and high-severity security findings** that must be remediated before mainnet. The most dangerous issues are: (1) the use of `--alice` and `--unsafe-rpc-external` in the AKS production deployment, (2) the `latest` mutable image tag used for deploys, (3) `continue-on-error: true` on the build job allowing failed builds to proceed to deploy, (4) absence of container image scanning, and (5) no Kubernetes security context, network policy, or persistent storage.

| Severity | Count |
|----------|-------|
| CRITICAL | 6 |
| HIGH     | 8 |
| MEDIUM   | 7 |
| LOW      | 4 |
| INFO     | 3 |
| **Total**| **28** |

---

## 1. CI/CD Pipeline — `deploy.yml`

### File Overview
- **Triggers**: push to `belizechain`, PR to `belizechain`, manual dispatch
- **Jobs**: test → coverage → benchmark → build → deploy
- **Concurrency**: deploy group with cancel-in-progress (✅ Good)
- **Environment**: `testnet` environment on deploy job (✅ Good — enables protection rules)

---

### CD-CRIT-001: `--alice` Flag in AKS Deployment
- **File**: `.github/workflows/deploy.yml` line 266
- **Severity**: **CRITICAL**
- **Description**: The AKS deployment uses `--alice` which injects the well-known Alice development account as the validator session key. The private key for Alice is publicly known (`0xe5be9a5092b81bca64be81d212e7f2f9ebb0f6`...). Any attacker who knows this (it's in every Substrate tutorial) can impersonate the validator, sign blocks, and steal staking rewards.
- **Impact**: Complete validator identity compromise. An attacker can produce blocks, execute equivocation attacks, and drain validator funds from day one.
- **Recommendation**: Generate proper session keys using `author_rotateKeys` RPC, insert them with `author_insertKey`, and remove `--alice` entirely. Use `--validator` without any well-known account flag.

### CD-CRIT-002: `--unsafe-rpc-external` Exposes RPC to Internet
- **File**: `.github/workflows/deploy.yml` line 280
- **Severity**: **CRITICAL**
- **Description**: The `--unsafe-rpc-external` flag binds the RPC server to `0.0.0.0`, exposing it to the internet without authentication. Combined with the `LoadBalancer` service type, the RPC port (9944) is directly accessible from the public internet.
- **Impact**: Enables unauthorized state queries, transaction submission, and potential denial-of-service. With `--rpc-methods Safe` this mitigates some risk, but still exposes the entire safe RPC surface to unauthenticated callers.
- **Recommendation**: Remove `--unsafe-rpc-external`. Use `--rpc-external` (which is less dangerous but still broad) behind an ingress controller with rate limiting and authentication. Better: use a `ClusterIP` service for RPC and expose via an authenticated API gateway.

### CD-CRIT-003: `--rpc-cors all` Without Restriction
- **File**: `.github/workflows/deploy.yml` line 278
- **Severity**: **CRITICAL**
- **Description**: `--rpc-cors all` allows any origin to make cross-origin requests to the node's RPC endpoint. Combined with `--unsafe-rpc-external` and a `LoadBalancer`, this means any website can interact with the node.
- **Impact**: Enables cross-origin attacks from malicious websites against users who visit them while having access to the node.
- **Recommendation**: Restrict CORS to specific trusted origins: `--rpc-cors "https://portal.belizechain.org,https://explorer.belizechain.org"`.

### CD-CRIT-004: Mutable `latest` Tag for Production Deployment
- **File**: `.github/workflows/deploy.yml` lines 172-173, 250
- **Severity**: **CRITICAL**
- **Description**: The image is tagged with both `sha-<commit>` and `latest`. However, the K8s deployment references `${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest`. Since `latest` is mutable, there is no guarantee which image version is actually running. If the registry is compromised, a malicious image could be pushed as `latest`.
- **Impact**: Supply chain attack vector. Non-reproducible deployments. Impossible to audit which exact code is running in production.
- **Recommendation**: Deploy using the immutable SHA-based tag: `belizechainacr.azurecr.io/belizechain-node:${{ github.sha }}`. Never use `latest` in production K8s manifests. Enable ACR content trust for image signing.

### CD-CRIT-005: `continue-on-error: true` on Build Job
- **File**: `.github/workflows/deploy.yml` line 159
- **Severity**: **CRITICAL**
- **Description**: The `build` job has `continue-on-error: true`, meaning the `deploy` job can proceed even if the Docker build or ACR push **fails**. The `deploy` job depends on `build` via `needs: build`, but `continue-on-error` makes that dependency meaningless — a failed build still allows deploy to run.
- **Impact**: A broken, partially-built, or not-pushed image could be "deployed" to AKS. In practice, AKS would pull the stale `latest` tag, meaning an old vulnerable version continues running while giving the illusion of a successful deploy.
- **Recommendation**: Remove `continue-on-error: true`. If there are intermittent ACR failures, add retry logic to the ACR login/push step instead.

### CD-CRIT-006: Node Key Passed as CLI Argument via Secret
- **File**: `.github/workflows/deploy.yml` lines 263-264
- **Severity**: **CRITICAL**
- **Description**: The node's libp2p identity key (`${{ secrets.NODE_KEY }}`) is passed as a CLI argument (`--node-key`). CLI arguments are visible in `/proc/<pid>/cmdline` on the host and may appear in K8s pod descriptions, audit logs, and monitoring tools.
- **Impact**: Node identity key exposure allows an attacker to impersonate the node on the P2P network, perform eclipse attacks, or intercept network traffic.
- **Recommendation**: Use `--node-key-file` with a Kubernetes Secret mounted as a file:
  ```yaml
  volumes:
    - name: node-key
      secret:
        secretName: belizechain-node-key
  volumeMounts:
    - name: node-key
      mountPath: /etc/belizechain/node-key
      readOnly: true
  args:
    - "--node-key-file"
    - "/etc/belizechain/node-key/key"
  ```

---

### CD-HIGH-001: No Container Image Scanning
- **File**: `.github/workflows/deploy.yml` (absent)
- **Severity**: **HIGH**
- **Description**: There is no Trivy, Snyk, Grype, or any container scanning step in the CI/CD pipeline. The documentation mentions Trivy/Snyk as recommendations (docs/security/SECURITY_BEST_PRACTICES.md, docs/architecture/INFRASTRUCTURE_GUIDE.md) but neither is actually implemented.
- **Impact**: Vulnerable OS packages (libssl3, ca-certificates, curl) in the runtime image could contain known CVEs that are deployed to production without detection.
- **Recommendation**: Add a Trivy scan step after `docker/build-push-action`:
  ```yaml
  - name: Scan image for vulnerabilities
    uses: aquasecurity/trivy-action@master
    with:
      image-ref: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
      severity: CRITICAL,HIGH
      exit-code: 1
  ```

### CD-HIGH-002: GitHub Actions Not Pinned by SHA
- **File**: `.github/workflows/deploy.yml` (all `uses:` lines)
- **Severity**: **HIGH**
- **Description**: All GitHub Actions are pinned by mutable tags (`@v4`, `@v3`, `@v2`, `@v5`, `@v6`, `@master`). Notably, `dtolnay/rust-toolchain@master` is pinned to a branch, not even a version tag.
- **Impact**: Supply chain attack. If any upstream action repository is compromised, malicious code runs in the CI pipeline with access to all secrets (ACR_PASSWORD, AZURE_CREDENTIALS, NODE_KEY, etc.).
- **Affected Actions**:
  - `actions/checkout@v4` — should be `actions/checkout@<sha>`
  - `dtolnay/rust-toolchain@master` — **worst offender**, pinned to branch HEAD
  - `actions/cache@v4`
  - `codecov/codecov-action@v4`
  - `docker/metadata-action@v5`
  - `docker/setup-buildx-action@v3`
  - `docker/login-action@v3`
  - `docker/build-push-action@v6`
  - `azure/login@v2`
  - `azure/aks-set-context@v4`
- **Recommendation**: Pin all actions by full SHA. Use Dependabot or Renovate to manage updates:
  ```yaml
  - uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11  # v4.1.1
  ```

### CD-HIGH-003: ACR Admin Credentials Instead of Managed Identity
- **File**: `.github/workflows/deploy.yml` lines 183-184
- **Severity**: **HIGH**
- **Description**: ACR authentication uses admin username/password (`ACR_USERNAME`/`ACR_PASSWORD`), which are long-lived static credentials. ACR admin access grants full push/pull rights to all repositories.
- **Impact**: Credential rotation burden. If compromised, attacker can push malicious images to ACR. Admin credentials cannot be scoped to specific repositories.
- **Recommendation**: Use OIDC federation with `azure/login` and workload identity to authenticate to ACR. Disable ACR admin account. Use AKS-managed identity with `acrpull` role for image pulls.

### CD-HIGH-004: LoadBalancer Service Exposes All Ports Publicly
- **File**: `.github/workflows/deploy.yml` lines 315-330
- **Severity**: **HIGH**
- **Description**: The K8s Service is `type: LoadBalancer`, exposing P2P (30333), RPC (9944), and Prometheus (9615) ports to the public internet. Prometheus metrics should never be public — they leak internal node state, peer information, block production metrics, and memory usage.
- **Impact**: Information disclosure via Prometheus. Combined with `--unsafe-rpc-external`, full RPC access without authentication.
- **Recommendation**: 
  - Use `ClusterIP` for RPC and Prometheus
  - Use `NodePort` or dedicated LB only for P2P (30333)
  - Expose metrics only via internal monitoring (Grafana agent scraping within cluster)

### CD-HIGH-005: No K8s Security Context
- **File**: `.github/workflows/deploy.yml` (K8s manifest, absent)
- **Severity**: **HIGH**
- **Description**: The pod spec has no `securityContext`. Missing: `runAsNonRoot`, `readOnlyRootFilesystem`, `allowPrivilegeEscalation: false`, `capabilities.drop: [ALL]`.
- **Impact**: Container runs as root by default (unless Dockerfile USER is respected, which it should be, but K8s should enforce it). No defense-in-depth.
- **Recommendation**:
  ```yaml
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    readOnlyRootFilesystem: true
    allowPrivilegeEscalation: false
    capabilities:
      drop: [ALL]
  ```

### CD-HIGH-006: Blockchain Data on `emptyDir` — Data Loss on Every Restart
- **File**: `.github/workflows/deploy.yml` line 308
- **Severity**: **HIGH**
- **Description**: Blockchain data volume uses `emptyDir: {}`, which is ephemeral. Pod restart/eviction/rescheduling causes complete data loss. The node must resync from genesis each time.
- **Impact**: Extended downtime during resync. Validator misses blocks during resync period. On mainnet with months of chain history, resync could take days.
- **Recommendation**: Use a PersistentVolumeClaim with Azure Managed Disk (`managed-csi` StorageClass). Already noted in Phase 4 audit (ND-C-003).

### CD-HIGH-007: No Network Policy
- **File**: `.github/workflows/deploy.yml` (absent)
- **Severity**: **HIGH**
- **Description**: No Kubernetes NetworkPolicy exists. Any pod in the cluster can communicate with the BelizeChain node on any port. When sibling services (ui, nawal, kinich, gem, pakit) are deployed to the same AKS cluster, they will have unrestricted network access to the validator node.
- **Impact**: Lateral movement. If any sibling service is compromised, it can reach the RPC endpoint and submit transactions.
- **Recommendation**: Deploy a NetworkPolicy that restricts ingress to only required ports and from specific namespaces:
  ```yaml
  apiVersion: networking.k8s.io/v1
  kind: NetworkPolicy
  metadata:
    name: belizechain-node-policy
    namespace: belizechain
  spec:
    podSelector:
      matchLabels:
        app: belizechain-node
    ingress:
      - ports:
          - port: 30333      # P2P - from anywhere
      - from:
          - namespaceSelector:
              matchLabels:
                name: monitoring
        ports:
          - port: 9615        # Prometheus - from monitoring only
    policyTypes:
      - Ingress
  ```

### CD-HIGH-008: `--force-authoring` in Production
- **File**: `.github/workflows/deploy.yml` line 267
- **Severity**: **HIGH**
- **Description**: `--force-authoring` forces the node to produce blocks even when it is not fully synced. In production, this can lead to authoring blocks on a stale fork, causing chain splits.
- **Impact**: Chain forks if node is un-synced. Equivocation risk if combined with another validator running the same keys.
- **Recommendation**: Remove `--force-authoring`. Only use during initial testnet bootstrap with full awareness.

---

### CD-MED-001: Coverage Gate Does Not Block Deploy
- **File**: `.github/workflows/deploy.yml` lines 59-99
- **Severity**: **MEDIUM**
- **Description**: The `coverage` job is not in the `needs:` chain for `build` or `deploy`. The build only depends on `test`. Coverage failures do not prevent deployment.
- **Impact**: Code with coverage regression can be deployed. The 100% coverage gate is advisory only.
- **Recommendation**: Add `coverage` to the build job's `needs:` array: `needs: [test, coverage]`.

### CD-MED-002: Benchmark Job Not in Deploy Gate
- **File**: `.github/workflows/deploy.yml` lines 104-141
- **Severity**: **MEDIUM**
- **Description**: Benchmarks are skipped on PRs (via `if: github.event_name != 'pull_request'`), which is fine. However, benchmark failures on pushes to main also don't block deploy since `benchmark` is not in the `needs:` chain.
- **Impact**: Weight regression (benchmarks producing different weights) can be deployed without detection.
- **Recommendation**: Add `benchmark` to build's `needs:` array for push events.

### CD-MED-003: Cargo Cache Poisoning Risk
- **File**: `.github/workflows/deploy.yml` lines 46-53
- **Severity**: **MEDIUM**
- **Description**: The cache key uses `hashFiles('**/Cargo.lock')` which is good, but the cached paths include compiled artifacts in `target/`. If a PR introduces a modified dependency that gets cached, subsequent runs on main could use the poisoned cache.
- **Impact**: Supply chain risk via cache. Mitigated by the fact that PR workflows and push workflows use different cache keys, but `restore-keys: cargo-test-` allows partial matches across branches.
- **Recommendation**: Scope cache keys by branch: `key: cargo-test-${{ github.ref }}-${{ hashFiles('**/Cargo.lock') }}`. Consider using `actions/cache/save` and `actions/cache/restore` separately.

### CD-MED-004: No CODEOWNERS File
- **File**: (absent)
- **Severity**: **MEDIUM**
- **Description**: No `.github/CODEOWNERS` file exists. This means anyone with write access can merge changes to security-critical files (deploy.yml, Dockerfile, deny.toml, runtime/src/lib.rs) without mandatory review from designated owners.
- **Impact**: Unreviewed changes to CI/CD or runtime configuration.
- **Recommendation**: Create `.github/CODEOWNERS`:
  ```
  /.github/workflows/  @BelizeChain/security-team
  /Dockerfile          @BelizeChain/security-team
  /runtime/            @BelizeChain/core-team
  /pallets/            @BelizeChain/core-team
  ```

### CD-MED-005: No Rollback Strategy
- **File**: `.github/workflows/deploy.yml` (absent)
- **Severity**: **MEDIUM**
- **Description**: The deploy step does `kubectl apply` followed by `kubectl rollout status`. There is no rollback step on failure. If the new image crashes, the deployment stays broken.
- **Impact**: Extended downtime until manual intervention.
- **Recommendation**: Add rollback on failure:
  ```yaml
  - name: Rollback on failure
    if: failure()
    run: kubectl rollout undo deployment/belizechain-node -n belizechain
  ```

### CD-MED-006: Inline K8s Manifest (Not Version-Controlled Separately)
- **File**: `.github/workflows/deploy.yml` lines 225-340
- **Severity**: **MEDIUM**
- **Description**: The entire K8s Deployment and Service manifest is embedded inline in the workflow file using a heredoc. Changes to infrastructure require modifying the CI/CD pipeline, which is error-prone and makes infrastructure review difficult.
- **Impact**: Increased risk of accidental breakage. Difficult to review K8s changes in isolation.
- **Recommendation**: Extract the K8s manifest to `k8s/testnet/deployment.yaml` and apply with `kubectl apply -f k8s/testnet/`.

### CD-MED-007: `--prometheus-external` Without Access Control
- **File**: `.github/workflows/deploy.yml` line 292
- **Severity**: **MEDIUM**
- **Description**: Prometheus metrics are exposed externally via `--prometheus-external` and the `LoadBalancer` Service publishes port 9615.
- **Impact**: Information disclosure — metrics reveal block height, peer count, transaction pool size, finality status, CPU/memory usage, and other operational data useful for attack planning.
- **Recommendation**: Remove `--prometheus-external`. Scrape metrics via service mesh or cluster-internal Prometheus only.

---

### CD-LOW-001: `workflow_dispatch` Without Input Validation
- **File**: `.github/workflows/deploy.yml` line 15
- **Severity**: **LOW**
- **Description**: `workflow_dispatch` allows anyone with write access to manually trigger a deploy. No inputs are required (e.g., confirmation string, target environment).
- **Recommendation**: Add required inputs for manual dispatch:
  ```yaml
  workflow_dispatch:
    inputs:
      confirm:
        description: 'Type DEPLOY to confirm'
        required: true
  ```

### CD-LOW-002: Codecov Token Not Configured
- **File**: `.github/workflows/deploy.yml` lines 96-100
- **Severity**: **LOW**
- **Description**: `codecov/codecov-action@v4` is used without a `token:` parameter. For private repos, this will fail. For public repos, it works but is less reliable.
- **Recommendation**: Add `token: ${{ secrets.CODECOV_TOKEN }}` if repo is private.

### CD-LOW-003: Hardcoded NODE_NAME in Env
- **File**: `.github/workflows/deploy.yml` line 20
- **Severity**: **LOW**
- **Description**: `NODE_NAME: "BelizeChain-Testnet-Validator-1"` is hardcoded. If multiple validators are deployed, this cannot be parameterized without modifying the workflow.
- **Recommendation**: Make it a `workflow_dispatch` input or derive from a matrix strategy.

### CD-LOW-004: No SBOM Generation
- **File**: `.github/workflows/deploy.yml` (absent)
- **Severity**: **LOW**
- **Description**: No Software Bill of Materials (SBOM) is generated during the Docker build. SBOMs are increasingly required for supply chain compliance.
- **Recommendation**: Add SBOM generation with `anchore/sbom-action` or `docker/build-push-action`'s `sbom: true` option.

---

## 2. Security Audit Workflow — `security-audit.yml`

### Overview
- **Triggers**: Push/PR that touches Cargo.toml/Cargo.lock, weekly schedule (Monday 6am UTC), manual
- **Jobs**: cargo audit, cargo deny, yanked crate check (3 separate jobs — ✅ Good)
- **Path filtering**: Only runs when dependency files change (✅ Efficient)

### SA-INFO-001: `cargo audit --deny warnings` with `continue-on-error: false`
- **Severity**: **INFO** (Positive finding)
- **Description**: `cargo audit` is configured to deny warnings, and the job does NOT use `continue-on-error`. This means any advisory will block the pipeline. ✅ Good.

### SA-MED-001: No Clippy Enforcement in CI
- **File**: `.github/workflows/security-audit.yml` (absent)
- **Severity**: **MEDIUM** (Noted — may exist elsewhere)
- **Description**: The security audit workflow does not run `cargo clippy -- -D warnings`. While clippy is installed in `rust-toolchain.toml`, no CI job enforces it.
- **Impact**: Clippy warnings about unsafe code, integer overflow, and other issues go undetected.
- **Recommendation**: Add a clippy job:
  ```yaml
  clippy:
    name: 📎 Clippy Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with: { toolchain: "1.90.0", components: clippy }
      - run: cargo clippy --workspace --all-targets -- -D warnings
  ```

### SA-INFO-002: Weekly Schedule Catches New Advisories
- **Severity**: **INFO** (Positive finding)
- **Description**: The weekly Monday schedule ensures new RustSec advisories are detected even when no code changes occur. ✅ Good practice.

---

## 3. Dockerfile

### Overview
- **Multi-stage**: ✅ Yes — `paritytech/ci-linux:production` builder → `debian:bookworm-slim` runtime
- **Non-root user**: ✅ `USER belizechain` (UID 1000)
- **Health check**: ✅ Present (RPC health via curl)
- **Exposed ports**: 30333, 9933, 9944, 9615 (✅ All expected)
- **VOLUME**: `/data` declared (✅ Good)

### DF-HIGH-001: Base Image Not Pinned by Digest
- **File**: `Dockerfile` lines 5, 23
- **Severity**: **HIGH**
- **Description**: Both base images use mutable tags:
  - `paritytech/ci-linux:production` — mutable tag, could change at any time
  - `debian:bookworm-slim` — mutable tag
- **Impact**: Builds are not reproducible. A compromised base image propagates to all future builds silently.
- **Recommendation**: Pin by digest:
  ```dockerfile
  FROM paritytech/ci-linux:production@sha256:<digest> AS builder
  FROM debian:bookworm-slim@sha256:<digest>
  ```

### DF-MED-001: `COPY . .` Copies Entire Workspace
- **File**: `Dockerfile` line 10
- **Severity**: **MEDIUM**
- **Description**: `COPY . .` copies the entire workspace into the builder. No `.dockerignore` file was found (search returned no results). This means `target/`, `.git/`, `audit_results/`, `docs/`, `tests/`, and potentially `.env` files are all copied into the build context.
- **Impact**: Larger build context (slower builds). Risk of secrets in `.env` or `.git/` being included. The `.git` directory alone can be hundreds of MB.
- **Recommendation**: Create a `.dockerignore`:
  ```
  target/
  .git/
  .github/
  audit_results/
  docs/
  tests/
  scripts/
  *.md
  .env*
  .venv/
  ```

### DF-LOW-001: Health Check Uses Localhost
- **File**: `Dockerfile` lines 49-52
- **Severity**: **LOW** (Positive with caveat)
- **Description**: Health check calls `http://localhost:9944`. This works correctly when `--rpc-external` or `--unsafe-rpc-external` is used, since it binds to `0.0.0.0`. If those flags are removed, health check will still work because localhost is always accessible.
- **Status**: ✅ Acceptable.

### DF-INFO-001: Chain Spec Built at Docker Build Time
- **File**: `Dockerfile` lines 16-18
- **Severity**: **INFO**
- **Description**: The chain spec is generated during Docker build with `--chain local`. This means the embedded chain spec is for `local` chain, not testnet or mainnet. The AKS deployment overrides this by mounting a different chain spec via configmap. This is fine but could be confusing.

---

## 4. `deny.toml`

### Overview
- **Advisory policy**: `vulnerability = "deny"`, `yanked = "deny"` ✅
- **License allow-list**: MIT, Apache-2.0, BSD, ISC, etc. ✅ Appropriate for MIT project
- **Source controls**: Unknown registries and git sources denied ✅
- **Trusted orgs**: paritytech, AcalaNetwork, nicira, AztecProtocol ✅

### DN-MED-001: 4 Ignored Advisories
- **File**: `deny.toml` lines 23-40
- **Severity**: **MEDIUM** (Acknowledged risk)
- **Description**: Four RustSec advisories are explicitly ignored:
  1. **RUSTSEC-2026-0020** (wasmtime WASI resource exhaustion) — justified: wasmtime-wasi not in deps
  2. **RUSTSEC-2026-0021** (wasmtime WASI fields panic) — justified: same reason
  3. **RUSTSEC-2025-0118** (wasmtime shared linear memory) — justified: Substrate doesn't use shared memory
  4. **RUSTSEC-2025-0009** (ring AES panic) — justified: only with overflow-checks, disabled in release
- **Assessment**: All have documented justifications. The ring advisory (RUSTSEC-2025-0009) is the most concerning since `ring` is the primary cryptographic library. The justification is valid for release builds but development/test builds with overflow-checks could trigger panics.
- **Recommendation**: Periodically re-evaluate. Track SDK upgrade for ring 0.17.x.

### DN-LOW-001: `multiple-versions = "warn"` Not Enforced
- **File**: `deny.toml` line 66
- **Severity**: **LOW**
- **Description**: Multiple versions of the same crate produce warnings, not errors. Substrate has many diamond dependencies, so this is pragmatic.
- **Impact**: Multiple crate versions increase binary size and audit surface. Not a direct security issue.

### DN-INFO-001: Trusted Org `nicira` and `AztecProtocol`
- **File**: `deny.toml` lines 76-77
- **Severity**: **INFO**
- **Description**: `nicira` (VMware/Broadcom networking) and `AztecProtocol` are trusted for git dependencies. Their presence suggests transitive dependencies from Substrate/Polkadot SDK. Verify these are still needed.

---

## 5. `rust-toolchain.toml`

### Overview
```toml
channel = "1.90.0"
components = ["rustfmt", "clippy", "rust-src"]
targets = ["wasm32-unknown-unknown", "wasm32v1-none"]
profile = "minimal"
```

### RT-INFO-001: Rust 1.90.0 (Current Stable)
- **Severity**: **INFO**
- **Description**: Rust 1.90.0 is a recent stable release. No known compiler vulnerabilities at time of audit. The `wasm32v1-none` target indicates modern WASM support. Profile `minimal` reduces unnecessary component downloads. ✅ Good.

---

## 6. Kubernetes Manifests & Helm Charts

### K8S-HIGH-001: No Standalone K8s Manifests or Helm Charts
- **Severity**: **HIGH**
- **Description**: No `deployment.yaml`, `service.yaml`, `ingress.yaml`, `networkpolicy.yaml`, or Helm charts exist in the repository. The entire K8s infrastructure is defined inline in the deploy workflow heredoc.
- **Impact**: 
  - Cannot be reviewed, tested, or validated independently of CI/CD changes
  - Cannot use `kubectl diff` or GitOps tools (ArgoCD in infra repo cannot reference these)
  - No separation of concerns between CI/CD and infrastructure
- **Recommendation**: Extract to `k8s/testnet/` directory with separate manifest files. Reference the infra repo's Helm charts.

### K8S-MED-001: No Resource Quota on Namespace
- **Severity**: **MEDIUM**
- **Description**: The `belizechain` namespace has no ResourceQuota. Any pod can consume all cluster resources (the single Standard_D2s_v3 node has 2 vCPU and 8 GiB RAM).
- **Recommendation**: Create a ResourceQuota limiting total namespace consumption.

---

## 7. Security-Sensitive Pattern Search

### Pattern: `--alice`
| Location | Context | Risk |
|----------|---------|------|
| `deploy.yml:266` | AKS production deployment | **CRITICAL** — well-known dev key in production |
| `scripts/run_all_tests.sh:46` | Test script | ✅ Acceptable for testing |
| `scripts/start_testnet.sh:36` | Local testnet | ✅ Acceptable for local dev |
| `tests/*/README.md` | Test documentation | ✅ Acceptable |
| `node/README.md:45` | Dev documentation | ✅ Acceptable |

### Pattern: `--unsafe-rpc-external`
| Location | Context | Risk |
|----------|---------|------|
| `deploy.yml:280` | AKS production deployment | **CRITICAL** |
| `node/README.md:188` | Checklist notes to disable | ✅ Awareness exists |

### Pattern: `--rpc-cors all`
| Location | Context | Risk |
|----------|---------|------|
| `deploy.yml:278` | AKS production deployment | **CRITICAL** |
| `docs/deployment/installation.md:199` | Deployment docs | ⚠️ Should note risk |
| `scripts/deploy/deploy_testnet.sh:83` | Testnet script | ⚠️ Medium risk |

### Pattern: `--rpc-external` / `--ws-external`
| Location | Context | Risk |
|----------|---------|------|
| `scripts/deploy/deploy_testnet.sh:83-84` | Testnet script | ⚠️ |
| `scripts/start_testnet.sh` (multiple) | Local testnet | ⚠️ |
| `docs/deployment/` (multiple) | Documentation | ⚠️ |

### Pattern: Hardcoded credentials
- **No hardcoded passwords or API keys found** in workflow files or scripts. ✅
- Secrets are properly referenced via `${{ secrets.* }}`. ✅
- `bootstrap_vm.sh` uses `ACR_PASSWORD` from environment, not hardcoded. ✅

### Pattern: `--dev`
| Location | Context | Risk |
|----------|---------|------|
| `scripts/run_all_tests.sh:46` | Test script | ✅ Acceptable |
| `scripts/build_release.sh:68` | Build smoke test | ✅ Acceptable |
| `node/README.md` (multiple) | Documentation | ✅ Acceptable |
| `deploy.yml` | **NOT present** | ✅ Good — not in deploy |

---

## 8. Cross-Reference with Prior Audits

Several findings in this audit were previously identified:
- **`--unsafe-rpc-external`**: Noted in Phase 4 (ND-DC-003, ND-H-005) — **still not fixed**
- **`emptyDir` data loss**: Noted in Phase 4 (ND-C-003) — **still not fixed**
- **`--alice` flag**: Noted in node/README.md checklist — **still not fixed**

---

## 9. Remediation Priority Matrix

| ID | Finding | Severity | Effort | Priority |
|----|---------|----------|--------|----------|
| CD-CRIT-001 | `--alice` in production | CRITICAL | 2h | **P0 — Immediate** |
| CD-CRIT-002 | `--unsafe-rpc-external` | CRITICAL | 1h | **P0 — Immediate** |
| CD-CRIT-003 | `--rpc-cors all` | CRITICAL | 30m | **P0 — Immediate** |
| CD-CRIT-004 | Mutable `latest` tag | CRITICAL | 30m | **P0 — Immediate** |
| CD-CRIT-005 | `continue-on-error: true` | CRITICAL | 5m | **P0 — Immediate** |
| CD-CRIT-006 | Node key as CLI arg | CRITICAL | 1h | **P0 — Immediate** |
| CD-HIGH-001 | No container scanning | HIGH | 1h | P1 |
| CD-HIGH-002 | Actions not pinned by SHA | HIGH | 2h | P1 |
| CD-HIGH-003 | ACR admin credentials | HIGH | 4h | P1 |
| CD-HIGH-004 | LoadBalancer exposes all ports | HIGH | 1h | P1 |
| CD-HIGH-005 | No K8s security context | HIGH | 30m | P1 |
| CD-HIGH-006 | emptyDir data loss | HIGH | 1h | P1 |
| CD-HIGH-007 | No network policy | HIGH | 1h | P1 |
| CD-HIGH-008 | `--force-authoring` | HIGH | 5m | P1 |
| DF-HIGH-001 | Base images not digest-pinned | HIGH | 30m | P1 |
| K8S-HIGH-001 | No standalone K8s manifests | HIGH | 2h | P1 |
| CD-MED-001 | Coverage not blocking deploy | MEDIUM | 5m | P2 |
| CD-MED-002 | Benchmark not blocking deploy | MEDIUM | 5m | P2 |
| CD-MED-003 | Cargo cache scope | MEDIUM | 15m | P2 |
| CD-MED-004 | No CODEOWNERS | MEDIUM | 15m | P2 |
| CD-MED-005 | No rollback strategy | MEDIUM | 30m | P2 |
| CD-MED-006 | Inline K8s manifest | MEDIUM | 2h | P2 |
| CD-MED-007 | Prometheus exposed | MEDIUM | 15m | P2 |
| SA-MED-001 | No clippy in CI | MEDIUM | 30m | P2 |
| DF-MED-001 | No .dockerignore | MEDIUM | 15m | P2 |
| DN-MED-001 | 4 ignored advisories | MEDIUM | Ongoing | P2 |
| K8S-MED-001 | No ResourceQuota | MEDIUM | 15m | P2 |

---

## 10. Summary

**The deployment pipeline is functional but not production-ready.** The six CRITICAL findings center on:

1. **Identity compromise** — `--alice` uses a publicly-known key as the validator identity
2. **Network exposure** — `--unsafe-rpc-external` + `--rpc-cors all` + `LoadBalancer` = validator RPC accessible to the entire internet
3. **Supply chain** — `latest` tag deployment + `continue-on-error` on build + unpinned actions
4. **Secret exposure** — Node key passed as CLI argument visible in process lists

The eight HIGH findings address defense-in-depth gaps: no image scanning, no K8s security context, ephemeral storage, and no network segmentation.

**Minimum viability for testnet**: Fix all 6 CRITICAL items.
**Minimum viability for mainnet**: Fix all CRITICAL + HIGH items (16 total).

---

*End of Phase 12 CI/CD, Docker & Deployment Security Audit*
