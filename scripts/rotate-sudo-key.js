#!/usr/bin/env node
/**
 * Rotate `Sudo.Key` away from a compromised dev key.
 *
 * Why this exists
 * ---------------
 * `pallet_sudo::set_key` is the one remediation for blocker #13 that is both
 * complete and low-risk: it is a single extrinsic, it needs no runtime upgrade,
 * no restart, and no epoch wait, and it is reversible by whoever holds the new
 * key. Everything else in that blocker (the BABE/GRANDPA authority set) is a
 * longer, riskier operation and is handled separately.
 *
 * The dangerous property of `pallet_sudo` is that it is only as private as
 * `Sudo.Key`. On the live chain that key was `//Alice`, whose mnemonic is in
 * every Substrate tutorial, so `sudo.sudo(System::set_code(..))` was available
 * to anyone who read one storage item over a public, unauthenticated RPC.
 *
 * Guards, because this is a live-chain write
 * ------------------------------------------
 * - It reads `Sudo.Key` first and aborts unless the account derived from
 *   `--current-seed` actually is the current sudo key. Running this against the
 *   wrong chain, or with a stale seed, would otherwise submit an extrinsic that
 *   fails with `RequireSudo` and look like a network problem.
 * - It verifies `Sudo.Key` afterwards and exits non-zero if the rotation did not
 *   take, rather than reporting success on a mined-but-failed extrinsic.
 * - `--current-seed` has no default. There is no "obvious" key to fall back to,
 *   and a default would make an accidental mainnet run possible.
 *
 * Usage
 * -----
 *   NODE_PATH=<repo>/ui/node_modules node scripts/rotate-sudo-key.js \
 *     --rpc wss://testnet.belizechain.org/ws \
 *     --current-seed '//Alice' \
 *     --new-key 5DD2xP514R5ZprsrPYaDaSYaCh1Pm2DBh1PdeX2sz2oYQ3yW
 *
 * `--new-key` accepts either SS58 or a 0x-prefixed 32-byte public key. Generate
 * the replacement **on the host that will hold it** and never transmit its
 * secret: `belizechain-node key generate --scheme sr25519 --output-type json`.
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, decodeAddress } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');

const SS58_PREFIX = 1981;

function parseArgs(argv) {
  const args = { rpc: null, currentSeed: null, newKey: null, dryRun: false };
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--rpc') args.rpc = argv[++i];
    else if (a === '--current-seed') args.currentSeed = argv[++i];
    else if (a === '--new-key') args.newKey = argv[++i];
    else if (a === '--dry-run') args.dryRun = true;
    else if (a === '--help' || a === '-h') args.help = true;
    else throw new Error(`unknown argument: ${a}`);
  }
  return args;
}

function usage() {
  console.log('usage: node scripts/rotate-sudo-key.js --rpc <url> --current-seed <uri> --new-key <ss58|0xhex>');
}

/** Turn a dispatch error into the pallet's own name and doc string. */
function describeDispatchError(dispatchError) {
  if (!dispatchError.isModule) return dispatchError.toString();
  const meta = dispatchError.asModule.registry.findMetaError(dispatchError.asModule);
  return `${meta.section}.${meta.name}: ${meta.docs.join(' ').trim()}`;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) return usage();
  if (!args.rpc || !args.currentSeed || !args.newKey) {
    usage();
    throw new Error('--rpc, --current-seed and --new-key are all required');
  }

  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519', ss58Format: SS58_PREFIX });
  const signer = keyring.addFromUri(args.currentSeed);

  const api = await ApiPromise.create({ provider: new WsProvider(args.rpc), noInitWarn: true });
  try {
    const chain = await api.rpc.system.chain();
    const head = await api.rpc.chain.getHeader();
    console.log('Sudo key rotation');
    console.log(`  chain:  ${chain}`);
    console.log(`  head:   #${head.number.toNumber()}`);
    console.log(`  rpc:    ${args.rpc}`);
    console.log(`  signer: ${signer.address}`);

    // Fail closed: the whole point is that this key is compromised, so a
    // mismatch might mean someone else already rotated it. Stop and let a
    // human look rather than fighting them for the slot.
    const current = (await api.query.sudo.key()).toString();
    const signerHex = u8aToHex(signer.publicKey);
    const currentHex = u8aToHex(decodeAddress(current));
    if (currentHex !== signerHex) {
      throw new Error(
        `Sudo.Key is ${current}, but --current-seed derives ${signer.address}. ` +
        'Refusing to submit: either the key was already rotated, or this is the wrong chain.',
      );
    }

    const newAccount = keyring.encodeAddress(decodeAddress(args.newKey));
    const newHex = u8aToHex(decodeAddress(args.newKey));
    if (newHex === currentHex) {
      throw new Error('--new-key is the current sudo key; nothing to do');
    }

    console.log(`  from:   ${current}`);
    console.log(`  to:     ${newAccount}`);
    console.log(`  new pk: ${newHex}`);

    if (args.dryRun) {
      console.log('\n  dry run: would submit sudo.sudo(sudo.setKey(..))');
      return null;
    }

    const tx = api.tx.sudo.sudo(api.tx.sudo.setKey(newAccount));
    const status = await new Promise((resolve, reject) => {
      let unsubscribe = null;
      tx.signAndSend(signer, ({ status: s, dispatchError }) => {
        if (dispatchError) {
          if (unsubscribe) unsubscribe();
          reject(new Error(describeDispatchError(dispatchError)));
          return;
        }
        if (s.isInBlock || s.isFinalized) {
          if (unsubscribe) unsubscribe();
          resolve(s);
        }
      })
        .then((handle) => { unsubscribe = handle; })
        .catch((e) => reject(new Error(e.message)));
    });
    console.log(`  included in ${status.isInBlock ? status.asInBlock.toHex() : 'block'}`);

    // Trust the chain, not the extrinsic's inclusion: a transaction can be
    // included and still not have moved the key.
    const after = (await api.query.sudo.key()).toString();
    console.log(`  Sudo.Key is now ${after}`);
    if (u8aToHex(decodeAddress(after)) !== newHex) {
      throw new Error(`rotation did not take: Sudo.Key is still ${after}`);
    }

    console.log('\nPASS: Sudo.Key rotated. The former dev key can no longer call sudo.');
    return after;
  } finally {
    await api.disconnect();
  }
}

main().catch((e) => {
  console.error(`\nFAILED: ${e.message}`);
  process.exit(1);
});
