#!/usr/bin/env node
/**
 * Bootstrap a BelizeX market that can actually trade.
 *
 * Why this exists
 * ---------------
 * `create_trading_pair` is governance-gated and genesis opens three pairs, so the
 * chain *looks* like it has a DEX. It does not have a market:
 *
 *   1. bBZD has no genesis issuance. `Economy::BBZDBalances` and
 *      `TotalBbzdSupply` are empty on a fresh chain, so every bBZD-quoted
 *      reserve is bookkeeping with no ledger behind it — nothing can settle.
 *   2. The one-time seed hook requires the treasury to already hold bBZD.
 *      Genesis never funds the treasury, so the seed waits forever.
 *   3. `add_liquidity` and `execute_trade` both require KYC level >= 1, and a
 *      fresh chain has no identities, so no account can open or use a market.
 *
 * This script walks the real production path — no storage pokes, no benchmark
 * helpers — in the only order that satisfies the pallets' own preconditions:
 *
 *   reserves -> minter authorization -> identity -> KYC -> mint bBZD
 *            -> trading pair -> liquidity -> a real swap to prove settlement
 *
 * Every step is idempotent, so re-running it converges instead of erroring.
 *
 * Usage
 * -----
 *   NODE_PATH=<repo>/ui/node_modules node scripts/bootstrap-dex.js \
 *     --rpc ws://127.0.0.1:9944 --seed 0x<hex seed of the sudo account>
 *
 * Options
 *   --rpc      WebSocket RPC endpoint (default ws://127.0.0.1:9944)
 *   --seed     secret seed of an account that can satisfy sudo AND act as the
 *              minter/liquidity provider. Required.
 *   --dalla    DALLA leg of the bootstrap liquidity (whole DALLA, default 100000)
 *   --bbzd     bBZD leg (whole bBZD, default 200000)
 *   --bridge-validators  secrets of accounts to register as bridge validators
 *   --evm-addresses      matching 0x secp256k1 addresses, same order and length
 *   --dry-run  run every precondition check and skip the extrinsics
 *
 * Testnet only: it mints a stablecoin, so it must never be pointed at mainnet.
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, randomAsU8a } = require('@polkadot/util-crypto');
const { u8aToHex } = require('@polkadot/util');

const DECIMALS = 12;
const UNIT = 10n ** BigInt(DECIMALS);

// Asset ids as declared by `pallet_belize_belizex::AssetId`.
const DALLA = 0;
const BBZD = 1;

// `pallet_belize_identity::AttributeType::Ssn`.
const ATTR_SSN = 0;

// `pallet_belize_interoperability::BridgeChain::Ethereum`.
const BRIDGE_CHAIN_ETHEREUM = 1;

// Bridge validator registration requires Full (L3) KYC.
const BRIDGE_KYC_LEVEL = 3;

// ML-DSA-87 public keys are a fixed 2592 bytes; the pallet bounds the field to
// exactly this size, so a shorter placeholder is rejected outright.
const ML_DSA_87_PUBLIC_KEY_BYTES = 2592;

// Mirrors the runtime's `MinOracleAgreement`: one KYC verification only
// finalises once this many distinct oracle operators submit the same vote.
const DEFAULT_MIN_ORACLE_AGREEMENT = 3;

const PAIR_FEE_BPS = 30;

function parseArgs(argv) {
  const args = { rpc: 'ws://127.0.0.1:9944', dalla: 100000n, bbzd: 200000n, dryRun: false, seed: null, oracleSeeds: [], minAgreement: DEFAULT_MIN_ORACLE_AGREEMENT, mintKycLevel: 2, bridgeValidators: [], evmAddresses: [] };
  const list = (value) => value.split(',').map((s) => s.trim()).filter(Boolean);
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--rpc') args.rpc = argv[++i];
    else if (a === '--seed') args.seed = argv[++i];
    else if (a === '--oracle-seeds') args.oracleSeeds = list(argv[++i]);
    else if (a === '--bridge-validators') args.bridgeValidators = list(argv[++i]);
    else if (a === '--evm-addresses') args.evmAddresses = list(argv[++i]);
    else if (a === '--min-agreement') args.minAgreement = Number(argv[++i]);
    else if (a === '--mint-kyc-level') args.mintKycLevel = Number(argv[++i]);
    else if (a === '--dalla') args.dalla = BigInt(argv[++i]);
    else if (a === '--bbzd') args.bbzd = BigInt(argv[++i]);
    else if (a === '--dry-run') args.dryRun = true;
    else if (a === '--help' || a === '-h') args.help = true;
    else throw new Error(`unknown argument: ${a}`);
  }
  return args;
}

function usage() {
  console.log('usage: node scripts/bootstrap-dex.js --seed 0x<hex> [--rpc ws://...] [--dry-run]');
}

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

/** Current KYC level, tolerant of runtimes without the convenience getter. */
async function kycLevel(api, address) {
  if (!api.query.identity.verifiedKycLevel) return 0;
  const level = await api.query.identity.verifiedKycLevel(address);
  return Number(level.toString());
}

