#!/usr/bin/env bash
set -e

CYAN="\033[36m"
GREEN="\033[32m"
MAGENTA="\033[35m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo -e "${RED}ERROR: '$1' is not installed or not in PATH.${RESET}"
        return 1
    fi
    return 0
}

echo -e "${CYAN}Checking for rustup...${RESET}"
check_command rustup || {
    echo -e "${RED}Install Rust from https://rustup.rs/ and try again.${RESET}"
    exit 1
}

echo -e "${CYAN}Checking for cargo...${RESET}"
check_command cargo || {
    echo -e "${RED}Install Rust from https://rustup.rs/ and try again.${RESET}"
    exit 1
}

echo -e "${CYAN}Adding wasm32 target...${RESET}"
if ! rustup target add wasm32-unknown-unknown; then
    echo -e "${RED}ERROR: Failed to add wasm32 target.${RESET}"
    exit 1
fi

echo -e "${CYAN}Checking for wasm-pack...${RESET}"
if ! command -v wasm-pack >/dev/null 2>&1; then
    echo -e "${YELLOW}wasm-pack not found. Installing...${RESET}"
    if ! cargo install wasm-pack; then
        echo -e "${RED}ERROR: Failed to install wasm-pack.${RESET}"
        exit 1
    fi
else
    echo -e "${GREEN}wasm-pack already installed.${RESET}"
fi

echo -e "${CYAN}Building WASM package...${RESET}"
if ! wasm-pack build --target web --out-dir pkg --release; then
    echo -e "${RED}ERROR: wasm-pack build failed.${RESET}"
    exit 1
fi

echo
echo -e "${GREEN}Build complete.${RESET}"
echo -e "${GREEN}Output directory: ./pkg${RESET}"
echo

echo -e "${MAGENTA}To publish manually:${RESET}"
echo -e "${MAGENTA}  npm login${RESET}"
echo -e "${MAGENTA}  cd pkg${RESET}"
echo -e "${MAGENTA}  npm publish --access public --no-git-checks${RESET}"
echo
