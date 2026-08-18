# Windows code signing

EDY Sentinel public Windows releases require a legitimate Authenticode publisher
certificate and a trusted timestamp. A self-signed certificate is not a production
substitute. Certificate files, private keys, tokens, passwords, and cloud-signing
credentials must never be committed, logged, embedded in React, or passed on a command
line.

The following artifacts must all validate before publication:

1. `src-tauri/target/release/edy-sentinel.exe`;
2. the MSI under `src-tauri/target/release/bundle/msi/`;
3. the NSIS setup executable under `src-tauri/target/release/bundle/nsis/`.

## Signing boundary and order

The supported local/runner path uses the Windows Certificate Store. A secured runner or
operator imports or exposes the legitimate certificate and non-exportable private key in
`Cert:\CurrentUser\My` before the build. The repository receives only the public SHA-1
certificate thumbprint. `scripts/build-release.mjs` creates an ephemeral Tauri config in
the operating-system temporary directory and removes it after the build.

Before every build the wrapper removes only the ignored bundle-output directory. This prevents a
stale installer from a previous version from entering the three-artifact verification set.

The required order is:

1. build and test the frontend and Rust binary;
2. Authenticode-sign the application EXE with SHA-256 and timestamp it;
3. package that signed EXE into MSI and NSIS;
4. Authenticode-sign and timestamp the outer MSI and NSIS setup executable;
5. verify publisher trust and a timestamp on all three artifacts;
6. calculate release hashes, then publish only the verified immutable files.

Tauri owns steps 2–4 when its Windows bundle configuration receives the certificate
thumbprint, digest and timestamp settings. The repository wrapper owns fail-closed input
validation, post-build verification, and the build-trust marker.

## Pipeline inputs

- `EDY_SENTINEL_AUTHENTICODE_THUMBPRINT`: required 40-hex SHA-1 thumbprint of the
  legitimate publisher certificate in the Windows Certificate Store. It identifies the
  public certificate and is not the private key.
- `EDY_SENTINEL_TIMESTAMP_URL`: required certificate-provider timestamp endpoint.
- `EDY_SENTINEL_TIMESTAMP_RFC3161`: optional `true` or `false`; use the value required by
  the provider. Default is `false` for legacy Authenticode timestamping.
- `EDY_SENTINEL_REQUIRE_SIGNED_RELEASE=1`: mandatory in a public release job. It makes a
  missing signing identity a hard failure.

The private-key credential is injected before this wrapper by the protected release
environment: hardware token/HSM, managed code-signing service, or secured certificate-store
bootstrap. Provider credentials and any PFX payload/password are CI secrets owned by that
bootstrap and are deliberately not named as ordinary project variables or handled by this
repository.

Build with the configured production identity:

```powershell
$env:EDY_SENTINEL_AUTHENTICODE_THUMBPRINT = '<production thumbprint>'
$env:EDY_SENTINEL_TIMESTAMP_URL = '<provider timestamp URL>'
$env:EDY_SENTINEL_TIMESTAMP_RFC3161 = 'true'
$env:EDY_SENTINEL_REQUIRE_SIGNED_RELEASE = '1'
pnpm release:build
```

The wrapper invokes `scripts/verify-authenticode.ps1`. Verification requires exactly one
application EXE, one MSI and one NSIS setup executable, `Status=Valid`, a signer certificate,
and a timestamp certificate for every artifact. A failure exits non-zero and blocks release.
The wrapper writes `BUILD-TRUST.txt` beside the generated bundles; this file is build output
and remains ignored by Git.

## Unsigned local, development, and release-candidate builds

Local development must remain buildable without any signing credential:

```powershell
pnpm release:build
```

When no thumbprint is available and signed release mode is not required, the wrapper reads the
package version. A prerelease matching `-rc.` prints and records:

`UNSIGNED RELEASE CANDIDATE`

Final local version `1.0.0` prints and records:

`UNSIGNED BUILD`

Other development versions print and record:

`UNSIGNED DEVELOPMENT BUILD`

Such artifacts may be used for local QA or human review but must not be published as a public signed release.
If `EDY_SENTINEL_REQUIRE_SIGNED_RELEASE=1` is set, missing thumbprint or timestamp input fails
before compilation. The wrapper never generates a certificate, never downgrades to a
self-signed identity, and never claims that an unsigned artifact is signed.

Configuration fields follow the official Tauri 2 Windows signing contract:
<https://v2.tauri.app/distribute/sign/windows/> and
<https://v2.tauri.app/reference/config/#windowsconfig>.
