#!/usr/bin/env bash
set -euo pipefail

# Manual: one strict full build, then `mkdocs serve`.
# `serve` runs MkDocs’ dev server with live reload on by default (watches docs/,
# mkdocs.yml, etc.; browser auto-refreshes). Disable with `mkdocs serve --no-livereload`.

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

if [[ -x "$root/.venv-docs/bin/mkdocs" ]]; then
  mkdocs_cmd=("$root/.venv-docs/bin/mkdocs")
elif command -v mkdocs >/dev/null 2>&1; then
  mkdocs_cmd=(mkdocs)
else
  echo "mkdocs not found. Create a venv and install deps:" >&2
  echo "  python3 -m venv .venv-docs" >&2
  echo "  .venv-docs/bin/pip install -r requirements-docs.txt" >&2
  exit 1
fi

if [[ -x "$root/.venv-docs/bin/python" ]]; then
  python_cmd=("$root/.venv-docs/bin/python")
else
  python_cmd=(python3)
fi

tcp_has_listener() {
  local host=$1 port=$2
  (echo -n '' >/dev/tcp/"$host"/"$port") &>/dev/null
}

pick_free_tcp_port() {
  "${python_cmd[@]}" -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()'
}

user_supplied_dev_addr=false
user_wants_no_strict_serve=false
user_has_strict_serve=false
no_livereload=false
for arg in "$@"; do
  case "$arg" in
    -a | --dev-addr | -a=* | --dev-addr=*)
      user_supplied_dev_addr=true
      ;;
    -s | --strict)
      user_has_strict_serve=true
      ;;
    --no-strict)
      user_wants_no_strict_serve=true
      ;;
    --no-livereload)
      no_livereload=true
      ;;
  esac
done

"${mkdocs_cmd[@]}" build --strict

serve_prefix=()
if [[ "$user_wants_no_strict_serve" == false && "$user_has_strict_serve" == false ]]; then
  serve_prefix+=(--strict)
fi

serve_args=("$@")
if [[ "$user_supplied_dev_addr" == false ]]; then
  addr="${MKDOCS_SERVE_ADDR:-127.0.0.1:8000}"
  port="${addr##*:}"
  host="${addr%:"$port"}"
  if tcp_has_listener "$host" "$port"; then
    free_port="$(pick_free_tcp_port)"
    addr="$host:$free_port"
    echo "mkdocs: ${MKDOCS_SERVE_ADDR:-127.0.0.1:8000} is already in use; serving at http://$addr" >&2
    echo "set MKDOCS_SERVE_ADDR or pass -a to choose a host:port" >&2
  fi
  serve_args=(-a "$addr" "$@")
fi

if [[ "$no_livereload" == false ]]; then
  echo "mkdocs: live reload on — edit files under docs/ (and mkdocs.yml); the site refreshes in the browser." >&2
else
  echo "mkdocs: live reload off (--no-livereload); restart this script after edits." >&2
fi

exec "${mkdocs_cmd[@]}" serve "${serve_prefix[@]}" "${serve_args[@]}"
