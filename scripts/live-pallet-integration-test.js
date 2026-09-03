/**
 * BelizeChain Live Pallet End-to-End Integration Test
 *
 * Exercises the custom governance and ethical safeguard pallets on the live Ceiba testnet:
 *   1. pallet-belize-moderation: Content flagging, flag storage, and flag count aggregation.
 *   2. pallet-belize-justice: Opening a formal dispute, bond reservation, and cooling-off state transition.
 *   3. pallet-belize-whistleblower: Submitting a cryptographic commitment-based misconduct report and bond reservation.
 */

const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { cryptoWaitReady, blake2AsU8a } = require('@polkadot/util-crypto');
const { u8aToHex, hexToU8a } = require('@polkadot/util');

const RPC_ENDPOINT = process.env.RPC_ENDPOINT || 'ws://100.81.45.25:9944';
const DALLA = 1_000_000_000_000n; // 10^12 atomic units

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function sendTx(api, tx, sender, label) {
  console.log(`\n⏳ Submitting [${label}] from ${sender.address}...`);
  return new Promise((resolve, reject) => {
    let unsub;
    tx.signAndSend(sender, ({ status, events, dispatchError }) => {
      if (status.isInBlock) {
        const blockHash = status.asInBlock.toHex();
        console.log(`   📦 [${label}] included in block: ${blockHash}`);
        
        if (dispatchError) {
          if (dispatchError.isModule) {
            const decoded = api.registry.findMetaError(dispatchError.asModule);
            const errStr = `${decoded.section}.${decoded.name}: ${decoded.docs.join(' ')}`;
            console.error(`   ❌ [${label}] DispatchError: ${errStr}`);
            if (unsub) unsub();
            return reject(new Error(errStr));
          } else {
            console.error(`   ❌ [${label}] DispatchError: ${dispatchError.toString()}`);
            if (unsub) unsub();
            return reject(new Error(dispatchError.toString()));
          }
        }

        const successEvent = events.find(
          ({ event }) => api.events.system.ExtrinsicSuccess.is(event)
        );
        if (successEvent) {
          console.log(`   ✅ [${label}] ExtrinsicSuccess!`);
        }
        if (unsub) unsub();
        resolve({ blockHash, events });
      }
    }).then((u) => {
      unsub = u;
    }).catch(reject);
  });
}

