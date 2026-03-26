import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    name: 'integration',
    include: ['tests/integration/**/*.test.ts'],
    timeout: 120000, // 2 minutes for container startup
    poolOptions: {
      threads: {
        singleThread: true, // Run tests sequentially to avoid container conflicts
      },
    },
    globalSetup: ['./tests/integration/setup.ts'],
  },
});
