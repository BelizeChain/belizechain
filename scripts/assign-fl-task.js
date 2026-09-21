// Assign a federated-learning task on chain.
//
// `staking.assignFLTask` is `ensure_root`, so this is submitted through
// `sudo.sudo(...)` signed by the on-chain sudo key. It also CLEARS
// `Staking.ModelSubmissions`, so assigning a new task invalidates every
// submission from the previous one — assign, then submit.
//
// Usage:
//   MODEL_HASH=0x... TASK_ID=1 node scripts/assign-fl-task.js
//
// Everything has preconditions checked before anything is signed.
const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { decodeAddress, encodeAddress } = require('@polkadot/util-crypto');

const WS_ENDPOINT = process.env.RPC_ENDPOINT || 'ws://100.81.45.25:9944';
const SUDO_SEED = process.env.SUDO_SEED || '//Alice';
const MODEL_HASH = process.env.MODEL_HASH;
const TASK_ID = Number(process.env.TASK_ID || 1);
// Seconds of training the task is expected to take — used for the timeliness score.
const COMPUTATION_TIME = Number(process.env.COMPUTATION_TIME || 300);
// Perbill: 1_000_000_000 == 100%.
const REWARD_MULTIPLIER = Number(process.env.REWARD_MULTIPLIER || 1_000_000_000);
// Relative to the current block. 5000 blocks ~= 8.3 h at 6 s.
const DEADLINE_BLOCKS = Number(process.env.DEADLINE_BLOCKS || 5000);

const hex = (a) => Buffer.from(decodeAddress(a)).toString('hex');

function submit(tx, signer, label) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`${label}: timeout`)), 120000);
    tx.signAndSend(signer, ({ status, dispatchError, events, txHash }) => {
      if (status.isInBlock) {
        clearTimeout(timer);
        const failed = events.find(({ event }) => event.section === 'system' && event.method === 'ExtrinsicFailed');
        if (dispatchError || failed) {
          reject(new Error(`${label}: ${(dispatchError || failed.event.data[0]).toString()}`));
          return;
        }
        console.log(`  ${label} included in ${status.asInBlock.toHex().slice(0, 18)}…  ext ${txHash.toHex().slice(0, 18)}…`);
        resolve();
      }
    }).catch((e) => { clearTimeout(timer); reject(new Error(`${label}: ${e.message}`)); });
  });
}

async function main() {
  if (!MODEL_HASH || !/^0x[0-9a-fA-F]{64}$/.test(MODEL_HASH)) {
    console.error('error: MODEL_HASH must be a 0x-prefixed 32-byte hex string');
    process.exit(1);
  }

  const api = await ApiPromise.create({ provider: new WsProvider(WS_ENDPOINT), noInitWarn: true });
  await api.isReady;

  const keyring = new Keyring({ type: 'sr25519' });
  const signer = keyring.addFromUri(SUDO_SEED);
  const ss58 = api.registry.chainSS58;

  const onChainSudo = (await api.query.sudo.key()).toString();
  if (hex(onChainSudo) !== hex(signer.address)) {
    console.error('error: signer is not the on-chain sudo key — refusing.');
    process.exit(1);
  }

  const before = await api.query.staking.activeFLTask();
  console.log('chain      :', api.runtimeVersion.specName.toString(), 'spec', api.runtimeVersion.specVersion.toString());
  console.log('sudo       :', encodeAddress(signer.address, ss58), '(confirmed)');
  console.log('task before:', before.isSome ? JSON.stringify(before.toHuman()) : '(none)');

  // polkadot-js renders the acronym as `Fl`, not `FL`.
  const call = api.tx.staking.assignFlTask(
    TASK_ID,
    MODEL_HASH,
    COMPUTATION_TIME,
    REWARD_MULTIPLIER,
    DEADLINE_BLOCKS,
  );

  console.log('\n>>> sudo.sudo(staking.assignFlTask) <<<');
  await submit(api.tx.sudo.sudo(call), signer, 'assignFlTask');

  const after = await api.query.staking.activeFLTask();
  const submissions = await api.query.staking.modelSubmissions.entries();
  console.log('\n--- on-chain state after ---');
  console.log('task after  :', JSON.stringify(after.toHuman()));
  console.log('submissions :', submissions.length, '(cleared by the assignment)');

  await api.disconnect();
}

main().catch((e) => { console.error('FAILED:', e.message); process.exit(1); });
