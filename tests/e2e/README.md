# E2E Testing Framework

## Overview

This directory contains the comprehensive end-to-end testing framework for the Axionvera Network project. The framework covers core protocol workflows, reward distributions, staking operations, treasury actions, and governance interactions.

## Directory Structure

The E2E testing framework is organized into the following components:

### Core Framework (tests/e2e/)

- **utils/** - Shared utilities and helpers
- **fixtures/** - Test data and configuration
- **workflows/** - Complete workflow test scenarios

### Framework Components

1. **Core Protocol Workflow Tests** (workflows/core.ts)
   - Complete user lifecycle scenarios
   - Multi-user concurrent operations
   - Reward distribution mechanics
   - Staking and withdrawal operations
   - Governance administrative controls

2. **Reward Distribution & Treasury Management Tests** (workflows/rewards.ts)
   - Treasury fund management
   - Batch reward distributions
   - Vesting reward schedules
   - Staking position rewards
   - Protocol governance workflows

3. **Governance & Parameter Management Tests** (workflows/governance.ts)
   - Protocol constitution and parameters
   - Admin transfer and permission management
   - Parameter upgrade procedures
   - Treasury management governance
   - Advanced governance features

### Utilities

The framework includes comprehensive utilities:

#### Core Utilities (utils/core.ts)

- RPC client management
- User and test data generation
- Retry mechanisms for flaky tests
- State management helpers

#### Environment Setup (utils/setup.ts)

- Docker container management
- Network connectivity checks
- Contract state validation
- Test environment configuration

#### Command Execution (utils/commands.ts)

- Cross-platform command execution
- Timeout handling
- Network connectivity checks

#### State Management (utils/state.ts)

- Test state tracking
- Reward distribution simulation
- Balance validation
- Transaction history management

### Test Data and Configuration (fixtures/types.ts)

- User account definitions
- Token configurations
- Expected event types
- System states and transitions
- Expected error messages

## Running E2E Tests

### Prerequisites

1. **Docker** (20.10+) - Required for Testcontainers
2. **Node.js 18+** - Runtime environment
3. **npm** - Package manager
4. **Rust toolchain** - For contract compilation
5. **Soroban CLI** - For contract interaction

### Installation

```bash
cd /workspaces/axionvera-network
npm install
rustup target add wasm32-unknown-unknown
```

### Building Contracts

```bash
npm run build:contracts
```

### Running Tests

#### Run All E2E Tests

```bash
npm run test:integration
```

#### Run Specific Test Suite

```bash
npx vitest run --config vitest.integration.config.ts tests/e2e/workflows/core.ts
npx vitest run --config vitest.integration.config.ts tests/e2e/workflows/rewards.ts
npx vitest run --config vitest.integration.config.ts tests/e2e/workflows/governance.ts
```

#### Run Tests with Cleanup

```bash
npm run test:integration:clean
```

### Development Setup

For local development with simulated environments:

```bash
# Create test environment variables
echo "TEST_NODE_HOST=localhost" >> .env
echo "TEST_NODE_PORT=50051" >> .env

# Run tests in watch mode
npx vitest --watch
```

## Test Configuration

### Test Environment Variables

| Variable            | Default                                                | Description              |
| ------------------- | ------------------------------------------------------ | ------------------------ |
| `TEST_NODE_HOST`    | `localhost`                                            | Network node hostname    |
| `TEST_NODE_PORT`    | `50051`                                                | Network node port        |
| `TEST_DATABASE_URL` | `postgresql://testuser:testpass@localhost:5432/testdb` | Test database connection |

### Test Timeouts

| Setting          | Value      | Description                            |
| ---------------- | ---------- | -------------------------------------- |
| `TEST_TIMEOUT`   | `300000ms` | Maximum timeout per test               |
| `GLOBAL_TIMEOUT` | `900000ms` | Maximum timeout for beforeAll/afterAll |

### Test Coverage Requirements

The framework enforces minimum coverage levels:

| Component           | Minimum Coverage |
| ------------------- | ---------------- |
| Core Protocol       | 90%              |
| Reward Distribution | 85%              |
| Staking Operations  | 80%              |
| Treasury Actions    | 85%              |
| Governance          | 90%              |

## Workflow Documentation

### Core Protocol Workflow

The core protocol workflow tests verify the complete lifecycle of user interactions with the Axionvera Network:

1. **User Registration and Initialization**
2. **Deposit Operations** - Users deposit tokens into the vault
3. **Reward Distribution** - Admin distributes rewards proportionally
4. **Reward Claiming** - Users claim earned rewards
5. **Withdrawal Operations** - Users withdraw deposited funds
6. **Multi-User Scenarios** - Concurrent operations across multiple users

### Reward Distribution Workflow

This workflow covers treasury management and reward distribution:

1. **Treasury Fund Management** - Deposits and withdrawals from treasury
2. **Batch Reward Distribution** - Distribution to multiple users simultaneously
3. **Vesting Schedules** - Reward vesting over time periods
4. **Staking Rewards** - Rewards for staked positions
5. **Protocol Governance** - Budget and parameter approval workflows

### Governance Workflow

This workflow tests protocol governance mechanisms:

1. **Parameter Governance** - Protocol parameter upgrades
2. **Admin Transfer** - Administrative control transfers
3. **Treasury Governance** - Budget and spending approvals
4. **Emergency Actions** - Fast-track emergency governance
5. **Delegation** - Governance power delegation

## Test Scenarios

### Complete User Lifecycle

```typescript
// Example: Complete user lifecycle
const depositAmount = "1000";
const rewardDistribution = "500000";

// 1. Multiple users deposit
await user1.deposit(depositAmount);
await user2.deposit("1500");

// 2. Admin distributes rewards
await admin.distributeRewards(rewardDistribution);

// 3. Users claim rewards
await user1.claimRewards();
await user2.claimRewards();

// 4. Users withdraw funds
await user1.withdraw(depositAmount);
await user2.withdraw("1500");
```

### Multi-User Concurrent Operations

```typescript
// Test parallel deposit operations
const depositPromises = users.map(async (user, index) => {
  return await user.deposit("2000", signature, nonce);
});

const results = await Promise.all(depositPromises);
results.forEach((result) => expect(result.success).toBe(true));
```

### Reward Distribution Proportionality

```typescript
// Test reward distribution is proportional to deposits
const aliceDeposit = "1000";
const bobDeposit = "2000";
const totalDeposits = aliceDeposit + bobDeposit;

const distributionAmount = "1000000";
const aliceExpected = Math.floor(
  (distributionAmount * aliceDeposit) / totalDeposits,
);
const bobExpected = Math.floor(
  (distributionAmount * bobDeposit) / totalDeposits,
);

await admin.distributeRewards(distributionAmount);

expect(aliceRewards).toBeCloseTo(aliceExpected, -2);
expect(bobRewards).toBeCloseTo(bobExpected, -2);
```

## Error Handling Tests

The framework includes comprehensive error scenario testing:

### Invalid Operations

| Scenario                   | Expected Error                        | Test Coverage |
| -------------------------- | ------------------------------------- | ------------- |
| Zero amount deposit        | `ValidationError.InvalidAmount`       | ✅            |
| Negative amount withdrawal | `ValidationError.NegativeAmount`      | ✅            |
| Insufficient balance       | `BalanceError.InsufficientBalance`    | ✅            |
| Invalid signature          | `AuthorizationError.InvalidSignature` | ✅            |
| Duplicate nonce            | `ValidationError.DuplicateNonce`      | ✅            |
| Unauthorized operation     | `AuthorizationError.Unauthorized`     | ✅            |

### Edge Cases

- Contract is paused
- Empty user addresses
- Missing fields in requests
- Zero reward distributions
- Large amount overflows

## CI/CD Integration

### GitHub Actions Workflow

The framework includes a GitHub Actions workflow for automated testing:

#### `.github/workflows/e2e-tests.yml`

**Features:**

- Multi-stage test execution
- Parallel test suites
- Artifact management
- Docker cleanup
- Test coverage reporting

#### Workflow Stages

1. **Setup Environment** - Install dependencies and build contracts
2. **E2E Tests** - Run parallel test suites (Core, Rewards, Governance)
3. **Comprehensive E2E** - Full test suite execution
4. **Cleanup** - Remove Docker resources and artifacts

### Test Reporting

The framework generates detailed test reports:

#### Output Files

- **`test-results.json`** - Test execution results
- **`test-coverage.json`** - Coverage metrics
- **Screenshots** - Visual evidence of failures
- **Logs** - System and application logs

#### Report Contents

- Test execution time
- Pass/fail ratios
- Component-specific coverage
- Error details and stack traces
- Performance metrics

## Framework Benefits

### 1. **Comprehensive Coverage**

- Covers all core protocol workflows
- Tests reward distribution mechanics
- Validates staking operations
- Verifies treasury actions
- Ensures governance integrity

### 2. **Automated Execution**

- CI/CD integration
- Parallel test execution
- Automatic retry mechanisms
- Clean environment setup

### 3. **Clear Failure Reporting**

- Detailed error messages
- Visual evidence of failures
- Stack trace analysis
- Performance metrics

### 4. **Complete Documentation**

- Workflow descriptions
- Code examples
- Configuration guides
- Best practices

## Maintenance and Updates

### Adding New Workflows

1. Add new test file in `tests/e2e/workflows/`
2. Follow existing patterns and conventions
3. Update `TEST_FRAMEWORK_CONFIG.md` if needed
4. Add to CI configuration if required

### Updating Test Data

Modify `tests/e2e/fixtures/types.ts`:

- Add new test users
- Update token configurations
- Modify expected values
- Add edge cases

### Fixing Test Issues

1. Check environment setup
2. Verify contract state
3. Review test configurations
4. Update test data if needed
5. Run tests with verbose logging

## Troubleshooting

### Common Issues

#### Docker Not Available

```bash
docker ps
# Start Docker Desktop (macOS/Windows)
# Or start Docker daemon (Linux)
sudo systemctl start docker
```

#### Port Conflicts

```bash
# Check what's using port 50051
lsof -i :50051

# Kill conflicting processes
kill -9 <PID>
```

#### Test Timeouts

```bash
# Increase timeout in vitest integration config
# vitest.integration.config.ts
timeout: 180000, // 3 minutes
```

#### Insufficient Permissions

```bash
# Add user to Docker group (Linux)
sudo usermod -aG docker $USER
# Then logout and login again
```

### Getting Help

If you encounter issues:

1. **Check Environment Logs**
   - Docker logs: `docker logs <container-name>`
   - Application logs: Check test artifacts

2. **Review Test Output**
   - Examine detailed test results
   - Check for error patterns

3. **Verify Configuration**
   - Confirm environment variables
   - Check test configuration files

4. **Run Minimal Tests**
   - Run single test files
   - Check for specific error patterns

## Support

For issues or questions:

1. **Check Documentation** - Review this guide
2. **Test Logs** - Examine test artifacts and logs
3. **Configuration** - Verify environment setup
4. **Framework Issues** - Report in GitHub repository

## Framework Version

**Version:** 1.0.0
**Updated:** $(date)
**Status:** Production Ready
