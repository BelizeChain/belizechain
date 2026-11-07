#!/usr/bin/env node
/**
 * Move identity-issuer authority off a compromised account.
 *
 * Why this is part of the sudo incident
 * -------------------------------------
 * Rotating `Sudo.Key` and the consensus authority closes the two routes that let
 * an attacker *change the chain*. They do not close the route that lets them
 * *forge identity data*. `pallet_belize_identity` keeps three independent
 * allow-lists — `SsnIssuers`, `PassportIssuers`, `BiometricIssuers` — and an
 * account on any of them can attest that attribute for any subject. Attestation
 * is the rung of the KYC ladder that gates trading, bridge operation and credit,
 * so a published dev key on those lists is a forgery capability that needs no
 * sudo at all.
 *
 * On the live chain each list held exactly one account, and it was the same
 * `//Alice` key. Discovery order matters here: `SsnIssuers` was found first and
 * fixed on its own, and only a later sweep of every privileged storage entry
 * turned up the other two. Hence one script for all three, and a `--attrs` flag
 * so a partial run is a deliberate choice rather than an oversight.
 *
 * Each attribute carries its **own** bond (`IssuerBonds` is keyed on
 * `(attr, account)`), so covering all three costs three bonds.
 *
 * Order matters
 * -------------
 * Add the replacement **before** removing the old one, per attribute. The reverse
 * order leaves that list empty until the second extrinsic lands, and an empty
 * list means no legitimate attestation of that kind can be issued.
 *
 * `add_issuer` requires the replacement to have posted `IssuerBondAmount`, and is
 * gated on `TechnicalCouncilSuperMajority`, whose `EitherOfDiverse` includes
 * `EnsureRoot` — so sudo satisfies it without a council vote.
 *
 * Usage
 * -----
 *   NODE_PATH=<repo>/ui/node_modules node scripts/rotate-identity-issuer.js \
 *     --rpc wss://testnet.belizechain.org/ws \
 *     --sudo-seed <suri> --issuer-seed <suri> --old-issuer <ss58> \
 *     [--attrs Ssn,Passport,Biometrics] [--dry-run]
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, decodeAddress } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');

const SS58_PREFIX = 1981;

/**
 * `pallet_belize_identity::AttributeType`, in declaration order.
 *
 * The runtime is inconsistent about how a variant is passed, so both forms are
 * named here rather than left as bare literals at the call sites:
 * `issuerDepositBond` / `addIssuer` / `removeIssuer` take the raw `u8` index,
 * while `IssuerBonds` is a double map keyed on the enum, which polkadot-js can
 * only resolve from the variant name.
 */
const ATTRS = Object.freeze([
  { index: 0, name: 'Ssn', list: 'ssnIssuers' },
  { index: 1, name: 'Passport', list: 'passportIssuers' },
  { index: 2, name: 'Biometrics', list: 'biometricIssuers' },
]);

const ALL_ATTR_NAMES = ATTRS.map((a) => a.name);

function parseArgs(argv) {
  const args = {
    rpc: null,
    sudoSeed: null,
    issuerSeed: null,
    oldIssuer: null,
    attrs: [...ALL_ATTR_NAMES],
    dryRun: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const flag = argv[i];
    if (flag === '--rpc') args.rpc = argv[++i];
    else if (flag === '--sudo-seed') args.sudoSeed = argv[++i];
    else if (flag === '--issuer-seed') args.issuerSeed = argv[++i];
    else if (flag === '--old-issuer') args.oldIssuer = argv[++i];
    else if (flag === '--attrs') args.attrs = argv[++i].split(',').map((s) => s.trim()).filter(Boolean);
    else if (flag === '--dry-run') args.dryRun = true;
    else if (flag === '--help' || flag === '-h') args.help = true;
    else throw new Error(`unknown argument: ${flag}`);
  }
  return args;
}

const log = (msg) => console.log(`  ${msg}`);
const step = (msg) => console.log(`\n${msg}`);
const hex = (value) => u8aToHex(decodeAddress(value));
const sameAccount = (a, b) => hex(a) === hex(b);

function resolveAttrs(names) {
  const unknown = names.filter((name) => !ALL_ATTR_NAMES.includes(name));
  if (unknown.length > 0) {
    throw new Error(`unknown attribute(s) ${unknown.join(', ')}; expected one of ${ALL_ATTR_NAMES.join(', ')}`);
  }
  return ATTRS.filter((attr) => names.includes(attr.name));
}

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

/** Attributes whose allow-list currently contains `account`. */
async function heldAttributes(api, account) {
  const held = [];
  for (const attr of ATTRS) {
    const holders = (await api.query.identity[attr.list]()).toJSON() ?? [];
    if (holders.some((entry) => sameAccount(entry, account))) held.push(attr);
  }
  return held;
}

