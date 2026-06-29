// E2E Testing Framework - Error Handling and Edge Case Utilities
// Comprehensive error handling utilities for E2E testing

import { TestUtils } from "./core";
import { E2E_TEST_CONFIG, EXPECTED_ERRORS } from "./fixtures/types";

export interface ErrorScenario {
  name: string;
  description: string;
  validationFn: (input: any) => boolean;
  expectedErrorCode: string;
  expectedErrorMessage?: string;
  severity: "low" | "medium" | "high" | "critical";
  recoverable: boolean;
}

export interface EdgeCaseConfig {
  boundaryValues: {
    positiveMin: string;
    positiveMax: string;
    negativeMin: string;
    zero: string;
  };
  invalidInputs: any[];
  specialValues: any[];
  timingRelated: any[];
}

export class ErrorHandlingUtils {
  static readonly ERROR_SCENARIOS: ErrorScenario[] = [
    {
      name: "ZERO_AMOUNT_DEPOSIT",
      description: "Test deposit with zero amount",
      validationFn: (input: any) => input.amount === "0",
      expectedErrorCode: "VALIDATION_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.INVALID_AMOUNT,
      severity: "medium",
      recoverable: true,
    },
    {
      name: "NEGATIVE_AMOUNT_WITHDRAWAL",
      description: "Test withdrawal with negative amount",
      validationFn: (input: any) =>
        input.amount && input.amount.startsWith("-"),
      expectedErrorCode: "VALIDATION_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.INVALID_AMOUNT,
      severity: "medium",
      recoverable: true,
    },
    {
      name: "INVALID_SIGNATURE",
      description: "Test transaction with invalid signature",
      validationFn: (input: any) =>
        input.signature && input.signature.includes("invalid"),
      expectedErrorCode: "AUTHORIZATION_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.REEJECT_SIGNATURE,
      severity: "high",
      recoverable: false,
    },
    {
      name: "DUPLICATE_NONCE",
      description: "Test transaction with duplicate nonce",
      validationFn: (input: any) => {
        // This would need state tracking to detect duplicates
        return false; // Placeholder
      },
      expectedErrorCode: "VALIDATION_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.DUPLICATE_NONCE,
      severity: "high",
      recoverable: true,
    },
    {
      name: "EMPTY_USER_ADDRESS",
      description: "Test transaction with empty user address",
      validationFn: (input: any) =>
        !input.user_address || input.user_address === "",
      expectedErrorCode: "VALIDATION_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.INVALID_ADDRESS,
      severity: "medium",
      recoverable: true,
    },
    {
      name: "INVALID_TOKEN_ADDRESS",
      description: "Test transaction with invalid token address",
      validationFn: (input: any) =>
        !input.token_address || !input.token_address.startsWith("C"),
      expectedErrorCode: "VALIDATION_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.INVALID_ADDRESS,
      severity: "medium",
      recoverable: true,
    },
    {
      name: "UNAUTHORIZED_OPERATION",
      description: "Test admin operation without proper authorization",
      validationFn: (input: any) =>
        input.operation === "admin" && !input.authorized,
      expectedErrorCode: "AUTHORIZATION_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.UNAUTHORIZED,
      severity: "critical",
      recoverable: false,
    },
    {
      name: "INSUFFICIENT_BALANCE",
      description: "Test withdrawal exceeding available balance",
      validationFn: (input: any) => {
        return input.amount > input.available_balance;
      },
      expectedErrorCode: "BALANCE_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.INSUFFICIENT_BALANCE,
      severity: "high",
      recoverable: true,
    },
    {
      name: "CONTRACT_PAUSED_OPERATION",
      description: "Test operation when contract is paused",
      validationFn: (input: any) => input.contract_paused,
      expectedErrorCode: "SYSTEM_ERROR",
      expectedErrorMessage: EXPECTED_ERRORS.CONTRACT_PAUSED,
      severity: "high",
      recoverable: false,
    },
    {
      name: "UNKNOWN_RPC_METHOD",
      description: "Test unknown RPC method call",
      validationFn: (input: any) =>
        !input.method || !input.supported_methods.includes(input.method),
      expectedErrorCode: "NETWORK_ERROR",
      expectedErrorMessage: "Unknown RPC method",
      severity: "medium",
      recoverable: true,
    },
  ];

