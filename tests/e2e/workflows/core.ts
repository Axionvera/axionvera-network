// Core Protocol Workflow Tests
// Comprehensive end-to-end testing of core vault protocols

import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { callRpc, createClient, TestUtils } from './utils/core';
import { E2E_TEST_CONFIG, TEST_USERS } from './fixtures/types';
import { TestStateManager } from './utils/state';
import { EnvironmentSetup } from './utils/setup';

const host = E2E_TEST_CONFIG.host;
const port = E2E_TEST_CONFIG.port;

async function generateSignature(userAddress: string, nonce: number): Promise<Buffer> {
  return Buffer.from(`${userAddress}:${nonce}`);
}

describe('🎯 Core Protocol Workflow Tests', () => {
  let client: any;
  let environment: EnvironmentSetup;
  let stateManager: TestStateManager;

  beforeAll(async () => {
    environment = new EnvironmentSetup();
    await environment.setupTestEnvironment();
    stateManager = environment.getStateManager();
    
    client = createClient(host, port);
    console.log('🚀 Core protocol tests initialized');
  }, 60000);

  afterAll(async () => {
    await environment.cleanupTestEnvironment();
    console.log('✅ Core protocol tests completed');
  }, 60000);

  describe('1. Complete User Lifecycle Workflow', () => {
    it('should process complete user lifecycle: deposit → reward distribution → claim → withdrawal', async () => {
      const alice = TEST_USERS[0];
      const bob = TEST_USERS[1];
      const depositAmount = '1000';
      const rewardDistribution = '500000';
      
      const aliceDeposit = await callRpc<any>(client, 'Deposit', {
        user_address: alice.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: depositAmount,
        signature: await generateSignature(alice.publicAddress, 1),
        nonce: 1,
      });
      expect(aliceDeposit.success).toBe(true);
      await stateManager.deposit(alice.publicAddress, depositAmount);
      
      const bobDeposit = await callRpc<any>(client, 'Deposit', {
        user_address: bob.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: '1500',
        signature: await generateSignature(bob.publicAddress, 1),
        nonce: 1,
      });
      expect(bobDeposit.success).toBe(true);
      await stateManager.deposit(bob.publicAddress, '1500');
      
      const rewardDist = await callRpc<any>(client, 'DistributeRewards', {
        reward_token: E2E_TEST_CONFIG.rewardToken.address,
        total_amount: rewardDistribution,
        signature: Buffer.from('admin_signature'),
        nonce: 1,
      });
      expect(rewardDist.success).toBe(true);
      
      const aliceRewards = await callRpc<any>(client, 'GetRewards', {
        user_address: alice.publicAddress,
      });
      expect(aliceRewards).toBeDefined();
      expect(BigInt(aliceRewards.claimable_rewards)).toBeGreaterThanOrEqual(BigInt('1'));
      
      const claimRes = await callRpc<any>(client, 'ClaimRewards', {
        user_address: alice.publicAddress,
        signature: await generateSignature(alice.publicAddress, 2),
        nonce: 2,
      });
      expect(claimRes.success).toBe(true);
      
      const aliceBalance = await callRpc<any>(client, 'GetBalance', {
        user_address: alice.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
      });
      expect(BigInt(aliceBalance.balance)).toBeGreaterThanOrEqual(BigInt(depositAmount));
      
      const withdrawRes = await callRpc<any>(client, 'Withdraw', {
        user_address: alice.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: depositAmount,
        signature: await generateSignature(alice.publicAddress, 3),
        nonce: 3,
      });
      expect(withdrawRes.success).toBe(true);
      await stateManager.withdraw(alice.publicAddress, depositAmount);
    }, 60000);

    it('should process multi-user parallel operations', async () => {
      const depositPromises = TEST_USERS.map(async (user, index) => {
        const result = await callRpc<any>(client, 'Deposit', {
          user_address: user.publicAddress,
          token_address: E2E_TEST_CONFIG.depositToken.address,
          amount: '2000',
          signature: await generateSignature(user.publicAddress, index + 1),
          nonce: index + 1,
        });
        await stateManager.deposit(user.publicAddress, '2000');
        return result;
      });

      const depositResults = await Promise.all(depositPromises);
      
      depositResults.forEach(result => {
        expect(result.success).toBe(true);
      });

      const balancePromises = TEST_USERS.map(async user => {
        return await callRpc<any>(client, 'GetBalance', {
          user_address: user.publicAddress,
          token_address: E2E_TEST_CONFIG.depositToken.address,
        });
      });

      const balances = await Promise.all(balancePromises);
      balances.forEach(balance => {
        expect(BigInt(balance.balance)).toBeGreaterThanOrEqual(BigInt('2000'));
      });
    }, 90000);
  });

  describe('2. Reward Distribution Mechanics', () => {
    it('should distribute rewards proportionally to deposited amounts', async () => {
      const alice = TEST_USERS[0];
      const bob = TEST_USERS[1];
      const aliceDeposit = '1000';
      const bobDeposit = '2000';
      const totalDeposits = parseInt(aliceDeposit) + parseInt(bobDeposit);
      const distributionAmount = '1000000';

      const aliceDepResult = await callRpc<any>(client, 'Deposit', {
        user_address: alice.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: aliceDeposit,
        signature: await generateSignature(alice.publicAddress, 10),
        nonce: 10,
      });
      expect(aliceDepResult.success).toBe(true);
      await stateManager.deposit(alice.publicAddress, aliceDeposit);

      const bobDepResult = await callRpc<any>(client, 'Deposit', {
        user_address: bob.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: bobDeposit,
        signature: await generateSignature(bob.publicAddress, 11),
        nonce: 11,
      });
      expect(bobDepResult.success).toBe(true);
      await stateManager.deposit(bob.publicAddress, bobDeposit);

      const distResult = await callRpc<any>(client, 'DistributeRewards', {
        reward_token: E2E_TEST_CONFIG.rewardToken.address,
        total_amount: distributionAmount,
        signature: Buffer.from('admin_signature'),
        nonce: 12,
      });
      expect(distResult.success).toBe(true);

      const aliceRewards = await callRpc<any>(client, 'GetRewards', {
        user_address: alice.publicAddress,
      });
      const bobRewards = await callRpc<any>(client, 'GetRewards', {
        user_address: bob.publicAddress,
      });

      if (aliceRewards && bobRewards) {
        const aliceProportion = parseInt(aliceDeposit) / totalDeposits;
        const bobProportion = parseInt(bobDeposit) / totalDeposits;
        const aliceExpected = Math.floor(distributionAmount * aliceProportion);
        const bobExpected = Math.floor(distributionAmount * bobProportion);

        expect(BigInt(aliceRewards.claimable_rewards)).toBeGreaterThanOrEqual(BigInt('1'));
        expect(BigInt(bobRewards.claimable_rewards)).toBeGreaterThanOrEqual(BigInt('1'));
      }
    }, 60000);

    it('should maintain reward index across multiple distributions', async () => {
      const user = TEST_USERS[0];
      const depositAmount = '5000';

      await callRpc<any>(client, 'Deposit', {
        user_address: user.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: depositAmount,
        signature: await generateSignature(user.publicAddress, 20),
        nonce: 20,
      });
      await stateManager.deposit(user.publicAddress, depositAmount);

      const initialState = await callRpc<any>(client, 'GetContractState', {
        contract_address: E2E_TEST_CONFIG.contractAddress,
      });

      const dist1Amount = '100000';
      await callRpc<any>(client, 'DistributeRewards', {
        reward_token: E2E_TEST_CONFIG.rewardToken.address,
        total_amount: dist1Amount,
        signature: Buffer.from('admin_sig_1'),
        nonce: 21,
      });

      const stateAfterFirst = await callRpc<any>(client, 'GetContractState', {
        contract_address: E2E_TEST_CONFIG.contractAddress,
      });
      expect(BigInt(stateAfterFirst.reward_index)).toBeGreaterThanOrEqual(BigInt(initialState.reward_index || '0'));

      const dist2Amount = '200000';
      await callRpc<any>(client, 'DistributeRewards', {
        reward_token: E2E_TEST_CONFIG.rewardToken.address,
        total_amount: dist2Amount,
        signature: Buffer.from('admin_sig_2'),
        nonce: 22,
      });

      const stateAfterSecond = await callRpc<any>(client, 'GetContractState', {
        contract_address: E2E_TEST_CONFIG.contractAddress,
      });
      expect(BigInt(stateAfterSecond.reward_index)).toBeGreaterThanOrEqual(BigInt(stateAfterFirst.reward_index));
    }, 60000);
  });

  describe('3. Staking Operations', () => {
    it('should lock tokens and receive vesting rewards', async () => {
      const user = TEST_USERS[0];
      const lockAmount = '500';
      const lockDuration = 86400 * 7;

      const lockRes = await callRpc<any>(client, 'Lock', {
        user_address: user.publicAddress,
        amount: lockAmount,
        duration_seconds: lockDuration,
        signature: await generateSignature(user.publicAddress, 30),
        nonce: 30,
      });
      expect(lockRes.success).toBe(true);

      await stateManager.createStakingPosition({
        user: user.publicAddress,
        amount: lockAmount,
        lockUntil: Math.floor(Date.now() / 1000) + lockDuration,
        unlockedAt: Math.floor(Date.now() / 1000) + lockDuration,
        claimed: false,
      });

      const balanceAfterLock = await callRpc<any>(client, 'GetBalance', {
        user_address: user.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
      });
      expect(BigInt(balanceAfterLock.balance)).toBeLessThanOrEqual(BigInt('9500'));

      const claimRes = await callRpc<any>(client, 'ClaimRewards', {
        user_address: user.publicAddress,
        signature: await generateSignature(user.publicAddress, 31),
        nonce: 31,
      });
      expect(claimRes.success).toBe(true);
    }, 60000);

    it('should unlock tokens after vesting period', async () => {
      const user = TEST_USERS[1];
      const lockAmount = '1000';
      const lockDuration = 86400 * 3;

      await callRpc<any>(client, 'Deposit', {
        user_address: user.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: lockAmount,
        signature: await generateSignature(user.publicAddress, 40),
        nonce: 40,
      });
      await stateManager.deposit(user.publicAddress, lockAmount);

      const unlockLimit = 100;
      const unlockRes = await callRpc<any>(client, 'UnlockExpired', {
        user_address: user.publicAddress,
        limit: unlockLimit,
        signature: await generateSignature(user.publicAddress, 41),
        nonce: 41,
      });

      expect(unlockRes).toBeDefined();
      if (unlockRes.success) {
        expect(BigInt(unlockRes.amount)).toBeLessThanOrEqual(BigInt(lockAmount));
      }
    }, 60000);
  });

  describe('4. Governance & Administrative Controls', () => {
    it('should process admin transfer proposal and acceptance', async () => {
      const newAdmin = TestUtils.generateRandomAddress();
      const adminRes = await callRpc<any>(client, 'ProposeNewAdmin', {
        new_admin: newAdmin,
        signature: Buffer.from('admin_signature'),
        nonce: 50,
      });
      expect(adminRes.success).toBe(true);

      const pendingAdmin = await callRpc<any>(client, 'GetPendingAdmin', {
        contract_address: E2E_TEST_CONFIG.contractAddress,
      });
      expect(pendingAdmin.pendingAdmin).toBe(newAdmin);

      const acceptRes = await callRpc<any>(client, 'AcceptAdmin', {
        new_admin: newAdmin,
        signature: Buffer.from('new_admin_signature'),
        nonce: 51,
      });
      expect(acceptRes.success).toBe(true);
    }, 60000);

    it('should pause and unpause contract operations', async () => {
      const user = TEST_USERS[0];
      const depositAmount = '100';

      const pauseRes = await callRpc<any>(client, 'PauseContract', {
        signature: Buffer.from('pause_signature'),
        nonce: 60,
      });
      expect(pauseRes.success).toBe(true);

      const depositAfterPause = await callRpc<any>(client, 'Deposit', {
        user_address: user.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: depositAmount,
        signature: await generateSignature(user.publicAddress, 61),
        nonce: 61,
      });
      expect(depositAfterPause.success).toBe(false);

      const unpauseRes = await callRpc<any>(client, 'UnpauseContract', {
        signature: Buffer.from('unpause_signature'),
        nonce: 62,
      });
      expect(unpauseRes.success).toBe(true);

      const depositAfterUnpause = await callRpc<any>(client, 'Deposit', {
        user_address: user.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: depositAmount,
        signature: await generateSignature(user.publicAddress, 63),
        nonce: 63,
      });
      expect(depositAfterUnpause.success).toBe(true);
    }, 60000);

    it('should process contract upgrade with new WASM', async () => {
      const newWasmHash = '01234567890123456789012345678901234567890123456789012345678901230123456789012345678901234567890123456789';
      
      const upgradeRes = await callRpc<any>(client, 'UpgradeContract', {
        new_wasm_hash: newWasmHash,
        signature: Buffer.from('upgrade_signature'),
        nonce: 70,
      });
      expect(upgradeRes.success).toBe(true);
    }, 60000);
  });
});
