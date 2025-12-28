#!/usr/bin/env bash
#
# Bootstrap script for initializing a project from this template.
# Use this if you don't have cargo-generate or prefer a simple shell-based setup.
#
# Usage: ./bootstrap.sh
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Project Initialization${NC}"
echo ""

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust is not installed. Please install Rust: https://rustup.rs${NC}"
    exit 1
fi

echo -e "${YELLOW}Building and launching project initialization tool...${NC}"

# Run the project-init tool
# We use --quiet to reduce cargo output noise, but the tool's output will still show
if ! cargo run --quiet --manifest-path tools/project-init/Cargo.toml; then
    echo -e "${RED}Project initialization failed or was aborted.${NC}"
    exit 1
fi

# Ask about removing this script
read -p "Remove this bootstrap script? (y/n) [y]: " REMOVE_BOOTSTRAP
REMOVE_BOOTSTRAP=${REMOVE_BOOTSTRAP:-y}

if [[ "$REMOVE_BOOTSTRAP" == "y" || "$REMOVE_BOOTSTRAP" == "Y" ]]; then
    rm -f bootstrap.sh
fi

echo ""
echo -e "${GREEN}✅ Project initialized successfully!${NC}"
echo ""
echo "Next steps:"
echo "  1. Review the changes: git diff"
echo "  2. Install dependencies: bun install"
echo "  3. Build the project: cargo build"
echo "  4. Run the dev server: cargo run"
echo ""