/**
 * Record an SSN attestation for `subject` in the **Identity** pallet's store.
 *
 * `issue_ssn` must be signed by an authorized issuer, and genesis seeds that
 * list from the chain spec's root_key *argument* — not from `sudo.key`. So a
 * chain whose sudo key was patched (every generated spec, including the swarm)
 * has a sudo holder that is NOT an issuer. Authorize `issuer` explicitly instead
 * of assuming the two are the same account.
 */
async function seedSsnAttestation(api, run, subject, issuer) {
  const address = subject.address;
  const issuers = await api.query.identity.ssnIssuers();
  const issuerKey = u8aToHex(issuer.publicKey);
  const alreadyIssuer = issuers.some(
    (a) => a.toString() === issuer.address || u8aToHex(a.toU8a()) === issuerKey,
  );
  log(`ssn issuer: ${alreadyIssuer}`);
  if (!alreadyIssuer) {
    await run('bond issuer', api.tx.identity.issuerDepositBond(ATTR_SSN), issuer);
    await run('authorize issuer', api.tx.sudo.sudo(api.tx.identity.addIssuer(ATTR_SSN, issuer.address)), issuer);
  }

  // Re-running must converge: the second call reports AlreadyAttested once an
  // SSN exists, which means the goal is already met.
  try {
    await run('issue ssn', api.tx.identity.issueSsn(
      address,
      api.createType('H256', randomAsU8a(32)),
      bytes('BZD-SSN-PLACEHOLDER'),
      true,
    ), issuer);
  } catch (e) {
    if (!/AlreadyAttested/.test(e.message)) throw e;
    log('issue ssn: already attested');
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
 * Sealing needs `MinOracleAgreement` distinct operators to submit the same vote,
 * and `verify_identity` requires each voter to be a seated operator — seated by
 * `OracleAdminOrigin`, which includes Root, so sudo can do it.
 */
async function seedOracleKyc(api, run, subject, operators, required) {
  const address = subject.address;
  const recorded = await api.query.oracle.identityVerifications(address);
  const recordedLevel = recorded.isSome ? Number(recorded.unwrap().kycLevel.index) : 0;
  log(`oracle kyc level: ${recordedLevel} (need ${required})`);
  if (recordedLevel >= required) {
    log('already sufficient');
    return;
  }

  // Every operator must submit the SAME (level, id_hash) pair for the votes to
  // tally; a differing hash becomes a competing leader and never reaches
  // agreement. An operator that already voted can change its vote.
  const idHash = randomAsU8a(32);
  for (const op of operators) {
    await run(`oracle kyc vote L${required} ${op.address.slice(0, 10)}…`,
      api.tx.oracle.verifyIdentity(address, required, idHash, bytes('kyc-seed'), false, true),
      op);
  }

  const after = await api.query.oracle.identityVerifications(address);
  const afterLevel = after.isSome ? Number(after.unwrap().kycLevel.index) : 0;
  if (afterLevel < required) {
    throw new Error(`oracle KYC reached L${afterLevel}, need L${required}`);
  }
  log(`oracle KYC recorded at L${afterLevel}`);
}

/** Seat every operator that is not already seated. Idempotent. */
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

async function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) return usage();
  if (!args.seed) return usage();

  await cryptoWaitReady();

  const api = await ApiPromise.create({ provider: new WsProvider(args.rpc) });
  // SS58 prefix 1981 is BelizeChain's; using the default would print addresses
  // that no other tool in this repo recognises.
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 1981 });
  const who = keyring.addFromUri(args.seed);
  const address = who.address;

  const chain = await api.rpc.system.chain();
  const header = await api.rpc.chain.getHeader();
  console.log(`BelizeChain DEX bootstrap`);
  console.log(`  chain: ${chain}`);
  console.log(`  head:  #${header.number.toNumber()}`);
  console.log(`  acct:  ${address}`);
  console.log(`  rpc:   ${args.rpc}`);

  const run = async (label, tx, signer = who) => {
    if (args.dryRun) {
      log(`${label}: would submit (dry run)`);
      return null;
    }
    return new Promise((resolve, reject) => {
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
          resolve(status);
        }
      })
        .then((handle) => { unsubscribe = handle; })
        .catch((e) => reject(new Error(`${label}: ${e.message}`)));
    });
  };

  // ── 1. Central Bank reserves ────────────────────────────────────────────
  // `mint_bbzd` refuses when new_supply > reserves, and `update_reserves`
  // refuses to go below circulating supply. Set it first, high enough for the
  // mint below.
  step('1. Central Bank reserves');
  const mintAmount = args.bbzd * UNIT;
  const reserves = await api.query.economy.centralBankReserves();
  log(`current reserves: ${reserves.toString()}`);
  if (BigInt(reserves.toString()) < mintAmount) {
    await run('set reserves', api.tx.sudo.sudo(api.tx.economy.updateReserves(mintAmount * 10n)));
  } else {
    log('already sufficient');
  }

  // ── 2. Minter authorization ─────────────────────────────────────────────
  step('2. Minter authorization');
  const isMinter = await api.query.economy.authorizedMinters(address);
  log(`authorized: ${isMinter.toString()}`);
  if (!isMinter.isTrue) {
    await run('authorize minter', api.tx.sudo.sudo(api.tx.economy.setMinterAuthorization(address, true)));
  } else {
    log('already authorized');
  }

  // ── 3. Identity + KYC ───────────────────────────────────────────────────
  // Both `add_liquidity` and `execute_trade` gate on KYC level >= 1, and
  // `mint_bbzd` requires it of the recipient.
  step('3. Identity and KYC level 1');
  const existing = await api.query.identity.identityOf(address);
  if (existing.isSome) {
    log('identity already registered');
  } else {
    await run('register identity', api.tx.identity.registerIdentity(
      bytes('BelizeX market maker'),
    ));
  }

  const level = await kycLevel(api, address);
  log(`kyc level: ${level}`);
  if (level < 1) {
    await seedSsnAttestation(api, run, who, who);
  } else {
    log('already KYC level 1+');
  }

  // ── 3b. Oracle KYC ──────────────────────────────────────────────────────
  step('3b. Oracle KYC (separate store from Identity)');
  if (args.oracleSeeds.length < args.minAgreement) {
    throw new Error(
      `minting bBZD needs ${args.minAgreement} distinct oracle operators to agree on KYC; `
      + `pass --oracle-seeds <seed1>,<seed2>,<seed3> (got ${args.oracleSeeds.length})`,
    );
  }

  const operators = args.oracleSeeds.map((s) => keyring.addFromUri(s));
  await seatOracleOperators(api, run, operators);
  await seedOracleKyc(api, run, who, operators.slice(0, args.minAgreement), args.mintKycLevel);

  // ── 4. Mint bBZD ────────────────────────────────────────────────────────
  // This is the step that does not exist on a fresh chain: bBZD has no genesis
  // issuance, so without it there is no stablecoin to quote a market against.
  step('4. Mint bBZD');
  const currentBbzd = await api.query.economy.bbzdBalances(address);
  log(`bBZD held: ${currentBbzd.toString()}`);
  if (BigInt(currentBbzd.toString()) < mintAmount) {
    await run('mint bBZD', api.tx.economy.mintBbzd(
      address,
      mintAmount,
      bytes('dex-bootstrap'),
    ));
  } else {
    log('already funded');
  }

  // ── 5. Trading pair ─────────────────────────────────────────────────────
  step('5. DALLA/bBZD trading pair');
  const pair = await api.query.belizeX.tradingPairs([DALLA, BBZD]);
  if (pair.isSome) {
    log('pair already listed');
  } else {
    await run('create pair', api.tx.sudo.sudo(api.tx.belizeX.createTradingPair(DALLA, BBZD, PAIR_FEE_BPS)));
  }

  // ── 6. Liquidity ────────────────────────────────────────────────────────
  step('6. Liquidity');
  const after = await api.query.belizeX.tradingPairs([DALLA, BBZD]);
  const baseReserve = BigInt(after.unwrap().baseReserve.toString());
  log(`base reserve: ${baseReserve.toString()}`);
  if (baseReserve === 0n) {
    await run('add liquidity', api.tx.belizeX.addLiquidity(
      DALLA,
      BBZD,
      args.dalla * UNIT,
      args.bbzd * UNIT,
      0,
    ));
  } else {
    log('pool already funded');
  }

  // ── 7. Prove settlement ─────────────────────────────────────────────────
  // A listed pair with reserves still proves nothing until a trade settles and
  // moves both ledgers. This is the check that actually retires the blocker.
  step('7. Settlement check');
  if (args.dryRun) {
    log('skipped (dry run)');
  } else {
    const bBzdBefore = BigInt((await api.query.economy.bbzdBalances(address)).toString());
    const dallaBefore = BigInt((await api.query.system.account(address)).data.free.toString());
    const tradeIn = 100n * UNIT;
    await run('execute trade (DALLA -> bBZD)', api.tx.belizeX.executeTrade(DALLA, BBZD, tradeIn, 0, false));
    const bBzdAfter = BigInt((await api.query.economy.bbzdBalances(address)).toString());
    const dallaAfter = BigInt((await api.query.system.account(address)).data.free.toString());

    log(`DALLA: ${dallaBefore} -> ${dallaAfter}`);
    log(`bBZD : ${bBzdBefore} -> ${bBzdAfter}`);
    if (dallaAfter >= dallaBefore) throw new Error('settlement failed: DALLA did not leave the trader');
    if (bBzdAfter <= bBzdBefore) throw new Error('settlement failed: bBZD did not reach the trader');
    console.log('\nPASS: a swap settled against real balances on both ledgers.');
  }

  // ── 8. Bridge validators ────────────────────────────────────────────────
  // Blocker #7 is the foreign-chain leg. Its two halves are the receiver
  // contract (deployed by `infra/relayer/scripts/deploy-receiver.mjs`) and this
  // pallet's validator set. Registration is the producer side: without it the
  // contract's secp256k1 set has nothing to reconcile against, and
  // `check-validator-sets.mjs` compares three addresses to zero.
  if (args.bridgeValidators.length > 0) {
    step('8. Bridge validators');
    if (args.evmAddresses.length !== args.bridgeValidators.length) {
      throw new Error(
        `--bridge-validators and --evm-addresses must pair up `
        + `(got ${args.bridgeValidators.length} and ${args.evmAddresses.length})`,
      );
    }

    // Sealing needs the same quorum the DEX steps used; reuse the seated set
    // rather than requiring a second list of operator keys.
    const kycQuorum = operators.slice(0, args.minAgreement);

    for (const [i, seed] of args.bridgeValidators.entries()) {
      const validator = keyring.addFromUri(seed);
      const label = `${validator.address.slice(0, 10)}…`;
      const evmAddress = args.evmAddresses[i];

      // The Identity pallet is not consulted for this gate, so only the Oracle
      // store is seeded — no SSN or passport attestation is needed.
      await seedOracleKyc(api, run, validator, kycQuorum, BRIDGE_KYC_LEVEL);

      const existingValidator = await api.query.interoperability.bridgeValidators(validator.address);
      if (existingValidator.isSome) {
        log(`${label}: already registered`);
      } else {
        // A placeholder PQ key: this rehearsal proves the extrinsic path and the
        // set reconciliation, not ML-DSA signing. The real key is inserted with
        // `author_insertKey` on a host that holds validator secrets.
        await run(`register validator ${label}`, api.tx.interoperability.registerBridgeValidator(
          u8aToHex(randomAsU8a(ML_DSA_87_PUBLIC_KEY_BYTES)),
          [BRIDGE_CHAIN_ETHEREUM],
        ), validator);
      }

      // `set_evm_address` requires registration first, and this is the field the
      // contract's validator set is reconciled against. Skipping when it already
      // matches keeps a re-run genuinely converged instead of re-writing it.
      const currentEvm = await api.query.interoperability.evmAddressOf(validator.address);
      if (currentEvm.isSome && u8aToHex(currentEvm.unwrap().toU8a(true)).toLowerCase() === evmAddress.toLowerCase()) {
        log(`${label}: evm address already set`);
        continue;
      }
      await run(`set evm address ${label}`, api.tx.interoperability.setEvmAddress(evmAddress), validator);
    }
  }

  await api.disconnect();
}

main().catch((e) => {
  console.error(`\nFAILED: ${e.message}`);
  process.exit(1);
});
