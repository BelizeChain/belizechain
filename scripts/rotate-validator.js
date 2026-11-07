#!/usr/bin/env node
/**
 * Move the consensus authority off a compromised account, without a re-genesis.
 *
 * Why this is not just `session.setKeys`
 * --------------------------------------
 * The live authority set contained one account whose secret is public. Rotating
 * only its session keys would change nothing durable: `session.set_keys` is
 * authorised by the *account*, which the attacker also holds, so they could
 * rotate the keys straight back. The account itself has to leave the set.
 *
 * How the set is actually built
 * -----------------------------
 * `BelizeSessionManager::new_session` (runtime/src/lib.rs) enumerates
 * `staking.Validators`, drops any entry with no `Session.NextKeys`, and — only
 * if that leaves nothing — falls back to the *current* set. The compromised
 * account is in the authority set purely through that fallback: it is not a
 * staking validator at all. So registering one new, properly-keyed staking
 * validator makes the non-empty branch win and the fallback stops being taken.
 * The old account disappears at the next session with no `leave_validators`
 * call and no unbonding wait.
 *
 * Why this cannot deadlock the chain
 * ----------------------------------
 * The switch is gated on `NextKeys` existing. If `set_keys` has not landed by
 * the session boundary, the new validator is filtered out, the set is empty,
 * and the fallback returns the current set — i.e. nothing changes and the next
 * session tries again. The dangerous state (a set naming a validator with no
 * keys) requires `set_keys` to succeed while the keystore lacks the key, which
 * is why this script checks the keystore precondition out of band and refuses
 * to continue if it cannot confirm it.
 *
 * Preconditions this script cannot check for you
 * ----------------------------------------------
 * The session keys must already be in the node's keystore. Insert them on the
 * host that runs the node, while it is running:
 *
 *   belizechain-node key insert --chain <spec> \
 *     --keystore-path <base>/chains/<id>/keystore \
 *     --scheme Sr25519 --suri "<suri>" --key-type babe
 *   belizechain-node key insert --chain <spec> \
 *     --keystore-path <base>/chains/<id>/keystore \
 *     --scheme Ed25519 --suri "<suri>" --key-type gran
 *
 * `key insert` loads a chain spec, and the fail-closed mainnet config rejects a
 * default run, so `--chain` is mandatory rather than optional.
 *
 * The `set_keys` proof is not optional
 * ------------------------------------
 * This runtime's `SessionKeys` uses the `impl_opaque_keys!` default, whose
 * `ownership_proof_is_valid` requires a SCALE tuple of one `ProofOfPossession`
 * per key — a signature by each session key over the account id — in struct
 * field order. `0x` fails with `session.InvalidProof`, so both seeds are needed
 * to build it, not just the account.
 *
 * Usage
 * -----
 *   NODE_PATH=<repo>/ui/node_modules node scripts/rotate-validator.js \
 *     --rpc ws://127.0.0.1:9944 \
 *     --sudo-seed <uri> --validator-seed <uri> --grandpa-seed <uri> \
 *     --babe 0x<32-byte pub> --grandpa 0x<32-byte pub> \
 *     --wait-epoch
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');
const { u8aConcat, u8aToHex } = require('@polkadot/util');

const SS58_PREFIX = 1981;

/** `MinValidatorStake` on the live runtime; used only as the default. */
const DEFAULT_STAKE = 1_000_000_000_000_000n;

/** Headroom above the stake so the account can pay fees for a long time. */
const FEE_HEADROOM = 10_000_000_000_000n;

const DEFAULT_COMPUTE_CAPACITY = 100;

/** Hex of "Belize City", matching the existing validator's record. */
const DEFAULT_LOCATION = '0x42656c697a652043697479';

const EPOCH_POLL_MS = 15_000;

/**
 * Context tag `sp_core::proof_of_possession::statement_of_ownership` prepends to
 * the owner before signing. The runtime verifies against `POP_ ++ owner`.
 */
const POP_CONTEXT_TAG = 'POP_';

function parseArgs(argv) {
  const args = {
    rpc: null,
    sudoSeed: null,
    validatorSeed: null,
    grandpaSeed: null,
    babe: null,
    grandpa: null,
    stake: DEFAULT_STAKE,
    computeCapacity: DEFAULT_COMPUTE_CAPACITY,
    location: DEFAULT_LOCATION,
    waitEpoch: false,
    dryRun: false,
    timeoutMinutes: 45,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--rpc') args.rpc = argv[++i];
    else if (a === '--sudo-seed') args.sudoSeed = argv[++i];
    else if (a === '--validator-seed') args.validatorSeed = argv[++i];
    else if (a === '--grandpa-seed') args.grandpaSeed = argv[++i];
    else if (a === '--babe') args.babe = argv[++i];
    else if (a === '--grandpa') args.grandpa = argv[++i];
    else if (a === '--stake') args.stake = BigInt(argv[++i]);
    else if (a === '--compute') args.computeCapacity = Number(argv[++i]);
    else if (a === '--location') args.location = argv[++i];
    else if (a === '--wait-epoch') args.waitEpoch = true;
    else if (a === '--dry-run') args.dryRun = true;
    else if (a === '--timeout-minutes') args.timeoutMinutes = Number(argv[++i]);
    else if (a === '--help' || a === '-h') args.help = true;
    else throw new Error(`unknown argument: ${a}`);
  }
  return args;
}

