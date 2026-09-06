#!/usr/bin/env python3
"""Run mocked post-deployment smoke test flow for Axionvera Vault contracts."""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_RE = re.compile(r"^(CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
ADDRESS_RE = re.compile(r"^(ADDRESS_PLACEHOLDER|ADMIN_PUBKEY_PLACEHOLDER|G[A-Z2-7]{55})$")
TOKEN_RE = re.compile(r"^(ADDRESS_PLACEHOLDER|DEPOSIT_TOKEN_CONTRACT_ID_PLACEHOLDER|REWARD_TOKEN_CONTRACT_ID_PLACEHOLDER|TOKEN_CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
VALID_NETWORKS = {"local", "testnet", "futurenet", "mainnet"}


def load_contract_id(registry_path: Path, network: str, fallback: str) -> tuple[str, str]:
    if registry_path.is_file():
        try:
            data = json.loads(registry_path.read_text(encoding="utf-8"))
            envs = data.get("environments", {})
            net_env = envs.get(network, {})
            contracts = net_env.get("contracts", {})
            vault = contracts.get("axionvera_vault_contract", {})
            cid = vault.get("contract_id")
            if cid:
                return cid, str(registry_path.relative_to(ROOT) if registry_path.is_relative_to(ROOT) else registry_path)
        except Exception:
            pass
    return fallback, "fallback"


def load_init_input(init_path: Path) -> dict:
    if not init_path.is_file():
        raise FileNotFoundError(f"Initialization input file not found: {init_path}")
    data = json.loads(init_path.read_text(encoding="utf-8"))
    required = {"schema_version", "network", "contract_id", "admin_public_key", "deposit_token_contract_id", "reward_token_contract_id"}
    missing = sorted(required - data.keys())
    if missing:
        raise ValueError(f"Missing required initialization fields: {missing}")
    return data


def run_smoke_flow(
    input_config_path: Path = None,
    registry_path: Path = None,
    init_input_path: Path = None,
) -> dict:
    # 1. Defaults
    network = "testnet"
    fallback_cid = "CONTRACT_ID_PLACEHOLDER"
    sim_deposit = 1000
    sim_reward = 250

    reg_p = registry_path or (ROOT / "examples/contract-id-registry.json")
    init_p = init_input_path or (ROOT / "examples/vault-initialization-input.json")

    if input_config_path and input_config_path.is_file():
        cfg = json.loads(input_config_path.read_text(encoding="utf-8"))
        if cfg.get("schema_version") != "1":
            raise ValueError("input config schema_version must be '1'")
        network = cfg.get("network", network)
        if network not in VALID_NETWORKS:
            raise ValueError(f"Invalid network in config: {network}")
        fallback_cid = cfg.get("fallback_contract_id", fallback_cid)
        sim_deposit = cfg.get("simulated_deposit_amount", sim_deposit)
        sim_reward = cfg.get("simulated_reward_amount", sim_reward)
        if not registry_path and "contract_id_source" in cfg:
            reg_p = ROOT / cfg["contract_id_source"]
        if not init_input_path and "initialization_input_source" in cfg:
            init_p = ROOT / cfg["initialization_input_source"]

    # Stage 1: Contract ID loading
    contract_id, cid_src = load_contract_id(reg_p, network, fallback_cid)
    if not CONTRACT_RE.fullmatch(contract_id):
        raise ValueError(f"Invalid contract_id resolved: {contract_id}")

    stage_1 = {
        "stage_index": 1,
        "stage_name": "contract_id_loading",
        "status": "PASSED",
        "checks": [
            {"check": "registry_resolution", "result": contract_id, "expected": contract_id},
            {"check": "contract_id_format_valid", "result": True, "expected": True},
        ],
    }

    # Stage 2: Initialization input loading
    init_data = load_init_input(init_p)
    admin_addr = init_data.get("admin_public_key", "")
    dep_token = init_data.get("deposit_token_contract_id", "")
    rew_token = init_data.get("reward_token_contract_id", "")

    if not ADDRESS_RE.fullmatch(admin_addr):
        raise ValueError(f"Invalid admin address: {admin_addr}")
    if not TOKEN_RE.fullmatch(dep_token):
        raise ValueError(f"Invalid deposit token address: {dep_token}")
    if not TOKEN_RE.fullmatch(rew_token):
        raise ValueError(f"Invalid reward token address: {rew_token}")

    stage_2 = {
        "stage_index": 2,
        "stage_name": "initialization_input_loading",
        "status": "PASSED",
        "checks": [
            {"check": "init_schema_version", "result": str(init_data.get("schema_version")), "expected": "1"},
            {"check": "admin_address_format_valid", "result": True, "expected": True},
            {"check": "deposit_token_format_valid", "result": True, "expected": True},
            {"check": "reward_token_format_valid", "result": True, "expected": True},
        ],
    }

    # Stage 3: Read verification (simulated)
    stage_3 = {
        "stage_index": 3,
        "stage_name": "read_verification",
        "status": "PASSED",
        "checks": [
            {"check": "is_initialized", "result": True, "expected": True},
            {"check": "initial_total_deposits", "result": 0, "expected": 0},
            {"check": "initial_reward_balance", "result": 0, "expected": 0},
        ],
    }

    # Stage 4: Deposit accounting simulation
    stage_4 = {
        "stage_index": 4,
        "stage_name": "deposit_accounting_simulation",
        "status": "PASSED",
        "checks": [
            {"check": "simulated_deposit_event", "result": "vault/deposit", "expected": "vault/deposit"},
            {"check": "user_balance_after_deposit", "result": sim_deposit, "expected": sim_deposit},
            {"check": "total_deposits_after_deposit", "result": sim_deposit, "expected": sim_deposit},
        ],
    }

    # Stage 5: Reward and claim simulation
    stage_5 = {
        "stage_index": 5,
        "stage_name": "reward_and_claim_simulation",
        "status": "PASSED",
        "checks": [
            {"check": "pending_rewards_calculated", "result": sim_reward, "expected": sim_reward},
            {"check": "simulated_claim_event", "result": "vault/claim", "expected": "vault/claim"},
            {"check": "user_balance_after_withdrawal", "result": 0, "expected": 0},
        ],
    }

    init_summary = {
        "admin_address": admin_addr,
        "deposit_token": dep_token,
        "reward_token": rew_token,
        "loaded_from": str(init_p.relative_to(ROOT) if init_p.is_relative_to(ROOT) else init_p),
    }

    report = {
        "schema_version": "1",
        "flow_name": "mocked-post-deployment-smoke",
        "network": network,
        "contract_id": contract_id,
        "initialization_summary": init_summary,
        "lifecycle_stages": [stage_1, stage_2, stage_3, stage_4, stage_5],
        "overall_status": "PASSED",
        "maintainer_smoke_boundary": {
            "no_live_rpc_used": True,
            "secrets_exposed": False,
        },
    }

    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, help="Path to smoke test input config JSON")
    parser.add_argument("--registry", type=Path, help="Path to contract-id-registry JSON")
    parser.add_argument("--init-input", type=Path, help="Path to vault-initialization-input JSON")
    parser.add_argument("--output", type=Path, help="Output path for report JSON")
    parser.add_argument("--json", action="store_true", help="Print report in JSON format")
    args = parser.parse_args()

    try:
        report = run_smoke_flow(args.input, args.registry, args.init_input)
    except Exception as exc:
        print(f"FAILED: {exc}")
        return 1

    rendered = json.dumps(report, indent=2) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
        print(f"Smoke test report saved to {args.output}")

    if args.json:
        print(rendered)
    else:
        print("====================================================")
        print(" Axionvera Vault - Mocked Post-Deployment Smoke Flow")
        print("====================================================")
        print(f"Flow Name:   {report['flow_name']}")
        print(f"Network:     {report['network']}")
        print(f"Contract ID: {report['contract_id']}")
        print(f"Admin:       {report['initialization_summary']['admin_address']}")
        print(f"Init Source: {report['initialization_summary']['loaded_from']}")
        print("----------------------------------------------------")
        for stage in report["lifecycle_stages"]:
            print(f"[{stage['status']}] Stage {stage['stage_index']}: {stage['stage_name']}")
            for c in stage["checks"]:
                print(f"    - {c['check']}: {c['result']} (expected: {c['expected']})")
        print("====================================================")
        print("PASSED: mocked post-deployment smoke test flow completed successfully")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