  static readonly EDGE_CASE_CONFIG: EdgeCaseConfig = {
    boundaryValues: {
      positiveMin: "1",
      positiveMax: "999999999999999999",
      negativeMin: "-999999999999999999",
      zero: "0",
    },
    invalidInputs: [
      "",
      "NaN",
      "Infinity",
      "-Infinity",
      "1.5", // Non-integer for integer fields
      "0x123", // Non-decimal
      "...invalid", // Malformed
      "  123  ", // With spaces
    ],
    specialValues: [
      "PRECISION_FACTOR",
      "MAX_UINT128",
      "MAX_UINT64",
      "MIN_INT128",
    ],
    timingRelated: [
      "future_timestamp",
      "past_timestamp",
      "unbounded_duration",
      "negative_duration",
    ],
  };

  static generateErrorTestCases(): ErrorScenario[] {
    return this.ERROR_SCENARIOS;
  }

  static getEdgeCaseValues(
    type: "positive" | "negative" | "boundary",
  ): string[] {
    const config = this.EDGE_CASE_CONFIG;
    switch (type) {
      case "positive":
        return [
          config.boundaryValues.positiveMin,
          "1",
          "1000",
          config.boundaryValues.positiveMax,
        ];
      case "negative":
        return ["-1", "-1000", config.boundaryValues.negativeMin];
      case "boundary":
        return [
          config.boundaryValues.zero,
          config.boundaryValues.positiveMin,
          config.boundaryValues.positiveMax,
        ];
    }
  }

  static isValidAddress(address: string): boolean {
    if (!address || address.length < 10 || address.length > 60) {
      return false;
    }
    return (
      address.startsWith("G") ||
      address.startsWith("C") ||
      address.startsWith("E")
    );
  }

  static isValidAmount(amount: string): boolean {
    if (!amount) return false;
    if (amount.includes(".")) return false; // Only integers allowed
    if (amount.startsWith("-0")) return false; // Invalid negative zero
    return (
      !isNaN(parseInt(amount)) &&
      parseInt(amount) >= Number.MIN_SAFE_INTEGER &&
      parseInt(amount) <= Number.MAX_SAFE_INTEGER
    );
  }

  static async simulateNetworkError<R>(
    operation: () => Promise<R>,
    shouldFail: boolean = false,
  ): Promise<R> {
    if (shouldFail) {
      throw new Error("Network timeout");
    }
    return await operation();
  }

  static async simulateResourceExhaustion<R>(
    operation: () => Promise<R>,
    shouldExhaust: boolean = false,
  ): Promise<R> {
    if (shouldExhaust) {
      throw new Error("Resource limit exceeded");
    }
    return await operation();
  }

  static generateMalformedSignature(): Buffer {
    return Buffer.from("malformed_signature_with_invalid_length_and_format");
  }

  static generateValidSignature(): Buffer {
    return Buffer.from(`valid_signature_${Date.now()}`);
  }
}

export class EdgeCaseTestUtils {
  static generateUserWithBoundaryData() {
    return [
      { address: "G0000000001111111111111111111111111111111", amount: "1" },
      {
        address: "G9999999999999999999999999999999999999999999",
        amount: "999999999999999999",
      },
      {
        address: "G1234567890123456789012345678901234567890",
        amount: "500000000000",
      },
      { address: "G0000000000000000000000000000000000000000", amount: "0" },
    ].map((user) => ({
      ...user,
      validSignature: TestUtils.generateSignature(user.address, 1),
      invalidSignature: Buffer.from("invalid"),
    }));
  }

  static generateTokenWithEdgeValues() {
    return [
      { address: "C0000000001111111111111111111111111111111", type: "deposit" },
      {
        address: "C9999999999999999999999999999999999999999999",
        type: "deposit",
      },
      {
        address: "CREWARD0000000000000000000000000000000000000",
        type: "reward",
      },
      { address: "", type: "deposit" }, // Invalid
    ].map((token) => ({
      ...token,
      validSignature: TestUtils.generateSignature(token.address, 2),
      invalidSignature: Buffer.from("invalid_token"),
    }));
  }

  static generateTimingEdgeCases() {
    const now = Math.floor(Date.now() / 1000);
    return [
      { timestamp: now - 86400 * 365, label: "1 year ago" }, // Ancient
      { timestamp: now - 86400 * 30, label: "1 month ago" }, // Recent past
      { timestamp: now, label: "now" }, // Current
      { timestamp: now + 86400, label: "1 day future" }, // Future
      { timestamp: now + 86400 * 365, label: "1 year future" }, // Distant future
      { timestamp: now + Number.MAX_SAFE_INTEGER / 1000, label: "max future" }, // Overflow
    ];
  }

