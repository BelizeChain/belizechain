#!/usr/bin/env node
/**
 * harden-genesis-spec.js — make a chain spec safe to re-genesis from.
 *
 * WHY THIS EXISTS
 * ---------------
 * `testnet-spec.json` is what Ceiba boots from and what a re-genesis would use.
 * It pinned well-known Substrate **development** keys in every privileged slot:
 *
 *   sudo.key                  = //Alice
 *   session.keys[*].babe      = //Alice
 *   session.keys[*].grandpa   = //Alice's ed25519 key
 *   identity.*Issuers (x3)    = //Alice
 *   governance.councilMembers = //Alice, //Bob, //Charlie
 *   balances                  = 1,000,000 DALLA to //Alice
 *
 * Every one of those seeds is published in the Substrate docs, so anyone could
 * sign as them. Rotating live storage (blocker #13) fixed the *running* chain;
 * a re-genesis from this file would have restored the entire exposure on a new
 * chain with nothing in the docs looking wrong. That is the failure this script
 * prevents.
 *
 * The script rewrites only the privileged slots, derives what it can, and then
 * REFUSES TO WRITE unless a freshly generated block of every common dev key is
 * absent from the output. It cannot be run "successfully" while leaving //Alice
 * anywhere — that is the point.
 *
 * USAGE
 * -----
 *   node scripts/harden-genesis-spec.js \
 *     --sudo    0x<32-byte hex or ss58> \
 *     --validator 0x<session key owner = BABE key> \
 *     --grandpa 0x<32-byte hex or ss58> \
 *     [--spec testnet-spec.json] [--out testnet-spec.json] [--check-only]
 *
 * Requires @polkadot/util-crypto, which is already in ui/node_modules:
 *   NODE_PATH=../ui/node_modules node scripts/harden-genesis-spec.js --help
 */

const fs = require('fs');
const path = require('path');

function loadCrypto() {
  try {
    return require('@polkadot/util-crypto');
  } catch {
    console.error(
      'Cannot load @polkadot/util-crypto. Run with:\n' +
        '  NODE_PATH=../ui/node_modules node scripts/harden-genesis-spec.js ...'
    );
    process.exit(2);
  }
}

const DEFAULT_SPEC = 'testnet-spec.json';

/** SS58 prefix 42 is the chain-agnostic form the existing spec uses. */
const SPEC_SS58_PREFIX = 42;

/**
 * Seeds whose private keys are in Substrate's public documentation. None of
 * these may appear anywhere in a spec that is safe to re-genesis from.
 */
const DEV_SEEDS = [
  '//Alice',
  '//Bob',
  '//Charlie',
  '//Dave',
  '//Eve',
  '//Ferdie',
  '//Alice//stash',
  '//Bob//stash',
  '//Charlie//stash',
  '//Dave//stash',
  '//Eve//stash',
  '//Ferdie//stash',
];

/**
 * Derive the addresses a dev seed would produce, in both key types: session
 * keys include an ed25519 grandpa key that shares the seed but not the address.
 */
function devAddresses(crypto) {
  const { Keyring } = require('@polkadot/keyring');
  const out = new Map();
  for (const seed of DEV_SEEDS) {
    for (const type of ['sr25519', 'ed25519']) {
      try {
        const pair = new Keyring({ type, ss58Format: SPEC_SS58_PREFIX }).addFromUri(seed);
        out.set(pair.address, `${seed} (${type})`);
      } catch {
        /* some seed/type combinations are not derivable; skip */
      }
    }
  }
  return out;
}

/** Every string value in a JSON tree, so dev keys can be found wherever they hide. */
function allStrings(node, out = []) {
  if (typeof node === 'string') out.push(node);
  else if (Array.isArray(node)) node.forEach((v) => allStrings(v, out));
  else if (node && typeof node === 'object') Object.values(node).forEach((v) => allStrings(v, out));
  return out;
}

function parseArgs(argv) {
  const args = { spec: DEFAULT_SPEC, out: null, checkOnly: false };
  for (let i = 2; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--help' || a === '-h') args.help = true;
    else if (a === '--check-only') args.checkOnly = true;
    else if (a === '--spec') args.spec = argv[++i];
    else if (a === '--out') args.out = argv[++i];
    else if (a === '--sudo') args.sudo = argv[++i];
    else if (a === '--validator') args.validator = argv[++i];
    else if (a === '--grandpa') args.grandpa = argv[++i];
    else throw new Error(`unknown argument: ${a}`);
  }
  return args;
}

function usage() {
  console.log(`harden-genesis-spec.js — remove publicly-known dev keys from a chain spec

  --sudo <hex|ss58>       account that becomes sudo.key and the identity issuer
  --validator <hex|ss58>  account that owns the session keys (its own BABE key)
  --grandpa <hex|ss58>    ed25519 grandpa key
  --spec <path>           input spec (default ${DEFAULT_SPEC})
  --out <path>            REQUIRED. Where to write the hardened spec. It must not
                          equal --spec (see the warning in the source).
  --check-only            verify the spec is clean, change nothing

Exits non-zero if any publicly-known dev key is still present.`);
}

/** Reject a spec that still contains any dev key, naming each offender. */
function assertNoDevKeys(spec, crypto, label) {
  const devs = devAddresses(crypto);
  const offenders = [];
  for (const value of allStrings(spec)) {
    if (devs.has(value)) offenders.push(`${value} → ${devs.get(value)}`);
  }
  if (offenders.length > 0) {
    const unique = [...new Set(offenders)];
    console.error(`✗ ${label}: ${unique.length} publicly-known dev key(s) still present:`);
    for (const o of unique) console.error(`    ${o}`);
    return false;
  }
  console.log(`✓ ${label}: no publicly-known dev key present`);
  return true;
}

