#!/usr/bin/env bash

set -euo pipefail

REPO_OWNER="${REPO_OWNER:-clionertr}"
REPO_NAME="${REPO_NAME:-Pebble}"
PEBBLE_REF="${PEBBLE_REF:-master}"
PEBBLE_VERSION="${PEBBLE_VERSION:-edge}"
PEBBLE_INSTALL_DIR="${PEBBLE_INSTALL_DIR:-./pebble}"
PEBBLE_HTTP_BIND="${PEBBLE_HTTP_BIND:-127.0.0.1:9191}"

IMAGE_OWNER="${REPO_OWNER,,}"
PEBBLE_BACKEND_IMAGE="${PEBBLE_BACKEND_IMAGE:-ghcr.io/${IMAGE_OWNER}/pebble:${PEBBLE_VERSION}}"
PEBBLE_FRONTEND_IMAGE="${PEBBLE_FRONTEND_IMAGE:-ghcr.io/${IMAGE_OWNER}/pebble-frontend:${PEBBLE_VERSION}}"
PEBBLE_RAW_BASE="${PEBBLE_RAW_BASE:-https://raw.githubusercontent.com/${REPO_OWNER}/${REPO_NAME}/${PEBBLE_REF}}"

log() {
  printf "\033[1;34m==>\033[0m %s\n" "$*" >&2
}

warn() {
  printf "\033[1;33mWarning:\033[0m %s\n" "$*" >&2
}

die() {
  printf "\033[1;31mError:\033[0m %s\n" "$*" >&2
  exit 1
}

has_tty() {
  [[ -t 0 || -t 1 || -t 2 ]] && [[ -r /dev/tty && -w /dev/tty ]]
}

prompt() {
  local message="$1"
  local default="${2:-}"
  local answer=""

  if has_tty; then
    if [[ -n "$default" ]]; then
      printf "%s [%s]: " "$message" "$default" > /dev/tty
    else
      printf "%s: " "$message" > /dev/tty
    fi
    IFS= read -r answer < /dev/tty || answer=""
  elif [[ -z "$default" ]]; then
    die "Interactive input is required for: ${message}. Set the matching environment variable and retry."
  fi

  printf "%s" "${answer:-$default}"
}

confirm() {
  local message="$1"
  local default="${2:-n}"
  local answer=""

  while true; do
    answer="$(prompt "${message} (y/n)" "$default")"
    case "${answer,,}" in
      y|yes) return 0 ;;
      n|no) return 1 ;;
      *) warn "Please answer y or n." ;;
    esac
  done
}

prompt_required() {
  local message="$1"
  local default="${2:-}"
  local value=""

  while true; do
    value="$(prompt "$message" "$default")"
    if [[ -n "$value" ]]; then
      printf "%s" "$value"
      return 0
    fi
    warn "This value cannot be empty."
  done
}

prompt_password_twice() {
  local password=""
  local repeated=""

  has_tty || die "A terminal is required to enter the login password. Set PEBBLE_PASSWORD or PEBBLE_RANDOM_PASSWORD=1 for non-interactive deployment."

  while true; do
    printf "Pebble login password: " > /dev/tty
    IFS= read -rs password < /dev/tty || password=""
    printf "\n" > /dev/tty

    printf "Repeat password: " > /dev/tty
    IFS= read -rs repeated < /dev/tty || repeated=""
    printf "\n" > /dev/tty

    if [[ -z "$password" ]]; then
      warn "Password cannot be empty."
    elif [[ "$password" != "$repeated" ]]; then
      warn "Passwords do not match."
    else
      printf "%s" "$password"
      return 0
    fi
  done
}

random_password() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -base64 24 | tr -d "\n"
  elif command -v od >/dev/null 2>&1; then
    od -An -tx1 -N24 /dev/urandom | tr -d " \n"
  else
    die "Cannot generate a random password because neither openssl nor od is available."
  fi
}

compose_escape() {
  sed 's/\$/$$/g'
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || die "$1 is required but was not found."
}

check_docker() {
  require_command docker
  docker compose version >/dev/null 2>&1 || die "Docker Compose v2 is required. Please install/enable 'docker compose'."
  docker info >/dev/null 2>&1 || die "Docker is installed, but the daemon is not reachable by the current user."
}

env_value() {
  local key="$1"
  [[ -f "$ENV_FILE" ]] || return 0
  sed -n -E "s/^${key}=//p" "$ENV_FILE" | tail -n 1
}

normalize_public_url() {
  local value="$1"
  value="${value%/}"
  case "$value" in
    http://*|https://*) printf "%s" "$value" ;;
    *) return 1 ;;
  esac
}

