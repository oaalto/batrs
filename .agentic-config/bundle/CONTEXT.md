# {Context Name}

{One or two sentences describing what this context is and why it exists.}

## Language

### {Optional Term Group}

**{Term}**:
{One sentence defining what the term is.}
_Avoid_: {Ambiguous or rejected alternatives}

## Draft Language

Use this section only for new projects before live sources exist.

**{Draft Term}**:
{One sentence defining the provisional term.}
_Status_: draft; confirm during the first implementation slice.
_Avoid_: {Ambiguous or rejected alternatives}

## Relationships

- A **{Term}** belongs to exactly one **{Other Term}**.
- A **{Term}** can have many **{Other Terms}**.

## Example Dialogue

> **Dev:** "{Question that uses project language.}"
> **Domain expert:** "{Answer that demonstrates the intended vocabulary and boundaries.}"

## Flagged Ambiguities

- "{ambiguous word}" was used to mean both **{Term A}** and **{Term B}**; resolved: {resolution}.

## To Complete

Agent instruction: When this section lists items, offer the user LLM-assisted follow-up to resolve them. Do not invent definitions silently.

- {Concrete backlog item the seed pass could not fill from chat}
