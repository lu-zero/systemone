#!/usr/bin/env bash
# release.sh — publish the workspace's published crates to crates.io, then push
# the release tag to GitHub.
#
# Usage: release.sh [--dry-run] [--yes] [--crate NAME]... [--no-push]
#   --dry-run        validate packaging via `cargo publish --dry-run`; no upload
#   --yes, -y        skip the upload confirmation prompt
#   --crate NAME     publish only NAME (repeatable); order stays topological
#   --no-push        publish crates but do not push master/tag to GitHub
#   --help, -h       show this help
#
# Publishes the four published crates in dependency order so each crate's
# dependencies are already on crates.io when it uploads. A crate already
# published at its current Cargo.toml version is skipped, so re-running
# after a partial failure resumes where it left off. systemone-bench is not
# published (publish = false).
#
# After publishing, pushes master and the version tag to origin and prints the
# GitHub release URL for the tag (create the release there with the tag message
# as the body).
#
# The crates.io sparse index can lag a few seconds between a dependency
# landing and its dependents resolving it; the script detects that specific
# failure ("candidate versions ... didn't match") and retries with backoff.
#
# Requires: cargo (logged in or CARGO_REGISTRY_TOKEN set), git remote `origin`,
# network access. Run from the workspace root.

set -euo pipefail

# Publish order = topological (dependencies before dependents).
ALL_ORDER=(
    systemone-core
    systemone
    systemone-macro
    systemone-facet
)

# How long to keep retrying a dependent whose just-published dep has not
# propagated through the crates.io sparse index yet. Override via env.
INDEX_LAG_RETRIES=${INDEX_LAG_RETRIES:-12}   # ~6 min ceiling at 30 s cadence
INDEX_LAG_WAIT=${INDEX_LAG_WAIT:-30}
CARGO=${CARGO:-cargo}
GIT=${GIT:-git}

DRY_RUN=no
ASSUME_YES=no
PUSH=yes
SELECTED=()

usage() {
    sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'
}

die() { echo "release.sh: $*" >&2; exit 1; }

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --dry-run)    DRY_RUN=yes; shift ;;
            --yes|-y)     ASSUME_YES=yes; shift ;;
            --no-push)    PUSH=no; shift ;;
            --crate)      [ $# -ge 2 ] || die "--crate needs a NAME"; SELECTED+=("$2"); shift 2 ;;
            --help|-h)    usage; exit 0 ;;
            *)            die "unknown flag: $1 (try --help)" ;;
        esac
    done
    # Default: every published crate.
    if [ ${#SELECTED[@]} -eq 0 ]; then
        SELECTED=("${ALL_ORDER[@]}")
    else
        # Re-project the selection onto the topological order so --crate
        # flags given out of order still publish deps before dependents.
        local filtered=() found
        for c in "${ALL_ORDER[@]}"; do
            for s in "${SELECTED[@]}"; do
                [ "$c" = "$s" ] && { filtered+=("$c"); break; }
            done
        done
        SELECTED=("${filtered[@]}")
        [ ${#SELECTED[@]} -gt 0 ] || die "no --crate matched a known published crate"
    fi
}

# Read the workspace version (all members share version.workspace).
workspace_version() {
    awk -F'"' '/^version[[:space:]]*=/ {print $2; exit}' Cargo.toml
}

# Classify a `cargo publish` failure stream. Echoes one of:
#   already   — version is already on crates.io (treat as success)
#   index-lag — a workspace dep was just published but the sparse index has
#               not caught up yet (retry)
#   other     — real error (abort)
classify_failure() {
    if echo "$1" | grep -qiE 'already (published|exists)|status 409|409 conflict'; then
        echo already
    elif echo "$1" | grep -qE "candidate versions? found which didn.t match|no matching package named"; then
        echo index-lag
    else
        echo other
    fi
}

publish_real() {
    local crate="$1" version attempt=0 out kind
    version=$(workspace_version)
    while :; do
        if out=$("$CARGO" publish -p "$crate" --color always 2>&1); then
            printf '  [ OK ]   %-18s %s\n' "$crate $version" "published"
            return 0
        fi
        kind=$(classify_failure "$out")
        case "$kind" in
            already)
                printf '  [SKIP]   %-18s %s\n' "$crate $version" "already on crates.io"
                return 0
                ;;
            index-lag)
                attempt=$((attempt + 1))
                if [ "$attempt" -le "$INDEX_LAG_RETRIES" ]; then
                    printf '  [WAIT]   %-18s index lag, retry %d/%d in %ds\n' \
                        "$crate $version" "$attempt" "$INDEX_LAG_RETRIES" "$INDEX_LAG_WAIT"
                    sleep "$INDEX_LAG_WAIT"
                    continue
                fi
                printf '  [FAIL]   %-18s index never propagated after %d retries\n' \
                    "$crate $version" "$INDEX_LAG_RETRIES"
                ;;
            *)
                printf '  [FAIL]   %-18s cargo publish failed\n' "$crate $version"
                ;;
        esac
        printf '%s\n' "$out" | sed 's/^/           /' >&2
        return 1
    done
}

