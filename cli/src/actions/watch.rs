// Structs
use crate::services::{self, Action};
use crate::services::{FgBg, Service};
use crate::types::{Commands, DetachableCommands, PostCommands, Trigger};

use pipelight_utils::git::{Flag, Special};
use pipelight_watcher::*;
use std::env;
// Globals
use std::sync::Arc;

use std::future::Future;
use watchexec::{action::ActionHandler, Config, Watchexec};
use watchexec_events::Event;
use watchexec_signals::Signal;

// Globals
use crate::globals::CLI;
// Error handling
use miette::{Diagnostic, IntoDiagnostic, Result};

/**
* The watcher main action.
*
* Modify the triggering env by setting the action to watch
* And try to trigger pipelines.
*/
pub fn watch_and_trigger() -> Result<()> {
    let flag = Some(String::from(&Flag::Special(Special::Watch)));
    let mut args = CLI.lock().unwrap().clone();
    args.commands = Commands::PostCommands(PostCommands::DetachableCommands(
        DetachableCommands::Trigger(Trigger { flag }),
    ));
    Service::new(services::Action::Trigger, Some(args))?.should_detach()?;
    Ok(())
}

/**
* Wrap the main action into a common signal handling action handler.
*/
pub fn default_action_handler(
    mut action: ActionHandler,
) -> Box<dyn Future<Output = ActionHandler> + Send + Sync> {
    // Pipeline execution
    watch_and_trigger().unwrap();

    // Handle Stop signals
    if action
        .signals()
        .any(|sig| sig == Signal::Interrupt || sig == Signal::Terminate)
    {
        action.quit();
    }

    if let Some(ignore_file) = get_ignore_path().ok() {
        for event in action.events.iter() {
            let paths: Vec<String> = event
                .paths()
                .map(|e| e.0.to_str().unwrap().to_owned())
                .collect();

            if paths.contains(&ignore_file) {
                // Self reconfigure when the ignore file changes
                // reconfigure(&w_clone, &r_clone, &action).await.unwrap();
            }
        }
    }
    // Actions
    return Box::new(async { action });
}

/**
* Build an appropriate watcher that:
* - self reconfigures on ignore file changes
* - ignores pipelight autogenerated tmp files
* - can trigger pipelines
*/
pub async fn build() -> Result<Watchexec> {
    // Create a Watchexec with action handler
    let watchexec = Watchexec::default();
    // let watchexec = Watchexec::new_async(default_action_handler)?;
    watchexec.config.on_action_async(default_action_handler);

    // Search for an ignore file to set a watch filter
    match get_ignore_path() {
        Ok(res) => {
            let filterer = make_filter_configuration(&res).await?;
            watchexec.config.filterer(Arc::new(filterer));
        }
        Err(_) => {
            let filterer = make_default_filter_configuration().await?;
            watchexec.config.filterer(Arc::new(filterer));
        }
    }
    // Watch only the current directory
    watchexec.config.pathset(vec![env::current_dir().unwrap()]);

    Ok(watchexec)
}

/**
Build and launch the custom watcher
*/
#[tokio::main]
pub async fn launch() -> Result<()> {
    // Kill already running watcher
    Watcher::kill_homologous()?;
    let watchexec = build().await?;
    watchexec.main().await.into_diagnostic()??;
    Ok(())
}