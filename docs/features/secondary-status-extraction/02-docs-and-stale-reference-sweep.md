# 02 — Docs and stale reference sweep

## Parent

`prd.md`

## What to build

After Secondary Status extraction lands, align engineering docs with the new ownership boundary so readers do not look for guild HUD state in Stats or follow superseded Nergal ticket guidance.

## Blocked by

01 — Extract Secondary Status module from Stats

## Status

ready-for-agent

## Acceptance criteria

- [ ] `CONTEXT.md` Secondary Status and Nergal Status sections match implemented behavior (verify against ticket 01; update if drift)
- [ ] Engineering wiki path-map / concept pages no longer describe guild HUD as stats-owned; Secondary Status referenced where HUD band is documented
- [ ] Stale references to `StatsEffect` guild HUD variants or stats render helpers for secondary rows removed or updated in wiki and feature docs
- [ ] `nergal-resource-status-ownership` slice tickets (`01`, `02`) noted as superseded in feature folder or left clearly unimplemented per parent PRD status
- [ ] Wiki log entry recorded per project wiki skill (update, ingest, or intentional skip)
