// Governance & Parameter Management Tests
// Comprehensive testing of protocol governance and parameter management

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

describe('🏛️ Governance & Parameter Management Tests', () => {
  let client: any;
  let environment: EnvironmentSetup;
  let stateManager: TestStateManager;

  beforeAll(async () => {
    environment = new EnvironmentSetup();
    await environment.setupTestEnvironment();
    stateManager = environment.getStateManager();
    
    client = createClient(host, port);
    console.log('🏛️ Governance tests initialized');
  }, 60000);

  afterAll(async () => {
    await environment.cleanupTestEnvironment();
    console.log('✅ Governance tests completed');
  }, 60000);

  describe('1. Protocol Constitution & Core Parameters', () => {
    it('should retrieve current protocol parameters', async () => {
      const paramsRes = await callRpc<any>(client, 'GetChainParameters', {});
      expect(paramsRes).toBeDefined();
      expect(paramsRes.chain_id).toBeDefined();
      expect(paramsRes.reward_rate).toBeDefined();
      expect(paramsRes.max_stake_amount).toBeDefined();
      expect(paramsRes.min_reward_distribution).toBeDefined();
      expect(paramsRes.vesting_period).toBeDefined();
    }, 60000);

    it('should query active protocol upgrades', async () => {
      const upgradesRes = await callRpc<any>(client, 'ListPendingParameterUpgrades', {});
      expect(upgradesRes).toBeDefined();
      expect(Array.isArray(upgradesRes.pending)).toBe(true);
    }, 60000);

    it('should validate protocol parameter bounds', async () => {
      const validationRes = await callRpc<any>(client, 'ValidateParameters', {
        parameters: {
          reward_rate: '2000',
          max_stake_amount: '1000000',
          min_reward_distribution: '1000',
        },
      });
      expect(validationRes).toBeDefined();
      expect(validationRes.valid).toBe(true);
    }, 60000);
  });

  describe('2. Admin Transfer & Permission Management', () => {
    it('should process complete admin transfer workflow', async () => {
      const currentAdmin = E2E_TEST_CONFIG.adminWallet.address;
      const newAdmin = TestUtils.generateRandomAddress();
      const mockNewAdmin = 'GBA_NEW_ADMIN1234567890ABCDEFGHIJ1234567890ABCDEFGHIJKL';

      const proposeRes = await callRpc<any>(client, 'ProposeNewAdmin', {
        new_admin: mockNewAdmin,
        signature: Buffer.from('admin_transfer_signature'),
        nonce: 800,
      });
      expect(proposeRes.success).toBe(true);

      const pendingRes = await callRpc<any>(client, 'GetPendingAdmin', {
        contract_address: E2E_TEST_CONFIG.contractAddress,
      });
      expect(pendingRes.pending_admin).toBe(mockNewAdmin);

      const acceptRes = await callRpc<any>(client, 'AcceptAdmin', {
        new_admin: mockNewAdmin,
        signature: Buffer.from('accept_admin_signature'),
        nonce: 801,
      });
      expect(acceptRes.success).toBe(true);

      const finalPendingRes = await callRpc<any>(client, 'GetPendingAdmin', {
        contract_address: E2E_TEST_CONFIG.contractAddress,
      });
      expect(finalPendingRes.pending_admin).toBeNull();

      const newAdminAfterAccept = await callRpc<any>(client, 'GetAdmin', {
        contract_address: E2E_TEST_CONFIG.contractAddress,
      });
      expect(newAdminAfterAccept.admin).toBe(mockNewAdmin);
    }, 60000);

    it('should reject unauthorized admin transfer attempts', async () => {
      const unauthorizedUser = TEST_USERS[0];
      const proposedNewAdmin = TestUtils.generateRandomAddress();

      const unauthorizedRes = await callRpc<any>(client, 'ProposeNewAdmin', {
        new_admin: proposedNewAdmin,
        signature: await generateSignature(unauthorizedUser.publicAddress, 802),
        nonce: 802,
      });
      expect(unauthorizedRes.success).toBe(false);
      expect(unauthorizedRes.error_message).toBeDefined();
    }, 60000);

    it('should handle admin transfer rejection prevention', async () => {
      const mockNewAdmin1 = 'GBA_NEW_ADMIN1123456789ABCDEFGHIJ1234567890ABCDEFGHIJKL';
      const mockNewAdmin2 = 'GBA_NEW_ADMIN2234567890ABCDEFGHIJ2345678901ABCDEFGHIJKM';

      const propose1 = await callRpc<any>(client, 'ProposeNewAdmin', {
        new_admin: mockNewAdmin1,
        signature: Buffer.from('proposal_1_signature'),
        nonce: 900,
      });
      expect(propose1.success).toBe(true);

      const unauthorizedTry = await callRpc<any>(client, 'ProposeNewAdmin', {
        new_admin: mockNewAdmin2,
        signature: Buffer.from('unauthorized_proposal_signature'),
        nonce: 901,
      });
      expect(unauthorizedTry.success).toBe(false);
    }, 60000);
  });

  describe('3. Parameter Upgrade Procedures', () => {
    it('should submit and execute parameter upgrade proposal', async () => {
      const upgradeRes = await callRpc<any>(client, 'SubmitParameterUpgrade', {
        name: 'reward_rate',
        new_value: '2500',
        deadline: Math.floor(Date.now() / 1000) + 86400 * 7,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('parameter_upgrade_signature'),
        nonce: 1000,
      });
      expect(upgradeRes.success).toBe(true);
      expect(upgradeRes.proposal_id).toBeDefined();

      const voteRes = await callRpc<any>(client, 'VoteOnParameterUpgrade', {
        proposal_id: upgradeRes.proposal_id,
        vote: 'yes',
        voter: TEST_USERS[1].publicAddress,
        signature: Buffer.from('vote_signature_yes'),
        nonce: 1001,
      });
      expect(voteRes.success).toBe(true);

      const anotherVoteRes = await callRpc<any>(client, 'VoteOnParameterUpgrade', {
        proposal_id: upgradeRes.proposal_id,
        vote: 'no',
        voter: TEST_USERS[2].publicAddress,
        signature: Buffer.from('vote_signature_no'),
        nonce: 1002,
      });
      expect(anotherVoteRes.success).toBe(true);

      const executeRes = await callRpc<any>(client, 'ExecuteParameterUpgrade', {
        proposal_id: upgradeRes.proposal_id,
        signature: Buffer.from('parameter_execution_signature'),
        nonce: 1003,
      });
      expect(executeRes.success).toBe(true);
    }, 60000);

    it('should reject parameter upgrade with insufficient votes', async () => {
      const proposalRes = await callRpc<any>(client, 'SubmitParameterUpgrade', {
        name: 'vesting_period',
        new_value: '30',
        deadline: Math.floor(Date.now() / 1000) + 86400,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('insufficient_votes_signature'),
        nonce: 1100,
      });
      expect(proposalRes.success).toBe(true);

      const executeFailRes = await callRpc<any>(client, 'ExecuteParameterUpgrade', {
        proposal_id: proposalRes.proposal_id,
        signature: Buffer.from('execution_signature_fail'),
        nonce: 1101,
      });
      expect(executeFailRes.success).toBe(false);
    }, 60000);

    it('should handle parameter upgrade deadline expiration', async () => {
      const expiredDeadline = Math.floor(Date.now() / 1000) - 3600;
      const expiredProposalRes = await callRpc<any>(client, 'SubmitParameterUpgrade', {
        name: 'protocol_fee',
        new_value: '500',
        deadline: expiredDeadline,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('expired_proposal_signature'),
        nonce: 1200,
      });
      expect(expiredProposalRes.success).toBe(true);

      const expiredExecuteRes = await callRpc<any>(client, 'ExecuteParameterUpgrade', {
        proposal_id: expiredProposalRes.proposal_id,
        signature: Buffer.from('expired_execution_signature'),
        nonce: 1201,
      });
      expect(expiredExecuteRes.success).toBe(false);
    }, 60000);
  });

  describe('4. Treasury Management Governance', () => {
    it('should process treasury budget proposal workflow', async () => {
      const budgetRes = await callRpc<any>(client, 'SubmitBudgetProposal', {
        amount: '2000000',
        description: 'Emergency infrastructure upgrade',
        deadline: Math.floor(Date.now() / 1000) + 86400 * 14,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('budget_proposal_signature'),
        nonce: 1300,
      });
      expect(budgetRes.success).toBe(true);

      const approveRes = await callRpc<any>(client, 'ApproveBudget', {
        proposal_id: budgetRes.proposal_id,
        approver: TEST_USERS[1].publicAddress,
        signature: Buffer.from('budget_approval_signature'),
        nonce: 1301,
      });
      expect(approveRes.success).toBe(true);

      const executeBudgetRes = await callRpc<any>(client, 'ExecuteBudget', {
        proposal_id: budgetRes.proposal_id,
        signature: Buffer.from('budget_execution_signature'),
        nonce: 1302,
      });
      expect(executeBudgetRes.success).toBe(true);
    }, 60000);

    it('should process multi-signature treasury governance', async () => {
      const multiSigRes = await callRpc<any>(client, 'ExecuteMultiSigTreasuryOperation', {
        operation_type: 'withdraw',
        amount: '5000000',
        recipient: TestUtils.generateRandomAddress(),
        required_signatures: 3,
        approvers: [
          TEST_USERS[0].publicAddress,
          TEST_USERS[1].publicAddress,
          TEST_USERS[2].publicAddress,
        ],
        signatures: [
          Buffer.from('multisig_sig_1'),
          Buffer.from('multisig_sig_2'),
          Buffer.from('multisig_sig_3'),
        ],
        nonce: 1400,
      });
      expect(multiSigRes.success).toBe(true);
    }, 60000);

    it('should handle emergency treasury withdrawal', async () => {
      const emergencyRes = await callRpc<any>(client, 'EmergencyTreasuryWithdrawal', {
        amount: '10000000',
        justification: 'Critical system maintenance',
        deadline: Math.floor(Date.now() / 1000) + 3600,
        signature: Buffer.from('emergency_treasury_signature'),
        nonce: 1500,
      });
      expect(emergencyRes.success).toBe(true);

      const fastTrackRes = await callRpc<any>(client, 'FastTrackEmergencyProposal', {
        proposal_id: emergencyRes.proposal_id,
        signature: Buffer.from('fast_track_treasury_signature'),
        nonce: 1501,
      });
      expect(fastTrackRes.success).toBe(true);

      const executeEmergencyRes = await callRpc<any>(client, 'ExecuteEmergencyProposal', {
        proposal_id: emergencyRes.proposal_id,
        signature: Buffer.from('emergency_treasury_execution_signature'),
        nonce: 1502,
      });
      expect(executeEmergencyRes.success).toBe(true);
    }, 60000);
  });

  describe('5. Advanced Governance Features', () => {
    it('should process proposal with complex voting weights', async () => {
      const complexProposalRes = await callRpc<any>(client, 'SubmitProposalWithWeights', {
        title: 'Complex Protocol Enhancement',
        description: 'Multi-faceted improvement',
        type: 'parameter',
        parameters: {
          'reward_rate': '3000',
          'vesting_period': '90',
        },
        targets: [
          E2E_TEST_CONFIG.contractAddress,
          'CBALANCE_TOKEN_ADDRESS',
        ],
        values: ['0', '1000000'],
        calldatas: ['encoded_data_1', 'encoded_data_2'],
        deadline: Math.floor(Date.now() / 1000) + 86400 * 10,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('complex_proposal_signature'),
        nonce: 1600,
      });
      expect(complexProposalRes.success).toBe(true);
      expect(complexProposalRes.proposal_id).toBeDefined();
    }, 60000);

    it('should handle delegation governance', async () => {
      const delegator = TEST_USERS[0].publicAddress;
      const delegatee = TEST_USERS[1].publicAddress;
      const delegatedAmount = '5000';

      const delegateRes = await callRpc<any>(client, 'DelegateGovernancePower', {
        delegatee: delegatee,
        amount: delegatedAmount,
        signature: Buffer.from('delegation_signature'),
        nonce: 1700,
      });
      expect(delegateRes.success).toBe(true);

      const voteOnDelegatedRes = await callRpc<any>(client, 'VoteWithDelegatedPower', {
        proposal_id: 'proposal_001',
        vote: 'yes',
        signature: Buffer.from('delegated_vote_signature'),
        nonce: 1701,
      });
      expect(voteOnDelegatedRes.success).toBe(true);
    }, 60000);

    it('should process constitution amendment', async () => {
      const amendmentRes = await callRpc<any>(client, 'ProposeConstitutionAmendment', {
        title: 'Protocol Constitution Update',
        description: 'Amendment to governance rules',
        changes: ['Article III', 'Article V'],
        required_majority: '2/3',
        deadline: Math.floor(Date.now() / 1000) + 86400 * 30,
        proposer: TEST_USERS[0].publicAddress,
        signature: Buffer.from('constitution_amendment_signature'),
        nonce: 1800,
      });
      expect(amendmentRes.success).toBe(true);
      expect(amendmentRes.proposal_id).toBeDefined();

      const executeAmendmentRes = await callRpc<any>(client, 'ExecuteConstitutionAmendment', {
        proposal_id: amendmentRes.proposal_id,
        signature: Buffer.from('constitution_execution_signature'),
        nonce: 1801,
      });
      expect(executeAmendmentRes.success).toBe(true);
    }, 60000);
  });
});
