// Struct
use utils::git::Flag;
// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
// Teleport
use utils::{logger::Logger, teleport::Portal};
// Logs
use workflow::{Config, Trigger};
// Cli
use clap::FromArgMatches;
use cli::services::FgBg;
use cli::services::{Action, Service};
use cli::types::Cli;
use cli::types::{Commands, DetachableCommands, PostCommands};
// Error Handling
use log::{info, trace};
use miette::Result;

// Global vars
use cli::globals::CLI;
use utils::globals::LOGGER;
use workflow::globals::CONFIG;

pub static PORTAL: Lazy<Arc<Mutex<Portal>>> = Lazy::new(|| Arc::new(Mutex::new(Portal::default())));

// Hydrate logs
pub fn early_hydrate_logger() -> Result<()> {
    let args;
    args = CLI.lock().unwrap().clone();
    // Set internal verbosity level
    let verbosity = args.verbose.log_level_filter();
    LOGGER.lock().unwrap().set_level(&verbosity)?;
    // Set verbosity level
    let verbosity = args.internal_verbose.log_level_filter();
    LOGGER.lock().unwrap().set_internal_level(&verbosity)?;
    Ok(())
}
// Hydrate logs
pub fn full_hydrate_logger() -> Result<()> {
    LOGGER.lock().unwrap().to_file();
    Ok(())
}

// Hydrate trigger
pub fn hydrate_trigger() -> Result<()> {
    let args;
    args = CLI.lock().unwrap().clone();
    let mut flag = None;
    match args.commands {
        Commands::PostCommands(post_commands) => match post_commands {
            PostCommands::DetachableCommands(detachable_commands) => match detachable_commands {
                DetachableCommands::Trigger(trigger) => {
                    flag = trigger.flag;
                }
                DetachableCommands::Run(pipeline) => {
                    flag = pipeline.trigger.flag;
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }
    if let Some(flag) = flag {
        Trigger::set(Some(Flag::from(&flag)))?;
    }
    Ok(())
}

// Hydrate portal
pub fn hydrate_portal() -> Result<()> {
    trace!("hydrate portal");
    let args;
    args = CLI.lock().unwrap().clone();

    let seed = if args.config.is_some() {
        args.config.unwrap()
    } else {
        "pipelight".to_owned()
    };
    let portal = Portal::new()?.seed(&seed).search()?;
    info!(
        "Found config file at: {}",
        portal.target.file_path.clone().unwrap()
    );
    *PORTAL.lock().unwrap() = portal;
    Ok(())
}

// Hydrate config
pub fn hydrate_config() -> Result<()> {
    trace!("hydrate config");
    let portal;
    let args;
    args = CLI.lock().unwrap().clone();
    portal = PORTAL.lock().unwrap().clone();

    let casted_config = cast::Config::load(&portal.target.file_path.unwrap(), args.raw.clone())?;
    let config = Config::from(&casted_config);
    *CONFIG.lock().unwrap() = config.clone();

    Ok(())
}

/**
Read the command line and the config file
then hydrate every globals.
*/
pub fn set_globals() -> Result<()> {
    trace!("Set globals [full]");
    let cond = *CONFIG.lock().unwrap() == Config::default()
        && *PORTAL.lock().unwrap() == Portal::default();
    if cond {
        // hydrate the CLI global var
        Cli::hydrate()?;
        hydrate_trigger()?;
        // early_hydrate_logger()?;
        // hydrate the PORTAL global var
        hydrate_portal()?;
        // hydrate the CONFIG global var
        (*PORTAL.lock().unwrap()).teleport()?;
        full_hydrate_logger()?;
        hydrate_config()?;
    }
    Ok(())
}

/**
Only read the command line and ignore the config file
then hydrate globals that can be hydrated.
*/
pub fn set_early_globals() -> Result<()> {
    trace!("Set globals [early]");
    let cond = *CONFIG.lock().unwrap() == Config::default()
        && *PORTAL.lock().unwrap() == Portal::default();
    if cond {
        // hydrate the CLI global var
        Cli::hydrate()?;
        early_hydrate_logger()?;
    }
    Ok(())
}
