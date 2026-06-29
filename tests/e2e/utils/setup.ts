// Environment setup utilities for E2E tests

import { CommandExecutor } from "./commands";
import { TestStateManager } from "./state";
import { E2E_TEST_CONFIG } from "./fixtures/types";

export class EnvironmentSetup {
  private stateManager: TestStateManager;

  constructor() {
    this.stateManager = new TestStateManager();
  }

  async setupTestEnvironment(): Promise<void> {
    console.log("🔧 Setting up E2E test environment...");

    // Check network connectivity
    const isNetworkAvailable = await CommandExecutor.checkNetworkConnectivity(
      E2E_TEST_CONFIG.host,
      E2E_TEST_CONFIG.port,
    );

    if (!isNetworkAvailable) {
      throw new Error(
        `Network not available at ${E2E_TEST_CONFIG.host}:${E2E_TEST_CONFIG.port}`,
      );
    }

    // Initialize test state
    await this.stateManager.initializeState();

    // Check if contract exists and is initialized
    const isContractInitialized = await CommandExecutor.checkContractStatus(
      E2E_TEST_CONFIG.contractAddress,
    );

    if (!isContractInitialized) {
      console.log(
        "⚠️ Contract not available, tests may be skipped or require setup",
      );
    }

    console.log("✅ Test environment setup complete");
  }

  async cleanupTestEnvironment(): Promise<void> {
    console.log("🧹 Cleaning up test environment...");
    this.stateManager.reset();
    console.log("✅ Test environment cleanup complete");
  }

  getStateManager(): TestStateManager {
    return this.stateManager;
  }
}
