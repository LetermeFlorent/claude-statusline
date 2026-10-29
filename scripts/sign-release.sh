#!/bin/sh
set -eu

usage() {
    echo "usage: sign-release.sh -t vX.Y.Z [-k private_key] [-r owner/repo] [-R]" >&2
    exit 2
}

TAG=""
KEY="$HOME/.ssh/statusline-release"
REPO="LetermeFlorent/claude-statusline"
REPLACE=0
while getopts "t:k:r:R" opt; do
    case "$opt" in
        t) TAG=$OPTARG ;;
        k) KEY=$OPTARG ;;
        r) REPO=$OPTARG ;;
        R) REPLACE=1 ;;
        *) usage ;;
    esac
done

PUBLIC_KEY="ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAc/fOxJG/NBMFcBqV6k6pwGdOCfLsloZuAqCWBFmRtN"
PRINCIPAL="statusline-release"
BINARIES="statusline-windows-x86_64.exe statusline-windows-arm64.exe statusline-linux-x86_64 statusline-linux-arm64 statusline-macos-arm64 statusline-macos-x86_64"
MANIFEST="release-manifest"
SIG="release-manifest.sig"

die() {
    echo "error: $*" >&2
    exit 1
}

printf '%s\n' "$TAG" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$' || die "tag must look like v1.2.3"
[ -f "$KEY" ] || die "private key not found: $KEY"
case "$KEY" in
    /*) ;;
    *) KEY=$PWD/$KEY ;;
esac
for tool in curl ssh-keygen python3; do
    command -v "$tool" >/dev/null 2>&1 || die "$tool is required"
done
if command -v sha256sum >/dev/null 2>&1; then
    sha256() { sha256sum "$1" | cut -d ' ' -f 1; }
elif command -v shasum >/dev/null 2>&1; then
    sha256() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
else
    die "sha256sum or shasum is required"
fi

TOKEN=${GITHUB_TOKEN:-}
if [ -z "$TOKEN" ]; then
    printf 'GitHub token: ' >&2
    stty -echo
    trap 'stty echo' EXIT INT TERM
    read -r TOKEN
    stty echo
    trap - EXIT INT TERM
    echo >&2
fi
[ -n "$TOKEN" ] || die "no token"

WORK=$(mktemp -d "${TMPDIR:-/tmp}/sign-release.XXXXXX")
cleanup() {
    rm -rf "$WORK"
}
trap cleanup EXIT
trap 'exit 1' INT TERM

api() {
    printf 'Authorization: Bearer %s\n' "$TOKEN" |
        curl -fsS -H @- -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" -A statusline-sign-release "$@"
}

PARSE='
import json, sys
tag = sys.argv[2]
with open(sys.argv[1], encoding="utf-8") as handle:
    releases = json.load(handle)
for release in releases:
    if release.get("tag_name") == tag:
        print("release", release["id"], "draft" if release.get("draft") else "published", release["html_url"])
        for asset in release.get("assets", []):
            print("asset", asset["id"], asset["name"], asset["url"])
        break
'

FOUND=""
page=1
while [ "$page" -le 5 ] && [ -z "$FOUND" ]; do
    api "https://api.github.com/repos/$REPO/releases?per_page=100&page=$page" -o "$WORK/page.json"
    out=$(python3 -I -c "$PARSE" "$WORK/page.json" "$TAG")
    if [ -n "$out" ]; then
        FOUND=$out
    elif [ "$(tr -d ' \n' <"$WORK/page.json")" = "[]" ]; then
        break
    fi
    page=$((page + 1))
done
[ -n "$FOUND" ] || die "no release with tag $TAG was found"

HEAD=$(printf '%s\n' "$FOUND" | sed -n '1p')
RELEASE_ID=$(printf '%s\n' "$HEAD" | cut -d ' ' -f 2)
STATE=$(printf '%s\n' "$HEAD" | cut -d ' ' -f 3)
HTML_URL=$(printf '%s\n' "$HEAD" | cut -d ' ' -f 4)
[ "$STATE" = "draft" ] || die "release $TAG is not a draft, refusing to touch it"
ASSETS=$(printf '%s\n' "$FOUND" | sed '1d')

asset_field() {
    printf '%s\n' "$ASSETS" | awk -v n="$1" -v f="$2" '$3 == n { print $f }'
}

for name in $(printf '%s\n' "$ASSETS" | awk '{ print $3 }'); do
    case " $BINARIES $MANIFEST $SIG SHA256SUMS " in
        *" $name "*) ;;
        *) die "unexpected asset on the draft: $name" ;;
    esac
done
for name in $BINARIES $MANIFEST; do
    [ -n "$(asset_field "$name" 2)" ] || die "the draft has no asset named $name"
done

OLD_SIG=$(asset_field "$SIG" 2)
if [ -n "$OLD_SIG" ] && [ "$REPLACE" -ne 1 ]; then
    die "$SIG is already on the draft, use -R to overwrite it"
fi

for name in $BINARIES $MANIFEST; do
    echo "Downloading $name"
    api -L -H "Accept: application/octet-stream" -o "$WORK/$name" "$(asset_field "$name" 4)"
done

MF="$WORK/$MANIFEST"
[ -z "$(tail -c 1 "$MF")" ] || die "the manifest does not end with a newline"
if grep -q "$(printf '\r')" "$MF"; then
    die "the manifest contains carriage returns"
fi
[ "$(sed -n '1p' "$MF")" = "tag $TAG" ] || die "the manifest first line is not 'tag $TAG'"
[ "$(wc -l <"$MF" | tr -d ' ')" -ge 2 ] || die "the manifest has no file lines"

SEEN=" "
while IFS= read -r line; do
    printf '%s\n' "$line" | grep -Eq '^[0-9a-f]{64}  [^ ]+$' || die "malformed manifest line: $line"
    want=${line%%  *}
    name=${line#*  }
    case " $BINARIES " in
        *" $name "*) ;;
        *) die "unexpected file in the manifest: $name" ;;
    esac
    case "$SEEN" in
        *" $name "*) die "listed twice in the manifest: $name" ;;
    esac
    SEEN="$SEEN$name "
    [ "$(sha256 "$WORK/$name")" = "$want" ] || die "SHA-256 mismatch for $name"
done <<EOF
$(sed '1d' "$MF")
EOF
for name in $BINARIES; do
    case "$SEEN" in
        *" $name "*) ;;
        *) die "missing from the manifest: $name" ;;
    esac
done

echo
cat "$MF"
printf 'The checksums match. Sign this manifest for %s? (y/N) ' "$TAG"
read -r answer
[ "$answer" = "y" ] || die "cancelled"

(cd "$WORK" && ssh-keygen -Y sign -n "$PRINCIPAL" -f "$KEY" "$MANIFEST")
[ -f "$WORK/$SIG" ] || die "signing failed"

printf '%s namespaces="%s" %s\n' "$PRINCIPAL" "$PRINCIPAL" "$PUBLIC_KEY" >"$WORK/allowed_signers"
ssh-keygen -Y verify -f "$WORK/allowed_signers" -I "$PRINCIPAL" -n "$PRINCIPAL" \
    -s "$WORK/$SIG" <"$MF" >"$WORK/verify.out" 2>&1 ||
    die "the new signature does not verify against the embedded public key"
echo "Signature verified against the embedded public key."

if [ -n "$OLD_SIG" ]; then
    api -X DELETE "https://api.github.com/repos/$REPO/releases/assets/$OLD_SIG" -o /dev/null
fi
api -X POST -H "Content-Type: application/octet-stream" --data-binary "@$WORK/$SIG" \
    "https://uploads.github.com/repos/$REPO/releases/$RELEASE_ID/assets?name=$SIG" -o /dev/null

echo
echo "Uploaded $SIG to the draft for $TAG."
echo "$HTML_URL"
echo "The release is still a draft. Read it, then publish it by hand as the latest release."
