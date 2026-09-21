// Recreate the A1 "Nawal AI Validator" registration on a fresh BelizeChain testnet.
//
// Recovered from the pre-reset chain state (2026-09-21) so the record is
// reproduced exactly rather than approximately:
//
//   identity    : id 2, name "Nawal AI Validator" (owner = the signer account)
//   KYC         : L2 — requires a *valid* SSN **and** Passport attestation
//                 (pallet_belize_identity::kyc_state, KycLevel::L2). The
//                 compliance pallet stays at `None`; validators gate on the
//                 identity pallet (`meets_validator_kyc` → requirement level 2).
//   staking     : stake 1000 DALLA, computeCapacity 100, location "Belize City"
//
// Order matters: the identity must exist before attestations can be issued
// (`IdentityNotFound`), and KYC L2 must be in place before `joinValidators`
// (`ValidatorKycInsufficient`).
//
// Usage:
//   NAWAL_SEED='<mnemonic or //dev-uri>' \
//     NODE_PATH=/path/to/@polkadot node scripts/register-nawal-signer.js
//
// `NAWAL_SEED` is never logged. The signer address is derived from it and must
// match SIGNER_ADDRESS, so a wrong key fails before anything is signed.
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { decodeAddress, encodeAddress, blake2AsHex } = require('@polkadot/util-crypto');
const { stringToU8a, u8aToHex } = require('@polkadot/util');

const WS_ENDPOINT = process.env.RPC_ENDPOINT || 'ws://100.81.45.25:9944';
const NAVAL_SEED = process.env.NAWAL_SEED;
const ISSUER_SEED = process.env.ISSUER_SEED || '//Alice';
const SIGNER_ADDRESS = process.env.SIGNER_ADDRESS || '5Gj3p3X5HLdBaLFQ7xuXCw8DRPLhaVdQ4N1XA22dYdj3hxvU';
const IDENTITY_NAME = process.env.IDENTITY_NAME || 'Nawal AI Validator';
const LOCATION = process.env.LOCATION || 'Belize City';
const CAPACITY = Number(process.env.CAPACITY || 100);
const STAKE_DALLA = BigInt(process.env.STAKE_DALLA || 1000);
const DALLA = 10n ** 12n;

const hex = (a) => Buffer.from(decodeAddress(a)).toString('hex');

// The runtime decodes these params as `Bytes` / `BoundedVec<u8, _>`, so they must
// be hex — passing a Uint8Array makes the API treat it as a byte-per-element
// sequence and overrun the length bound.
const bytes = (s) => u8aToHex(stringToU8a(s));

