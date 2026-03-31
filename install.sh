#!/usr/bin/env bash
set -euo pipefail

REPO="vaibhav/vibes-copy-manager"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="vcm"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { printf "${CYAN}info${NC}  %s\n" "$1"; }
ok()    { printf "${GREEN}  ok${NC}  %s\n" "$1"; }
warn()  { printf "${YELLOW}warn${NC}  %s\n" "$1"; }
err()   { printf "${RED}error${NC} %s\n" "$1" >&2; }

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        CYGWIN*|MINGW*|MSYS*)
            err "Windows is not supported via this installer."
            echo ""
            echo "Download the .msi installer from:"
            echo "  https://github.com/${REPO}/releases/latest"
            exit 1
            ;;
        *)
            err "Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)   arch="aarch64" ;;
        *)
            err "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

download() {
    local url="$1" dest="$2"
    if command -v curl &>/dev/null; then
        curl -fSL --progress-bar -o "$dest" "$url"
    elif command -v wget &>/dev/null; then
        wget -q --show-progress -O "$dest" "$url"
    else
        err "Neither curl nor wget found. Install one and try again."
        exit 1
    fi
}

check_path() {
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) return ;;
    esac

    echo ""
    warn "${INSTALL_DIR} is not in your PATH."
    echo ""
    echo "  Add it by running:"
    echo ""

    local shell_name
    shell_name="$(basename "${SHELL:-/bin/bash}")"

    case "$shell_name" in
        zsh)
            echo "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
            echo "    source ~/.zshrc"
            ;;
        fish)
            echo "    fish_add_path ~/.local/bin"
            ;;
        *)
            echo "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo "    source ~/.bashrc"
            ;;
    esac
}

main() {
    printf "\n"
    printf "  ${BOLD}Vibes Copy Manager${NC} — Installer\n"
    printf "\n"

    local platform
    platform="$(detect_platform)"
    info "Detected platform: ${platform}"

    mkdir -p "$INSTALL_DIR"

    local base_url="https://github.com/${REPO}/releases/latest/download"
    local download_url

    case "$platform" in
        linux-*)
            download_url="${base_url}/vcm-${platform}"
            ;;
        macos-*)
            download_url="${base_url}/vcm-${platform}"
            ;;
    esac

    info "Downloading from GitHub Releases..."

    local tmpfile
    tmpfile="$(mktemp)"
    trap 'rm -f "$tmpfile"' EXIT

    if download "$download_url" "$tmpfile" 2>/dev/null; then
        mv "$tmpfile" "${INSTALL_DIR}/${BINARY_NAME}"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
        ok "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
    else
        warn "Pre-built binary not available for ${platform}."
        info "Falling back to build from source..."
        build_from_source
    fi

    check_path

    echo ""
    printf "  ${GREEN}${BOLD}Installation complete!${NC}\n"
    echo ""
    echo "  Get started:"
    echo "    vcm --help        Show all commands"
    echo "    vcm list          List clipboard history"
    echo "    vcm push \"text\"   Add text to history"
    echo "    vcm settings      View configuration"
    echo ""
    echo "  GUI app (if installed separately):"
    echo "    Download from https://github.com/${REPO}/releases/latest"
    echo ""
}

build_from_source() {
    if ! command -v cargo &>/dev/null; then
        err "Rust is required to build from source."
        echo ""
        echo "  Install Rust:"
        echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    if ! command -v git &>/dev/null; then
        err "git is required to build from source."
        exit 1
    fi

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$tmpdir/vcm" 2>/dev/null || {
        err "Could not clone repository."
        exit 1
    }

    info "Building vcm CLI (this may take a minute)..."
    (cd "$tmpdir/vcm/src-tauri" && cargo build --release --bin vcm --no-default-features) || {
        err "Build failed."
        exit 1
    }

    cp "$tmpdir/vcm/src-tauri/target/release/vcm" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    ok "Built and installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

main "$@"
