---
name: orchestrator
description: Orchestrate multiple sub-agents safely — delegate discovery/research lanes proactively during the Understand phase, the gated phase sequence for complex work with implementation fan-out gated on settled seams, an async-first recon/research posture, one-writer-per-workspace, evidence-not-authority, test/review lanes as delegated implementation, verification, and review, narrow non-clone lane prompts, and escalate-up on unapproved decisions. Tool-agnostic; the concrete orchestration engine is named by the target repo's review prompt.
---

# Orchestrating multiple sub-agents

Adopt this posture when a task is large enough to warrant splitting into distinct sub-agent lanes. It names no orchestration engine, extension, or command — the concrete tooling and role definitions are decided per repo, not hardcoded here.

You remain the **single decision-maker**. Children are advisors and executors, never authorities. Keep one writer per workspace, keep lanes narrow and non-overlapping, and only delegate implementation work whose boundaries you have already settled.

## Proactive discovery/research delegation

Delegate **discovery and research** sub-agent lanes by default — it is the proactive posture, not a tolerated exception. During the **Understand** phase, launch recon lanes rather than serializing the whole exploration through your own context:

- Recon and research are read-only: cheap to parallelize and safe, with no writer contention.
- Launch them proactively and **async-first** (see below); only wait at a dependency barrier.
- Use lanes for recon, analysis, and adversarial review as a matter of course.

Trivial, single-context work still does not need a lane, and **implementation** lanes stay gated: only fan out implementation when the work is large, concurrency-safe, and its seams are already settled.

## Gated phase sequence for complex work

Implementation lanes do **not** fan out early. Follow the gated phases in order, settling seams and contracts before parallel implementation streams diverge:

**Understand → Decide → Design → Implement → Verify → Iterate → Ship**

- **Understand** — read the task and the code it touches end to end; this is where **discovery/research lanes are launched proactively**, before you branch into implementation.
- **Decide** — make the product/architecture/scope decisions yourself, up front.
- **Design** — settle the public seams, contracts, and interfaces before delegating implementation.
- **Implement** — only now fan out implementation lanes against agreed contracts.
- **Verify** — observe the actual artifact, not just exit codes.
- **Iterate** — fix in bounded waves.
- **Ship** — gate through an adversarial fresh-context review before ship.

Discovery/research fan-out is welcome from the **Understand** phase onward; only _implementation_ fan-out before the seams are settled is a mistake, not progress.

## One writer per workspace

Never run two lanes that write the same files in parallel. A single writer must own a workspace path at a time:

- Keep **one writer for the baseline** before parallel streams, so later merge work is minimized.
- Use separate workspaces or sequential writing for anything that would otherwise contend.
- If two lanes need to touch the same file, serialize them or split the file — do not race.

## Async-first posture for recon/research lanes

Launch recon/research lanes in the background **by default** and keep working while they run. Only **wait on their results at a dependency barrier** — the point where the parent's next step actually consumes them. Do not block the parent on a lane whose output nothing needs yet. Async-first is the norm for discovery/research; only implementation lanes that gate on each other's output wait eagerly.

## Distinct, non-clone lane prompts

Each lane gets a **narrow, non-overlapping prompt**. Never hand multiple lanes the same ticket number or the same generic instruction — identical clones drift and collide. One lane, one specific deliverable, one workspace.

## Evidence-not-authority

Receipts, CI results, review-bot reports, and lane outputs are **evidence that informs** your decision — they never grant merge, close, or release authority. Authority stays with you, or with a human/operator you escalate to. "CI passed" is a report to weigh, not a permission to merge.

## Escalate-up on unapproved decisions

Escalate to a supervisor, operator, or human on any unapproved decision involving:

- product direction
- architecture
- scope changes
- merge, close, or release
- credentials or secrets

Never take these calls unilaterally. Escalation is mandatory, not optional.

## Verification by observation

In verification, observe the **actual artifact** — the produced output, the rendered result, the running behaviour — not just tool exit codes. "Passing" only means correct when you have looked at the thing that was produced, not merely the process that claimed to produce it.

## Test and review lanes

Test and review work are valid lanes to delegate, with distinct placements:

- **Test-writing** is an **implementation lane**, gated like any implementation: settle the seams first, keep one writer per workspace, and do not fan it out before **Design** is settled.
- **Test-execution** is a **verification lane** for **Verify**: its output is evidence-not-authority, and it passes only when you observe the produced artifact, not just the exit codes (see Evidence-not-authority and Verification by observation). Gate order and fail-fast remain the parent's responsibility — do not delegate the whole gate table.
- **Review** is a named lane kind: an **adversarial fresh-context reviewer** in **Ship**, as in Adversarial fresh-context review before ship. Its methodology lives in the `review` skill; do not restate it here.

## Bounded fix waves

Review findings arrive in cycles. Apply **one fix wave per accepted defect class**, then re-review — do not endlessly chase every marginal nit across unbounded loops. Triage defects into accepted classes, fix each class in one pass, and keep churn bounded.

## Adversarial fresh-context review before ship

Before shipping, run an **adversarial fresh-context review** by a reviewer with no accumulated context bias. Disposition its findings, re-run the gates after fixes, and only then ship. Ship quality is the reviewer's job — and yours to honour by acting on the disposition.

## Post-install tailoring

This skill carries its guardrails in the body rather than as a scoped rule; treat the discipline here as the operative guidance when orchestrating lanes. The concrete orchestration engine, agent roles, and workspace tooling are repo-specific — the target repo's post-install review prompt names them. Do not invent references to any particular extension or tool in this skill's body.

### Repo-specific tailoring for this repository

- **Engine**: Pi subagents (`subagent` workflows / child runs) are the concrete orchestration mechanism in this repo.
- **Lane launch**: use Pi child runs with narrow prompts and non-overlapping deliverables; prefer async recon/research lanes first, then wait at dependency barriers.
- **Workspace discipline**: keep one writer per workspace. If two Pi child lanes need to edit concurrently, isolate them to separate worktrees or serialize them.
- **Barrier waits**: let read-only recon/research lanes run in the background and use Pi wait/barrier behavior only when the parent actually needs their output to proceed.
- **Fresh-context reviewer**: use a fresh-context reviewer lane in Pi before ship so review is adversarial and not biased by the implementation context.
- **Escalate-up path**: unresolved product, architecture, scope, credentials, merge, or release decisions escalate to the human operator in the main chat; child lanes do not decide them.
