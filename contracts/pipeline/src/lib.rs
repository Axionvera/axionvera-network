#![no_std]

use soroban_sdk::{contract, contractimpl, Env, InvokeError, Symbol};

use axionvera_events::{
    self, ACT_PIPE_COMPL, ACT_PIPE_FAIL, ACT_PIPE_START, ACT_STAGE_COMPL, ACT_STAGE_START,
    EVENT_VERSION, PROTOCOL,
};
use axionvera_interfaces::{
    PipelineAction, PipelineDefinition, PipelineError, PipelineReceipt, PipelineRunner,
    PipelineStatus,
};

#[contract]
pub struct PipelineContract;

#[contractimpl]
impl PipelineRunner for PipelineContract {
    fn execute_pipeline(
        e: Env,
        pipeline: PipelineDefinition,
    ) -> Result<PipelineReceipt, PipelineError> {
        if pipeline.stages.is_empty() {
            return Err(PipelineError::EmptyPipeline);
        }

        emit_pipeline_started(&e, pipeline.id.clone());

        for stage in pipeline.stages.iter() {
            emit_stage_started(&e, pipeline.id.clone(), stage.id.clone());

            // 1. Validation Hooks
            for validation in stage.validation.iter() {
                if let Err(_) = invoke_action(&e, &validation) {
                    let receipt = PipelineReceipt {
                        pipeline_id: pipeline.id.clone(),
                        status: PipelineStatus::Failed,
                        failed_stage: Some(stage.id.clone()),
                        error_code: Some(PipelineError::StageValidationFailed as u32),
                    };
                    emit_pipeline_failed(&e, &receipt);
                    return Ok(receipt);
                }
            }

            // 2. Execution
            if let Err(_) = invoke_action(&e, &stage.execution) {
                let receipt = PipelineReceipt {
                    pipeline_id: pipeline.id.clone(),
                    status: PipelineStatus::Failed,
                    failed_stage: Some(stage.id.clone()),
                    error_code: Some(PipelineError::StageExecutionFailed as u32),
                };
                emit_pipeline_failed(&e, &receipt);
                return Ok(receipt);
            }

            // 3. Post-processing Hooks
            for post_hook in stage.post_hook.iter() {
                if let Err(_) = invoke_action(&e, &post_hook) {
                    let receipt = PipelineReceipt {
                        pipeline_id: pipeline.id.clone(),
                        status: PipelineStatus::Failed,
                        failed_stage: Some(stage.id.clone()),
                        error_code: Some(PipelineError::PostHookFailed as u32),
                    };
                    emit_pipeline_failed(&e, &receipt);
                    return Ok(receipt);
                }
            }

            emit_stage_completed(&e, pipeline.id.clone(), stage.id.clone());
        }

        let receipt = PipelineReceipt {
            pipeline_id: pipeline.id.clone(),
            status: PipelineStatus::Completed,
            failed_stage: None,
            error_code: None,
        };
        emit_pipeline_completed(&e, pipeline.id.clone());
        Ok(receipt)
    }
}

fn invoke_action(e: &Env, action: &PipelineAction) -> Result<(), ()> {
    match e.try_invoke_contract::<(), InvokeError>(
        &action.target,
        &action.function,
        action.args.clone(),
    ) {
        Ok(Ok(())) => Ok(()),
        _ => Err(()),
    }
}

fn emit_pipeline_started(e: &Env, pipeline_id: Symbol) {
    e.events().publish(
        (PROTOCOL, ACT_PIPE_START),
        axionvera_events::PipelineStartedEvent {
            event_version: EVENT_VERSION,
            pipeline_id,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_pipeline_completed(e: &Env, pipeline_id: Symbol) {
    e.events().publish(
        (PROTOCOL, ACT_PIPE_COMPL),
        axionvera_events::PipelineCompletedEvent {
            event_version: EVENT_VERSION,
            pipeline_id,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_pipeline_failed(e: &Env, receipt: &PipelineReceipt) {
    let failed_stage = receipt
        .failed_stage
        .clone()
        .unwrap_or(Symbol::new(e, "unknown"));
    let error_code = receipt.error_code.unwrap_or(0);
    e.events().publish(
        (PROTOCOL, ACT_PIPE_FAIL),
        axionvera_events::PipelineFailedEvent {
            event_version: EVENT_VERSION,
            pipeline_id: receipt.pipeline_id.clone(),
            failed_stage,
            error_code,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_stage_started(e: &Env, pipeline_id: Symbol, stage_id: Symbol) {
    e.events().publish(
        (PROTOCOL, ACT_STAGE_START),
        axionvera_events::StageStartedEvent {
            event_version: EVENT_VERSION,
            pipeline_id,
            stage_id,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

fn emit_stage_completed(e: &Env, pipeline_id: Symbol, stage_id: Symbol) {
    e.events().publish(
        (PROTOCOL, ACT_STAGE_COMPL),
        axionvera_events::StageCompletedEvent {
            event_version: EVENT_VERSION,
            pipeline_id,
            stage_id,
            timestamp: axionvera_events::ledger_timestamp(e),
        },
    );
}

#[cfg(test)]
mod test;
