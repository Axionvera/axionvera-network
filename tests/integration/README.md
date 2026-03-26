# Integration Test Suite

This directory contains integration tests that validate end-to-end data flow with real, ephemeral dependencies.

## Prerequisites

- Docker (20.10+) - Required for Testcontainers
- Node.js 18+
- npm

## Setup

The integration tests use [Testcontainers](https://www.testcontainers.org/) to spin up real PostgreSQL databases and other dependencies in Docker containers. This ensures tests run in an isolated, production-like environment.

### Install Dependencies

```bash
npm install
```

This will install:
- `testcontainers` - Core library for container management
- `@testcontainers/postgresql` - PostgreSQL container provider
- `node-fetch` - HTTP client for testing endpoints

## Running Tests

### Run All Integration Tests

```bash
npm run test:integration
```

### Run Tests with Cleanup

This command cleans up any dangling containers before running tests:

```bash
npm run test:integration:clean
```

### Run Specific Test File

```bash
npx vitest run --config vitest.integration.config.ts tests/integration/network-node.test.ts
```

## Test Structure

### Global Setup (`setup.ts`)

The global setup file handles:
1. Starting PostgreSQL container with random ports
2. Setting environment variables for tests
3. Cleaning up containers after all tests complete

### Network Node Tests (`network-node.test.ts`)

Tests for the network node service including:
- Health check endpoints (`/health/liveness`, `/health/readiness`)
- Metrics endpoint (`/metrics`)
- Data ingestion and retrieval routes

## Test Environment

### Containers Started

1. **PostgreSQL 15 (Alpine)**
   - Database: `testdb`
   - Username: `testuser`
   - Password: `testpass`
   - Port: Randomly mapped (not exposed on host)

2. **Network Node** (optional, can be run locally)
   - Binds to: `localhost:8080`
   - Connects to test PostgreSQL instance

### Environment Variables

The setup automatically sets:
- `TEST_DATABASE_URL` - Connection string for test database
- `TEST_POSTGRES_HOST` - Host of test database
- `TEST_POSTGRES_PORT` - Port of test database

## Cleanup Process

The teardown process ensures no pollution of your local environment:

1. **Stop all containers** - Gracefully stops PostgreSQL and network-node containers
2. **Remove containers** - Deletes stopped containers
3. **Prune volumes** - Removes anonymous volumes
4. **Clean system** - Runs `docker system prune` to remove unused resources

### Manual Cleanup

If tests fail and leave containers running:

```bash
# Stop all test containers
docker ps --filter "name=testcontainer" -q | xargs docker stop

# Remove all test containers
docker ps -a --filter "name=testcontainer" -q | xargs docker rm

# Clean up volumes
docker volume prune -f

# Full system cleanup
docker system prune -af
```

## Writing New Integration Tests

### Example Test

```typescript
import { describe, it, expect } from 'vitest';
import fetch from 'node-fetch';

describe('My Integration Test', () => {
  it('should connect to database and return data', async () => {
    const dbUrl = process.env.TEST_DATABASE_URL;
    expect(dbUrl).toBeDefined();
    
    // Your test logic here
    const response = await fetch(`http://localhost:8080/health`);
    expect(response.status).toBe(200);
  });
});
```

### Best Practices

1. **Use environment variables** - Always use `TEST_DATABASE_URL` instead of hardcoded values
2. **Clean up resources** - Close database connections in `afterEach` hooks
3. **Isolate tests** - Each test should be independent and runnable in any order
4. **Handle failures gracefully** - Use try-catch for external service calls
5. **Avoid hardcoding ports** - Let Testcontainers assign random ports

## Troubleshooting

### Docker Not Running

```bash
# Check if Docker is running
docker ps

# Start Docker Desktop (macOS/Windows)
# Or start Docker daemon (Linux)
sudo systemctl start docker
```

### Container Startup Failures

Check Docker logs:
```bash
docker logs <container-id>
```

Increase timeout in `vitest.integration.config.ts`:
```typescript
timeout: 180000, // 3 minutes
```

### Port Conflicts

Testcontainers uses random ports, but if you encounter conflicts:
```bash
# Check what's using port 8080
lsof -i :8080

# Kill the process
kill -9 <PID>
```

### Permission Issues (Linux)

Add user to Docker group:
```bash
sudo usermod -aG docker $USER
# Then logout and login again
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-test:
    runs-on: ubuntu-latest
    
    services:
      docker:
        image: docker:dind
        options: --privileged
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run integration tests
        run: npm run test:integration
```

## Performance Tips

1. **Run tests sequentially** - Already configured in vitest config
2. **Reuse containers** - Use global setup to start containers once
3. **Parallel test suites** - Use different container prefixes for parallel suites
4. **Cache Docker images** - Pre-pull images in CI

## Security Considerations

- Test containers run with default security settings
- No privileged mode required
- Containers are isolated from host network
- Credentials are randomly generated per test run

## Additional Resources

- [Testcontainers Documentation](https://www.testcontainers.org/)
- [Vitest Configuration](https://vitest.dev/config/)
- [PostgreSQL Container Image](https://hub.docker.com/_/postgres)
