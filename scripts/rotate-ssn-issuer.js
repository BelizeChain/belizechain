#!/usr/bin/env node
/**
 * Move SSN issuance off a compromised account.
 *
 * Why this is part of the same incident
 * -------------------------------------
 * Rotating `Sudo.Key` and the consensus authority closes the two routes that let
 * an attacker *change the chain*. They do not close the route that lets them
 * *forge identity data*: `identity.SsnIssuers` is a separate allow-list, and while
 * a published dev key sits on it, anyone can issue SSN attestations — which is the
 * first rung of the KYC ladder that gates trading, bridge operation and credit.
 *
 * On the live chain that list held exactly one account, and it was the same
 * `//Alice` key. This hands the role to an account whose secret is held on the
 * host, keeping the list at one entry so nothing about *how many* issuers exist
 * changes — only who they are.
 *
 * Order matters
 * -------------
 * Add the replacement **before** removing the old one. Doing it the other way
 * leaves the chain with zero issuers for as long as the second extrinsic takes,
 * which is a window in which no legitimate identity can be onboarded.
 *
 * `add_issuer` requires the replacement to have posted `IssuerBondAmount`, and is
 * gated on `TechnicalCouncilSuperMajority`, whose `EitherOfDiverse` includes
 * `EnsureRoot` — so sudo satisfies it without a council vote.
 *
 * Usage
 * -----
 *   NODE_PATH=<repo>/ui/node_modules node scripts/rotate-ssn-issuer.js \
 *     --rpc wss://testnet.belizechain.org/ws \
 *     --sudo-seed <suri> --issuer-seed <suri> --old-issuer <ss58> [--dry-run]
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, decodeAddress } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');

const SS58_PREFIX = 1981;

/**
 * `pallet_belize_identity::AttributeType::Ssn`.
 *
 * The runtime is inconsistent about how this variant is passed, so both forms
 * are named here rather than left as bare literals at the call sites:
 * `issuerDepositBond` / `addIssuer` / `removeIssuer` take the raw `u8` index,
 * while `IssuerBonds` is a double map keyed on the enum, which polkadot-js can
 * only resolve from the variant name.
 */
const ATTR_SSN_INDEX = 0;
const ATTR_SSN = 'Ssn';

function parseArgs(argv) {
  const args = { rpc: null, sudoSeed: null, issuerSeed: null, oldIssuer: null, dryRun: false };
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--rpc') args.rpc = argv[++i];
    else if (a === '--sudo-seed') args.sudoSeed = argv[++i];
    else if (a === '--issuer-seed') args.issuerSeed = argv[++i];
    else if (a === '--old-issuer') args.oldIssuer = argv[++i];
    else if (a === '--dry-run') args.dryRun = true;
    else if (a === '--help' || a === '-h') args.help = true;
    else throw new Error(`unknown argument: ${a}`);
  }
  return args;
}

const log = (msg) => console.log(`  ${msg}`);
const step = (msg) => console.log(`\n${msg}`);
const hex = (value) => u8aToHex(decodeAddress(value));

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

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) return null;
  for (const [flag, value] of Object.entries({
    '--rpc': args.rpc,
    '--sudo-seed': args.sudoSeed,
    '--issuer-seed': args.issuerSeed,
    '--old-issuer': args.oldIssuer,
  })) {
    if (!value) throw new Error(`${flag} is required`);
  }

  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519', ss58Format: SS58_PREFIX });
  const sudo = keyring.addFromUri(args.sudoSeed);
  const issuer = keyring.addFromUri(args.issuerSeed);

  const api = await ApiPromise.create({ provider: new WsProvider(args.rpc), noInitWarn: true });
  try {
    console.log('SSN issuer handover');
    log(`chain : ${(await api.rpc.system.chain()).toString()}`);
    log(`sudo  : ${sudo.address}`);
    log(`issuer: ${issuer.address}`);
    log(`old   : ${args.oldIssuer}`);

    const sudoKey = (await api.query.sudo.key()).toString();
    if (hex(sudoKey) !== hex(sudo.address)) {
      throw new Error(`Sudo.Key is ${sudoKey}, but --sudo-seed derives ${sudo.address}`);
    }

    const issuers = (await api.query.identity.ssnIssuers()).toJSON();
    log(`current issuers: ${JSON.stringify(issuers)}`);
    if (!issuers.some((i) => hex(i) === hex(args.oldIssuer))) {
      throw new Error(`${args.oldIssuer} is not a current SSN issuer; nothing to remove`);
    }
    if (issuers.some((i) => hex(i) === hex(issuer.address))) {
      throw new Error(`${issuer.address} is already an SSN issuer`);
    }

    step('1. Bond for the replacement issuer');
    const required = BigInt((await api.query.identity.issuerBondAmount()).toString());
    const bonded = BigInt(
      (await api.query.identity.issuerBonds(ATTR_SSN, issuer.address)).toString(),
    );
    log(`bond required ${required}, currently ${bonded}`);
    if (bonded < required) {
      const free = BigInt((await api.query.system.account(issuer.address)).data.free.toString());
      log(`issuer free balance ${free}`);
      if (free < required) {
        throw new Error(
          `issuer holds ${free} but the bond is ${required}. Fund it first — this script will not `
          + 'mint, because an issuer bond is a deposit, not a supply change.',
        );
      }
      if (args.dryRun) log('would issuerDepositBond');
      else {
        await submit(
          api,
          api.tx.identity.issuerDepositBond(ATTR_SSN_INDEX),
          issuer,
          'issuerDepositBond',
        );
      }
    }

    step('2. Add the replacement (before removing the old one)');
    if (args.dryRun) log('would sudo(addIssuer)');
    else {
      await submit(
        api,
        api.tx.sudo.sudo(api.tx.identity.addIssuer(ATTR_SSN_INDEX, issuer.address)),
        sudo,
        'addIssuer',
      );
    }

    step('3. Remove the compromised issuer');
    if (args.dryRun) {
      log('would sudo(removeIssuer)');
      console.log('\n  dry run: no change made');
      return null;
    }
    await submit(
      api,
      api.tx.sudo.sudo(api.tx.identity.removeIssuer(ATTR_SSN_INDEX, args.oldIssuer)),
      sudo,
      'removeIssuer',
    );

    step('4. Verify');
    const after = (await api.query.identity.ssnIssuers()).toJSON();
    log(`issuers now: ${JSON.stringify(after)}`);
    if (after.some((i) => hex(i) === hex(args.oldIssuer))) {
      throw new Error(`${args.oldIssuer} is still an SSN issuer`);
    }
    if (!after.some((i) => hex(i) === hex(issuer.address))) {
      throw new Error(`${issuer.address} did not become an SSN issuer`);
    }
    if (after.length === 0) {
      throw new Error('issuer list is now empty; no identity can be onboarded');
    }

    console.log('\nPASS: SSN issuance has moved off the compromised account.');
    return after;
  } finally {
    await api.disconnect();
  }
}

main().catch((e) => {
  console.error(`\nFAILED: ${e.message}`);
  process.exit(1);
});
