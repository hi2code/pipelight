// Structs
use crate::services;
use crate::services::{FgBg, Service};
use crate::types::{Commands, DetachableCommands, PostCommands, Trigger};
use utils::git::{Flag, Special};
use utils::teleport::Portal;
// use crate::trigger;
// Globals
use std::sync::Arc;
// Watchexec
use ignore_files::{IgnoreFile, IgnoreFilter};
use watchexec::{
    // Trait
    action::{Action, Outcome},
    config::{InitConfig, RuntimeConfig},
    handler::PrintDebug,
    Watchexec,
};
use watchexec_events::Event;
use watchexec_filterer_ignore::IgnoreFilterer;
use watchexec_signals::Signal;
// Env
use std::env;
use std::path::Path;
// Globals
use crate::globals::CLI;
// Error handling
use miette::{Diagnostic, IntoDiagnostic, Result};
use thiserror::Error;

// Watchexec - Bug fix Struct
#[derive(Debug, Error, Diagnostic)]
#[error("stub")]
struct MietteStub;

/**
 * Retrieve an ignore file fullpath if any.
*/
fn get_ignore_path() -> Result<String> {
    // Search an ignore file to set a filter
    let mut portal = Portal::new()?;
    portal.seed(".pipelight_ignore");
    match portal.search() {
        Ok(res) => Ok(res.target.file_path.unwrap()),
        Err(_) => {
            let mut portal = Portal::new()?;
            portal.seed(".gitignore");
            match portal.search() {
                Ok(res) => Ok(res.target.file_path.unwrap()),
                Err(err) => return Err(err),
            }
        }
    }
}

/**
Build an appropriate watcher that:
- self reconfigures on ignore file change
- ignores pipelight generated tmp files
- can trigger pipelines
*/
pub async fn build() -> Result<(Arc<Watchexec>, RuntimeConfig)> {
    // Default config
    let mut init = InitConfig::default();
    init.on_error(PrintDebug(std::io::stderr()));
    let mut runtime = RuntimeConfig::default();

    // Search an ignore file to set a filter
    match get_ignore_path() {
        Ok(res) => {
            let filterer = filter_configuration(&res).await?;
            runtime.filterer(Arc::new(filterer));
        }
        Err(_) => {}
    }

    // Watch cwd only
    runtime.pathset(vec![env::current_dir().unwrap()]);

    // Create WE instance
    let watchexec = Watchexec::new(init, runtime.clone()).unwrap();
    let w_clone = watchexec.clone();
    let r_clone = runtime.clone();

    runtime.on_action(move |action: Action| {
        let w_clone = w_clone.clone();
        let r_clone = r_clone.clone();

        async move {
            // Self reconfigure on ignore file change
            reconfigure(&w_clone, &r_clone, &action).await.unwrap();
            // Pipeline execution
            watch_trigger().unwrap();
            // Handle Stop signals
            let sigs = action
                .events
                .iter()
                .flat_map(Event::signals)
                .collect::<Vec<_>>();
            if sigs.iter().any(|sig| sig == &Signal::Interrupt) {
                action.outcome(Outcome::Exit);
            } else {
                action.outcome(Outcome::if_running(
                    Outcome::DoNothing,
                    Outcome::both(Outcome::Clear, Outcome::Start),
                ));
            }
            Ok(())
      // (not normally required! ignore this when implementing)
      as std::result::Result<_, MietteStub>
        }
    });
    Ok((watchexec, runtime))
}

/**
Self reconfigure when the IgnoreFile changes.
*/
pub async fn reconfigure(
    watchexec: &Arc<Watchexec>,
    runtime: &RuntimeConfig,
    action: &Action,
) -> Result<()> {
    if let Some(ignore_path) = get_ignore_path().ok() {
        for event in action.events.iter() {
            // if event.paths().any(|(p, _)| p.ends_with(ignorefile)) {
            if event
                .paths()
                .any(|(p, _)| p.to_str().unwrap() == ignore_path)
            {
                // Set Filter
                let filterer = filter_configuration(&ignore_path).await?;
                let mut r = runtime.clone();
                r.filterer(Arc::new(filterer));
                watchexec.reconfigure(r).unwrap();
                break;
            }
        }
    }
    Ok(())
}

/**
Create action filter
Do not watch some files to avoid recursive watching
 */
pub async fn filter_configuration(path: &str) -> Result<IgnoreFilterer> {
    let path = Path::new(path);
    // Set Filter
    let applies_in = env::current_dir().into_diagnostic()?;
    let file = IgnoreFile {
        path: path.into(),
        applies_in: Some(applies_in.clone()),
        applies_to: None,
    };
    let globs = [".pipelight/*", ".git", ".cargo", ".node_modules"];
    let mut filter: IgnoreFilter = IgnoreFilter::empty(applies_in.clone());
    filter
        .add_globs(&globs, Some(&applies_in))
        .into_diagnostic()?;
    filter.add_file(&file).await.into_diagnostic()?;
    let filterer = IgnoreFilterer(filter);
    Ok(filterer)
}

/**
Modify the triggering env by setting the action to watch
And try to trigger pipelines.
*/
pub fn watch_trigger() -> Result<()> {
    let flag = Some(String::from(&Flag::Special(Special::Watch)));
    let mut args = CLI.lock().unwrap().clone();
    args.commands = Commands::PostCommands(PostCommands::DetachableCommands(
        DetachableCommands::Trigger(Trigger { flag }),
    ));
    Service::new(services::Action::Trigger, Some(args))?.should_detach()?;
    Ok(())
}