function usage() {
  console.log('usage: node scripts/rotate-validator.js --rpc <url> --sudo-seed <uri> \\\n'
    + '         --validator-seed <uri> --grandpa-seed <uri> --babe 0x.. --grandpa 0x.. [--wait-epoch]');
}

const log = (msg) => console.log(`  ${msg}`);
const step = (msg) => console.log(`\n${msg}`);
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

function describeDispatchError(dispatchError) {
  if (!dispatchError.isModule) return dispatchError.toString();
  const meta = dispatchError.asModule.registry.findMetaError(dispatchError.asModule);
  return `${meta.section}.${meta.name}: ${meta.docs.join(' ').trim()}`;
}

async function submit(api, tx, signer, label) {
  await new Promise((resolve, reject) => {
    let unsubscribe = null;
    tx.signAndSend(signer, ({ status, dispatchError }) => {
      if (dispatchError) {
        if (unsubscribe) unsubscribe();
        reject(new Error(`${label}: ${describeDispatchError(dispatchError)}`));
        return;
      }
      if (status.isInBlock || status.isFinalized) {
        if (unsubscribe) unsubscribe();
        log(`${label}: ok`);
        resolve();
      }
    })
      .then((handle) => { unsubscribe = handle; })
      .catch((e) => reject(new Error(`${label}: ${e.message}`)));
  });
}

