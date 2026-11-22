# Security

## Reporting a vulnerability

Report it privately through this repository's
[private vulnerability reporting](https://github.com/shuwariafrica/pamba/security/advisories/new).
Please do not open a public issue for it.

## Verifying a release

Every release of pamba is built, signed and published by this repository's release workflow,
from a tag the owner signed, and recorded with its signed checksums and attestations in its
[releases](https://github.com/shuwariafrica/pamba/releases). Its signing key is
pamba's own, whose fingerprint is

```
7C96D0926D61D7F76544BD010E0E86A8608B7CC5
```

pamba is four crates, each released under the same tag: `pamba`, `pamba-host`, `pamba-windows`
and `pamba-windows-build`. In a new, empty directory, fetch the one you are verifying, with
`<crate>` here and below its name and `<version>` its version:

```
curl -fsSLo <crate>-<version>.crate https://static.crates.io/crates/<crate>/<crate>-<version>.crate
```

With a GitHub login, this prints the attested repository, tag and commit, and prints nothing
unless the attestation is of this tag:

```
gh attestation verify <crate>-<version>.crate \
  --repo shuwariafrica/pamba \
  --signer-workflow shuwariafrica/pamba/.github/workflows/release.yml \
  --source-ref refs/tags/v<version> \
  --predicate-type https://shuwari.africa/attestations/crate-release/v1 \
  --format json \
  --jq '.[].verificationResult.statement.predicate | select(.tag == "v<version>") | "\(.repository) \(.tag) \(.commit)"'
```

Without one, check the release's signed checksums against the signing key's certificate as
[public-crates](https://github.com/shuwariafrica/public-crates) holds it, with keys.openpgp.org's
copy of the key beside it, so a revocation published there makes `sqv` refuse. Run it in the
directory that holds the `.crate`, as one command whose exit status is its verdict: it passes only
when `sqv` names this key as the signer. The fingerprint in it is the one above, which the crate's
README on crates.io states too; compare the two before you run it.

```
base=https://github.com/shuwariafrica/pamba/releases/download/v<version> && \
curl -fsSLO "$base/SHA256SUMS" -fsSLO "$base/SHA256SUMS.asc" \
  -fsSLo signing.asc https://raw.githubusercontent.com/shuwariafrica/public-crates/main/signing-keys/pamba.asc \
  -fsSLo published.asc https://keys.openpgp.org/vks/v1/by-fingerprint/7C96D0926D61D7F76544BD010E0E86A8608B7CC5 && \
test "$(sqv --keyring signing.asc --keyring published.asc SHA256SUMS.asc SHA256SUMS)" = 7C96D0926D61D7F76544BD010E0E86A8608B7CC5 && \
sha256sum -c --ignore-missing SHA256SUMS
```

The `.crate` crates.io serves has the index's checksum, which `SHA256SUMS` lists.
