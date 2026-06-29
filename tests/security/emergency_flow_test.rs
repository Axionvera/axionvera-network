// Write an integration test that deploys Admin, Core, and Security contracts.
// 1. Assert Core critical function succeeds.
// 2. Call Admin -> trigger_emergency_pause().
// 3. Assert Core critical function panics/reverts.
// 4. Call Admin -> trigger_recovery().
// 5. Assert Core critical function succeeds again.