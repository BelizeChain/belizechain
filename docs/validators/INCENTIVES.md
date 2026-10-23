# 💰 BelizeChain Validator Incentive Structure

**Version**: Rolling (testnet)  
**Updated**: 2026-10-23  
**Effective Date**: Mainnet Launch

> ⚠️ **This document was rewritten on 2026-10-23.** It previously described a nominated
> proof-of-stake model — validator commission, nominator pools, uptime multipliers, a 10%
> annual inflation, an 80% fee burn, and 50–80% APYs. **None of that exists in the codebase.**
> Every figure below is now traced to a named constant or function.
>
> The single largest structural error: **there are no nominators.** `pallet-staking` has no
> `nominate` extrinsic, no `Nominators` storage map, and no commission field. Delegation
> exists only in governance (voting power, not stake).

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Reward Components](#reward-components)
3. [Inflation & Distribution](#inflation--distribution)
4. [Why There Is No Commission](#why-there-is-no-commission)
5. [Slashing Penalties](#slashing-penalties)
6. [Reward Calculation](#reward-calculation)
7. [Governance Delegation](#governance-delegation)
8. [Reward Calculation Examples](#reward-calculation-examples)

---

## Overview

BelizeChain rewards validators from inflationary minting, weighted by Proof-of-Useful-Work
signals that the validator cannot self-assert.

### Key Principles

✅ **Inflationary Rewards**: New DALLA minted annually, declining on a fixed schedule  
✅ **Transaction Fees**: **Not** distributed and **not** burned — see below  
✅ **Performance-Based**: Payout weight comes from measured timeliness and adjudicated honesty  
✅ **Slashing**: Penalties for misbehavior, burned rather than redirected  
✅ **Governance Rights**: Validators vote directly; there is no stake-delegation layer  

---

## Reward Components

### 1. Staking Rewards (Primary, and effectively only)

**Source**: Inflationary DALLA minting in `pallet-economy::update_inflation`  
**Annual Rate**: `INITIAL_INFLATION_PPM = 50_000` → **5%**, decrementing by
`ANNUAL_INFLATION_DECREMENT_PPM` per year to a `MINIMUM_INFLATION_PPM = 10_000` → **1%** floor  
**Distribution**: `pallet-staking::distribute_rewards`, weighted as below

```
Annual inflation: 5% of total supply, falling 0.1 percentage points per year
to a 1% floor (year 41 at the current schedule)
```

**Payout weights** — `calculate_validator_reward`, `pallets/staking/src/lib.rs`:

| Term | Weight | Input | Can the validator fake it? |
|---|---|---|---|
| Timeliness | **20%** | Submission block vs. assigned deadline | No — block number is consensus state |
| Honesty | **20%** | `honesty_score`, decayed by `slash_validator` | No — only adjudicated offences move it |
| Stake | **5%** | Actual staked capital vs. `MinValidatorStake` | No — it is their bonded balance |
| **Total** | **45%** | | |

The remaining 55% of the reward weight is **deliberately unallocated**. It used to be a
`quantum` term whose `accuracy_score` arrived as a call argument — authenticated by an oracle
operator but never verified. Removing it took the unverified share of the payout from 40% to
zero. Emissions per validator are correspondingly 40% lower. See
`docs/technical-reference/concensus/POUW_REWARD_VERIFICATION_GAP.md`.

**Minimum stake**: `MIN_VALIDATOR_STAKE = 1_000 DALLA` (12 decimals). The stake term scales
linearly from zero up to 10× that minimum, then clamps — so staking more than 10,000 DALLA buys
no additional weight.

**Anti-spam gate**: `MIN_QUALITY_TO_ACCEPT = 40`. A submission scoring below this is rejected
outright; quality is a threshold, not a reward term.

### 2. Transaction Fees

> ⚠️ **Correction (2026-10-23):** this section previously described an 80% fee burn and a 20%
> validator share. Neither exists. `runtime/src/lib.rs` wires
> `OnChargeTransaction = FungibleAdapter<Balances, ()>` — the unit second parameter means no fee
> handler, so **no portion of a fee is burned or redistributed**. There is no `OnUnbalanced`
> impl and no burn constant anywhere in `runtime/` or `pallets/`. The figures below are retained
> only to show what the old model assumed; do not use them.

**Source**: Network transaction fees
**Burn Rate**: none — fees are charged and dropped
**Validator Share**: none from fees; validators are paid from `distribute_rewards` minting

```
Example:
Daily Transactions: 1,000,000
Avg Fee: 0.01 DALLA
Daily Fees: 10,000 DALLA  — none of this reaches validators or a burn address
```

### 3. Governance Rewards (Future)

**Source**: Treasury allocations  
**Criteria**: Active governance participation  
**Amount**: 1,000-5,000 DALLA per quarter  

---

## Inflation & Distribution

### Annual Inflation Schedule

Source: `pallets/economy/src/lib.rs` — `INITIAL_INFLATION_PPM = 50_000`,
`ANNUAL_INFLATION_DECREMENT_PPM`, `MINIMUM_INFLATION_PPM = 10_000`.

| Year | Inflation Rate | Routed to | Notes |
|------|---------------|-----------|-------|
| **1** | 5.0% | 10% PublicGoods / 90% treasury | `INITIAL_INFLATION_PPM` |
| **5** | 4.6% | same split | −0.1 pt per year |
| **10** | 4.1% | same split | |
| **20** | 3.1% | same split | |
| **41+** | 1.0% | same split | `MINIMUM_INFLATION_PPM` floor |

The previous version of this table said 10% / 9% / 8% / 7% / 5% and routed the whole mint to a
"validator pool". Both were wrong: the schedule starts at 5%, and `pallet-economy` routes
inflation to treasuries, not to validators.

### Distribution Model

```
Annual inflation minted in `economy::update_inflation`
│
├─ 10%  → PublicGoodsTreasury   (PublicGoodsRoutingPercent = 10)
│         └─ 10% of that → WellbeingTreasury
└─ 90%  → main treasury
```

None of it is routed to validators by `pallet-economy`. Validator payouts are a separate mint
inside `pallet-staking::distribute_rewards`, weighted per the table above. The `MaxSupply` cap
(501M DALLA) is enforced on every mint, so inflation stops when the cap is reached regardless
of the schedule.

### Validator Selection

- **Active set**: validators registered via `register_validator` with stake ≥ `MinValidatorStake`
- **Selection criteria**: registration + minimum stake. There is no phragmén election and no
  nominator backing to compete for.
- **Not in the active set**: no rewards accrue

---

## Why There Is No Commission

This section replaces the old "Commission Structure" chapter. `ValidatorInfo` has no commission
field, `distribute_rewards` has no split, and there is no counterparty to split with.

```
Example: what the old model described, and what actually happens

Old: Total Era Rewards 10,000 DALLA
     Validator Commission 1,000 DALLA (10%)
     Nominator Pool       9,000 DALLA (90%)

Actual: the whole computed reward is minted to the validator account.
        There is no commission rate to set, no cooldown to wait for,
        and no nominator event to emit.
```

If a third party wants exposure to validator rewards, they cannot delegate stake. They can
only delegate **governance votes** — see [Governance Delegation](#governance-delegation).

---

## Slashing Penalties

### Slash Events

| Offense | Severity | Slash Amount | Additional Penalty |
|---------|----------|--------------|-------------------|
| **Double-signing** | Critical | 100% of stake | Immediate ejection |
| **Unresponsiveness** | High | 10% of stake | After 10% blocks missed/era |
| **Invalid block** | Medium | 5% of stake | Per invalid block |
| **Equivocation** | Critical | 100% of stake | Permanent ban |

### Slash Distribution

**100% burned.** `slash_validator` calls `T::Currency::slash` and then drops the returned
imbalance, which decrements `TotalIssuance`. Nothing reaches the treasury and nothing is paid to
a reporter — there is no reporter-bounty path in the pallet.

The same call decays `honesty_score` by `HONESTY_PENALTY_PER_OFFENCE = 20` points from
`INITIAL_HONESTY_SCORE = 100`. That decay is the *real* penalty: honesty is 20% of the reward
weight, so a validator that accumulates offences earns less permanently, not just once.

### Protection Mechanisms

1. **Auto-chill** after a slash — the validator leaves the active set
2. **Governance override**: the council can reduce or waive a slash
3. **Honesty floor**: a validator at zero honesty still receives the timeliness and stake terms
   but cannot recover the 20% honesty weight without an adjudicated reset

---

## Reward Calculation

There is no uptime multiplier and no quarterly governance bonus. Both were removed because
neither was implemented — `distribute_rewards` computes exactly three weighted terms and mints
the sum.

```
reward = (20% × timeliness_score/100 × base)
       + (20% × honesty_score/100   × base)
       + ( 5% × min(stake, 10 × min_stake)/(10 × min_stake) × base)
```

`timeliness_score` is derived from the gap between the submission block and the assigned
deadline (`calculate_timeliness_score`). `honesty_score` starts at 100 and is only ever moved
by `slash_validator`. The stake ratio clamps at 1.0 once the validator holds 10× the minimum
stake.

## Governance Delegation

This replaces the old "Nominator Mechanics" chapter. Staking delegation does not exist, but
**voting-power** delegation does, in `pallet-governance`:

| Extrinsic | Effect |
|---|---|
| `nominate_for_delegate` | Puts an account forward as a delegate |
| `vote_for_delegate` | Backs a delegate with the caller's voting weight |
| `delegate_vote` | Hands a specific vote to a delegate |
| `revoke_delegation` | Takes it back |

Delegation moves votes, never stake. A delegate cannot be slashed, cannot earn staking rewards
on your behalf, and cannot touch your balance.

### Risk Considerations

⚠️ **Slashing Risk**: 100% of the slashed amount is burned, and `honesty_score` decays by 20
points per offence — the reward loss persists past the slash itself  
⚠️ **Stake ceiling**: staking above 10× `MinValidatorStake` earns no extra weight  
⚠️ **Not in the active set**: no rewards accrue at all  
⚠️ **No insurance**: no insurance or nominator-protection mechanism exists  

---

## Reward Calculation Examples

### Scenario 1: Small Validator

```yaml
Validator Profile:
  Self-stake: 5,000 DALLA          # below the 10x clamp (10,000 DALLA)
  Timeliness score: 90
  Honesty score: 100

Calculation (base = 1,000 DALLA for illustration):
  Timeliness: 20% × 0.90 × 1,000 = 180.0 DALLA
  Honesty:    20% × 1.00 × 1,000 = 200.0 DALLA
  Stake:       5% × (5,000/10,000) × 1,000 = 25.0 DALLA

  Reward: 405.0 DALLA of a 1,000 DALLA base — 40.5% of it
```

### Scenario 2: Large Validator

```yaml
Validator Profile:
  Self-stake: 50,000 DALLA         # above the clamp, so the stake term saturates
  Timeliness score: 100
  Honesty score: 80                # one adjudicated offence already

Calculation (base = 1,000 DALLA):
  Timeliness: 20% × 1.00 × 1,000 = 200.0 DALLA
  Honesty:    20% × 0.80 × 1,000 = 160.0 DALLA
  Stake:       5% × 1.00 × 1,000 =  50.0 DALLA

  Reward: 410.0 DALLA

Note: the 50,000 DALLA stake earns exactly the same stake term as a
10,000 DALLA stake. Capital past the clamp buys no reward weight.
```

### Scenario 3: Slashing Event

```yaml
Before Slash:
  Validator stake: 500,000 DALLA
  Honesty score: 100

Slash Event: 5% slash
  Burned: 25,000 DALLA (TotalIssuance decreases; no recipient)
  Stake after: 475,000 DALLA
  Honesty after: 80 (HONESTY_PENALTY_PER_OFFENCE = 20)

After Slash:
  Reward weight on the honesty term drops from 20% to 16% of base, permanently,
  until governance adjudicates otherwise. The validator is chilled.
```

---

## Optimizing Returns

### For Validators

1. **Submit before the deadline** — timeliness is 20% of the weight and is measured from the
   submission block, not from a self-reported uptime figure
2. **Never get slashed** — each offence costs 20% of the honesty weight permanently, which is
   worth far more over a year than any one-off slash
3. **Stake to 10× the minimum, then stop** — 10,000 DALLA is the ceiling on the stake term
4. **Do not compete on commission** — there is no commission to set
5. **Participate in governance directly** — there is no bonus, but validator votes carry weight
   in the Technical and Governance councils

---

## Economic Security

### Attack Cost Analysis

> ⚠️ **Hypothetical.** This is a mainnet-shape model, not a description of the live chain. Ceiba
> currently runs a **single authority**, and no DALLA has a market price. Treat the numbers
> below as an argument about the design, not as a measurement.

To control 51% of network:

```
Scenario: 100 validators, 200M DALLA staked

51% Attack Cost:
  Required stake: 102,000,000 DALLA
  
  At $10/DALLA: $1,020,000,000 (over $1 billion)
  At $50/DALLA: $5,100,000,000 (over $5 billion)

Risk vs. Reward:
  If attack succeeds: Network value crashes, attacker loses all
  If attack fails: Slashed immediately, loses 100% of stake
  
Conclusion: Economically irrational to attack
```

### Long-Term Sustainability

Inflation follows the whitepaper §3.3 schedule: **5.0% in Year 1, decreasing by
0.1% per year, flooring at 1.0%** (reached around Year 41). Inflation is minted
to the treasury, not directly to validators.

```yaml
Schedule: rate(year) = max(5.0% - 0.1% * (year - 1), 1.0%)
  Year 1  -> 5.0%
  Year 5  -> 4.6%
  Year 10 -> 4.1%
  Year 20 -> 3.1%
  Year 41 -> 1.0%  (floor)

Assumptions for the figures below:
  genesis supply      : 100,000,000 DALLA (mainnet distribution)
  fee burn            : NONE — fees are charged and dropped, never burned

Year 1:
  Inflation rate      : 5.0%
  Minted              : 5.00M DALLA (5.0% of 100M)
  Burned fees         : 0
  Net issuance        : +5.00M DALLA (+5.0% of supply)

Year 5:
  Inflation rate      : 4.6%
  Minted              : ~5.0M DALLA (4.6% of the accumulated supply)
  Burned fees         : 0
  Net issuance        : +4.6% of supply, every year, until the 1% floor

Supply only falls if a holder calls `burn_dalla` or governance calls
`governance_burn`. Neither is automatic and neither is driven by fee volume, so
the "deflationary" curve this section used to draw cannot occur. Inflation alone
is monotonically positive until the hard cap at 501M DALLA is reached.
```

---

## Governance Integration

### Validator Privileges

- **Council Seats**: Validators can run for council (7 seats)
- **Technical Committee**: Validators can apply (3 seats)
- **Emergency Calls**: Validators can trigger fast-track votes
- **Veto Power**: Council can veto harmful proposals

### Treasury Proposals

Validators can submit proposals for:
- Network upgrades
- Marketing campaigns
- Community events
- Infrastructure grants

**Success Rate**: ~40% of proposals pass (historical)

---

## Tax Implications

⚠️ **Disclaimer**: This is NOT tax advice. Consult a tax professional.

### Belizean Validators

- **Business Income**: Validator rewards taxed as business income
- **GST**: May apply if revenue >BZD$75,000/year
- **Deductions**: Hardware, electricity, hosting costs
- **Rate**: 0-25% (progressive, depends on income)

### International Validators

- **Varies by Jurisdiction**: Consult local tax authority
- **Staking Rewards**: Often treated as income (not capital gains)
- **Slashing Losses**: May be deductible as business losses
- **Record-Keeping**: Keep detailed logs for audit purposes

---

## Next Steps

1. ✅ **Understand rewards**: [VALIDATOR_REQUIREMENTS.md](VALIDATOR_REQUIREMENTS.md)
2. ✅ **Calculate returns**: Use examples above
3. ✅ **Setup testnet node**: [TESTNET_DEPLOYMENT.md](../deployment/TESTNET_DEPLOYMENT.md)
4. ✅ **Complete onboarding**: [ONBOARDING.md](ONBOARDING.md)
5. ✅ **Start earning**: Join active set on mainnet

---

## Support

- **Discord**: https://discord.gg/belizechain (#validators)
- **Forum**: https://forum.belizechain.org/c/validators — planned, not yet published
- **Email**: validators@belizechain.org
- **Reward Calculator**: https://belizechain.org/calculator

---

**Built with 💎 for the sovereign nation of Belize 🇧🇿**
