#![allow(unused_variables)]
#[allow(dead_code)]
use log::error;
use shared::trigger::trigger;
use std::error::Error;
use std::process::exit;

fn main() {
    handler().unwrap_or_else(|e| {
        error!("{}", e);
        exit(1)
    })
}

/// Launch detached subprocess
fn handler() -> Result<(), Box<dyn Error>> {
    // Detached process
    trigger(false)?;
    Ok(())
}
