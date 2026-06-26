// Reward Distribution & Treasury Management Tests
// Comprehensive testing of reward distribution and treasury operations

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

describe('💰 Reward Distribution & Treasury Management Tests', () => {
  let client: any;
  let environment: EnvironmentSetup;
  let stateManager: TestStateManager;

  beforeAll(async () => {
    environment = new EnvironmentSetup();
    await environment.setupTestEnvironment();
    stateManager = environment.getStateManager();
    
    client = createClient(host, port);
    console.log('💰 Reward and treasury tests initialized');
  }, 60000);

  afterAll(async () => {
    await environment.cleanupTestEnvironment();
    console.log('✅ Reward and treasury tests completed');
  }, 60000);

  describe('1. Treasury Fund Management', () => {
    it('should process treasury deposits and withdrawals', async () => {
      const depositAmount = '1000000';
      const withdrawAmount = '500000';
      const user = TEST_USERS[0];

      const initialTreasury = await callRpc<any>(client, 'GetTreasuryBalance', {});
      expect(initialTreasury).toBeDefined();

      const depositRes = await callRpc<any>(client, 'DepositToTreasury', {
        user_address: user.publicAddress,
        amount: depositAmount,
        signature: await generateSignature(user.publicAddress, 100),
        nonce: 100,
      });
      expect(depositRes.success).toBe(true);

      const treasuryAfterDeposit = await callRpc<any>(client, 'GetTreasuryBalance', {});
      expect(BigInt(treasuryAfterDeposit.balance)).toBeGreaterThanOrEqual(BigInt(initialTreasury.balance));

      const withdrawRes = await callRpc<any>(client, 'WithdrawFromTreasury', {
        recipient: TestUtils.generateRandomAddress(),
        amount: withdrawAmount,
        signature: await generateSignature(user.publicAddress, 101),
        nonce: 101,
      });
      expect(withdrawRes.success).toBe(true);

      const treasuryAfterWithdraw = await callRpc<any>(client, 'GetTreasuryBalance', {});
      expect(BigInt(treasuryAfterWithdraw.balance)).toBeLessThanOrEqual(BigInt(treasuryAfterDeposit.balance));
    }, 60000);

    it('should process multi-signature treasury operations', async () => {
      const amount = '2000000';
      const approvers = [TEST_USERS[0].publicAddress, TEST_USERS[1].publicAddress];
      const requiredSignatures = 2;

      const multiSigRes = await callRpc<any>(client, 'ExecuteMultiSigTreasuryOperation', {
        operation_type: 'withdraw',
        amount: amount,
        recipient: TestUtils.generateRandomAddress(),
        required_signatures: requiredSignatures,
        approvers: approvers,
        signatures: approvers.map((_, index) => Buffer.from(`signature_${index}`)),
        nonce: 102,
      });
      expect(multiSigRes.success).toBe(true);
    }, 60000);

    it('should handle emergency withdrawal scenarios', async () => {
      const emergencyAmount = '5000000';
      const user = TEST_USERS[0];

      const emergencyRes = await callRpc<any>(client, 'EmergencyWithdraw', {
        amount: emergencyAmount,
        justification: 'Security concern',
        signature: await generateSignature(user.publicAddress, 103),
        nonce: 103,
      });
      expect(emergencyRes.success).toBe(true);

      const triggerThreshold = '10000000';
      const checkThresholdRes = await callRpc<any>(client, 'CheckEmergencyWithdrawalThreshold', {
        threshold: triggerThreshold,
      });
      expect(checkThresholdRes).toBeDefined();
    }, 60000);
  });

  describe('2. Reward Distribution Mechanics', () => {
    it('should process batch reward distribution to many users', async () => {
      const users = TEST_USERS.slice(0, 3);
      const totalDeposits = '10000';

      const distributionPromises = users.map(async (user, index) => {
        await callRpc<any>(client, 'Deposit', {
          user_address: user.publicAddress,
          token_address: E2E_TEST_CONFIG.depositToken.address,
          amount: totalDeposits,
          signature: await generateSignature(user.publicAddress, 200 + index),
          nonce: 200 + index,
        });
        await stateManager.deposit(user.publicAddress, totalDeposits);
      });

      await Promise.all(distributionPromises);

      const distributionAmount = '1500000';
      const batchDistRes = await callRpc<any>(client, 'DistributeRewards', {
        reward_token: E2E_TEST_CONFIG.rewardToken.address,
        total_amount: distributionAmount,
        signature: Buffer.from('batch_admin_signature'),
        nonce: 204,
      });
      expect(batchDistRes.success).toBe(true);

      const rewardPromises = users.map(async user => {
        return await callRpc<any>(client, 'GetRewards', {
          user_address: user.publicAddress,
        });
      });

      const userRewards = await Promise.all(rewardPromises);
      userRewards.forEach(rewards => {
        expect(rewards).toBeDefined();
        expect(rewards.total_rewards).toBeDefined();
        expect(rewards.claimable_rewards).toBeDefined();
      });
    }, 90000);

    it('should handle rewards with vesting schedules', async () => {
      const user = TEST_USERS[0];
      const depositAmount = '3000';
      const lockDuration = 86400 * 30;

      await callRpc<any>(client, 'Deposit', {
        user_address: user.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: depositAmount,
        signature: await generateSignature(user.publicAddress, 300),
        nonce: 300,
      });
      await stateManager.deposit(user.publicAddress, depositAmount);

      await callRpc<any>(client, 'Lock', {
        user_address: user.publicAddress,
        amount: depositAmount,
        duration_seconds: lockDuration,
        signature: await generateSignature(user.publicAddress, 301),
        nonce: 301,
      });

      const claimRes = await callRpc<any>(client, 'ClaimRewards', {
        user_address: user.publicAddress,
        signature: await generateSignature(user.publicAddress, 302),
        nonce: 302,
      });
      expect(claimRes.success).toBe(true);

      const vestingRes = await callRpc<any>(client, 'GetVestedRewards', {
        user_address: user.publicAddress,
      });
      expect(vestingRes).toBeDefined();
      expect(BigInt(vestingRes.vested_amount)).toBeGreaterThanOrEqual(BigInt('0'))
    }, 60000);

    it('should process rewards for staked positions', async () => {
      const user = TEST_USERS[1];
      const stakeAmount = '2000';
      const lockDuration = 86400 * 7;

      await callRpc<any>(client, 'Deposit', {
        user_address: user.publicAddress,
        token_address: E2E_TEST_CONFIG.depositToken.address,
        amount: stakeAmount,
        signature: await generateSignature(user.publicAddress, 400),
        nonce: 400,
      });
      await stateManager.deposit(user.publicAddress, stakeAmount);

      await callRpc<any>(client, 'Lock', {
        user_address: user.publicAddress,
        amount: stakeAmount,
        duration_seconds: lockDuration,
        signature: await generateSignature(user.publicAddress, 401),
        nonce: 401,
      });

      const stakedAmountRes = await callRpc<any>(client, 'GetStakedBalance', {
        user_address: user.publicAddress,
      });
      expect(BigInt(stakedAmountRes.balance)).toBeGreaterThanOrEqual(BigInt('1'));

      const distributeRes = await callRpc<any>(client, 'DistributeRewards', {
        reward_token: E2E_TEST_CONFIG.rewardToken.address,
        total_amount: '500000',
        signature: Buffer.from('staking_admin_signature'),
        nonce: 402,
      });
      expect(distributeRes.success).toBe(true);

      const rewardRes = await callRpc<any>(client, 'GetRewards', {
        user_address: user.publicAddress,
      });
      expect(rewardRes).toBeDefined();
      expect(BigInt(rewardRes.claimable_rewards)).toBeGreaterThanOrEqual(BigInt('1'));
    }, 90000);
  });

  describe('3. Protocol Governance & Treasury', () => {
    it('should process budget approval workflow', async () => {
      const budgetAmount = '1000000';
      const description = 'Infrastructure upgrade';
      const deadline = Math.floor(Date.now() / 1000) + 86400 * 7;

      const submitRes = await callRpc<any>(client, 'SubmitBudgetProposal', {
        amount: budgetAmount,
        description: description,
        deadline: deadline,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('budget_proposal_signature'),
        nonce: 500,
      });
      expect(submitRes.success).toBe(true);

      const approveRes = await callRpc<any>(client, 'ApproveBudget', {
        proposal_id: submitRes.proposal_id,
        approver: TEST_USERS[1].publicAddress,
        signature: Buffer.from('approval_signature'),
        nonce: 501,
      });
      expect(approveRes.success).toBe(true);

      const executeRes = await callRpc<any>(client, 'ExecuteBudget', {
        proposal_id: submitRes.proposal_id,
        signature: Buffer.from('execution_signature'),
        nonce: 502,
      });
      expect(executeRes.success).toBe(true);
    }, 60000);

    it('should process parameter upgrade proposals', async () => {
      const parameterName = 'reward_rate';
      const newValue = '1500';
      const deadline = Math.floor(Date.now() / 1000) + 86400 * 14;

      const paramRes = await callRpc<any>(client, 'SubmitParameterUpgrade', {
        name: parameterName,
        new_value: newValue,
        deadline: deadline,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('parameter_upgrade_signature'),
        nonce: 600,
      });
      expect(paramRes.success).toBe(true);

      const voteRes = await callRpc<any>(client, 'VoteOnParameterUpgrade', {
        proposal_id: paramRes.proposal_id,
        vote: 'yes',
        voter: TEST_USERS[1].publicAddress,
        signature: Buffer.from('vote_signature'),
        nonce: 601,
      });
      expect(voteRes.success).toBe(true);

      const executeParamRes = await callRpc<any>(client, 'ExecuteParameterUpgrade', {
        proposal_id: paramRes.proposal_id,
        signature: Buffer.from('parameter_execution_signature'),
        nonce: 602,
      });
      expect(executeParamRes.success).toBe(true);
    }, 60000);

    it('should process emergency governance actions', async () => {
      const emergencyAction = 'freeze_all_operations';
      const justification = 'Security vulnerability detected';
      const deadline = Math.floor(Date.now() / 1000) + 3600;

      const emergencyRes = await callRpc<any>(client, 'SubmitEmergencyProposal', {
        action: emergencyAction,
        justification: justification,
        deadline: deadline,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('emergency_proposal_signature'),
        nonce: 700,
      });
      expect(emergencyRes.success).toBe(true);

      const fastTrackRes = await callRpc<any>(client, 'FastTrackEmergencyProposal', {
        proposal_id: emergencyRes.proposal_id,
        signature: Buffer.from('fast_track_signature'),
        nonce: 701,
      });
      expect(fastTrackRes.success).toBe(true);

      const executeEmergencyRes = await callRpc<any>(client, 'ExecuteEmergencyProposal', {
        proposal_id: emergencyRes.proposal_id,
        signature: Buffer.from('emergency_execution_signature'),
        nonce: 702,
      });
      expect(executeEmergencyRes.success).toBe(true);
    }, 60000);
  });
});
