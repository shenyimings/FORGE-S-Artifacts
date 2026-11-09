import { afterAll, beforeAll, describe, expect, test } from 'vitest';
import { createTestSuiteContext, expectMasked } from '../../testutil';
import { initializeZBTCVault } from './zbtc.init';

describe('solv.zBTC test', async () => {
  const testCtx = initializeZBTCVault(await createTestSuiteContext());

  beforeAll(() => testCtx.initializationTasks);
  afterAll(() => testCtx.validator.quit());

  /** 1. configuration **/
  const { validator, feePayer, solv, initializationTasks, admin } = testCtx;
  const ctx = solv.zBTC;

  test(`solv.zBTC initializationTasks snapshot`, async () => {
    await expectMasked(initializationTasks).resolves.toMatchSnapshot();
  });

  test(`solv.zBTC.resolve`, async () => {
    await expect(ctx.resolve(true)).resolves.toMatchInlineSnapshot(`
      {
        "admin": "7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE",
        "delegateRewardTokenAdmin": "7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE",
        "delegatedRewardTokens": [
          {
            "amount": 0n,
            "delegate": {
              "__option": "Some",
              "value": "7hc9hViLRVvoTkJj8515EPM6CuJa7Sep8v5AFoTE8jYE",
            },
            "mint": "ZEUS1aR7aX8DFFJf5QjWj2ftDDdNTroMNGo8YoQm3Gq",
          },
        ],
        "receiptTokenDecimals": 8,
        "receiptTokenLockedAmount": 0n,
        "receiptTokenMint": "DNLsKFnrBjTBKp1eSwt8z1iNu2T2PL3MnxZFsGEEpQCf",
        "receiptTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "receiptTokenSupply": 0n,
        "supportedTokenAmount": 0n,
        "supportedTokenDecimals": 8,
        "supportedTokenMint": "zBTCug3er3tLyffELcvDNrKkCymbPWysGcWihESYfLg",
        "supportedTokenProgram": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      }
    `);
  });
});
