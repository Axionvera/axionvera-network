use std::{env, process};

use axionvera_network_node::load_config;

fn main() {
    let mut args = env::args_os();
    let program = args.next().unwrap_or_default();
    let Some(path) = args.next() else {
        eprintln!("usage: {} <config.json>", program.to_string_lossy());
        process::exit(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {} <config.json>", program.to_string_lossy());
        process::exit(2);
    }

    match load_config(&path) {
        Ok(config) => {
            println!(
                "VALID: network_name={}, rpc_url={}, environment={}",
                config.network_name, config.rpc_url, config.environment
            );
        }
        Err(error) => {
            eprintln!("INVALID: {error}");
            process::exit(1);
        }
    }
}
