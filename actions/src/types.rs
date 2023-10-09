// Struct
use cli::types::Cli;
// Error Handling
use miette::Result;

#[derive(Debug, Clone)]
pub enum Action {
    // Parameter: pipeline name
    Run(Option<String>),
    // Parameter: flag name
    Trigger(Option<String>),
    Watch,
}

#[derive(Debug, Clone)]
pub struct Run;

impl Action {
    pub fn start(&self) -> Result<()> {
        match self {
            Action::Trigger(flag) => {
                /**
                Filter pipeline by trigger and run
                */
                let mut pipelines = Pipeline::get()?;
                pipelines.par_iter_mut().for_each(|pipeline| {
                    if pipeline.is_triggerable().is_ok() {
                        pipeline.run().unwrap();
                    }
                });
            }
        }
        Ok(())
    }
}