async function ensureBond(api, attr, issuer, { dryRun }) {
  const required = BigInt((await api.query.identity.issuerBondAmount()).toString());
  const bonded = BigInt(
    (await api.query.identity.issuerBonds(attr.name, issuer.address)).toString(),
  );
  log(`${attr.name}: bond required ${required}, currently ${bonded}`);
  if (required === 0n || bonded >= required) return;

  const free = BigInt((await api.query.system.account(issuer.address)).data.free.toString());
  log(`${attr.name}: issuer free balance ${free}`);
  if (free < required) {
    throw new Error(
      `issuer holds ${free} but the ${attr.name} bond is ${required}. Fund it first — this `
      + 'script will not mint, because an issuer bond is a deposit, not a supply change.',
    );
  }
  if (dryRun) {
    log(`${attr.name}: would issuerDepositBond`);
    return;
  }
  await submit(api, api.tx.identity.issuerDepositBond(attr.index), issuer, `issuerDepositBond(${attr.name})`);
}

async function handOver(api, attr, { sudo, issuer, oldIssuer, dryRun }) {
  step(`${attr.name}`);

  await ensureBond(api, attr, issuer, { dryRun });

  if (dryRun) {
    log('would sudo(addIssuer), then sudo(removeIssuer)');
    return { attr, skipped: true };
  }

  await submit(
    api,
    api.tx.sudo.sudo(api.tx.identity.addIssuer(attr.index, issuer.address)),
    sudo,
    `addIssuer(${attr.name})`,
  );
  await submit(
    api,
    api.tx.sudo.sudo(api.tx.identity.removeIssuer(attr.index, oldIssuer)),
    sudo,
    `removeIssuer(${attr.name})`,
  );

  const after = (await api.query.identity[attr.list]()).toJSON() ?? [];
  log(`${attr.name} issuers now: ${JSON.stringify(after)}`);
  if (after.some((entry) => sameAccount(entry, oldIssuer))) {
    throw new Error(`${oldIssuer} is still a ${attr.name} issuer`);
  }
  if (!after.some((entry) => sameAccount(entry, issuer.address))) {
    throw new Error(`${issuer.address} did not become a ${attr.name} issuer`);
  }
  if (after.length === 0) {
    throw new Error(`${attr.name} issuer list is empty; no attestation of that kind can be issued`);
  }
  return { attr, skipped: false };
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
  const attrs = resolveAttrs(args.attrs);

  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519', ss58Format: SS58_PREFIX });
  const sudo = keyring.addFromUri(args.sudoSeed);
  const issuer = keyring.addFromUri(args.issuerSeed);

  const api = await ApiPromise.create({ provider: new WsProvider(args.rpc), noInitWarn: true });
  try {
    console.log('Identity issuer handover');
    log(`chain : ${(await api.rpc.system.chain()).toString()}`);
    log(`sudo  : ${sudo.address}`);
    log(`issuer: ${issuer.address}`);
    log(`old   : ${args.oldIssuer}`);

    const sudoKey = (await api.query.sudo.key()).toString();
    if (!sameAccount(sudoKey, sudo.address)) {
      throw new Error(`Sudo.Key is ${sudoKey}, but --sudo-seed derives ${sudo.address}`);
    }

    const oldHolds = await heldAttributes(api, args.oldIssuer);
    log(`old account holds: ${oldHolds.map((a) => a.name).join(', ') || 'nothing'}`);
    if (oldHolds.length === 0) {
      throw new Error(`${args.oldIssuer} is not an issuer of any kind; nothing to rotate`);
    }

    const oldByIndex = new Map(oldHolds.map((attr) => [attr.name, attr]));
    const pending = attrs.filter((attr) => oldByIndex.has(attr.name));
    const notHeld = attrs.filter((attr) => !oldByIndex.has(attr.name));
    if (notHeld.length > 0) {
      log(`not held, so not rotated: ${notHeld.map((a) => a.name).join(', ')}`);
    }
    if (pending.length === 0) {
      console.log('\nPASS: requested attributes are already off the old account.');
      return null;
    }

    const newHolds = await heldAttributes(api, issuer.address);
    const alreadyThere = pending.filter((attr) => newHolds.some((h) => h.name === attr.name));
    if (alreadyThere.length > 0) {
      throw new Error(`${issuer.address} is already a ${alreadyThere.map((a) => a.name).join(', ')} issuer`);
    }

    const results = [];
    for (const attr of pending) {
      results.push(await handOver(api, attr, { sudo, issuer, oldIssuer: args.oldIssuer, dryRun: args.dryRun }));
    }

    if (args.dryRun) {
      console.log('\n  dry run: no change made');
      return results;
    }

    const stillHeld = await heldAttributes(api, args.oldIssuer);
    if (stillHeld.length > 0) {
      throw new Error(`partial: old account still holds ${stillHeld.map((a) => a.name).join(', ')}`);
    }
    const remainingGap = ALL_ATTR_NAMES.filter(
      (name) => !attrs.some((attr) => attr.name === name),
    );
    if (remainingGap.length > 0) {
      console.log(`\nNOTE: not requested this run: ${remainingGap.join(', ')} — the old account may still hold them.`);
    }

    console.log('\nPASS: requested identity-issuer authority has moved off the compromised account.');
    return results;
  } finally {
    await api.disconnect();
  }
}

main().catch((e) => {
  console.error(`\nFAILED: ${e.message}`);
  process.exit(1);
});
