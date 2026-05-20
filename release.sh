#!/usr/bin/env bash
# Bump version, commit, tag, and push to trigger the GitHub Actions release workflow.
#
# Usage:
#   ./release.sh 1.0.2              # version (tag will be v1.0.2)
#   ./release.sh v1.0.2             # same
#   ./release.sh --dry-run 1.0.2    # preview only
#   ./release.sh -y 1.0.2           # skip confirmation prompt
#
# Prerequisites:
#   - On branch master, working tree clean, up to date with origin/master
#   - Push access to origin; tag push triggers .github/workflows/release.yml
#   - GitHub Actions release job runs only when github.actor is Vibes4

set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { printf "${CYAN}info${NC}  %s\n" "$1"; }
ok()    { printf "${GREEN}  ok${NC}  %s\n" "$1"; }
warn()  { printf "${YELLOW}warn${NC}  %s\n" "$1" >&2; }
err()   { printf "${RED}error${NC} %s\n" "$1" >&2; exit 1; }

DRY_RUN=0
ASSUME_YES=0
NO_PUSH=0
VERSION=""

usage() {
    sed -n '2,14p' "$0" | sed 's/^# \{0,1\}//'
    exit "${1:-0}"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help) usage 0 ;;
        -n|--dry-run) DRY_RUN=1; shift ;;
        -y|--yes) ASSUME_YES=1; shift ;;
        --no-push) NO_PUSH=1; shift ;;
        -*) err "unknown option: $1 (try --help)" ;;
        *)
            if [[ -n "$VERSION" ]]; then
                err "unexpected argument: $1"
            fi
            VERSION="$1"
            shift
            ;;
    esac
done

[[ -n "$VERSION" ]] || usage 1

# Normalize: v1.2.3 -> 1.2.3 for files; tag always vX.Y.Z
VERSION="${VERSION#v}"
TAG="v${VERSION}"

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
    err "version must look like MAJOR.MINOR.PATCH (e.g. 1.0.2), got: $VERSION"
fi

CARGO_TOML="src-tauri/Cargo.toml"
TAURI_CONF="src-tauri/tauri.conf.json"
CARGO_LOCK="src-tauri/Cargo.lock"

for f in "$CARGO_TOML" "$TAURI_CONF"; do
    [[ -f "$f" ]] || err "missing $f"
done

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    err "not a git repository"
fi

BRANCH="$(git branch --show-current)"
if [[ "$BRANCH" != "master" ]]; then
    err "must be on branch master (current: $BRANCH)"
fi

if [[ -n "$(git status --porcelain)" ]]; then
    err "working tree is not clean; commit or stash changes first"
fi

CURRENT="$(grep -E '^version = ' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')"
info "current version: $CURRENT"
info "new version:     $VERSION (tag: $TAG)"

if git rev-parse "$TAG" >/dev/null 2>&1; then
    err "tag $TAG already exists locally"
fi

if git ls-remote --exit-code --tags origin "refs/tags/$TAG" >/dev/null 2>&1; then
    err "tag $TAG already exists on origin"
fi

info "fetching origin/master..."
git fetch origin master

LOCAL="$(git rev-parse HEAD)"
REMOTE="$(git rev-parse origin/master)"
if [[ "$LOCAL" != "$REMOTE" ]]; then
    if git merge-base --is-ancestor "$LOCAL" "$REMOTE" 2>/dev/null; then
        err "master is behind origin/master — run: git pull origin master"
    elif git merge-base --is-ancestor "$REMOTE" "$LOCAL" 2>/dev/null; then
        info "master is ahead of origin/master (release commit will be pushed)"
    else
        err "master has diverged from origin/master — reconcile before releasing"
    fi
fi

echo ""
printf "${BOLD}Release plan${NC}\n"
echo "  1. Set version to $VERSION in $CARGO_TOML and $TAURI_CONF"
echo "  2. cargo check (update $CARGO_LOCK)"
echo "  3. git commit: Release $TAG."
echo "  4. git tag -a $TAG"
if [[ "$NO_PUSH" -eq 0 ]]; then
    echo "  5. git push origin master && git push origin $TAG"
else
    echo "  5. (skipped) --no-push: commit and tag stay local"
fi
echo ""
echo "  Then GitHub Actions builds Linux / macOS / Windows and publishes:"
echo "  https://github.com/Vibes4/vibes-copy-manager/releases/tag/$TAG"
echo ""
warn "Release workflow only creates the GitHub Release when the push is by GitHub user Vibes4."
echo ""

if [[ "$DRY_RUN" -eq 1 ]]; then
    info "dry-run: no files or git state changed"
    exit 0
fi

if [[ "$ASSUME_YES" -eq 0 ]]; then
    read -r -p "Continue? [y/N] " ans
    case "$ans" in
        y|Y|yes|YES) ;;
        *) info "aborted"; exit 0 ;;
    esac
fi

update_version_in_file() {
    local file="$1"
    local pattern="$2"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        sed -i '' -E "$pattern" "$file"
    else
        sed -i -E "$pattern" "$file"
    fi
}

info "updating version in project files..."
update_version_in_file "$CARGO_TOML" "s/^version = \".*\"/version = \"${VERSION}\"/"
update_version_in_file "$TAURI_CONF" "s/\"version\": \"[^\"]+\"/\"version\": \"${VERSION}\"/"

info "running cargo check..."
(cd src-tauri && cargo check --features gui --quiet)

git add "$CARGO_TOML" "$TAURI_CONF" "$CARGO_LOCK"
git commit -m "$(cat <<EOF
Release ${TAG}.

EOF
)"

git tag -a "$TAG" -m "$(cat <<EOF
${TAG}
EOF
)"
ok "created commit and tag $TAG"

if [[ "$NO_PUSH" -eq 0 ]]; then
    info "pushing master and $TAG..."
    git push origin master
    git push origin "$TAG"
    ok "pushed — watch CI: https://github.com/Vibes4/vibes-copy-manager/actions"
    ok "release (when ready): https://github.com/Vibes4/vibes-copy-manager/releases/tag/$TAG"
else
    warn "not pushed (--no-push). When ready:"
    echo "  git push origin master"
    echo "  git push origin $TAG"
fi