  static createErrorTestTemplate(
    scenario: ErrorScenario,
    input: any,
    client: any,
  ) {
    return async () => {
      console.log(`🧪 Testing error scenario: ${scenario.name}`);
      console.log(`   Input: ${JSON.stringify(input, null, 2)}`);

      if (!scenario.validationFn(input)) {
        throw new Error(`Validation failed for scenario ${scenario.name}`);
      }

      // Simulate the error
      const result = await this.simulateError(
        input,
        scenario.expectedErrorCode,
      );

      // Verify error response
      this.verifyErrorResponse(result, scenario);

      console.log(`   ✅ Error test passed: ${scenario.name}`);
    };
  }

  private static async simulateError(
    input: any,
    expectedErrorCode: string,
  ): Promise<any> {
    // Simulate error based on scenario type
    switch (expectedErrorCode) {
      case "VALIDATION_ERROR":
        throw new Error(`Validation error: ${input.reason}`);
      case "AUTHORIZATION_ERROR":
        throw new Error(
          `Authorization error: Invalid signature or permissions`,
        );
      case "BALANCE_ERROR":
        throw new Error(`Balance error: Insufficient balance for operation`);
      case "NETWORK_ERROR":
        throw new Error(`Network error: Unknown RPC method`);
      case "SYSTEM_ERROR":
        throw new Error(`System error: Contract is paused`);
      default:
        throw new Error(`Unknown error: ${expectedErrorCode}`);
    }
  }

  private static verifyErrorResponse(
    result: any,
    scenario: ErrorScenario,
  ): void {
    if (result.success === true) {
      throw new Error(
        `Expected error but got success response for ${scenario.name}`,
      );
    }
    if (result.error_code !== scenario.expectedErrorCode) {
      throw new Error(
        `Expected error code ${scenario.expectedErrorCode} but got ${result.error_code}`,
      );
    }
    if (
      scenario.expectedErrorMessage &&
      !result.error_message.includes(scenario.expectedErrorMessage)
    ) {
      throw new Error(
        `Expected error message containing '${scenario.expectedErrorMessage}' but got '${result.error_message}'`,
      );
    }
  }
}

export class ErrorAnalysis {
  static analyzeErrorPattern(errors: any[]): ErrorPatternAnalysis {
    const analysis: ErrorPatternAnalysis = {
      totalErrors: errors.length,
      errorsByCategory: {},
      errorsBySeverity: {},
      topErrorCodes: [],
      trends: [],
      recommendations: [],
    };

    errors.forEach((error) => {
      const category = this.categorizeError(error);
      const severity = this.determineSeverity(error);

      analysis.errorsByCategory[category] =
        (analysis.errorsByCategory[category] || 0) + 1;
      analysis.errorsBySeverity[severity] =
        (analysis.errorsBySeverity[severity] || 0) + 1;
    });

    analysis.topErrorCodes = Object.entries(analysis.errorsByCategory)
      .sort(([, a], [, b]) => b - a)
      .slice(0, 5)
      .map(([code, count]) => ({ code, count }));

    return analysis;
  }

  private static categorizeError(error: any): string {
    const message = error.message.toLowerCase();
    if (message.includes("signature")) return "AUTHORIZATION";
    if (message.includes("balance")) return "BALANCE";
    if (message.includes("amount")) return "VALIDATION";
    if (message.includes("pause")) return "SYSTEM";
    if (message.includes("network")) return "NETWORK";
    return "UNKNOWN";
  }

  private static determineSeverity(error: any): string {
    const message = error.message.toLowerCase();
    if (message.includes("critical") || message.includes("unauthorized"))
      return "critical";
    if (message.includes("error") || message.includes("failed")) return "high";
    if (message.includes("warning") || message.includes("insufficient"))
      return "medium";
    return "low";
  }
}

export interface ErrorPatternAnalysis {
  totalErrors: number;
  errorsByCategory: Record<string, number>;
  errorsBySeverity: Record<string, number>;
  topErrorCodes: Array<{ code: string; count: number }>;
  trends: any[];
  recommendations: string[];
}
