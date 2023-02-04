use crate::types::{Config, Pipeline, Trigger};
use log::warn;
use std::error::Error;

pub trait Getters<T> {
    /// Return every instances of the struct.
    fn get() -> Result<Vec<T>, Box<dyn Error>>;
    /// Return an instance of the struct.
    fn get_by_name(name: &str) -> Result<T, Box<dyn Error>>;
}

impl Getters<Pipeline> for Pipeline {
    fn get() -> Result<Vec<Pipeline>, Box<dyn Error>> {
        let config = Config::new();
        let optional = config.pipelines;
        match optional {
            Some(p) => return Ok(p),
            None => {
                let message = "Couldn't retrieve pipelines";
                return Err(Box::from(message));
            }
        };
    }
    fn get_by_name(name: &str) -> Result<Pipeline, Box<dyn Error>> {
        let pipelines = Pipeline::get()?;
        let optional = pipelines.iter().filter(|p| p.name == name).next();
        match optional {
            Some(res) => return Ok(res.to_owned()),
            None => {
                let message = format!("Couldn't find pipeline {:?}", name);
                warn!("{}", message);
                return Err(Box::from(message));
            }
        };
    }
}

impl Getters<Trigger> for Trigger {
    fn get() -> Result<Vec<Trigger>, Box<dyn Error>> {
        let pipelines = Pipeline::get()?;
        let mut triggers = pipelines
            .iter()
            .map(|p| p.triggers.clone().unwrap_or_default())
            .collect::<Vec<Vec<Trigger>>>()
            .into_iter()
            .flatten()
            .collect::<Vec<Trigger>>();
        triggers.sort();
        triggers.dedup();
        Ok(triggers)
    }
    fn get_by_name(name: &str) -> Result<Trigger, Box<dyn Error>> {
        let triggers = Trigger::get();
        let binding = triggers?;
        let trigger = binding.iter().next().unwrap();
        Ok(trigger.to_owned())
    }
}