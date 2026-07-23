---
name: grill-with-docs
description: A relentless interview to sharpen a plan or design, which also creates docs (ADR's and glossary) as we go.
disable-model-invocation: true
---

Interview the user relentlessly about every aspect of their plan until you reach a shared understanding. Walk down each branch of the decision tree, resolving dependencies one-by-one. For each question, provide your recommended answer.

Ask questions **one at a time**, waiting for feedback before continuing.

While grilling, actively sharpen the domain model:

- When the user uses a term that conflicts with `CONTEXT.md`, call it out. "Your glossary defines X as Y, but you seem to mean Z — which is it?"
- When the user uses vague terms, propose a precise canonical term. "You're saying 'account' — do you mean Customer or User?"
- Stress-test domain relationships with concrete edge-case scenarios.
- When the user states how something works, check whether the code agrees and surface contradictions.

Update `CONTEXT.md` and glossary terms inline as they crystallise. Offer ADRs only when all three apply: hard to reverse, surprising without context, result of a real trade-off.

Consult the engineering wiki (`docs/wiki/`). Load `.agents/skills/repo-navigation/SKILL.md` for exploration routing. Delegate background research and exploration to sub-agents; keep the grilling conversation inline.

Delegate to sub-agents for:

- **Wiki consultation** — delegate to a `/wiki` sub-agent: "Query the engineering wiki for context on [topic/area]. Report findings as a structured summary."
- **CONTEXT.md / ADR lookup** — delegate to a sub-agent: "Read `CONTEXT.md` and any ADRs in [area]. Report any terms, decisions, or constraints relevant to [topic]."
- **Research tickets** — delegate to a `/research` sub-agent when knowledge outside the working directory is required.

**Termination.** Stop the interview when the user has answered every question, confirmed they have nothing more to add, or explicitly signals agreement (e.g. "looks good", "let's go with that", "that works").

**On completion.** Produce a structured summary of every decision agreed upon — each decision on its own line, with the chosen option and any open constraints. Do not start implementing. After the summary, suggest: "Want me to run `to-spec` to turn this into a formal spec?" — do not proceed further without an explicit command from the user.
