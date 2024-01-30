use std::path::Path;
use pretty_assertions::{assert_eq, assert_ne};

mod framework;

#[test]
fn test_basic_both_changed() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/basic-both-changed.diff")?;

    assert_eq!(run.stdout, r"
    this is stdout
");
    assert_eq!(run.exit_code, 0);

    Ok(())
}
