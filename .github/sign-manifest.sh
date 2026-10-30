#!/bin/bash
set -eu
umask 077

usage() {
  echo "usage: sign-manifest.sh <manifest> '<key type> <base64 key>'" >&2
  echo "Signs <manifest> into <manifest>.sig with the OpenSSH private key held in RELEASE_SIGNING_KEY," >&2
  echo "then checks the signature against the given public key." >&2
  echo "Exit codes: 0 signed and verified, 1 signing or checking failed, 2 usage, 3 RELEASE_SIGNING_KEY empty or unset." >&2
  exit 2
}

die() {
  echo "sign-manifest.sh: $*" >&2
  exit 1
}

[ "$#" -eq 2 ] || usage
manifest=$1
read -r key_type key_base64 _ <<< "$2" || true
principal=statusline-release
signature="$manifest.sig"

[ "${key_type:-}" = ssh-ed25519 ] || die "the expected public key is not an ssh-ed25519 key"
[[ "${key_base64:-}" =~ ^[A-Za-z0-9+/]+=*$ ]] || die "the expected public key is malformed"
[ -f "$manifest" ] || die "no manifest at $manifest"

case "${RELEASE_SIGNING_KEY:-}" in
  *[![:space:]]*) ;;
  *)
    echo "sign-manifest.sh: RELEASE_SIGNING_KEY is empty or unset, nothing was signed" >&2
    exit 3
    ;;
esac

signed=no
work=""
cleanup() {
  [ -z "$work" ] || rm -rf "$work"
  [ "$signed" = yes ] || rm -f "$signature"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

work=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/sign-manifest.XXXXXX")
printf '%s\n' "$RELEASE_SIGNING_KEY" | tr -d '\r' > "$work/key"
rm -f "$signature"

derived=$(ssh-keygen -y -P "" -f "$work/key" 2> /dev/null < /dev/null) || die "RELEASE_SIGNING_KEY is not a readable private key without a passphrase"
read -r derived_type derived_base64 _ <<< "$derived" || true
[ "$derived_type $derived_base64" = "$key_type $key_base64" ] || die "RELEASE_SIGNING_KEY does not match the expected public key"

ssh-keygen -q -Y sign -n "$principal" -f "$work/key" "$manifest" < /dev/null || die "ssh-keygen could not sign $manifest"
[ -s "$signature" ] || die "ssh-keygen wrote no signature"

printf '%s namespaces="%s" %s %s\n' "$principal" "$principal" "$key_type" "$key_base64" > "$work/allowed_signers"
if ! ssh-keygen -Y verify -f "$work/allowed_signers" -I "$principal" -n "$principal" -s "$signature" < "$manifest" > "$work/verify.out" 2>&1; then
  cat "$work/verify.out" >&2
  die "the new signature does not verify against the expected public key"
fi
cat "$work/verify.out"
signed=yes
