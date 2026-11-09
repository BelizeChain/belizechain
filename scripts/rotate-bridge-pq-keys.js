#!/usr/bin/env node
/**
 * Replace placeholder bridge-validator keys with real ML-DSA-87 keys.
 *
 * Why this exists
 * ---------------
 * `bootstrap-dex.js` used to register validators with `randomAsU8a(2592)` as the
 * PQ public key. That proved the extrinsic path and the set reconciliation, but
 * it left records nobody can sign under: the private key for random bytes does
 * not exist. `PqVerifier::verify` compares against that stored key, so every
 * signature from those validators fails, forever.
 *
 * `register_bridge_validator` refuses an account that is already registered
 * (`AlreadyExecuted`), and the PQ key is not updatable in place, so the only
 * route is remove -> re-register. Removal is gated by `GovernanceOrigin`
 * (`GovernanceCouncilMajority`), which includes Root, so sudo can do it.
 *
 * Usage
 * -----
 *   BRIDGE_SUDO_SEED=<seed> BRIDGE_VALIDATOR_SEEDS=<s1>,<s2>,<s3> \
 *   BRIDGE_ORACLE_SEEDS=<o1>,<o2>,<o3> \
 *     node scripts/rotate-bridge-pq-keys.js \
 *       --rpc ws://127.0.0.1:9944 \
 *       --pq-keys /secure/bridge-1.pub,/secure/bridge-2.pub,/secure/bridge-3.pub
 *
 * Options
 *   --rpc           WebSocket RPC endpoint (default ws://127.0.0.1:9944)
 *   --pq-keys       ML-DSA-87 public keys, one per validator, same order
 *   --pq-host       SSH target holding the matching `.pem` files (e.g.
 *                   wicked@ceiba). Required when the private keys are not on
 *                   this machine, which is the production case: the derived
 *                   public key is compared over SSH, so no private key is copied.
 *   --pq-remote-dir directory holding the `.pem` files on --pq-host
 *   --chain-indices `BridgeChain` discriminants to support (default 1 = Ethereum)
 *   --prune-stale   remove every registered validator that is NOT one of the
 *                   accounts given here. Required in practice: the placeholders
 *                   this script replaces were registered under other accounts,
 *                   so leaving them behind would create a validator set half of
 *                   which can never produce a verifiable signature.
 *   --dry-run       check every precondition and report, sign nothing
 *   --help          this text
 *
 * Environment (secrets never go on the command line; `ps` shows argv)
 *   BRIDGE_SUDO_SEED        a sudo-capable secret seed
 *   BRIDGE_VALIDATOR_SEEDS  comma-separated validator secret seeds
 *   BRIDGE_ORACLE_SEEDS     comma-separated oracle operator seeds, for the
 *                           L3 KYC quorum that registration requires
 *
 * Testnet only: it rewrites a live validator set.
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');
const {
  log,
  step,
  describeDispatchError,
  oracleKycLevel,
  seatOracleOperators,
  seedOracleKyc,
} = require('./lib/oracle-kyc');
const { assertKeypairMatches, readPqPublicKey } = require('./lib/mldsa');

// `pallet_belize_interoperability::BridgeChain::Ethereum`.
const BRIDGE_CHAIN_ETHEREUM = 1;

// Bridge validator registration requires Full (L3) KYC.
const BRIDGE_KYC_LEVEL = 3;

const list = (value) => (value || '').split(',').map((s) => s.trim()).filter(Boolean);

function parseArgs(argv) {
  const args = {
    rpc: 'ws://127.0.0.1:9944',
    pqKeys: [],
    pqHost: null,
    pqRemoteDir: null,
    chainIndices: [BRIDGE_CHAIN_ETHEREUM],
    pruneStale: false,
    dryRun: false,
    sudoSeed: process.env.BRIDGE_SUDO_SEED || null,
    validatorSeeds: list(process.env.BRIDGE_VALIDATOR_SEEDS),
    oracleSeeds: list(process.env.BRIDGE_ORACLE_SEEDS),
  };
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--rpc') args.rpc = argv[++i];
    else if (a === '--pq-keys') args.pqKeys = list(argv[++i]).map(readPqPublicKey);
    else if (a === '--pq-host') args.pqHost = argv[++i];
    else if (a === '--pq-remote-dir') args.pqRemoteDir = argv[++i];
    else if (a === '--prune-stale') args.pruneStale = true;
    else if (a === '--chain-indices') args.chainIndices = list(argv[++i]).map(Number);
    else if (a === '--dry-run') args.dryRun = true;
    else if (a === '--help' || a === '-h') args.help = true;
    else throw new Error(`unknown argument: ${a}`);
  }
  return args;
}

function usage() {
  console.log('usage: BRIDGE_SUDO_SEED=<seed> BRIDGE_VALIDATOR_SEEDS=<s1>,<s2> \\');
  console.log('       BRIDGE_ORACLE_SEEDS=<o1>,<o2>,<o3> \\');
  console.log('         node scripts/rotate-bridge-pq-keys.js --rpc ws://... \\');
  console.log('           --pq-keys <1.pub>,<2.pub> [--pq-host wicked@ceiba]');
  console.log('           [--pq-remote-dir /opt/belizechain/keys/bridge] [--dry-run]');
  console.log('           [--prune-stale]');
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) return usage();

  if (!args.sudoSeed) {
    throw new Error('missing sudo seed: set BRIDGE_SUDO_SEED');
  }
  if (args.validatorSeeds.length === 0) {
    throw new Error('missing validators: set BRIDGE_VALIDATOR_SEEDS');
  }
  if (args.pqKeys.length !== args.validatorSeeds.length) {
    throw new Error(
      `--pq-keys must pair up with validators (got ${args.pqKeys.length} keys `
      + `for ${args.validatorSeeds.length} seeds)`,
    );
  }
  if (args.oracleSeeds.length === 0) {
    // Registration calls `Identity::verify_bridge_operator`, which falls back to
    // the Oracle store. Without a quorum there is no way to reach L3.
    throw new Error('missing oracle operators: set BRIDGE_ORACLE_SEEDS');
  }

  // Preflight: every public key must have its private key on disk and they must
  // agree. Done before connecting, so a mismatched file cannot reach the chain.
  const fingerprints = args.pqKeys.map(
    (k) => assertKeypairMatches(k.path, k.hex, { host: args.pqHost, remoteDir: args.pqRemoteDir }),
  );
  if (new Set(fingerprints).size !== fingerprints.length) {
    throw new Error('two validators were given the same keypair; each needs its own');
  }

  await cryptoWaitReady();

  const api = await ApiPromise.create({ provider: new WsProvider(args.rpc) });
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 1981 });
  const sudo = keyring.addFromUri(args.sudoSeed);
  const validators = args.validatorSeeds.map((seed) => keyring.addFromUri(seed));
  const operators = args.oracleSeeds.map((seed) => keyring.addFromUri(seed));

  const chain = await api.rpc.system.chain();
  const header = await api.rpc.chain.getHeader();
  console.log('Bridge validator PQ key rotation');
  console.log(`  chain:      ${chain}`);
  console.log(`  head:       #${header.number.toNumber()}`);
  console.log(`  rpc:        ${args.rpc}`);
  console.log(`  sudo:       ${sudo.address}`);
  console.log(`  validators: ${validators.length}`);
  console.log(`  operators:  ${operators.length}`);
  if (args.dryRun) console.log('  MODE:       dry run — nothing will be signed');

  /** Sign and wait, or (dry run) report and continue. */
  const run = async (label, tx, signer = sudo) => {
    if (args.dryRun) {
      log(`${label}: would submit ${tx.method.section}.${tx.method.method}`);
      return;
    }
    return new Promise((resolve, reject) => {
      tx.signAndSend(signer, ({ status, dispatchError }) => {
        if (dispatchError) {
          reject(new Error(`${label}: ${describeDispatchError(dispatchError)}`));
          return;
        }
        if (status.isInBlock) {
          log(`${label}: in ${status.asInBlock.toHex().slice(0, 10)}…`);
          resolve();
        }
      }).catch(reject);
    });
  };

  // The registration extrinsic requires a seated operator set to seal KYC, so
  // seat first — and do it once rather than per validator.
  step('1. Seat oracle operators');
  await seatOracleOperators(api, run, operators);

  // Identify records outside the target set: the placeholders were registered
  // under their own accounts, so "replace" means "register new" AND "remove
  // old". The check is fail-fast — done before anything is signed — but the
  // removal itself happens after registration, so the validator set is never
  // empty in between and the bridge is never briefly unusable.
  step('2. Check for validators outside the target set');
  const targetAddresses = new Set(validators.map((v) => v.address));
  const stale = (await api.query.interoperability.bridgeValidators.keys())
    .map((key) => key.args[0].toString())
    .filter((address) => !targetAddresses.has(address));
  if (stale.length === 0) {
    log('no stale validators');
  } else if (!args.pruneStale) {
    // Refusing is deliberate: silently leaving unverifiable validators behind
    // would look like success while leaving the defect in place.
    throw new Error(
      `${stale.length} registered validator(s) are not in the target set:\n`
      + stale.map((a) => `    ${a}`).join('\n')
      + '\n  re-run with --prune-stale to remove them',
    );
  } else {
    log(`${stale.length} to remove after registration:`);
    for (const address of stale) log(`  ${address}`);
  }

  for (const [i, validator] of validators.entries()) {
    const label = validator.address;
    const pqKey = args.pqKeys[i].hex;
    step(`3.${i + 1} ${label}  (key ${fingerprints[i]}…, private key present)`);

    const existing = await api.query.interoperability.bridgeValidators(validator.address);
    if (existing.isSome) {
      const current = existing.unwrap();
      const currentKey = `0x${Buffer.from(current.pqPublicKey.toU8a(true)).toString('hex')}`;
      if (currentKey.toLowerCase() === pqKey.toLowerCase()) {
        // Converging re-run: the record already carries this key. Re-writing it
        // would need a remove first, which is a live-authority change for no gain.
        log('already registered with this exact key — nothing to do');
        continue;
      }
      log(`existing record has a different key (${current.pqPublicKey.length} bytes)`);
      await run(
        'remove placeholder record',
        api.tx.sudo.sudo(api.tx.interoperability.removeBridgeValidator(validator.address)),
      );
      if (args.dryRun) {
        log('would re-register after removal (dry run)');
        continue;
      }
    }

    // Registration calls `verify_bridge_operator`, which reads the Oracle store.
    await seedOracleKyc(api, run, validator, operators, BRIDGE_KYC_LEVEL, args.dryRun);

    await run(
      'register with real ML-DSA-87 key',
      api.tx.interoperability.registerBridgeValidator(pqKey, args.chainIndices),
      validator,
    );

    if (args.dryRun) continue;

    // Assert the write, rather than trusting the extrinsic's success.
    const after = await api.query.interoperability.bridgeValidators(validator.address);
    if (after.isNone) {
      throw new Error(`${label}: registration reported success but no record exists`);
    }
    const stored = `0x${Buffer.from(after.unwrap().pqPublicKey.toU8a(true)).toString('hex')}`;
    if (stored.toLowerCase() !== pqKey.toLowerCase()) {
      throw new Error(`${label}: stored key does not match the key supplied`);
    }
    log(`verified: stored key matches, ${after.unwrap().pqPublicKey.length} bytes`);
  }

  step('4. Remove validators outside the target set');
  if (stale.length === 0) {
    log('nothing to remove');
  } else {
    for (const address of stale) {
      // Report the lock before it is released: `remove_bridge_validator` calls
      // `Currency::remove_lock`, and for an account whose secret is not held,
      // that stake becomes unspendable rather than recovered. Saying so here is
      // the only record that the value moved out of play.
      const locks = await api.query.balances.locks(address);
      log(`${address}  releases ${locks.map((l) => l.amount.toString()).join(', ') || 'no'} locks`);
      await run(
        `remove ${address}`,
        api.tx.sudo.sudo(api.tx.interoperability.removeBridgeValidator(address)),
      );
    }
  }

  step('5. Summary');
  const entries = await api.query.interoperability.bridgeValidators.entries();
  log(`registered bridge validators: ${entries.length}`);
  for (const [key, value] of entries) {
    const record = value.unwrap();
    const address = key.args[0].toString();
    const expected = fingerprints[validators.findIndex((v) => v.address === address)];
    log(`  ${address}  kyc-L${await oracleKycLevel(api, address)}`
      + `  pk=${record.pqPublicKey.length}B`
      + `${expected ? ` key=${expected}…` : ' key=not-supplied-here'}`);
  }

  await api.disconnect();
}

main().catch((e) => {
  console.error(`\nFAILED: ${e.message}`);
  process.exit(1);
});