// polkadot-js invokes the callback with a single result object; the tx hash is
// `txHash` (there is no second `extrinsic` argument).
function submit(api, tx, signer, label) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${label}: timeout waiting for inclusion`)), 180000);
    tx.signAndSend(signer, (result) => {
      const { status, dispatchError, events, txHash } = result;
      if (status.isInBlock) {
        clearTimeout(timer);
        const failed = events.find(({ event }) => event.section === 'system' && event.method === 'ExtrinsicFailed');
        if (dispatchError || failed) {
          reject(new Error(`${label}: ${(dispatchError || failed.event.data[0]).toString()}`));
          return;
        }
        console.log(`  ${label.padEnd(22)} in block ${status.asInBlock.toHex().slice(0, 18)}…  ext ${txHash.toHex().slice(0, 18)}…`);
        resolve();
      }
    }).catch((e) => { clearTimeout(timer); reject(new Error(`${label}: ${e.message}`)); });
  });
}

async function main() {
  if (!NAVAL_SEED) {
    console.error('error: NAWAL_SEED is required (the signer mnemonic/URI). Not printed, by design.');
    process.exit(1);
  }

  const api = await ApiPromise.create({ provider: new WsProvider(WS_ENDPOINT), noInitWarn: true });
  await api.isReady;

  const keyring = new Keyring({ type: 'sr25519' });
  const signer = keyring.addFromUri(NAVAL_SEED);
  const issuer = keyring.addFromUri(ISSUER_SEED);

  if (hex(signer.address) !== hex(SIGNER_ADDRESS)) {
    console.error('error: NAWAL_SEED derives a different account than SIGNER_ADDRESS — refusing.');
    console.error('       derived:', encodeAddress(signer.address, api.registry.chainSS58));
    process.exit(1);
  }

  console.log('chain      :', api.runtimeVersion.specName.toString(), 'spec', api.runtimeVersion.specVersion.toString());
  console.log('signer     :', encodeAddress(signer.address, api.registry.chainSS58));
  console.log('issuer     :', encodeAddress(issuer.address, api.registry.chainSS58));

  const nameBytes = bytes(IDENTITY_NAME);

  // --- 0. fund the signer (fees + stake) ---------------------------------
  const stake = STAKE_DALLA * DALLA;
  const minFree = stake + 100n * DALLA;
  const account = await api.query.system.account(signer.address);
  if (BigInt(account.data.free.toString()) < minFree) {
    const topUp = (2000n * DALLA) - BigInt(account.data.free.toString());
    console.log('\n[0] funding signer with', (Number(topUp) / 1e12).toFixed(2), 'DALLA from the issuer');
    await submit(api, api.tx.balances.transferKeepAlive(signer.address, api.createType('Balance', topUp.toString())), issuer, 'fund signer');
  } else {
    console.log('\n[0] signer already funded');
  }

  // --- 1. identity --------------------------------------------------------
  const existing = await api.query.identity.identityOf(signer.address);
  let identityId;
  if (!existing.isSome) {
    console.log('\n[1] identity.registerIdentity');
    await submit(api, api.tx.identity.registerIdentity(nameBytes), signer, 'registerIdentity');
    identityId = (await api.query.identity.identityOf(signer.address)).unwrap().toString();
  } else {
    identityId = existing.unwrap().toString();
    console.log('\n[1] identity already registered as id', identityId);
  }

  // --- 2. KYC L2 = valid SSN attestation + valid Passport attestation -----
  const ssn = await api.query.identity.ssnAttestations(identityId);
  if (ssn.isNone) {
    console.log('\n[2] identity.issueSsn (issuer = the genesis SSN issuer)');
    await submit(api, api.tx.identity.issueSsn(
      signer.address,
      blake2AsHex(stringToU8a(`ssn:${IDENTITY_NAME}`)),
      bytes('nawal-signer-ssn-anchor'),
      true,
    ), issuer, 'issueSsn');
  } else {
    console.log('\n[2] SSN attestation already present');
  }

  const passport = await api.query.identity.passportAttestations(identityId);
  if (passport.isNone) {
    console.log('[2] identity.issuePassport');
    await submit(api, api.tx.identity.issuePassport(
      signer.address,
      blake2AsHex(stringToU8a(`passport:${IDENTITY_NAME}`)),
      bytes('nawal-signer-passport-anchor'),
      true,
    ), issuer, 'issuePassport');
  } else {
    console.log('[2] Passport attestation already present');
  }

  // --- 3. PoUW validator registration ------------------------------------
  const validator = await api.query.staking.validators(signer.address);
  const alreadyValidator = Boolean(validator.isSome && validator.unwrap().account);
  if (alreadyValidator) {
    console.log('\n[3] already a staking validator');
  } else {
    console.log('\n[3] staking.joinValidators');
    await submit(api, api.tx.staking.joinValidators(
      api.createType('Balance', stake.toString()),
      CAPACITY,
      bytes(LOCATION),
    ), signer, 'joinValidators');
  }

  // --- verify -------------------------------------------------------------
  console.log('\n=== on-chain record after registration ===');
  const id = await api.query.identity.identityOf(signer.address);
  const identityKey = id.isSome ? id.unwrap().toString() : null;
  console.log('  identity id           :', identityKey ?? '(none)');
  if (identityKey) {
    const ssnAfter = await api.query.identity.ssnAttestations(identityKey);
    const passAfter = await api.query.identity.passportAttestations(identityKey);
    console.log('  SSN attestation       :', ssnAfter.isSome ? 'present' : 'MISSING');
    console.log('  Passport attestation  :', passAfter.isSome ? 'present' : 'MISSING');
  }
  const v = await api.query.staking.validators(signer.address);
  console.log('  staking validator     :', v.isSome ? JSON.stringify(v.toHuman()) : '(none)');
  console.log('  validator count       :', (await api.query.staking.validatorCount()).toString());

  await api.disconnect();
}

main().catch((e) => { console.error('FAILED:', e.message); process.exit(1); });
