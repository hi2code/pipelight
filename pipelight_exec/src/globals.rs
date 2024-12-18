// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// Error Handling
use log::trace;
use miette::Result;
use pipelight_error::PipelightError;

use std::env;

/**
* A lazy global that contains the default shell to be used by the invoked processes.
*/
pub static SHELL: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new("sh".to_owned())));

/**
* A lazy global that contains the default output directory to be used by the invoked processes
* that pipe their outputs(stdout/stderr) into files.
*/
pub static OUTDIR: Lazy<Arc<Mutex<String>>> =
    Lazy::new(|| Arc::new(Mutex::new(".pipelight/proc".to_owned())));

/**
* Returns the user session shell or fallback to default "sh".
*/
pub fn get_shell() -> Result<(), std::io::Error> {
    trace!("Get shell");
    let user_shell = env::var("SHELL");
    match user_shell {
        Ok(res) => {
            *SHELL.lock().unwrap() = res.to_owned();
        }
        Err(_) => {}
    }
    Ok(())
}
