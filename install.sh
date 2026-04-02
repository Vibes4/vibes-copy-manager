#!/bin/sh
set -eu

REPO="vibes4/vibes-copy-manager"
INSTALL_DIR="${HOME}/.local/bin"

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

detect_os() {
    _os="$(uname -s)"
    case "$_os" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "macos" ;;
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
            echo "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
            ;;
        fish)
            echo "    fish_add_path ~/.local/bin"
            ;;
        *)
            echo "    echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
            ;;
    esac
}

install_binary() {
    _name="$1"
    _url="$2"
    _dest="${INSTALL_DIR}/${_name}"

    _tmpfile="$(mktemp)"
    if download "$_url" "$_tmpfile" 2>/dev/null; then
        mv "$_tmpfile" "$_dest"
        chmod +x "$_dest"
        ok "Installed ${_name} to ${_dest}"
        return 0
    else
        rm -f "$_tmpfile"
        return 1
    fi
}

main() {
    printf "\n"
    printf "  ${BOLD}Vibes Copy Manager${NC} — Installer\n"
    printf "\n"

    _os="$(detect_os)"
    info "Detected OS: ${_os}"

    mkdir -p "$INSTALL_DIR"

    _base="https://github.com/${REPO}/releases/latest/download"
    _installed=0

    # ── Install GUI (AppImage or deb on Linux, binary on macOS) ──
    case "$_os" in
        linux)
            printf "\n"
            printf "  Select GUI package format:\n"
            printf "    ${BOLD}1${NC}) AppImage  (portable, no root needed)\n"
            printf "    ${BOLD}2${NC}) .deb      (system install, requires sudo)\n"
            printf "\n"
            printf "  Choice [1]: "
            _choice="1"
            if read _input </dev/tty 2>/dev/null; then
                case "$_input" in
                    2) _choice="2" ;;
                    *) _choice="1" ;;
                esac
            fi

            if [ "$_choice" = "2" ]; then
                info "Installing GUI (.deb package)..."
                _tmpfile="$(mktemp --suffix=.deb)"
                if download "${_base}/vcm-linux.deb" "$_tmpfile" 2>/dev/null; then
                    info "Running: sudo dpkg -i (may prompt for password)"
                    if sudo dpkg -i "$_tmpfile" </dev/tty; then
                        ok "Installed .deb package"
                        _installed=1
                    else
                        warn ".deb install failed. Falling back to AppImage..."
                        rm -f "$_tmpfile"
                        if install_binary "vcm-gui" "${_base}/vcm-linux.AppImage"; then
                            _installed=1
                        fi
                    fi
                    rm -f "$_tmpfile"
                else
                    rm -f "$_tmpfile"
                    warn "GUI .deb not available. Download manually from:"
                    echo "      https://github.com/${REPO}/releases/latest"
                fi
            else
                info "Installing GUI (AppImage)..."
                if install_binary "vcm-gui" "${_base}/vcm-linux.AppImage"; then
                    _installed=1
                else
                    warn "GUI AppImage not available. Download manually from:"
                    echo "      https://github.com/${REPO}/releases/latest"
                fi
            fi
            ;;
        macos)
            info "Installing GUI..."
            if install_binary "vcm-gui" "${_base}/vcm-macos"; then
                _installed=1
            else
                warn "GUI binary not available. Download the .dmg from:"
                echo "      https://github.com/${REPO}/releases/latest"
            fi
            ;;
    esac

    # ── Install CLI ──
    info "Installing CLI..."
    if install_binary "vcm" "${_base}/vcm-${_os}"; then
        _installed=1
    else
        if [ "$_installed" -eq 0 ]; then
            warn "Pre-built binaries not available."
            info "Falling back to build from source..."
            build_from_source
        else
            warn "CLI binary not available. Build from source with:"
            echo "      cd src-tauri && cargo build --release --bin vcm --no-default-features"
        fi
    fi

    check_path

    echo ""
    printf "  ${GREEN}${BOLD}Installation complete!${NC}\n"
    echo ""
    echo "  Get started:"
    echo "    vcm               Open GUI (clipboard popup)"
    echo "    vcm --help        Show all CLI commands"
    echo "    vcm list          List clipboard history"
    echo "    vcm push \"text\"   Add text to history"
    echo "    vcm settings      View configuration"
    echo ""
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

    cp "$_tmpdir/vcm/src-tauri/target/release/vcm" "${INSTALL_DIR}/vcm"
    chmod +x "${INSTALL_DIR}/vcm"
    ok "Built and installed vcm to ${INSTALL_DIR}/vcm"
}

main "$@"
