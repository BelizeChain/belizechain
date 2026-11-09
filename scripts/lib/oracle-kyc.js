/**
 * Helpers shared by the operator scripts that have to satisfy the Oracle
 * pallet's KYC gate before their real work can proceed.
 *
 * Both `bootstrap-dex.js` and `rotate-bridge-pq-keys.js` need to seat operators
 * and seal a KYC level, and the logic is subtle enough that two copies would
 * drift: the votes must carry an identical `(level, id_hash)` pair or they never
 * tally, and the Oracle store is a *different* store from the Identity pallet's.
 * Keeping one implementation is the point of this module.
 */

const log = (msg) => console.log(`  ${msg}`);
const step = (msg) => console.log(`\n${msg}`);

/** `BoundedVec<u8, _>` args must go in as hex, not as a Buffer. */
const bytes = (text) => `0x${Buffer.from(text, 'utf8').toString('hex')}`;

/** Turn a dispatch error into the pallet's own name and doc string. */
function describeDispatchError(dispatchError) {
  if (!dispatchError.isModule) return dispatchError.toString();
  const meta = dispatchError.asModule.registry.findMetaError(dispatchError.asModule);
  return `${meta.section}.${meta.name}: ${meta.docs.join(' ').trim()}`;
}

/** KYC as recorded in the Oracle pallet — the one store readable over RPC. */
async function oracleKycLevel(api, address) {
  const recorded = await api.query.oracle.identityVerifications(address);
  return recorded.isSome ? Number(recorded.unwrap().kycLevel.index) : 0;
}

/**
 * Seat every operator that is not already seated. Idempotent.
 *
 * `verify_identity` requires each voter to be a seated operator, and seating is
 * gated by `OracleAdminOrigin` (which includes Root), so this goes through sudo.
 */
async function seatOracleOperators(api, run, operators) {
  for (const op of operators) {
    const seated = await api.query.oracle.oracleOperators(op.address);
    if (seated.isTrue) {
      log(`operator ${op.address.slice(0, 10)}…: already seated`);
      continue;
    }
    await run(`seat operator ${op.address.slice(0, 10)}…`,
      api.tx.sudo.sudo(api.tx.oracle.addOperator(op.address)));
  }
}

/**
 * Seal KYC level `required` for `subject` in the **Oracle** pallet's store.
 *
 * This is a second, unsynchronised store. The Economy pallet does NOT read the
 * Identity pallet: its `Oracle` shim routes to `pallet_belize_oracle`, which
 * trusts only its own `IdentityVerifications` map. So a fresh chain needs KYC
 * recorded twice and a mint fails with `KycVerificationRequired` until this one
 * exists. `Interoperability::verify_bridge_operator` falls back to this store
 * too, which is why bridge validators are seeded here and nowhere else.
 *
 * Sealing needs `MinOracleAgreement` distinct operators to submit the same vote.
 */
async function seedOracleKyc(api, run, subject, operators, required, dryRun) {
  const address = subject.address;
  const recordedLevel = await oracleKycLevel(api, address);
  log(`oracle kyc level: ${recordedLevel} (need ${required})`);
  if (recordedLevel >= required) {
    log('already sufficient');
    return;
  }

  // Every operator must submit the SAME (level, id_hash) pair for the votes to
  // tally; a differing hash becomes a competing leader and never reaches
  // agreement. An operator that already voted can change its vote.
  const idHash = require('@polkadot/util-crypto').randomAsU8a(32);
  for (const op of operators) {
    await run(`oracle kyc vote L${required} ${op.address.slice(0, 10)}…`,
      api.tx.oracle.verifyIdentity(address, required, idHash, bytes('kyc-seed'), false, true),
      op);
  }

  const afterLevel = await oracleKycLevel(api, address);
  // A dry run submitted nothing, so asserting the outcome would always fail.
  // Reporting what the votes WOULD produce is the point of the flag; the real
  // assertion stays below for a live run.
  if (dryRun) {
    log(`oracle KYC would reach L${required} (dry run)`);
    return;
  }
  if (afterLevel < required) {
    throw new Error(`oracle KYC reached L${afterLevel}, need L${required}`);
  }
  log(`oracle KYC recorded at L${afterLevel}`);
}

module.exports = {
  log,
  step,
  bytes,
  describeDispatchError,
  oracleKycLevel,
  seatOracleOperators,
  seedOracleKyc,
};