publish_dry() {
    local crate="$1" version out kind
    version=$(workspace_version)
    if out=$("$CARGO" publish -p "$crate" --dry-run --color always 2>&1); then
        printf '  [ OK ]   %-18s %s\n' "$crate $version" "packaging valid"
        return 0
    fi
    kind=$(classify_failure "$out")
    # In dry-run, index-lag is expected for every non-leaf crate (its dep is
    # not on the registry yet). Report it as expected, not a failure.
    if [ "$kind" = index-lag ]; then
        printf '  [ OK ]   %-18s %s\n' "$crate $version" "packaging valid (dep not on registry yet)"
        return 0
    fi
    printf '  [FAIL]   %-18s packaging invalid\n' "$crate $version"
    printf '%s\n' "$out" | sed 's/^/           /' >&2
    return 1
}

push_release() {
    local version tag remote
    version=$(workspace_version)
    tag="v$version"
    "$GIT" rev-parse --verify --quiet "refs/tags/$tag" >/dev/null \
        || die "tag $tag does not exist — create it first"
    remote=$("$GIT" remote get-url origin 2>/dev/null) || die "no git remote 'origin'"
    printf 'Pushing master and %s to origin...\n' "$tag"
    "$GIT" push origin master
    "$GIT" push origin "$tag"
    printf 'Create the GitHub release at:\n  %s/releases/new?tag=%s\n' \
        "${remote%.git}" "$tag"
}

main() {
    parse_args "$@"

    # Guard: must be at the workspace root.
    grep -q '^\[workspace\]' Cargo.toml 2>/dev/null \
        || die "no [workspace] Cargo.toml here — run from the workspace root"

    local version
    version=$(workspace_version)

    printf 'Publishing %d crate(s) at v%s, topological order:\n' "${#SELECTED[@]}" "$version"
    for c in "${SELECTED[@]}"; do
        printf '    %s\n' "$c"
    done
    echo

    if [ "$DRY_RUN" = yes ]; then
        echo "Dry-run — validating packaging only, no upload, no push."
        echo "Non-leaf crates report 'dep not on registry yet'; that is expected."
        echo
        local rc=0
        for c in "${SELECTED[@]}"; do publish_dry "$c" || rc=1; done
        echo
        if [ "$rc" -eq 0 ]; then echo "All packaging valid."; else echo "Some crates failed."; fi
        return "$rc"
    fi

    if [ "$ASSUME_YES" = no ]; then
        echo "About to UPLOAD to crates.io. Dependencies must already be published"
        echo "in order; a failure stops the chain. Type 'yes' to proceed:"
        local ans
        read -r ans || die "no confirmation (stdin closed — pass --yes)"
        [ "$ans" = "yes" ] || { echo "aborted."; exit 1; }
        echo
    fi

    local rc=0
    for c in "${SELECTED[@]}"; do
        publish_real "$c" || rc=1
        # A dependent cannot resolve until its dep is published, so a real
        # failure stops the chain immediately.
        if [ "$rc" -ne 0 ]; then
            echo "Stopping: $c failed; its dependents cannot be published yet."
            echo "Fix the issue and re-run — already-published crates are skipped."
            break
        fi
    done
    echo
    if [ "$rc" -eq 0 ]; then echo "All crates published."; else echo "Publish incomplete."; return "$rc"; fi

    if [ "$PUSH" = yes ]; then
        echo
        push_release
    fi
}

main "$@"