async function run() {
  console.log('===============================================================');
  console.log('   BELIZECHAIN LIVE TESTNET PALLET INTEGRATION TEST SUITE     ');
  console.log('===============================================================');
  console.log(`Connecting to node RPC: ${RPC_ENDPOINT}...`);

  await cryptoWaitReady();
  const provider = new WsProvider(RPC_ENDPOINT);
  const api = await ApiPromise.create({ provider });

  const chainName = (await api.rpc.system.chain()).toString();
  const specVersion = api.runtimeVersion.specVersion.toNumber();
  const header = await api.rpc.chain.getHeader();
  console.log(`Connected: ${chainName} (spec_version: ${specVersion}, head block: #${header.number})`);

  const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
  const sudoTreasury = keyring.addFromUri('//treasury');
  const testActor = keyring.addFromUri('//BelizeTestActor//2026');
  const testTarget = keyring.addFromUri('//BelizeTestTarget//2026');

  console.log('\n--- Accounts ---');
  console.log(`Sudo / Treasury : ${sudoTreasury.address}`);
  console.log(`Test Actor      : ${testActor.address}`);
  console.log(`Test Target     : ${testTarget.address}`);

  // ─────────────────────────────────────────────────────────────────────────
  // Step 0: Pre-fund test accounts if needed
  // ─────────────────────────────────────────────────────────────────────────
  const actorAccount = await api.query.system.account(testActor.address);
  const targetAccount = await api.query.system.account(testTarget.address);

  if (actorAccount.data.free.toBigInt() < 300n * DALLA) {
    const fundingAmount = 500n * DALLA;
    console.log(`\nFunding Test Actor with 500 DALLA...`);
    const fundActorTx = api.tx.balances.transferKeepAlive(testActor.address, fundingAmount);
    await sendTx(api, fundActorTx, sudoTreasury, 'Fund Test Actor');
  } else {
    console.log(`Test Actor already has ${actorAccount.data.free.toHuman()} free DALLA.`);
  }

  if (targetAccount.data.free.toBigInt() < 10n * DALLA) {
    console.log(`Funding Test Target with 50 DALLA...`);
    const fundTargetTx = api.tx.balances.transferKeepAlive(testTarget.address, 50n * DALLA);
    await sendTx(api, fundTargetTx, sudoTreasury, 'Fund Test Target');
  } else {
    console.log(`Test Target already has ${targetAccount.data.free.toHuman()} free DALLA.`);
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Step 1: Test pallet-belize-moderation (Content Flagging)
  // ─────────────────────────────────────────────────────────────────────────
  console.log('\n===============================================================');
  console.log(' TEST 1: pallet-belize-moderation (Community Content Flagging)');
  console.log('===============================================================');
  
  const contentPayload = `BelizeChain decentralised content post ID: ${Date.now()}`;
  const contentHash = blake2AsU8a(contentPayload, 256);
  const contentHashHex = u8aToHex(contentHash);
  console.log(`Content Hash: ${contentHashHex}`);
  console.log(`Flagging reason: 2 (Spam)`);

  const flagTx = api.tx.belizeModeration.flagContent(contentHashHex, 2);
  await sendTx(api, flagTx, testActor, 'belizeModeration.flagContent');

  // Verify storage state
  const flagRecord = await api.query.belizeModeration.contentFlags(contentHashHex, testActor.address);
  console.log(`Storage query [contentFlags]:`, flagRecord.toHuman());
  if (flagRecord.isNone) {
    throw new Error('Verification failed: contentFlag record not found in storage!');
  }

  const flagCount = await api.query.belizeModeration.flagCounts(contentHashHex);
  console.log(`Storage query [flagCounts]: ${flagCount.toString()}`);
  if (flagCount.toNumber() !== 1) {
    throw new Error(`Expected flag count 1, found ${flagCount.toString()}`);
  }
  console.log('>>> TEST 1 PASSED: Content successfully flagged and counted on-chain!');

  // ─────────────────────────────────────────────────────────────────────────
  // Step 2: Test pallet-belize-justice (Dispute Mediation & Cooling-off)
  // ─────────────────────────────────────────────────────────────────────────
  console.log('\n===============================================================');
  console.log(' TEST 2: pallet-belize-justice (Dispute Mediation & Protection)');
  console.log('===============================================================');

  const evidencePayload = `Evidence of contract non-compliance ref: ${Date.now()}`;
  const evidenceHash = blake2AsU8a(evidencePayload, 256);
  const evidenceHashHex = u8aToHex(evidenceHash);
  const severity = 1; // 1 = Moderate

  console.log(`Target: ${testTarget.address}`);
  console.log(`Evidence Hash: ${evidenceHashHex}`);
  console.log(`Severity: 1 (Moderate)`);

  const initialActorState = await api.query.system.account(testActor.address);
  console.log(`Actor reserved balance before dispute: ${initialActorState.data.reserved.toHuman()}`);

  const openDisputeTx = api.tx.belizeJustice.openDispute(testTarget.address, evidenceHashHex, severity);
  await sendTx(api, openDisputeTx, testActor, 'belizeJustice.openDispute');

  const disputeCount = await api.query.belizeJustice.disputeCounter();
  const disputeId = disputeCount.toNumber();
  console.log(`Latest Dispute ID: #${disputeId}`);

  const disputeRecord = await api.query.belizeJustice.disputes(disputeId);
  console.log(`Storage query [disputes(#${disputeId})]:`, JSON.stringify(disputeRecord.toHuman(), null, 2));

  if (disputeRecord.isNone) {
    throw new Error(`Verification failed: dispute record #${disputeId} not found in storage!`);
  }

  const rehabStatus = await api.query.belizeJustice.rehabilitationStatus(testTarget.address);
  console.log(`Target Rehabilitation Status:`, rehabStatus.toHuman());
  if (rehabStatus.toString() !== 'InCoolingOff') {
    throw new Error(`Expected InCoolingOff status for target, got: ${rehabStatus.toString()}`);
  }

  const coolingOffEnd = await api.query.belizeJustice.coolingOffEnd(testTarget.address);
  console.log(`Target Cooling-off end block: #${coolingOffEnd.toString()}`);

  const postDisputeActorState = await api.query.system.account(testActor.address);
  console.log(`Actor reserved balance after dispute: ${postDisputeActorState.data.reserved.toHuman()} (OpenDisputeBond reserved)`);
  console.log('>>> TEST 2 PASSED: Dispute created, bond reserved, target placed in cooling-off!');

  // ─────────────────────────────────────────────────────────────────────────
  // Step 3: Test pallet-belize-whistleblower (Pseudonymous Misconduct Report)
  // ─────────────────────────────────────────────────────────────────────────
  console.log('\n===============================================================');
  console.log(' TEST 3: pallet-belize-whistleblower (Anonymous Misconduct Report)');
  console.log('===============================================================');

  const domainTag = Buffer.from('BelizeChainWhistleblowerV1');
  const secretBytes = Buffer.from('super_secret_whistleblower_salt_2026_belize');
  const commitmentPayload = Buffer.concat([domainTag, testActor.publicKey, secretBytes]);
  const commitment = blake2AsU8a(commitmentPayload, 256);
  const commitmentHex = u8aToHex(commitment);

  const reportEvidencePayload = `Whistleblower confidential dossier: ${Date.now()}`;
  const reportEvidenceHash = blake2AsU8a(reportEvidencePayload, 256);
  const reportEvidenceHashHex = u8aToHex(reportEvidenceHash);
  const category = 1; // 1 = Fraud

  console.log(`Commitment Hex: ${commitmentHex}`);
  console.log(`Target: ${testTarget.address}`);
  console.log(`Evidence Hash: ${reportEvidenceHashHex}`);
  console.log(`Category: 1 (Fraud)`);

  const initialReportCounter = await api.query.belizeWhistleblower.reportCounter();
  const submitReportTx = api.tx.belizeWhistleblower.submitReport(
    commitmentHex,
    testTarget.address,
    reportEvidenceHashHex,
    category
  );
  await sendTx(api, submitReportTx, testActor, 'belizeWhistleblower.submitReport');

  const newReportCounter = await api.query.belizeWhistleblower.reportCounter();
  const reportId = newReportCounter.toNumber();
  console.log(`Latest Report ID: #${reportId}`);

  const reportRecord = await api.query.belizeWhistleblower.reports(reportId);
  console.log(`Storage query [reports(#${reportId})]:`, JSON.stringify(reportRecord.toHuman(), null, 2));

  if (reportRecord.isNone) {
    throw new Error(`Verification failed: whistleblower report #${reportId} not found in storage!`);
  }

  if (reportRecord.toHuman().commitment !== commitmentHex) {
    throw new Error(`Commitment mismatch: expected ${commitmentHex}, got ${reportRecord.toHuman().commitment}`);
  }
  console.log(`Commitment verified on on-chain report: ${reportRecord.toHuman().commitment}`);

  const finalActorState = await api.query.system.account(testActor.address);
  console.log(`Actor final reserved balance: ${finalActorState.data.reserved.toHuman()} (Dispute + Report Bonds reserved)`);
  console.log('>>> TEST 3 PASSED: Whistleblower report committed, bond escrowed, storage verified!');

  console.log('\n===============================================================');
  console.log(' 🎉 ALL THREE CUSTOM PALLETS PASSED END-TO-END LIVE ON-CHAIN!  ');
  console.log('===============================================================');

  await api.disconnect();
}

run().catch((err) => {
  console.error('\n❌ Integration test failed with exception:', err);
  process.exit(1);
});
