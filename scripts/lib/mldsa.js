/**
 * Read and validate ML-DSA-87 public keys in the byte layout the chain stores.
 *
 * The pallet verifies with `fips204` 0.4.6 (`ml_dsa_87`, context
 * `belizechain-bridge-v1`, PK_LEN 2592, SIG_LEN 4627). Keys are produced by
 * `scripts/generate-bridge-pq-keys.sh`, which writes a `<name>.pem` private key
 * beside each `<name>.pub` hex public key.
 *
 * Shared by `bootstrap-dex.js` and `rotate-bridge-pq-keys.js` so the format is
 * enforced in one place: a key that fails here would otherwise be written to
 * storage and only reveal itself when a signature fails to verify.
 */

const fs = require('fs');
const { execFileSync } = require('child_process');

/** ML-DSA-87 public keys are exactly this long; the pallet bounds the field. */
const ML_DSA_87_PUBLIC_KEY_BYTES = 2592;

/**
 * Read one raw ML-DSA-87 public key and return the hex the pallet stores.
 *
 * The file is hex on one line, no `0x`. Validating length here means a truncated
 * or wrong-algorithm key fails before any extrinsic is signed.
 */
function readPqPublicKey(path) {
  const raw = fs.readFileSync(path, 'utf8').trim().replace(/^0x/i, '');
  if (!/^[0-9a-fA-F]+$/.test(raw)) {
    throw new Error(`${path}: not hex — expected the output of generate-bridge-pq-keys.sh`);
  }
  const byteLength = raw.length / 2;
  if (byteLength !== ML_DSA_87_PUBLIC_KEY_BYTES) {
    throw new Error(
      `${path}: ${byteLength} bytes, expected ${ML_DSA_87_PUBLIC_KEY_BYTES} for ML-DSA-87`,
    );
  }
  return { hex: `0x${raw.toLowerCase()}`, path };
}

/**
 * Prove that a `.pub` file belongs to the `.pem` beside it, returning a short
 * fingerprint.
 *
 * This is the only cheap check that distinguishes a real keypair from bytes that
 * merely look key-shaped. Counting distinct byte values does NOT discriminate: a
 * genuine 2592-byte ML-DSA-87 public key uses all 256 values, exactly as
 * uniformly random bytes do (measured 2026-11-09). Re-deriving the public key
 * from the private key is a fact about the pair, not a guess about its shape.
 *
 * Requiring the private key at all is the point: a registered public key whose
 * private key nobody holds cannot produce a verifiable signature, which is the
 * defect this replaced (`randomAsU8a(2592)` placeholders).
 *
 * `options.host` runs the derivation on another machine over SSH, looking for the
 * private key under `options.remoteDir`. That exists because the production keys
 * live on Ceiba, which has no Node runtime: the rotation script runs on a
 * workstation, and a private key must not be copied off the host that holds it.
 * Only the derived public key crosses the wire.
 */
function assertKeypairMatches(pubPath, pubHex, options = {}) {
  const derived = options.host
    ? derivePublicKeyOverSsh(options.host, pubPath, options.remoteDir)
    : derivePublicKeyLocally(pubPath);
  if (`0x${derived}` !== pubHex) {
    throw new Error(
      `${pubPath}: public key does not match the private key${options.host ? ` on ${options.host}` : ''}`,
    );
  }
  return derived.slice(0, 16);
}

/** Derive the raw public key from the `.pem` next to `pubPath`, on this machine. */
function derivePublicKeyLocally(pubPath) {
  const pemPath = pubPath.replace(/\.pub$/, '.pem');
  if (!fs.existsSync(pemPath)) {
    throw new Error(
      `${pubPath}: no matching private key at ${pemPath}. A public key without its `
      + `private key cannot sign, so registering it would recreate the placeholder defect.`,
    );
  }
  const der = execFileSync(
    'openssl', ['pkey', '-in', pemPath, '-pubout', '-outform', 'DER'],
    { maxBuffer: 1 << 20 },
  );
  return der.subarray(der.length - ML_DSA_87_PUBLIC_KEY_BYTES).toString('hex');
}

/**
 * Same derivation, executed on `host` over SSH.
 *
 * `remoteDir` is where the `.pem` files live on that host; it is a separate
 * argument because the local `.pub` copies are ordinary files that can sit
 * anywhere, while the private keys stay in one guarded directory.
 *
 * `pipefail` matters: without it the exit status is `tail`'s, so a missing or
 * unreadable key still exits 0 and the failure shows up only as a zero-length
 * read. The error message would then blame the key length rather than the
 * missing file that caused it.
 */
function derivePublicKeyOverSsh(host, pubPath, remoteDir) {
  const pemName = require('path').basename(pubPath).replace(/\.pub$/, '.pem');
  const pemPath = remoteDir
    ? `${remoteDir.replace(/\/$/, '')}/${pemName}`
    : pemName;
  const remote = `set -o pipefail; openssl pkey -in '${pemPath}' -pubout -outform DER `
    + `| tail -c ${ML_DSA_87_PUBLIC_KEY_BYTES}`;
  let raw;
  try {
    raw = execFileSync('ssh', ['-o', 'BatchMode=yes', host, remote], {
      maxBuffer: ML_DSA_87_PUBLIC_KEY_BYTES * 2,
    });
  } catch (e) {
    throw new Error(
      `${pubPath}: could not read the private key at ${pemPath} on ${host}. `
      + `A public key without its private key cannot sign, so registering it would `
      + `recreate the placeholder defect.`,
    );
  }
  if (raw.length !== ML_DSA_87_PUBLIC_KEY_BYTES) {
    throw new Error(
      `${pubPath}: ${host}:${pemPath} produced ${raw.length} bytes, expected ${ML_DSA_87_PUBLIC_KEY_BYTES}`,
    );
  }
  return raw.toString('hex');
}

module.exports = {
  ML_DSA_87_PUBLIC_KEY_BYTES,
  readPqPublicKey,
  assertKeypairMatches,
};
