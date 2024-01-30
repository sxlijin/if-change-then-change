use assert_cmd::prelude::*;

use std::fmt;
use std::fs::File;
use std::process::{Command, Stdio};

#[derive(Debug, Eq, PartialEq)]
pub struct ToolOutput {
    pub stdout: String,
    pub exit_code: i32,
}

// data_path is relative to repository root
pub fn run_tool(data_path: &str) -> anyhow::Result<ToolOutput> {
    let mut cmd = Command::cargo_bin("to-be-named")?;

    cmd.env("RUST_LOG", "debug");
    cmd.stdin(File::open(data_path)?);
    cmd.stderr(Stdio::inherit());

    let output = cmd.output()?;

    return Ok(ToolOutput {
        stdout: String::from_utf8(output.stdout)?,
        exit_code: output.status.code().ok_or("No exit code - process was cancelled, maybe?")?,
    });
}