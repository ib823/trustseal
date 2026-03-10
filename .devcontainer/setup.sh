#!/bin/bash
# .devcontainer/setup.sh
# Runs once when the Codespace is created
set -e

echo "==> Setting up Sahi development environment..."

# ── RUST TOOLS ────────────────────────────────────────────────────────
echo "==> Installing Rust toolchain components..."
rustup component add clippy rustfmt
rustup target add wasm32-unknown-unknown   # for WASM bindings
rustup target add aarch64-linux-android    # for Flutter FFI (optional)

cargo install cargo-audit --locked
cargo install cargo-watch --locked
cargo install sqlx-cli --no-default-features --features postgres --locked
cargo install wasm-pack --locked
cargo install cargo-expand --locked
cargo install cargo-flamegraph --locked

echo "==> Rust tools installed."

# ── NODE / PNPM ───────────────────────────────────────────────────────
echo "==> Installing pnpm and Node tools..."
npm install -g pnpm@latest
npm install -g turbo@latest
pnpm --version

echo "==> Node tools installed."

# ── SYSTEM DEPENDENCIES ───────────────────────────────────────────────
echo "==> Installing system dependencies..."
sudo apt-get update -q
sudo apt-get install -y -q \
  libssl-dev \
  pkg-config \
  libpq-dev \
  libsqlite3-dev \
  libudev-dev \
  libusb-1.0-0-dev \
  cmake \
  clang \
  libclang-dev \
  protobuf-compiler \
  jq \
  httpie \
  postgresql-client-16 \
  redis-tools

echo "==> System dependencies installed."

# ── FLUTTER ───────────────────────────────────────────────────────────
echo "==> Configuring Flutter..."
flutter config --no-analytics
flutter doctor --android-licenses 2>/dev/null || true
flutter precache --ios --android 2>/dev/null || true
echo "==> Flutter configured."

# ── ANDROID SDK (for Flutter wallet build) ────────────────────────────
# Skipped in Codespace — mobile builds run on local machine or CI
# Codespace is for backend + web frontend development

# ── ENVIRONMENT FILE ──────────────────────────────────────────────────
echo "==> Setting up .env..."
if [ ! -f .env ]; then
  cp .env.example .env
  echo "  .env created from .env.example — review and fill in secrets"
fi

# ── DATABASE INITIALIZATION ───────────────────────────────────────────
echo "==> Waiting for Postgres to be ready..."
# Docker services start via postStartCommand, not here
# Migrations run via: cargo sqlx migrate run

# ── GIT HOOKS ─────────────────────────────────────────────────────────
echo "==> Installing git hooks from .githooks/..."
git config core.hooksPath .githooks
echo "  Git hooks path set to .githooks/"

echo ""
echo "==> Sahi development environment ready."
echo ""
echo "  Quick start:"
echo "  1. docker-compose up -d          # start Postgres, Redis, MQTT"
echo "  2. cargo sqlx migrate run        # run DB migrations"
echo "  3. /checkpoint                   # check Claude Code state"
echo "  4. /scaffold                     # create monorepo structure (first time)"
echo "  5. /module F1                    # begin Phase 0, Module F1"
echo ""
