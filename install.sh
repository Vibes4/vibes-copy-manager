#!/bin/sh
set -eu

REPO="vibes4/vibes-copy-manager"
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
warn()  { printf "${YELLOW}warn${NC}  %s\n" "$1" >&2; }
err()   { printf "${RED}error${NC} %s\n" "$1" >&2; }

detect_platform() {
    _os="$(uname -s)"
    _arch="$(uname -m)"

    case "$_os" in
        Linux*)  _os="linux" ;;
        Darwin*) _os="macos" ;;
        CYGWIN*|MINGW*|MSYS*)
            err "Windows is not supported via this installer."
            echo ""
            echo "Download the .msi installer from:"
            echo "  https://github.com/${REPO}/releases/latest"
            exit 1
            ;;
        *)
            err "Unsupported OS: $_os"
            exit 1
            ;;
    esac

    case "$_arch" in
        x86_64|amd64)   _arch="x86_64" ;;
        aarch64|arm64)   _arch="aarch64" ;;
        *)
            err "Unsupported architecture: $_arch"
            exit 1
            ;;
    esac

    echo "${_os}-${_arch}"
}

download() {
    _url="$1"
    _dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fSL --progress-bar -o "$_dest" "$_url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q --show-progress -O "$_dest" "$_url"
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

    _shell_name="$(basename "${SHELL:-/bin/bash}")"

    case "$_shell_name" in
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

build_from_source() {
    if ! command -v cargo >/dev/null 2>&1; then
        err "Rust is required to build from source."
        echo ""
        echo "  Install Rust:"
        echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    if ! command -v git >/dev/null 2>&1; then
        err "git is required to build from source."
        exit 1
    fi

    _tmpdir="$(mktemp -d)"
    trap 'rm -rf "$_tmpdir"' EXIT

    info "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$_tmpdir/vcm" 2>/dev/null || {
        err "Could not clone repository."
        exit 1
    }

    info "Building vcm CLI (this may take a minute)..."
    (cd "$_tmpdir/vcm/src-tauri" && cargo build --release --bin vcm --no-default-features) || {
        err "Build failed."
        exit 1
    }

    cp "$_tmpdir/vcm/src-tauri/target/release/vcm" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    ok "Built and installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

main() {
    printf "\n"
    printf "  ${BOLD}Vibes Copy Manager${NC} — Installer\n"
    printf "\n"

    _platform="$(detect_platform)"
    info "Detected platform: ${_platform}"

    mkdir -p "$INSTALL_DIR"

    _base_url="https://github.com/${REPO}/releases/latest/download"
    _download_url="${_base_url}/vcm-${_platform}"

    info "Downloading from GitHub Releases..."

    _tmpfile="$(mktemp)"
    trap 'rm -f "$_tmpfile"' EXIT

    if download "$_download_url" "$_tmpfile" 2>/dev/null; then
        mv "$_tmpfile" "${INSTALL_DIR}/${BINARY_NAME}"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
        ok "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
    else
        warn "Pre-built binary not available for ${_platform}."
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

main "$@"