function toSpecAddress(crypto, value, label) {
  const { decodeAddress, encodeAddress } = crypto;
  let bytes;
  try {
    bytes = decodeAddress(value);
  } catch (e) {
    throw new Error(`${label}: cannot decode ${value} (${e.message})`);
  }
  if (bytes.length !== 32) throw new Error(`${label}: expected a 32-byte key, got ${bytes.length}`);
  return encodeAddress(bytes, SPEC_SS58_PREFIX);
}

async function main() {
  const args = parseArgs(process.argv);
  if (args.help) {
    usage();
    return 0;
  }
  const crypto = loadCrypto();
  await crypto.cryptoWaitReady();

  const specPath = path.resolve(args.spec);
  if (!fs.existsSync(specPath)) {
    console.error(`✗ no such spec: ${specPath}`);
    return 2;
  }
  const spec = JSON.parse(fs.readFileSync(specPath, 'utf8'));

  if (args.checkOnly) {
    return assertNoDevKeys(spec, crypto, path.basename(specPath)) ? 0 : 1;
  }

  for (const required of ['sudo', 'validator', 'grandpa', 'out']) {
    if (!args[required]) {
      console.error(`✗ --${required} is required (specs are not rewritten by guesswork)`);
      usage();
      return 2;
    }
  }

  const sudo = toSpecAddress(crypto, args.sudo, '--sudo');
  const validator = toSpecAddress(crypto, args.validator, '--validator');
  const grandpa = toSpecAddress(crypto, args.grandpa, '--grandpa');

  const patch = spec?.genesis?.runtimeGenesis?.patch;
  if (!patch) {
    console.error('✗ spec has no genesis.runtimeGenesis.patch; refusing to guess its layout');
    return 2;
  }

  const changes = [];

  // 1. Root. This is the one that makes a re-genesis catastrophic.
  patch.sudo = { ...(patch.sudo ?? {}), key: sudo };
  changes.push(`sudo.key               → ${sudo}`);

  // 2. Consensus. BABE/GRANDPA authorities are derived from the session keys at
  //    genesis, so this single slot decides who can author and finalise.
  const sessionKeys = { babe: validator, grandpa };
  patch.session = { ...(patch.session ?? {}), keys: [[validator, validator, sessionKeys]] };
  changes.push(`session.keys           → 1 authority (${validator})`);

  // 3. Identity issuers. Blocker #13 rotated all three lists off //Alice; a
  //    re-genesis that restores them re-arms the same privilege.
  if (patch.identity) {
    patch.identity.initialSsnIssuers = [sudo];
    patch.identity.initialPassportIssuers = [sudo];
    patch.identity.initialBiometricIssuers = [sudo];
    changes.push('identity.*Issuers (x3) → sudo account');
  }

  // 4. Council. Genesis stamped every member Founder against max_count() = 1,
  //    and two of the three were //Bob and //Charlie — votes anyone can cast.
  //    One seat, held by a real account, is the only defensible genesis today.
  if (patch.governance) {
    patch.governance.councilMembers = [[sudo, 1, 0]];
    changes.push('governance.councilMembers → 1 seat (sudo)');
  }

  // 5. Balances. 1,000,000 DALLA in //Alice is spendable by anyone; move that
  //    allocation to the real root account rather than leaving it behind.
  const balances = patch?.balances?.balances;
  if (Array.isArray(balances)) {
    const devs = devAddresses(crypto);
    let moved = 0;
    for (const entry of balances) {
      if (devs.has(entry[0])) {
        entry[0] = sudo;
        moved += 1;
      }
    }
    if (moved > 0) changes.push(`balances              → ${moved} dev allocation(s) moved to sudo`);
  }

  console.log(`Rewriting ${path.relative(process.cwd(), specPath)}:`);
  for (const c of changes) console.log(`  ${c}`);

  // The gate. Nothing is written until the output is proven clean.
  if (!assertNoDevKeys(patch, crypto, 'patched genesis')) return 1;

  const outPath = path.resolve(args.out);
  if (outPath === specPath) {
    // A live chain's boot spec MUST reproduce that chain's genesis. Rewriting it
    // in place changes the genesis hash under the same chain id, and the node
    // then refuses the chain data it has been running on -- an outage, not a
    // hardening. Measured 2026-11-07: this repo's testnet-spec.json was md5
    // 31171e10... and Ceiba's /data/chain/testnet-spec.json was identical, so an
    // in-place edit here would have shipped as a broken boot spec.
    console.error(
      '\u2717 --out must differ from --spec.\n' +
        '  A live boot spec has to reproduce the genesis that already happened;\n' +
        '  rewriting it in place changes the genesis hash and the node will refuse\n' +
        '  its own chain data. Write the hardened spec to a new path (for example\n' +
        '  chain-specs/genesis-hardened-<date>.json) and keep the operational spec\n' +
        '  untouched.'
    );
    return 2;
  }
  fs.writeFileSync(outPath, `${JSON.stringify(spec, null, 2)}\n`);
  console.log(`\nWrote ${path.relative(process.cwd(), outPath)}`);
  return 0;
}

main()
  .then((code) => process.exit(code))
  .catch((e) => {
    console.error(`✗ ${e.message}`);
    process.exit(1);
  });
