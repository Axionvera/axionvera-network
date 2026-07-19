# Protocol Execution Pipeline Framework

The Protocol Execution Pipeline Framework provides a reusable and deterministic way to break down complex protocol workflows into sequential processing stages.

## Overview

Complex protocol operations often involve multiple steps that must be executed in a specific order. Each step may require validation, execution logic, and post-execution hooks. This framework standardizes these operations to improve readability, extensibility, and error handling.

## Architecture

A Pipeline consists of multiple `PipelineStage`s executed in sequence.

### Pipeline Definition

- `id`: A unique symbol identifying the pipeline.
- `stages`: A list of stages to be executed.

### Pipeline Stage

Each stage contains:
1. **Validation Hooks**: A list of `PipelineAction`s executed before the main logic. If any validation fails, the pipeline halts.
2. **Execution**: The main `PipelineAction` for the stage.
3. **Post-processing Hooks**: A list of `PipelineAction`s executed after the main logic.

### Pipeline Action

A `PipelineAction` represents a cross-contract call:
- `target`: The address of the contract to call.
- `function`: The symbol of the function to invoke.
- `args`: The arguments for the call.

## Lifecycle Events

The framework emits events at each stage of the pipeline:

- `pipe_str`: Pipeline started.
- `stg_str`: Stage started.
- `stg_com`: Stage completed.
- `pipe_com`: Pipeline completed successfully.
- `pipe_fai`: Pipeline failed (with details on the failed stage and error code).

## Failure Handling

If any action within a stage fails:
1. The pipeline halts execution immediately.
2. A `PipelineReceipt` is returned with `status: Failed`.
3. The `failed_stage` and `error_code` are recorded in the receipt.
4. A `pipe_fai` event is emitted.

*Note: In the current implementation, any state changes made by successfully completed stages (or actions within the failed stage before the failure) are persisted to the ledger. Users should ensure their actions are designed with this in mind or implement compensating actions if full atomicity is required.*
