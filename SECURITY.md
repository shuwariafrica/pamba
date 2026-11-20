# Security

## Reporting a vulnerability

Report it privately through this repository's
[private vulnerability reporting](https://github.com/shuwariafrica/pamba/security/advisories/new).
Please do not open a public issue for it.

## Verifying a release

Every release of pamba is built and published by
[`shuwariafrica/public-crates`](https://github.com/shuwariafrica/public-crates) from a tag the
owner signed. Its releases are
[`pamba-` releases there](https://github.com/shuwariafrica/public-crates/releases?q=pamba-).

Pamba is three crates, each released under the same tag: `pamba`, `pamba-windows` and
`pamba-windows-build`. Fetch the one you are verifying, with `<crate>` here and below its name and
`<version>` its version:

```
curl -fsSLo <crate>-<version>.crate https://static.crates.io/crates/<crate>/<crate>-<version>.crate
```

With a GitHub login, this prints `verified` only where the release's attestation names this
project and its tag:

```
gh attestation verify <crate>-<version>.crate \
  --repo shuwariafrica/public-crates \
  --signer-workflow shuwariafrica/public-crates/.github/workflows/release.yml \
  --source-ref refs/heads/main \
  --predicate-type https://shuwari.africa/attestations/crate-release/v1 \
  --format json \
  --jq '.[].verificationResult.statement.predicate | select(.repository == "shuwariafrica/pamba" and .tag == "v<version>")' \
  | grep -q . && echo verified
```

Without one, check the release's signed checksums against the signing key, whose fingerprint is

```
<SIGNING KEY FINGERPRINT: to be filled when the owner rules on the key>
```

```
base=https://github.com/shuwariafrica/public-crates/releases/download/pamba-v<version>
curl -fsSLO "$base/SHA256SUMS" -fsSLO "$base/SHA256SUMS.asc" -fsSLo signing.asc <SIGNING KEY CERTIFICATE URL: to be filled with the fingerprint>
gpg --show-keys --with-fingerprint signing.asc   # the fingerprint above
sqv --keyring signing.asc SHA256SUMS.asc SHA256SUMS && sha256sum -c --ignore-missing SHA256SUMS
```

The `.crate` crates.io serves has the index's checksum, which `SHA256SUMS` lists.