/** Normalise a hex public key for comparison against a decoded storage value. */
const hex = (bytes) => (typeof bytes === 'string' ? bytes.toLowerCase() : bytes.toHex().toLowerCase());

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) return usage();
  for (const [flag, value] of Object.entries({
    '--rpc': args.rpc,
    '--sudo-seed': args.sudoSeed,
    '--validator-seed': args.validatorSeed,
    '--grandpa-seed': args.grandpaSeed,
    '--babe': args.babe,
    '--grandpa': args.grandpa,
  })) {
    if (!value) {
      usage();
      throw new Error(`${flag} is required`);
    }
  }

  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519', ss58Format: SS58_PREFIX });
  const sudo = keyring.addFromUri(args.sudoSeed);
  const validator = keyring.addFromUri(args.validatorSeed);
  const grandpaKeyring = new Keyring({ type: 'ed25519', ss58Format: SS58_PREFIX });
  const grandpaPair = grandpaKeyring.addFromUri(args.grandpaSeed);

  // `ownership_proof_is_valid` verifies one proof of possession per session key,
  // in the struct's field order (babe then grandpa). Both sign the *ownership
  // statement*, not the bare account id: `sp_core::proof_of_possession::
  // statement_of_ownership` prefixes the owner with the 4-byte context tag
  // `POP_`, so signing the account id alone produces a valid-looking signature
  // that the runtime rejects with `session.InvalidProof`. Fixed-size signatures
  // encode with no length prefix, so the tuple is just the two concatenated.
  const owner = validator.publicKey;
  const statement = u8aConcat(new TextEncoder().encode(POP_CONTEXT_TAG), owner);
  const ownershipProof = u8aToHex(u8aConcat(validator.sign(statement), grandpaPair.sign(statement)));

  const api = await ApiPromise.create({ provider: new WsProvider(args.rpc), noInitWarn: true });
  try {
    console.log('Validator rotation');
    console.log(`  chain: ${(await api.rpc.system.chain()).toString()}`);
    console.log(`  head : #${(await api.rpc.chain.getHeader()).number.toNumber()}`);
    log(`sudo signer : ${sudo.address}`);
    log(`new validator: ${validator.address}`);
    log(`babe         : ${args.babe}`);
    log(`grandpa      : ${args.grandpa}`);

    // Same fail-closed guard as the sudo rotation: if the signer is not the
    // sudo key, every Root call below fails with an opaque error.
    const sudoKey = (await api.query.sudo.key()).toString();
    if (sudoKey !== sudo.address) {
      throw new Error(`Sudo.Key is ${sudoKey}, but --sudo-seed derives ${sudo.address}`);
    }

    const desiredKeys = { babe: args.babe, grandpa: args.grandpa };
    const required = args.stake + FEE_HEADROOM;

    step('1. Fund the new validator');
    const free = BigInt((await api.query.system.account(validator.address)).data.free.toString());
    log(`free: ${free} (needs ${required})`);
    if (free < required) {
      if (args.dryRun) log('would forceSetBalance');
      else await submit(api, api.tx.sudo.sudo(
        api.tx.balances.forceSetBalance(validator.address, required),
      ), sudo, 'forceSetBalance');
    } else {
      log('already funded');
    }

    step('2. Register as a staking validator');
    const existing = await api.query.staking.validators(validator.address);
    if (existing.isSome) {
      log('already a staking validator');
    } else if (args.dryRun) {
      log('would forceJoinValidator');
    } else {
      await submit(api, api.tx.sudo.sudo(api.tx.staking.forceJoinValidator(
        validator.address,
        args.stake,
        args.computeCapacity,
        args.location,
      )), sudo, 'forceJoinValidator');
    }

    step('3. Register session keys');
    const current = await api.query.session.nextKeys(validator.address);
    const currentJson = current.isSome ? current.unwrap().toJSON() : null;
    log(`current: ${JSON.stringify(currentJson)}`);
    if (currentJson && hex(currentJson.babe) === hex(args.babe) && hex(currentJson.grandpa) === hex(args.grandpa)) {
      log('already set');
    } else if (args.dryRun) {
      log(`would session.setKeys with a ${(ownershipProof.length - 2) / 2}-byte ownership proof`);
    } else {
      await submit(api, api.tx.session.setKeys(desiredKeys, ownershipProof), validator, 'setKeys');
      const after = await api.query.session.nextKeys(validator.address);
      const afterJson = after.isSome ? after.unwrap().toJSON() : null;
      if (!afterJson || hex(afterJson.babe) !== hex(args.babe) || hex(afterJson.grandpa) !== hex(args.grandpa)) {
        throw new Error(`setKeys did not take: ${JSON.stringify(afterJson)}`);
      }
      log(`nextKeys confirmed: ${JSON.stringify(afterJson)}`);
    }

    if (args.dryRun) {
      console.log('\n  dry run: no authority change observed');
      return null;
    }

    if (!args.waitEpoch) {
      console.log('\n  Registered. The authority set changes one session boundary later — '
        + 'run again with --wait-epoch to watch it happen.');
      return null;
    }

    step('4. Wait for the authority switch');
    // A validator registered mid-session does NOT take over at the next boundary.
    // `SessionManager::new_session` computes the set for the *following* session
    // and `rotate_session` enacts the set queued earlier, so the change lands one
    // session later than intuition suggests. Watching the set directly — rather
    // than counting boundaries — is what makes this correct either way; the queued
    // set is printed alongside so a stuck queue is visible as such instead of
    // looking like a silent no-op.
    const startSet = (await api.query.session.validators()).toJSON();
    log(`current set ${JSON.stringify(startSet)}`);
    log(`waiting up to ${args.timeoutMinutes} min for ${validator.address} to take over`);

    const deadline = Date.now() + args.timeoutMinutes * 60_000;
    let switched = false;
    let lastEpoch = (await api.query.babe.epochIndex()).toString();
    while (Date.now() < deadline) {
      const nowSet = (await api.query.session.validators()).toJSON();
      if (nowSet.length === 1 && nowSet[0] === validator.address) {
        switched = true;
        log(`authority set ${JSON.stringify(nowSet)}`);
        break;
      }

      const nowEpoch = (await api.query.babe.epochIndex()).toString();
      if (nowEpoch !== lastEpoch) {
        lastEpoch = nowEpoch;
        const queued = (await api.query.session.queuedKeys()).toJSON();
        log(`epoch ${nowEpoch}: set ${JSON.stringify(nowSet)}, queued ${JSON.stringify(queued.map((k) => k[0]))}`);
      }
      await sleep(EPOCH_POLL_MS);
    }

    if (!switched) {
      const queued = (await api.query.session.queuedKeys()).toJSON();
      throw new Error(
        `the new validator did not become the authority within ${args.timeoutMinutes} min. `
        + `Current set ${JSON.stringify((await api.query.session.validators()).toJSON())}, `
        + `queued ${JSON.stringify(queued.map((k) => k[0]))}. `
        + 'If it is queued, the chain is healthy and one more boundary will apply it; '
        + 'if the queue is empty, the keystore key is probably missing.',
      );
    }

    step('5. Confirm the chain is still producing and finalizing');
    const before = await api.rpc.chain.getHeader();
    await sleep(EPOCH_POLL_MS);
    const after = await api.rpc.chain.getHeader();
    const finalized = await api.rpc.chain.getFinalizedHead();
    const finalizedHeader = await api.rpc.chain.getHeader(finalized);
    log(`best ${before.number.toNumber()} -> ${after.number.toNumber()}`);
    log(`finalized #${finalizedHeader.number.toNumber()}`);

    if (after.number.toNumber() <= before.number.toNumber()) {
      throw new Error('best block did not advance after the switch');
    }
    if (finalizedHeader.number.toNumber() < after.number.toNumber() - 10) {
      throw new Error(`finality is lagging: best ${after.number.toNumber()}, finalized ${finalizedHeader.number.toNumber()}`);
    }

    console.log(`\nPASS: the new validator is the sole authority at ${validator.address}, `
      + 'and the chain is producing and finalizing.');
    return validator.address;
  } finally {
    await api.disconnect();
  }
}

main().catch((e) => {
  console.error(`\nFAILED: ${e.message}`);
  process.exit(1);
});
