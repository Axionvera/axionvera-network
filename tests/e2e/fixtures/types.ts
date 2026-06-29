// Type definitions and fixtures for E2E testing

export interface User {
  publicAddress: string;
  privateKey?: string;
  nonce?: number;
  balance?: string;
  staked?: boolean;
}

export interface Token {
  address: string;
  type: "deposit" | "reward";
  owner?: string;
}

export interface VaultState {
  contractAddress: string;
  admin: string;
  depositToken: Token;
  rewardToken: Token;
  totalDeposits: string;
  totalRewards: string;
  lastDistribution: number;
}

export interface RewardDistribution {
  totalAmount: string;
  recipients: Array<{ user: string; amount: string; rewardIndex: string }>;
  timestamp: number;
}

export interface TreasuryOperation {
  operationType: "deposit" | "withdraw" | "transfer";
  amount: string;
  status: "pending" | "approved" | "rejected" | "completed";
  timestamp: number;
  approvers: string[];
}

export interface GovernanceProposal {
  proposalId: string;
  proposer: string;
  title: string;
  description: string;
  type: "parameter" | "treasury" | "upgrade";
  status: "pending" | "active" | "passed" | "rejected" | "executed";
  votes: { for: number; against: number; abstain: number };
  threshold: number;
  deadline: number;
  targets?: string[];
  values?: string[];
  calldatas?: string[];
}

export interface StakingPosition {
  user: string;
  amount: string;
  lockUntil: number;
  unlockedAt: number;
  claimed: boolean;
}

// Test accounts and configurations
export const E2E_TEST_CONFIG = {
  host: process.env.TEST_NODE_HOST || "localhost",
  port: process.env.TEST_NODE_PORT || "50051",
  adminWallet: {
    address: "GBA_ADMIN0000000000000000000000000000000000000000000000",
    privateKey: "admin_private_key_123",
  },
  depositToken: {
    address: "CDA_DEPOSIT_TOKEN_ADDRESS",
    name: "Deposit Token",
  },
  rewardToken: {
    address: "CREWARD_REWARD_TOKEN_ADDRESS",
    name: "Reward Token",
  },
  contractAddress: "CAXIONVERA001",
  tokensPerUser: {
    alice: "1000",
    bob: "1500",
    charlie: "2000",
    dave: "500",
    eve: "3000",
  },
  rewardDistributionAmount: "500000",
  minStakeAmount: "100",
  minRewardDistribution: "1000",
};

// Test user definitions
export const TEST_USERS: User[] = [
  {
    publicAddress: "GALICE1234567890ABCDEFGHIJ1234567890ABCDEFGHIJKL",
    privateKey: "alice_private_key_123",
    nonce: 1,
  },
  {
    publicAddress: "GBOB9876543210ABCDEFGHIJKLMNOPQRSTUVWXYZ0987654321",
    privateKey: "bob_private_key_456",
    nonce: 1,
  },
  {
    publicAddress: "GCHARLIE111222333444555666777888999000111222333444",
    privateKey: "charlie_private_key_789",
    nonce: 1,
  },
  {
    publicAddress: "GDAVE444555666777888999000111222333444555666777888",
    privateKey: "dave_private_key_012",
    nonce: 1,
  },
  {
    publicAddress: "GEVE999000111222333444555666777888999000111222333",
    privateKey: "eve_private_key_345",
    nonce: 1,
  },
];

// Expected event types
export enum ExpectedEventTypes {
  DEPOSIT = "Deposit",
  WITHDRAWAL = "Withdrawal",
  REWARD_DISTRIBUTION = "RewardDistribution",
  REWARDS_CLAIMED = "RewardsClaimed",
  ADMIN_TRANSFER_PROPOSED = "AdminTransferProposed",
  ADMIN_TRANSFER_ACCEPTED = "AdminTransferAccepted",
  LOCK = "Lock",
  UNLOCK = "Unlock",
}

// System states and transitions
export const SYSTEM_STATES = {
  NOT_INITIALIZED: "Not Initialized",
  INITIALIZED: "Initialized",
  PAUSED: "Paused",
  ACTIVE: "Active",
};

// Expected error messages
export const EXPECTED_ERRORS = {
  INSUFFICIENT_BALANCE: "Insufficient balance",
  INVALID_AMOUNT: "Invalid amount",
  UNAUTHORIZED: "Unauthorized operation",
  REEJECT_SIGNATURE: "Invalid signature",
  DUPLICATE_NONCE: "Nonce already used",
  INSUFFICIENT_REWARDS: "Insufficient rewards",
  CONTRACT_PAUSED: "Contract is paused",
  INVALID_ADDRESS: "Invalid address",
};
