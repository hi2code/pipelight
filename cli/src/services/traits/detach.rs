// Struct
use crate::services::types::{Action, Service};
use crate::types::{Cli, Commands, DetachableCommands, PostCommands};
use crate::types::{Pipeline, Trigger, Watch};
use exec::Status;
use utils::git::Flag;
// Process manipulation
use exec::SelfProcess;
// Error Handling
use log::trace;
use miette::Result;

pub trait FgBg {
    /**
    Fork action/process end send to background
    */
    fn detach(&self) -> Result<()>;
    /**
    Fork action/process end keep in foreground
    */
    fn attach(&self) -> Result<()>;
    /**
    Inspect the parsed command line arguments (CLI global, attach flag)
    and determine whether to detach the subprocess or not.
    */
    fn should_detach(&mut self) -> Result<()>;
}

impl FgBg for Service {
    fn attach(&self) -> Result<()> {
        if let Some(args) = self.args.clone() {
            SelfProcess::run_fg_with_cmd(&String::from(&args))?;
        }
        Ok(())
    }
    fn detach(&self) -> Result<()> {
        if let Some(args) = self.args.clone() {
            SelfProcess::run_bg_with_cmd(&String::from(&args))?;
        }
        Ok(())
    }
    fn should_detach(&mut self) -> Result<()> {
        if let Some(mut args) = self.args.clone() {
            match args.attach.clone() {
                true => {
                    trace!("pipelight process is attached");
                    self.attach()?;
                }
                false => {
                    trace!("detach pipelight process");
                    // Exit the detach loop
                    self.args.as_mut().map(|e| e.attach = true);
                    self.detach()?;
                }
            };
        }
        Ok(())
    }
}
