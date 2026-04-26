---
name: "BelizeChain Review"
description: "Run a strict BelizeChain review for a PR, diff, file, issue, or audit finding with findings-first output and false-positive filtering."
argument-hint: "PR number, diff, file path, issue, audit artifact, or review target"
agent: "BelizeChain Review"
model: "GPT-5 (copilot)"
---
Review the provided BelizeChain target using a strict evidence-first standard.

Requirements:
- Prioritize true positives over coverage theater.
- Start from the narrowest concrete anchor in the provided context.
- Use code, tests, command output, logs, config, or a direct contract violation before calling something a defect.
- Distinguish confirmed defects, probable risks, and open questions.
- Calibrate severity realistically. Reserve highest severity for consensus, funds, identity, compliance, bridge, or production-ops impact.
- Mention stale docs, missing tests, rollback gaps, or validation gaps when they materially affect confidence.

Output format:
- Findings first, ordered by severity.
- Each finding must include location, evidence, impact, and the most direct fix or validation path.
- If no concrete finding remains after review, state that explicitly and mention any residual verification gaps.