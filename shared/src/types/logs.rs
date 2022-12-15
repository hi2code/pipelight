#![allow(dead_code)]
use chrono::{DateTime, Local, NaiveDateTime, Offset, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::{Result, Value};
use std::clone::Clone;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PipelineState {
    Started,
    Succeeded,
    Failed,
    Running,
    Aborted,
    Never,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineLog<'a> {
    pub state: PipelineState,
    pub date: String,
    pub name: &'a str,
    pub step: Option<StepLog<'a>>,
}
impl<'a> PipelineLog<'a> {
    pub fn new(name: &'a str) -> Self {
        PipelineLog {
            name,
            date: Utc::now().to_string(),
            step: Default::default(),
            state: PipelineState::Started,
        }
    }
    pub fn state(&mut self, state: PipelineState) -> &Self {
        self.state = state;
        return self;
    }
    pub fn step(&mut self, step: &'a str) -> &Self {
        self.step = Some(StepLog::new(step).to_owned());
        return self;
    }
    pub fn command(&mut self, cmd: &'a str, stdout: &'a str) -> &Self {
        self.step.as_mut().unwrap().command = CommandLog::new().to_owned();
        self.step.as_mut().unwrap().command.stdin = cmd.to_owned();
        self.step.as_mut().unwrap().command.stdout = stdout.to_owned();
        return self;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StepLog<'a> {
    pub name: &'a str,
    pub command: CommandLog,
}
impl<'a> StepLog<'a> {
    pub fn new(name: &'a str) -> Self {
        StepLog {
            name,
            command: CommandLog::new(),
        }
    }
    pub fn name(&mut self, name: &'a str) -> &Self {
        self.name = name;
        return self;
    }
    pub fn command(&mut self, command: &'a str) -> &mut Self {
        self.command = CommandLog::new();
        return self;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandLog {
    pub stdin: String,
    pub stdout: String,
    pub stderr: String,
}
impl CommandLog {
    fn new() -> Self {
        CommandLog {
            stdin: "".to_owned(),
            stdout: "".to_owned(),
            stderr: "".to_owned(),
        }
    }
    pub fn stdin(&mut self, cmd: String) -> &mut Self {
        self.stdin = cmd;
        return self;
    }
}