read_public_url() {
  local existing="$1"
  local value="${PEBBLE_PUBLIC_URL:-}"

  while true; do
    if [[ -z "$value" ]]; then
      value="$(prompt "Public URL for Pebble, for example https://mail.closev.com" "$existing")"
    fi

    if normalized="$(normalize_public_url "$value")"; then
      printf "%s" "$normalized"
      return 0
    fi

    if [[ -n "${PEBBLE_PUBLIC_URL:-}" ]]; then
      die "PEBBLE_PUBLIC_URL must start with http:// or https://"
    fi
    warn "The public URL must start with http:// or https://."
    value=""
  done
}

fetch_compose_template() {
  if [[ -n "${PEBBLE_COMPOSE_TEMPLATE:-}" ]]; then
    [[ -f "$PEBBLE_COMPOSE_TEMPLATE" ]] || die "PEBBLE_COMPOSE_TEMPLATE does not exist: $PEBBLE_COMPOSE_TEMPLATE"
    cp "$PEBBLE_COMPOSE_TEMPLATE" "$COMPOSE_FILE"
    return 0
  fi

  require_command curl
  curl -fsSL "${PEBBLE_RAW_BASE}/deploy/compose.prod.yml" -o "$COMPOSE_FILE" \
    || die "Failed to download deploy/compose.prod.yml from GitHub."
}

resolve_password() {
  if [[ -n "${PEBBLE_PASSWORD:-}" ]]; then
    printf "%s" "$PEBBLE_PASSWORD"
  elif [[ "${PEBBLE_RANDOM_PASSWORD:-}" == "1" ]]; then
    GENERATED_PASSWORD="$(random_password)"
    printf "%s" "$GENERATED_PASSWORD"
  else
    prompt_password_twice
  fi
}

generate_password_hash() {
  local password="$1"
  local hash=""

  log "Pulling backend image for password hashing: ${PEBBLE_BACKEND_IMAGE}"
  docker pull "$PEBBLE_BACKEND_IMAGE" >/dev/null \
    || die "Cannot pull ${PEBBLE_BACKEND_IMAGE}. If this is a GHCR image, check that the GitHub package is public."

  hash="$(printf "%s" "$password" | docker run --rm -i "$PEBBLE_BACKEND_IMAGE" pebble hash-password)" \
    || die "Failed to generate bcrypt password hash with the backend image."

  case "$hash" in
    \$2a\$*|\$2b\$*|\$2x\$*|\$2y\$*) printf "%s" "$hash" | compose_escape ;;
    *) die "Generated password hash does not look like bcrypt output." ;;
  esac
}

configure_oauth() {
  GOOGLE_CLIENT_ID="${GOOGLE_CLIENT_ID:-$(env_value GOOGLE_CLIENT_ID)}"
  GOOGLE_CLIENT_SECRET="${GOOGLE_CLIENT_SECRET:-$(env_value GOOGLE_CLIENT_SECRET)}"
  MICROSOFT_CLIENT_ID="${MICROSOFT_CLIENT_ID:-$(env_value MICROSOFT_CLIENT_ID)}"
  MICROSOFT_CLIENT_SECRET="${MICROSOFT_CLIENT_SECRET:-$(env_value MICROSOFT_CLIENT_SECRET)}"

  if confirm "Configure Google OAuth for Gmail now?" "n"; then
    GOOGLE_CLIENT_ID="$(prompt_required "Google Client ID" "$GOOGLE_CLIENT_ID" | compose_escape)"
    GOOGLE_CLIENT_SECRET="$(prompt_required "Google Client Secret" "$GOOGLE_CLIENT_SECRET" | compose_escape)"
  fi

  if confirm "Configure Microsoft OAuth for Outlook now?" "n"; then
    MICROSOFT_CLIENT_ID="$(prompt_required "Microsoft Client ID" "$MICROSOFT_CLIENT_ID" | compose_escape)"
    MICROSOFT_CLIENT_SECRET="$(prompt "Microsoft Client Secret (optional for public/native apps)" "$MICROSOFT_CLIENT_SECRET" | compose_escape)"
  fi
}

