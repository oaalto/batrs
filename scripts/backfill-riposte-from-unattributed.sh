#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
db="${1:?usage: $0 <combat_damage.db>}"

if [[ ! -f "$db" ]]; then
  echo "error: database not found: $db" >&2
  exit 1
fi

cd "$repo_root"
export BATRS_DAMAGE_DB="$db"
cargo test -q backfill_user_damage_db -- --ignored
