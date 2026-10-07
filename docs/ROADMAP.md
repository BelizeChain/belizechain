# BelizeChain Roadmap

Project status, verified milestones, and planned development.

> [!NOTE]
> **Testbed Protocol Notice**: All monetary metrics on testbed environments represent simulated protocol parameters. bBZD operates as a testnet statutory stablecoin model pegged 1:1 to BZD for architectural evaluation prior to live Central Bank fiat reserve custodianship.

**Status (October 2026):** BelizeChain is in **active testnet development** on
the single-node Ceiba testnet. **Mainnet has not launched.** Items marked ✅ are
live and verifiable; items marked 📅 are plans, not commitments.
**Last verified:** 2026-10-06.

---

## Where the network is today

### Runtime and pallets

- Runtime **spec_version 109**, hot-upgraded on the live testnet on 2026-10-04
  (block #70616) with no node restart
- **19 custom pallets** at runtime indexes 20–38, plus **14 standard Substrate
  pallets** at indexes 0–13
- 6-second block times, BABE block production with GRANDPA finality, SS58
  address prefix `1981`
- ~2,000 TPS target throughput for the runtime; no verified public benchmark
  figures are published yet

**Custom pallets:** Economy (DALLA + bBZD, treasury), Identity (BelizeID,
3-level KYC), Governance (district councils, proposals), Compliance (KYC/AML,
sanctions), Staking (PoUW validator staking, federated-learning rewards),
Oracle (merchant verification, price feeds), Payroll (enterprise and
government), Interoperability (bridge logic, design stage), BelizeX (on-chain
DEX/AMM), LandLedger (property registry), Consensus (Proof of Useful Work),
Quantum (quantum job registry), Community (grants), BNS (`.bz` domain
registry), Mesh (Meshtastic LoRa relay), Justice, Whistleblower, Moderation,
and StorageProof (Pakit storage-proof verification).

### Smart contracts (GEM)

Eight ink! 5.1.1 contracts in the `gem` repository: DALLA token (PSP22),
BeliNFT (PSP34), PSP37 multi-token, BelizeX DEX (factory + pair + router AMM),
Simple DAO, Faucet, Access Control library, and the Hello BelizeChain example.

> Contracts deployed on the previous chain were wiped by the 2026-09-29
> re-genesis and have not been redeployed yet.

### Services live on the Ceiba testnet

Maya Wallet (`wallet.belizechain.org`), Blue Hole Portal (`portal.belizechain.org`),
the belizechain.org site (`belizechain.org`), Pakit storage API, Nawal AI API,
Kinich quantum API, IPFS gateway, block explorer, nginx TLS front end,
Prometheus + Grafana + Alertmanager (Telegram alerting live since 2026-10-02),
and nightly Postgres / chain / Pakit backups with integrity manifests.

### Repositories

`belizechain` (runtime + node), `ui` (Maya Wallet + Blue Hole Portal), `infra`
(Ceiba stack), `gem` (contracts), `pakit-storage`, `nawal-ai`,
`kinich-quantum`, and `belizechain.org` (website).

## Verified development timeline

Dates below are verified from repository history and the Ceiba operations
records. Earlier versions of this file claimed 2024–2025 milestones (genesis,
hundreds of validators, production metrics) that never occurred; they have
been removed.

| Date | Milestone |
|------------|-----------------------------------------------------------------------|
| 2026-05-02 | Ceiba Phase 2: Pakit, Nawal, Kinich, GEM contracts, and Blue Hole Portal deployed to the testnet stack |
| 2026-09-17 | Non-destructive backup/restore drill completed |
| 2026-09-21 | Maya Wallet exposed on the testnet (originally the `/wallet/` path route) |
| 2026-09-23 | Chain stalled after a zero-authority BABE epoch; root cause diagnosed |
| 2026-09-29 | Chain re-genesis; session-key filtering fix prevents a repeat of the empty-authority stall |
| 2026-10-02 | Alertmanager → Telegram alerting armed and delivery-verified; nightly backups verified |
| 2026-10-04 | Runtime 109 hot-upgraded on the live chain (rollback blob retained) |
| 2026-10-07 | Hostname routing: apex/`www` serve the org site, `testnet.*`/`portal.*` the Portal, `wallet.*` the wallet, `explorer.*` the explorer |





---

## In progress

- **Public DNS and TLS** — subdomain map and bring-up plan live in
  `infra/docs/DNS_AND_SUBDOMAIN_PLAN.md`; registrar records and a real TLS
  certificate are pending
- **Testnet stabilization** — live-restore drill (needs a maintenance window),
  host stability items (memtest / BIOS / kernel), and expansion from the
  current single node to a multi-node testnet
- **GEM redeployment** — the eight contracts need to be rebuilt and redeployed
  on the post-re-genesis chain

## Pre-mainnet requirements (planned)

- **Third-party security audit** — required before mainnet; no firm has been
  engaged yet
- **Bug bounty program** — documentation is in place; launch is planned, not
  yet live
- **Penetration testing** — planned; no vendor engaged
- **Validator decentralization** — public onboarding docs and a multi-operator
  test have to precede mainnet
- **Governance handover** — `pallet_sudo` and `EnsureRoot` fallbacks are
  scheduled for removal pre-mainnet
- **Mainnet genesis** — spec, launch criteria, and ceremony to be planned
- **Regulatory engagement** — FSC engagement is part of the pre-mainnet plan
  (see `docs/security/COMPLIANCE_REGULATORY.md`)



------




## Long-term direction (not commitments)

High-level ambitions carried in the design:

- Government services on-chain: payroll, LandLedger property records, and
  BelizeID adoption
- BelizeX DEX liquidity growth and tourism-cashback merchant expansion
- BNS `.bz` registration and Pakit-hosted websites
- Cross-chain bridges via the Interoperability pallet (design stage; relayers
  not deployed)
- Off-grid mesh networking via the Mesh pallet (pallet in the runtime; field
  rollout pending)
- Nawal (AI) and Kinich (quantum) service maturation, including PoUW
  integration
- Regional cooperation with Caribbean neighbours

## Metrics

No verified public metrics (network, user, or economic) are published yet.
Testnet figures are simulated protocol parameters, as noted at the top of this
document. Live network statistics will be published once public access (DNS,
TLS, status surface) is up.

---

## How to follow

- **Repositories**: https://github.com/BelizeChain
- **Website**: https://belizechain.org (DNS and TLS bring-up in progress)
- **Release notes**: [CHANGELOG.md](../CHANGELOG.md) in this repository

---

*This roadmap was corrected on 2026-10-06: earlier revisions described
milestones, metrics, and partnerships that never existed. See the git history
of this file for what was removed.*
