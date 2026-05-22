#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

WATCH_PATHS=(
  "main.rs"
  "build.rs"
  "Cargo.toml"
  "Cargo.lock"
  "src"
  "content"
  "styles"
  "assets"
  "macros"
)

server_pid=""

start_server() {
  echo "starting blog at http://127.0.0.1:3000"
  cargo run &
  server_pid=$!
}

stop_server() {
  if [[ -n "${server_pid}" ]] && kill -0 "${server_pid}" 2>/dev/null; then
    kill "${server_pid}" 2>/dev/null || true
    wait "${server_pid}" 2>/dev/null || true
  fi

  server_pid=""
}

cleanup() {
  stop_server
}

file_signature() {
  local path="$1"

  if stat --format="%Y:%n" "$path" >/dev/null 2>&1; then
    stat --format="%Y:%n" "$path"
  else
    stat -f "%m:%N" "$path"
  fi
}

build_snapshot() {
  local path

  for path in "${WATCH_PATHS[@]}"; do
    if [[ -d "$path" ]]; then
      while IFS= read -r -d '' file; do
        file_signature "$file"
      done < <(find "$path" -type f -print0)
    elif [[ -e "$path" ]]; then
      file_signature "$path"
    fi
  done | LC_ALL=C sort
}

run_with_polling() {
  local interval="${DEV_POLL_INTERVAL:-1}"
  local previous_snapshot=""
  local current_snapshot=""

  trap cleanup EXIT INT TERM

  current_snapshot="$(build_snapshot)"
  previous_snapshot="$current_snapshot"
  start_server

  while true; do
    sleep "$interval"
    current_snapshot="$(build_snapshot)"

    if [[ "$current_snapshot" != "$previous_snapshot" ]]; then
      previous_snapshot="$current_snapshot"
      echo "change detected, restarting server"
      stop_server
      start_server
    fi
  done
}

if command -v cargo-watch >/dev/null 2>&1; then
  exec cargo watch \
    -w main.rs \
    -w build.rs \
    -w Cargo.toml \
    -w Cargo.lock \
    -w src \
    -w content \
    -w styles \
    -w assets \
    -w macros \
    -x run
fi

echo "cargo-watch not found, using built-in polling watcher"
run_with_polling
