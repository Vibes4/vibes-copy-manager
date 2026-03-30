#!/usr/bin/env bash
set -euo pipefail

REPO="vaibhav/vibes-copy-manager"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="vcm"

# ── Detect OS and architecture ────────────────────────────────

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        *)
            echo "Error: Unsupported OS: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            echo "Error: Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# ── Main ──────────────────────────────────────────────────────

main() {
    echo "╔══════════════════════════════════════╗"
    echo "║  Vibes Copy Manager — Installer      ║"
    echo "╚══════════════════════════════════════╝"
    echo ""

    local platform
    platform="$(detect_platform)"
    echo "Detected platform: ${platform}"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Check if we have a pre-built binary URL (GitHub Releases)
    local download_url="https://github.com/${REPO}/releases/latest/download/vcm-${platform}"

    echo "Downloading from: ${download_url}"
    echo ""

    if command -v curl &>/dev/null; then
        if curl -fSL --progress-bar -o "${INSTALL_DIR}/${BINARY_NAME}" "$download_url" 2>/dev/null; then
            chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
            echo ""
            echo "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
        else
            echo ""
            echo "Download failed. Falling back to build from source..."
            build_from_source
        fi
    elif command -v wget &>/dev/null; then
        if wget -q --show-progress -O "${INSTALL_DIR}/${BINARY_NAME}" "$download_url" 2>/dev/null; then
            chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
            echo ""
            echo "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
        else
            echo ""
            echo "Download failed. Falling back to build from source..."
            build_from_source
        fi
    else
        echo "Neither curl nor wget found. Building from source..."
        build_from_source
    fi

    # Verify PATH
    check_path

    echo ""
    echo "Installation complete! Run 'vcm --help' to get started."
}

build_from_source() {
    if ! command -v cargo &>/dev/null; then
        echo "Error: Rust is required to build from source."
        echo "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi

    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    echo "Cloning repository..."
    git clone --depth 1 "https://github.com/${REPO}.git" "$tmpdir/vcm" 2>/dev/null || {
        echo "Error: Could not clone repository."
        echo "You can build manually:"
        echo "  cd src-tauri && cargo build --release --bin vcm --no-default-features"
        exit 1
    }

    echo "Building vcm CLI..."
    cd "$tmpdir/vcm/src-tauri"
    cargo build --release --bin vcm --no-default-features

    cp "target/release/vcm" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

    echo "Built and installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

check_path() {
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*)
            return
            ;;
    esac

    echo ""
    echo "NOTE: ${INSTALL_DIR} is not in your PATH."
    echo ""
    echo "Add it by appending to your shell config:"
    echo ""

    local shell_name
    shell_name="$(basename "${SHELL:-/bin/bash}")"

    case "$shell_name" in
        zsh)
            echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
            echo "  source ~/.zshrc"
            ;;
        fish)
            echo "  fish_add_path ~/.local/bin"
            ;;
        *)
            echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo "  source ~/.bashrc"
            ;;
    esac
}

main "$@"
