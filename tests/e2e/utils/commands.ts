// Command execution utilities for E2E tests
// Cross-platform command execution with timeout support

import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

export interface CommandResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export class CommandExecutor {
  static async execute(
    command: string,
    timeout: number = 30000,
  ): Promise<CommandResult> {
    try {
      const result = await Promise.race([
        execAsync(command),
        new Promise<never>((_, reject) =>
          setTimeout(
            () => reject(new Error(`Command timed out after ${timeout}ms`)),
            timeout,
          ),
        ),
      ]);
      return {
        stdout: result.stdout,
        stderr: result.stderr,
        exitCode: 0,
      };
    } catch (error) {
      if (error instanceof Error && error.message.includes("timed out")) {
        throw new Error(`Command timed out after ${timeout}ms: ${command}`);
      }
      return {
        stdout: "",
        stderr: error instanceof Error ? error.message : String(error),
        exitCode: 1,
      };
    }
  }

  static async checkNetworkConnectivity(
    host: string,
    port: string,
  ): Promise<boolean> {
    const command = `nc -z ${host} ${port}`;
    try {
      const result = await this.execute(command, 5000);
      return result.exitCode === 0;
    } catch {
      return false;
    }
  }

  static async checkContractStatus(contractAddress: string): Promise<boolean> {
    const command = `soroban contract info --id ${contractAddress}`;
    try {
      const result = await this.execute(command, 10000);
      return result.exitCode === 0 && result.stdout.includes("isInitialized");
    } catch {
      return false;
    }
  }
}
