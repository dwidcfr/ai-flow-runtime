#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
API_PORT="${API_PORT:-8080}"
WEB_PORT="${WEB_PORT:-3000}"
API_RESTART="${API_RESTART:-true}"
API_PID=""
API_STARTED_BY_SCRIPT=false

cleanup() {
  if [[ "$API_STARTED_BY_SCRIPT" == true && -n "$API_PID" ]] && kill -0 "$API_PID" 2>/dev/null; then
    echo "Stopping ai-api (pid $API_PID)..."
    kill "$API_PID" 2>/dev/null || true
    wait "$API_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

if ! command -v npm >/dev/null 2>&1; then
  echo "npm not found. Install Node.js."
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust toolchain not found. Install cargo to run ai-api."
  exit 1
fi

if [[ -f "$REPO_ROOT/.env" ]]; then
  set -a
  # shellcheck disable=SC1091
  source "$REPO_ROOT/.env"
  set +a
  echo "Loaded environment from $REPO_ROOT/.env"
fi

export API_DATA_ROOT="${API_DATA_ROOT:-$REPO_ROOT/ai-flow-runtime/data}"
export API_HOST="${API_HOST:-127.0.0.1}"
export API_PORT="${API_PORT}"

port_in_use() {
  local port="$1"
  if command -v ss >/dev/null 2>&1; then
    ss -ltn | grep -q ":${port} "
  else
    nc -z 127.0.0.1 "$port" 2>/dev/null
  fi
}

port_listener_pids() {
  local port="$1"
  if command -v ss >/dev/null 2>&1; then
    ss -ltnp "sport = :${port}" 2>/dev/null \
      | grep -oP 'pid=\K[0-9]+' \
      | sort -u
  elif command -v fuser >/dev/null 2>&1; then
    fuser -n tcp "$port" 2>/dev/null || true
  fi
}

stop_port_listeners() {
  local port="$1"
  local pids
  pids="$(port_listener_pids "$port" || true)"
  if [[ -z "$pids" ]]; then
    return 0
  fi
  echo "Stopping process(es) on port ${port}: $pids"
  # shellcheck disable=SC2086
  kill $pids 2>/dev/null || true
  sleep 0.5
  # shellcheck disable=SC2086
  kill -9 $pids 2>/dev/null || true
}

resolve_llm_mode() {
  if [[ "${API_USE_GEMINI:-}" == "false" ]]; then
    echo "mock"
    return
  fi
  if [[ -n "${GEMINI_API_KEY:-}" ]]; then
    echo "gemini"
    return
  fi
  if [[ "${API_USE_GEMINI:-}" == "true" ]]; then
    echo "gemini"
    return
  fi
  echo "mock"
}

LLM_MODE="$(resolve_llm_mode)"
CARGO_FEATURES=()
if [[ "$LLM_MODE" == "gemini" ]]; then
  if [[ -z "${GEMINI_API_KEY:-}" ]]; then
    echo "ERROR: API_USE_GEMINI=true but GEMINI_API_KEY is not set."
    exit 1
  fi
  CARGO_FEATURES=(--features gemini)
  echo "LLM mode: gemini"
else
  echo "LLM mode: mock"
fi

if port_in_use "$API_PORT"; then
  if [[ "$API_RESTART" == "true" ]]; then
    echo "Port $API_PORT is in use — restarting ai-api..."
    stop_port_listeners "$API_PORT"
  else
    echo "Port $API_PORT already in use — reusing existing server."
  fi
fi

if ! port_in_use "$API_PORT"; then
  echo "Starting ai-api on http://${API_HOST}:${API_PORT} ..."
  (
    cd "$REPO_ROOT"
    cargo run -p ai-api "${CARGO_FEATURES[@]}"
  ) &
  API_PID=$!
  API_STARTED_BY_SCRIPT=true

  ready=false
  for _ in $(seq 1 120); do
    if curl -sf "http://127.0.0.1:${API_PORT}/health" >/dev/null 2>&1; then
      echo "ai-api is ready."
      ready=true
      break
    fi
    if ! kill -0 "$API_PID" 2>/dev/null; then
      echo "ERROR: ai-api exited during startup."
      exit 1
    fi
    sleep 0.5
  done
  if [[ "$ready" != true ]]; then
    echo "ERROR: ai-api did not become healthy within 60s."
    exit 1
  fi
fi

echo "Smoke: GET /studio/projects"
curl -sf "http://127.0.0.1:${API_PORT}/studio/projects" >/dev/null
echo "Smoke checks passed."

cd "$ROOT"
echo "Running npm install..."
npm install

echo ""
echo "========================================"
echo " LLM:        ${LLM_MODE}"
echo " API:        http://127.0.0.1:${API_PORT}"
echo " Platform:   http://127.0.0.1:${WEB_PORT}"
echo "========================================"
echo ""

npm run dev -- --host 127.0.0.1 --port "${WEB_PORT}"
