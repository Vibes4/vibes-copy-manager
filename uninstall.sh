#!/bin/sh
set -eu

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

INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/vcm"
AUTOSTART_FILE="${HOME}/.config/autostart/vibes-copy-manager.desktop"

FILES_TO_REMOVE="
${INSTALL_DIR}/vcm
${INSTALL_DIR}/vcm-gui
"

main() {
    printf "\n"
    printf "  ${BOLD}Vibes Copy Manager${NC} — Uninstaller\n"
    printf "\n"

    printf "  This will remove:\n"
    printf "    - CLI binary:     ${INSTALL_DIR}/vcm\n"
    printf "    - GUI binary:     ${INSTALL_DIR}/vcm-gui\n"
    printf "    - Config:         ${CONFIG_DIR}/\n"
    printf "    - Autostart:      ${AUTOSTART_FILE}\n"
    printf "\n"
    printf "  Continue? [y/N]: "

    _confirm="n"
    if read _input </dev/tty 2>/dev/null; then
        case "$_input" in
            y|Y|yes|YES) _confirm="y" ;;
            *) _confirm="n" ;;
        esac
    fi

    if [ "$_confirm" != "y" ]; then
        info "Uninstall cancelled."
        exit 0
    fi

    printf "\n"

    for _file in ${FILES_TO_REMOVE}; do
        _file="$(echo "$_file" | tr -d '[:space:]')"
        [ -z "$_file" ] && continue
        if [ -f "$_file" ]; then
            rm -f "$_file"
            ok "Removed $_file"
        else
            info "Not found: $_file (skipped)"
        fi
    done

    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        ok "Removed config directory: ${CONFIG_DIR}"
    else
        info "Config directory not found (skipped)"
    fi

    if [ -f "$AUTOSTART_FILE" ]; then
        rm -f "$AUTOSTART_FILE"
        ok "Removed autostart entry"
    fi

    _deb_installed=""
    if command -v dpkg >/dev/null 2>&1; then
        if dpkg -l "vibes-copy-manager" >/dev/null 2>&1; then
            _deb_installed="vibes-copy-manager"
        fi
    fi

    if [ -n "$_deb_installed" ]; then
        printf "\n"
        info "Detected system .deb package: ${_deb_installed}"
        printf "  Remove it with sudo? [y/N]: "
        _remove_deb="n"
        if read _input </dev/tty 2>/dev/null; then
            case "$_input" in
                y|Y|yes|YES) _remove_deb="y" ;;
            esac
        fi
        if [ "$_remove_deb" = "y" ]; then
            if sudo dpkg --purge "$_deb_installed" </dev/tty; then
                ok "Removed .deb package"
            else
                warn "Failed to remove .deb package. Try manually: sudo dpkg --purge ${_deb_installed}"
            fi
        fi
    fi

    printf "\n"
    printf "  ${GREEN}${BOLD}Uninstall complete.${NC}\n"
    printf "\n"
}

main "$@"
