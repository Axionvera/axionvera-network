#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Events, vec, Address, Env, IntoVal, Symbol};

use axionvera_events::{
    ACT_PIPE_COMPL, ACT_PIPE_FAIL, ACT_PIPE_START, ACT_STAGE_COMPL, ACT_STAGE_START, PROTOCOL,
};
use axionvera_interfaces::{
    PipelineAction, PipelineDefinition, PipelineStage, PipelineStatus,
};

#[soroban_sdk::contract]
pub struct MockContract;

#[soroban_sdk::contractimpl]
impl MockContract {
    pub fn success(_e: Env) {}
    pub fn fail(_e: Env) {
        panic!("Mock failure");
    }
}

fn setup_test(e: &Env) -> (Address, PipelineContractClient<'_>) {
    let mock_addr = e.register_contract(None, MockContract);
    let pipeline_addr = e.register_contract(None, PipelineContract);
    let client = PipelineContractClient::new(e, &pipeline_addr);
    (mock_addr, client)
}

#[test]
fn test_successful_pipeline() {
    let e = Env::default();
    let (mock_addr, client) = setup_test(&e);

    let stage1 = PipelineStage {
        id: Symbol::new(&e, "stage1"),
        validation: vec![
            &e,
            PipelineAction {
                target: mock_addr.clone(),
                function: Symbol::new(&e, "success"),
                args: vec![&e],
            },
        ],
        execution: PipelineAction {
            target: mock_addr.clone(),
            function: Symbol::new(&e, "success"),
            args: vec![&e],
        },
        post_hook: vec![
            &e,
            PipelineAction {
                target: mock_addr.clone(),
                function: Symbol::new(&e, "success"),
                args: vec![&e],
            },
        ],
    };

    let pipeline = PipelineDefinition {
        id: Symbol::new(&e, "test_pipe"),
        stages: vec![&e, stage1],
    };

    let receipt = client.execute_pipeline(&pipeline);

    assert_eq!(receipt.status, PipelineStatus::Completed);
    assert_eq!(receipt.failed_stage, None);
    assert_eq!(receipt.error_code, None);

    let events = e.events().all();
    assert!(events.iter().any(|ev| ev.1 == vec![&e, PROTOCOL.into_val(&e), ACT_PIPE_START.into_val(&e)]));
    assert!(events.iter().any(|ev| ev.1 == vec![&e, PROTOCOL.into_val(&e), ACT_STAGE_START.into_val(&e)]));
    assert!(events.iter().any(|ev| ev.1 == vec![&e, PROTOCOL.into_val(&e), ACT_STAGE_COMPL.into_val(&e)]));
    assert!(events.iter().any(|ev| ev.1 == vec![&e, PROTOCOL.into_val(&e), ACT_PIPE_COMPL.into_val(&e)]));
}

#[test]
fn test_pipeline_validation_failure() {
    let e = Env::default();
    let (mock_addr, client) = setup_test(&e);

    let stage1 = PipelineStage {
        id: Symbol::new(&e, "stage1"),
        validation: vec![
            &e,
            PipelineAction {
                target: mock_addr.clone(),
                function: Symbol::new(&e, "fail"),
                args: vec![&e],
            },
        ],
        execution: PipelineAction {
            target: mock_addr.clone(),
            function: Symbol::new(&e, "success"),
            args: vec![&e],
        },
        post_hook: vec![&e],
    };

    let pipeline = PipelineDefinition {
        id: Symbol::new(&e, "test_pipe"),
        stages: vec![&e, stage1],
    };

    let receipt = client.execute_pipeline(&pipeline);

    assert_eq!(receipt.status, PipelineStatus::Failed);
    assert_eq!(receipt.failed_stage, Some(Symbol::new(&e, "stage1")));
    assert_eq!(
        receipt.error_code,
        Some(PipelineError::StageValidationFailed as u32)
    );

    let events = e.events().all();
    assert!(events.iter().any(|ev| ev.1 == vec![&e, PROTOCOL.into_val(&e), ACT_PIPE_FAIL.into_val(&e)]));
}

#[test]
fn test_pipeline_execution_failure() {
    let e = Env::default();
    let (mock_addr, client) = setup_test(&e);

    let stage1 = PipelineStage {
        id: Symbol::new(&e, "stage1"),
        validation: vec![&e],
        execution: PipelineAction {
            target: mock_addr.clone(),
            function: Symbol::new(&e, "fail"),
            args: vec![&e],
        },
        post_hook: vec![&e],
    };

    let pipeline = PipelineDefinition {
        id: Symbol::new(&e, "test_pipe"),
        stages: vec![&e, stage1],
    };

    let receipt = client.execute_pipeline(&pipeline);

    assert_eq!(receipt.status, PipelineStatus::Failed);
    assert_eq!(receipt.failed_stage, Some(Symbol::new(&e, "stage1")));
    assert_eq!(
        receipt.error_code,
        Some(PipelineError::StageExecutionFailed as u32)
    );
}

#[test]
fn test_pipeline_hook_failure() {
    let e = Env::default();
    let (mock_addr, client) = setup_test(&e);

    let stage1 = PipelineStage {
        id: Symbol::new(&e, "stage1"),
        validation: vec![&e],
        execution: PipelineAction {
            target: mock_addr.clone(),
            function: Symbol::new(&e, "success"),
            args: vec![&e],
        },
        post_hook: vec![
            &e,
            PipelineAction {
                target: mock_addr.clone(),
                function: Symbol::new(&e, "fail"),
                args: vec![&e],
            },
        ],
    };

    let pipeline = PipelineDefinition {
        id: Symbol::new(&e, "test_pipe"),
        stages: vec![&e, stage1],
    };

    let receipt = client.execute_pipeline(&pipeline);

    assert_eq!(receipt.status, PipelineStatus::Failed);
    assert_eq!(receipt.failed_stage, Some(Symbol::new(&e, "stage1")));
    assert_eq!(
        receipt.error_code,
        Some(PipelineError::PostHookFailed as u32)
    );
}
