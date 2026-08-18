import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const bundleRoot = join(repositoryRoot, 'src-tauri', 'target', 'release', 'bundle')
const trustMarker = join(bundleRoot, 'BUILD-TRUST.txt')
const thumbprint = (process.env.EDY_SENTINEL_AUTHENTICODE_THUMBPRINT ?? '')
  .replaceAll(/\s/g, '')
  .toUpperCase()
const timestampUrl = process.env.EDY_SENTINEL_TIMESTAMP_URL ?? ''
const requireSigned = process.env.EDY_SENTINEL_REQUIRE_SIGNED_RELEASE === '1'
const tspValue = process.env.EDY_SENTINEL_TIMESTAMP_RFC3161 ?? 'false'
const packageManagerScript = process.env.npm_execpath
const packageVersion = JSON.parse(readFileSync(join(repositoryRoot, 'package.json'), 'utf8')).version
const unsignedLabel = /-rc\./i.test(packageVersion)
  ? 'UNSIGNED RELEASE CANDIDATE'
  : 'UNSIGNED DEVELOPMENT BUILD'

function run(command, args) {
  const result = spawnSync(command, args, { cwd: repositoryRoot, stdio: 'inherit' })
  if (result.error) throw result.error
  if (result.status !== 0) process.exit(result.status ?? 1)
}

function writeTrustMarker(lines) {
  mkdirSync(bundleRoot, { recursive: true })
  writeFileSync(trustMarker, `${lines.join('\n')}\n`, 'utf8')
}

function runTauriBuild(extraArguments = []) {
  if (!packageManagerScript) {
    console.error('Run this wrapper through `pnpm release:build`.')
    process.exit(2)
  }
  // Prevent a prior version from being mistaken for a current release artifact.
  rmSync(bundleRoot, { recursive: true, force: true })
  run(process.execPath, [packageManagerScript, 'exec', 'tauri', 'build', ...extraArguments, ...process.argv.slice(2)])
}

if (!thumbprint) {
  if (requireSigned) {
    console.error('Signed release required, but EDY_SENTINEL_AUTHENTICODE_THUMBPRINT is absent.')
    process.exit(2)
  }
  console.warn(unsignedLabel)
  runTauriBuild()
  writeTrustMarker([
    unsignedLabel,
    `Version: ${packageVersion}`,
    'Public distribution is blocked until Authenticode signing and timestamp verification pass.',
  ])
  process.exit(0)
}

if (process.platform !== 'win32') {
  console.error('Authenticode release signing must run on Windows.')
  process.exit(2)
}
if (!/^[A-F0-9]{40}$/.test(thumbprint)) {
  console.error('EDY_SENTINEL_AUTHENTICODE_THUMBPRINT must be a 40-character SHA-1 certificate thumbprint.')
  process.exit(2)
}
if (!timestampUrl) {
  console.error('EDY_SENTINEL_TIMESTAMP_URL is required for a signed release build.')
  process.exit(2)
}
if (!['true', 'false'].includes(tspValue)) {
  console.error('EDY_SENTINEL_TIMESTAMP_RFC3161 must be true or false.')
  process.exit(2)
}

const temporaryDirectory = mkdtempSync(join(tmpdir(), 'edy-sentinel-signing-'))
const signingConfig = join(temporaryDirectory, 'tauri.signing.conf.json')
try {
  writeFileSync(signingConfig, JSON.stringify({
    bundle: {
      windows: {
        certificateThumbprint: thumbprint,
        digestAlgorithm: 'sha256',
        timestampUrl,
        tsp: tspValue === 'true',
      },
    },
  }), 'utf8')
  runTauriBuild(['--config', signingConfig])
  run('powershell.exe', [
    '-NoProfile',
    '-File',
    join(repositoryRoot, 'scripts', 'verify-authenticode.ps1'),
    '-BundleRoot',
    bundleRoot,
    '-ExecutablePath',
    join(repositoryRoot, 'src-tauri', 'target', 'release', 'edy-sentinel.exe'),
    '-RequireTimestamp',
  ])
  writeTrustMarker([
    'SIGNED RELEASE BUILD',
    `Certificate thumbprint: ${thumbprint}`,
    `Timestamp URL: ${timestampUrl}`,
    'Authenticode and timestamp verification: PASS',
  ])
} finally {
  rmSync(temporaryDirectory, { recursive: true, force: true })
}
