use pretty_assertions::assert_eq;

mod framework;

#[test]
fn both_changed_both_added_lines_in_if_change() -> anyhow::Result<()> {
    // both a.sh and b.sh changed
    //   a.sh and b.sh contain if-change-then-change
    //   a.sh and b.sh changed in if-change block
    let run =
        framework::run_tool("tests/data/basic/both-changed-both-added-lines-in-if-change.diff")?;

    assert_eq!(run.stdout, "");
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn both_changed_both_removed_lines_in_if_change() -> anyhow::Result<()> {
    // both a.sh and b.sh changed
    //   a.sh and b.sh contain if-change-then-change
    //   a.sh and b.sh changed in if-change block
    let run =
        framework::run_tool("tests/data/basic/both-changed-both-removed-lines-in-if-change.diff")?;

    assert_eq!(run.stdout, "");
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn both_changed_one_in_if_change() -> anyhow::Result<()> {
    // both a.sh and b.sh changed
    //   a.sh and b.sh contain if-change-then-change
    //   a.sh changed in if-change block
    //   b.sh changed outside if-change block
    let run = framework::run_tool("tests/data/basic/both-changed-one-in-if-change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/basic/b.sh:4 - expected change here due to if-change in tests/data/basic/a.sh
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn both_changed_one_missing_if_change() -> anyhow::Result<()> {
    // both c.sh and d.sh changed
    //   c.sh contains if-change-then-change
    //   c.sh changed in if-change block
    //   d.sh does not contain an if-change-then-change block
    let run = framework::run_tool("tests/data/basic/both-changed-one-missing-if-change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/basic/d.sh - expected if-change-then-change in this file due to if-change in tests/data/basic/c.sh
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn one_changed_in_if_change() -> anyhow::Result<()> {
    // a.sh changed in if-change, then-change points at b.sh
    // b.sh contains an if-change-then-change pointing back at a.sh
    let run = framework::run_tool("tests/data/basic/one-changed-in-if-change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/basic/b.sh:4 - expected change here due to if-change in tests/data/basic/a.sh
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn one_changed_in_if_change_other_missing_if_change() -> anyhow::Result<()> {
    // c.sh changed in if-change, then-change points at d.sh
    // d.sh does not contain an if-change-then-change block
    let run = framework::run_tool("tests/data/basic/one-changed-other-missing-if-change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/basic/d.sh - expected if-change-then-change in this file due to if-change in tests/data/basic/c.sh
tests/data/basic/d.sh - expected change here due to if-change in tests/data/basic/c.sh
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

// TODO- need tests for when thenchange references a file that does not exist
