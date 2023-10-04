// Detach
use crate::utils::should_detach;
// Struct
use exec::Status;
use workflow::{Getters, Node, Pipeline};
// Error Handling
use miette::{Error, Result};
// Async
use std::future::Future;

/**
Run the pipeline.
Detach it if needed.
and return fancy log to stdout.
*/
pub fn launch(pipeline_name: &str) -> Result<()> {
    // Guard
    let mut pipeline = Pipeline::get_by_name(&pipeline_name)?;
    if pipeline.is_triggerable()? {
        // Execute pipeline
        match should_detach()? {
            false => action(&mut pipeline)?,
            true => {}
        };
    }
    Ok(())
}

pub fn action(pipeline: &mut Pipeline) -> Result<()> {
    // Action
    pipeline.run()?;
    // Return pipeline log
    println!("{}", Node::from(&*pipeline));
    match pipeline.status {
        Some(Status::Succeeded) => Ok(()),
        Some(Status::Failed) => {
            let message = "Pipeline status: Failed";
            Err(Error::msg(message))
        }
        _ => Ok(()),
    }
}
