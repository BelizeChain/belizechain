---
name: "Docs Alignment Sweep"
description: "Verify every state claim in the BelizeChain repos' docs against live reality, correct what drifted, and leave dated evidence behind. Run monthly, before milestones, or whenever docs are suspected stale."
argument-hint: "Scope: all repos, one repo, or one doc area (e.g. 'kinich docs', 'Ceiba ops docs')"
agent: "BelizeChain"
---

# Docs Alignment Sweep

Docs across all BelizeChain repos have repeatedly drifted from reality: claims
that something is "not done" when it shipped weeks ago, image tags that no
longer match the host, links to deleted files, and features documented as
working whose target extrinsics do not exist. This prompt exists so that
alignment is a repeatable, evidence-driven procedure — never guesswork.

**Scope: $ARGUMENTS** (default: all seven repos in the workspace).

## Step 0 — mechanical sweep first (deterministic)

Run the checker from the workspace root:

```
./infra/deploy/check-doc-claims.sh /home/wicked/Projects/Belizechain
```

It reports dead markdown links, compose vars missing from `.env.example`,
Node-version misalignment, image-pin drift (`--host`), and extrinsic calls in
sibling repos whose target functions do not exist in the runtime. Fix these
findings first — they are facts, not judgment calls. Archive snapshots
(`old_audits/`, `docs/archive/`, `ui-docs-archive/`) are excluded by design;
do not "fix" frozen history unless it links to something that misleads about
the PRESENT.

## Step 1 — inventory the semantic claims

State words that rot: **live, armed, deployed, active, pending, not yet, TODO,
complete, enabled, verified, in progress**. Grep for them across each repo's
`docs/`, `README*`, `.github/copilot-instructions.md`, and operation records.

## Step 2 — verify each claim against a live source

| Claim type | Verification source |
|---|---|
| Service live/healthy | `ssh wicked@100.81.45.25 'docker ps'`, health endpoints |
| Timer/schedule claims | `systemctl cat <unit>` AND `systemctl show <unit> -p <prop>` — effective config, not declared |
| Chain state (block, spec, epoch) | `curl -s -H 'Content-Type: application/json' -d '{"id":1,"jsonrpc":"2.0","method":"chain_getHeader","params":[]}' http://100.81.45.25:9944` |
| Alerting armed | live config in container + `.env` key names (never values) |
| CI/PR/alert state | `gh run list`, `gh pr list`, `gh api .../dependabot/alerts` |
| Feature works | code path check: caller exists, target extrinsic exists in `belizechain/pallets` or `runtime/src/lib.rs` |
| Backup/restore | timer logs (`journalctl -u belizechain-backup.service`), newest archive + manifest |

## Step 3 — correct with dated evidence

- Update the doc to the verified state. Add `(verified YYYY-MM-DD)` to the
  corrected line or section.
- Historical records (incident reports, dated drills) describe the past — leave
  their body alone; if their conclusions are superseded, add a dated status
  update block instead of rewriting history.
- If a claim cannot be verified right now, write exactly that: "not verified".
  Never pad with plausible guesses.

## Step 4 — report

Findings-first table: claim | doc file | verdict (true / stale / false) |
live evidence used | correction made. List anything left unverified.

## Known traps (learned 2026-10-03, verify before trusting)

- **Memory files can be stale** — repo memory (`/memories/repo/`) is often
  fresher than user memory; cross-check before any "X is not done" claim.
- **Absence of success logs is NOT failure** — alertmanager logs notify
  success only at DEBUG level; look for ERROR lines instead.
- **Compare accounts by `public_key` bytes** — the chain uses SS58 prefix 1981,
  so rendered addresses differ between tools.
- **mtime ≠ staleness** — wasm-builder is content-addressed; trust the hash.
- **`/opt/belizechain` is NOT a git checkout** — deploy is copy-based; docs
  saying `git pull` there are wrong.
- **A green CI step can be a no-op** — check that a step actually does
  something before citing it as evidence (placeholder integration steps).
- **`steps=0` job failures are infrastructure** (billing/runner), not code —
  read the job annotation before diagnosing.
