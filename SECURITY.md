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

pamba is three crates, each released under the same tag: `pamba`, `pamba-windows` and
`pamba-windows-build`. Fetch the one you are verifying, with `<crate>` here and below its name and
`<version>` its version:

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
keys.openpgp.org publishes it, so a published revocation makes `sqv` refuse:

```
base=https://github.com/shuwariafrica/pamba/releases/download/v<version>
curl -fsSLO "$base/SHA256SUMS" -fsSLO "$base/SHA256SUMS.asc" \
  -fsSLo signing.asc https://keys.openpgp.org/vks/v1/by-fingerprint/7C96D0926D61D7F76544BD010E0E86A8608B7CC5
gpg --show-keys --with-fingerprint signing.asc   # compare with the fingerprint above
sqv --keyring signing.asc SHA256SUMS.asc SHA256SUMS && sha256sum -c --ignore-missing SHA256SUMS
```

The `.crate` crates.io serves has the index's checksum, which `SHA256SUMS` lists.
