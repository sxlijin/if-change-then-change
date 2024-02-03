use pretty_assertions::assert_eq;
use test_log::test;

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
tests/data/basic/b.sh:4 - expected change here due to change in tests/data/basic/a.sh:3-4
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
    let run = framework::run_tool("tests/data/one-file-missing-if-change/both-changed.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/one-file-missing-if-change/d.sh - expected if-change-then-change in this file due to if-change in tests/data/one-file-missing-if-change/c.sh
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
tests/data/basic/b.sh:4 - expected change here due to change in tests/data/basic/a.sh:3-4
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn one_changed_in_if_change_other_missing_if_change() -> anyhow::Result<()> {
    // c.sh changed in if-change, then-change points at d.sh
    // d.sh does not contain an if-change-then-change block
    let run = framework::run_tool("tests/data/one-file-missing-if-change/one-changed.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/one-file-missing-if-change/d.sh - expected if-change-then-change in this file due to if-change in tests/data/one-file-missing-if-change/c.sh
tests/data/one-file-missing-if-change/d.sh - expected change here due to change in tests/data/one-file-missing-if-change/c.sh:3-4
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn post_diff_path_is_dev_null() -> anyhow::Result<()> {
    // this is the "file was deleted" case
    let run = framework::run_tool("tests/data/path-validation/post-diff-path-is-dev-null.diff")?;

    assert_eq!(run.stdout, "");
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn post_diff_path_is_nonexistent() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/path-validation/post-diff-path-is-nonexistent.diff")?;

    assert_eq!(
        run.stdout,
        "\
- - diff references file that does not exist: 'nonexistent.sh'
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn then_change_references_nonexistent_file() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/path-validation/then-change-references-nonexistent-file.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/path-validation/z.sh:4-6 - then-change references file that does not exist: 'nonexistent.cfg'
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn invalid_diff_produces_diagnostics() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/path-validation/invalid-before-after-paths.diff")?;

    assert_eq!(
        run.stdout,
        "\
- - invalid git diff: expected a/before.path -> b/after.path, but got 'invalid-before0.txt' -> 'b/invalid-after0.txt'
- - invalid git diff: expected a/before.path -> b/after.path, but got 'a/invalid-before1.txt' -> 'invalid-after1.txt'
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

// TODO- add test case for renamed file
// TODO- add test case for LFS diff
