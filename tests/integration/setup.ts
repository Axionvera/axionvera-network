import { PostgreSqlContainer } from '@testcontainers/postgresql';
import { GenericContainer, Wait } from 'testcontainers';
import { execSync } from 'child_process';

let postgresContainer: any;
let networkNodeContainer: any;

export async function setup() {
  console.log('🚀 Starting integration test environment...');
  
  try {
    // Start PostgreSQL container
    console.log('📦 Starting PostgreSQL container...');
    postgresContainer = await new PostgreSqlContainer('postgres:15-alpine')
      .withDatabase('testdb')
      .withUsername('testuser')
      .withPassword('testpass')
      .withExposedPorts(5432)
      .start();
    
    const dbHost = postgresContainer.getHost();
    const dbPort = postgresContainer.getMappedPort(5432);
    const dbUrl = `postgresql://testuser:testpass@${dbHost}:${dbPort}/testdb`;
    
    console.log(`✅ PostgreSQL started on ${dbHost}:${dbPort}`);
    
    // Set environment variables for tests
    process.env.TEST_DATABASE_URL = dbUrl;
    process.env.TEST_POSTGRES_HOST = dbHost;
    process.env.TEST_POSTGRES_PORT = dbPort.toString();
    
    // Optionally start network-node container if needed
    // For now, we'll test against the locally built binary
    
    console.log('✅ Integration test environment ready');
  } catch (error) {
    console.error('❌ Failed to setup integration test environment:', error);
    throw error;
  }
}

export async function teardown() {
  console.log('🧹 Cleaning up integration test environment...');
  
  try {
    // Stop PostgreSQL container
    if (postgresContainer) {
      console.log('⏹️  Stopping PostgreSQL container...');
      await postgresContainer.stop({ timeout: 10000 });
      console.log('✅ PostgreSQL container stopped');
    }
    
    // Stop network-node container if running
    if (networkNodeContainer) {
      console.log('⏹️  Stopping network-node container...');
      await networkNodeContainer.stop({ timeout: 10000 });
      console.log('✅ Network-node container stopped');
    }
    
    // Clean up any dangling containers and volumes
    console.log('🧹 Cleaning up Docker resources...');
    try {
      execSync('docker container prune -f', { stdio: 'ignore' });
      execSync('docker volume prune -f', { stdio: 'ignore' });
      console.log('✅ Docker cleanup complete');
    } catch (cleanupError) {
      console.warn('⚠️  Docker cleanup failed (non-critical):', cleanupError);
    }
    
    console.log('✅ Integration test environment cleaned up');
  } catch (error) {
    console.error('❌ Error during teardown:', error);
    // Don't throw in teardown to avoid masking test failures
  }
}

export default {
  setup,
  teardown,
};
