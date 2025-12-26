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

# Prompt for project name
read -p "Project name (kebab-case) [my-app]: " PROJECT_NAME
PROJECT_NAME=${PROJECT_NAME:-my-app}

# Validate kebab-case
if [[ ! "$PROJECT_NAME" =~ ^[a-z][a-z0-9-]*$ ]]; then
    echo -e "${RED}Error: Project name must be kebab-case (lowercase letters, numbers, hyphens)${NC}"
    exit 1
fi

# Prompt for description
read -p "Description [An agentic AI application]: " PROJECT_DESC
PROJECT_DESC=${PROJECT_DESC:-An agentic AI application}

# Prompt for author
read -p "Author name [Developer]: " AUTHOR_NAME
AUTHOR_NAME=${AUTHOR_NAME:-Developer}

# Prompt for GitHub org
read -p "GitHub organization/username [my-org]: " GITHUB_ORG
GITHUB_ORG=${GITHUB_ORG:-my-org}

# Feature toggles
read -p "Include Tauri desktop support? (y/n) [y]: " ENABLE_TAURI
ENABLE_TAURI=${ENABLE_TAURI:-y}

read -p "Include Docker configuration? (y/n) [y]: " ENABLE_DOCKER
ENABLE_DOCKER=${ENABLE_DOCKER:-y}

read -p "Include SDK scaffolding? (y/n) [y]: " INCLUDE_SDKS
INCLUDE_SDKS=${INCLUDE_SDKS:-y}

# Convert project name to crate name (replace - with _)
CRATE_NAME=$(echo "$PROJECT_NAME" | tr '-' '_')

echo ""
echo -e "${YELLOW}Configuration:${NC}"
echo "  Project Name: $PROJECT_NAME"
echo "  Crate Name:   $CRATE_NAME"
echo "  Description:  $PROJECT_DESC"
echo "  Author:       $AUTHOR_NAME"
echo "  GitHub Org:   $GITHUB_ORG"
echo ""
read -p "Proceed with initialization? (y/n) [y]: " CONFIRM
CONFIRM=${CONFIRM:-y}

if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo -e "${BLUE}🔄 Applying replacements...${NC}"

# Files to process
FILES=(
    "Cargo.toml"
    "package.json"
    "README.md"
    "src-tauri/Cargo.toml"
    "src-tauri/tauri.conf.json"
    "docker-compose.dev.yaml"
    "docker-compose.prod.yaml"
    "docker-compose.test.yaml"
)

# Add SDK files if they exist
if [[ -d "sdks" ]]; then
    FILES+=(
        "sdks/rust/Cargo.toml"
        "sdks/typescript/package.json"
        "sdks/python/pyproject.toml"
    )
fi

for file in "${FILES[@]}"; do
    if [[ -f "$file" ]]; then
        echo "  Processing: $file"
        
        # Use sed with backup (works on both macOS and Linux)
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/axum-leptos-htmx-wc/${PROJECT_NAME}/g" "$file"
            sed -i '' "s/axum_leptos_htmx_wc/${CRATE_NAME}/g" "$file"
            sed -i '' "s/Prometheus-AGS/${GITHUB_ORG}/g" "$file"
        else
            sed -i "s/axum-leptos-htmx-wc/${PROJECT_NAME}/g" "$file"
            sed -i "s/axum_leptos_htmx_wc/${CRATE_NAME}/g" "$file"
            sed -i "s/Prometheus-AGS/${GITHUB_ORG}/g" "$file"
        fi
    fi
done

# Process Rust SDK files if they exist
if [[ -d "sdks/rust/src" ]]; then
    echo "  Processing: sdks/rust/src/*.rs"
    find sdks/rust/src -name "*.rs" -exec sed -i${OSTYPE:+''} "s/axum_leptos_htmx_wc/${CRATE_NAME}/g" {} \;
fi

# Remove Tauri if disabled
if [[ "$ENABLE_TAURI" != "y" && "$ENABLE_TAURI" != "Y" ]]; then
    echo -e "${YELLOW}  Removing Tauri support...${NC}"
    rm -rf src-tauri
fi

# Remove Docker if disabled
if [[ "$ENABLE_DOCKER" != "y" && "$ENABLE_DOCKER" != "Y" ]]; then
    echo -e "${YELLOW}  Removing Docker configuration...${NC}"
    rm -f Dockerfile docker-compose.*.yaml
fi

# Remove SDKs if disabled
if [[ "$INCLUDE_SDKS" != "y" && "$INCLUDE_SDKS" != "Y" ]]; then
    echo -e "${YELLOW}  Removing SDK scaffolding...${NC}"
    rm -rf sdks
fi

echo ""
echo -e "${BLUE}🧹 Cleaning up template files...${NC}"

# Remove template-specific files
rm -f cargo-generate.toml 2>/dev/null || true
rm -f .github/workflows/template-cleanup.yml 2>/dev/null || true

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
