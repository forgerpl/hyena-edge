use log::*;
use dotenv::dotenv;
use std::env::{var_os, set_var};
use colored_logger::FormatterBuilder;

mod cli;
mod nanoserver;

fn configure_logging_env() {
    let hyena_debug_env = "HYENA_DEBUG";
    let rust_log_env = "RUST_LOG";
    let default_logging_params = "info";
    let default_debug_logging_params = "hyena=debug,hyena_engine=debug,hyena_api=debug,\
        hyena_common=debug,nanomsg_multi_server=debug";

    let set_rust_log = |value: &str| {
        set_var(rust_log_env, value);
    };

    dotenv().ok();

    // RUST_LOG overwrites everything
    if var_os(&rust_log_env).is_none() {
        if let Some(debug) = var_os(hyena_debug_env) {
            set_rust_log(match debug.to_str() {
                Some("1") => default_debug_logging_params,
                _ => default_logging_params,
            })
        } else {
            set_rust_log(default_logging_params);
        }
    }
}

fn main() {
    configure_logging_env();

    let options = cli::app().get_matches();

    let formatter = FormatterBuilder::default().build();
    flexi_logger::Logger::with_env()
        .format(formatter)
        .start()
        .expect("Logger initialization failed");

    info!("Starting Hyena");

    debug!("Data directory: {}", options.value_of("data_dir").unwrap());

    nanoserver::run(options);
}
