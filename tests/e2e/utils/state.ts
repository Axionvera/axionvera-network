// State management utilities for E2E tests

import {
  VaultState,
  RewardDistribution,
  TreasuryOperation,
  GovernanceProposal,
  StakingPosition,
  E2E_TEST_CONFIG,
} from "./fixtures/types";

export class TestStateManager {
  private state: Partial<VaultState> = {};

  async initializeState(): Promise<void> {
    this.state = {
      contractAddress: E2E_TEST_CONFIG.contractAddress,
      admin: E2E_TEST_CONFIG.adminWallet.address,
      depositToken: {
        address: E2E_TEST_CONFIG.depositToken.address,
        type: "deposit" as const,
        owner: E2E_TEST_CONFIG.adminWallet.address,
      },
      rewardToken: {
        address: E2E_TEST_CONFIG.rewardToken.address,
        type: "reward" as const,
        owner: E2E_TEST_CONFIG.adminWallet.address,
      },
      totalDeposits: "0",
      totalRewards: "0",
      lastDistribution: 0,
    };
  }

  async deposit(userAddress: string, amount: string): Promise<void> {
    const currentDeposits = BigInt(this.state.totalDeposits || "0");
    const newDeposits = currentDeposits + BigInt(amount);
    this.state.totalDeposits = newDeposits.toString();
  }

  async withdraw(userAddress: string, amount: string): Promise<void> {
    const currentDeposits = BigInt(this.state.totalDeposits || "0");
    const newDeposits = currentDeposits - BigInt(amount);
    if (newDeposits < 0) {
      throw new Error("Insufficient balance");
    }
    this.state.totalDeposits = newDeposits.toString();
  }

  async distributeRewards(distribution: RewardDistribution): Promise<void> {
    const totalRewards = this.state.totalRewards
      ? BigInt(this.state.totalRewards)
      : 0n;
    const newTotal = totalRewards + BigInt(distribution.totalAmount);
    this.state.totalRewards = newTotal.toString();
    this.state.lastDistribution = distribution.timestamp;
  }

  async executeTreasuryOperation(operation: TreasuryOperation): Promise<void> {
    if (operation.operationType === "deposit") {
      const totalRewards = this.state.totalRewards
        ? BigInt(this.state.totalRewards)
        : 0n;
      const newTotal = totalRewards + BigInt(operation.amount);
      this.state.totalRewards = newTotal.toString();
    } else if (operation.operationType === "withdraw") {
      const totalRewards = this.state.totalRewards
        ? BigInt(this.state.totalRewards)
        : 0n;
      const newTotal = totalRewards - BigInt(operation.amount);
      if (newTotal < 0) {
        throw new Error("Insufficient treasury balance");
      }
      this.state.totalRewards = newTotal.toString();
    }
  }

  async createGovernanceProposal(proposal: GovernanceProposal): Promise<void> {
    const newProposalId = `proposal_${Date.now()}`;
    proposal.proposalId = newProposalId;
  }

  async createStakingPosition(position: StakingPosition): Promise<void> {
    if (BigInt(position.amount) < BigInt(E2E_TEST_CONFIG.minStakeAmount)) {
      throw new Error("Amount below minimum stake requirement");
    }
  }

  getState(): VaultState {
    return this.state as VaultState;
  }

  reset(): void {
    this.state = {};
  }
}