write_env_file() {
  cat > "$ENV_FILE" <<EOF
# Pebble Docker deployment
PEBBLE_BACKEND_IMAGE=${PEBBLE_BACKEND_IMAGE}
PEBBLE_FRONTEND_IMAGE=${PEBBLE_FRONTEND_IMAGE}
PEBBLE_HTTP_BIND=${PEBBLE_HTTP_BIND}

# Backend runtime
PEBBLE_PASSWORD_HASH=${PEBBLE_PASSWORD_HASH_VALUE}
PEBBLE_PORT=3000
OAUTH_REDIRECT_URL=${OAUTH_REDIRECT_URL}
ALLOWED_ORIGIN=

# Optional Google OAuth
GOOGLE_CLIENT_ID=${GOOGLE_CLIENT_ID}
GOOGLE_CLIENT_SECRET=${GOOGLE_CLIENT_SECRET}

# Optional Microsoft OAuth
MICROSOFT_CLIENT_ID=${MICROSOFT_CLIENT_ID}
MICROSOFT_CLIENT_SECRET=${MICROSOFT_CLIENT_SECRET}

# Optional Gmail realtime push
GMAIL_PUBSUB_TOPIC=${GMAIL_PUBSUB_TOPIC:-$(env_value GMAIL_PUBSUB_TOPIC)}
GMAIL_WEBHOOK_SECRET=${GMAIL_WEBHOOK_SECRET:-$(env_value GMAIL_WEBHOOK_SECRET)}
EOF
  chmod 600 "$ENV_FILE"
}

compose_cmd() {
  docker compose --project-directory "$INSTALL_DIR" --env-file "$ENV_FILE" -f "$COMPOSE_FILE" "$@"
}

wait_for_http() {
  require_command curl

  local port="${PEBBLE_HTTP_BIND##*:}"
  local health_url="${PEBBLE_HEALTH_URL:-http://127.0.0.1:${port}}"
  local attempts="${PEBBLE_HEALTH_ATTEMPTS:-60}"

  log "Waiting for Pebble at ${health_url}"
  for ((i = 1; i <= attempts; i++)); do
    if curl -fsS -o /dev/null "$health_url"; then
      log "Pebble is reachable: ${health_url}"
      return 0
    fi
    sleep 2
  done

  warn "Pebble did not become reachable at ${health_url}."
  compose_cmd ps || true
  compose_cmd logs --tail=80 backend frontend || true
  return 1
}

main() {
  check_docker

  mkdir -p "$PEBBLE_INSTALL_DIR"
  INSTALL_DIR="$(cd "$PEBBLE_INSTALL_DIR" && pwd)"
  ENV_FILE="${INSTALL_DIR}/.env"
  COMPOSE_FILE="${INSTALL_DIR}/compose.yml"
  GENERATED_PASSWORD=""

  log "Installing Pebble into ${INSTALL_DIR}"
  fetch_compose_template
  mkdir -p "${INSTALL_DIR}/data"

  OAUTH_REDIRECT_URL="$(read_public_url "$(env_value OAUTH_REDIRECT_URL)")"

  local existing_hash
  existing_hash="$(env_value PEBBLE_PASSWORD_HASH)"
  PEBBLE_PASSWORD_HASH_VALUE="$existing_hash"

  if [[ -n "${PEBBLE_PASSWORD:-}" || "${PEBBLE_RANDOM_PASSWORD:-}" == "1" || "${RESET_PASSWORD:-}" == "1" ]]; then
    PEBBLE_PASSWORD_HASH_VALUE="$(generate_password_hash "$(resolve_password)")"
  elif [[ -n "$existing_hash" ]]; then
    if confirm "Existing login password found. Reset it now?" "n"; then
      PEBBLE_PASSWORD_HASH_VALUE="$(generate_password_hash "$(resolve_password)")"
    else
      log "Keeping existing login password."
    fi
  else
    PEBBLE_PASSWORD_HASH_VALUE="$(generate_password_hash "$(resolve_password)")"
  fi

  configure_oauth
  write_env_file

  log "Validating compose configuration"
  compose_cmd config --quiet

  log "Pulling Pebble images"
  compose_cmd pull \
    || die "Failed to pull Pebble images. If GHCR returns denied/not found, set the packages to Public in GitHub Packages."

  log "Starting Pebble"
  compose_cmd up -d
  compose_cmd ps

  wait_for_http

  log "Deployment complete."
  printf "Install dir: %s\n" "$INSTALL_DIR"
  printf "Local URL:   http://127.0.0.1:%s\n" "${PEBBLE_HTTP_BIND##*:}"
  printf "Public URL:  %s\n" "$OAUTH_REDIRECT_URL"
  if [[ -n "$GENERATED_PASSWORD" ]]; then
    printf "Generated login password: %s\n" "$GENERATED_PASSWORD"
    printf "Save this password now; it will not be shown again.\n"
  fi
}

main "$@"
