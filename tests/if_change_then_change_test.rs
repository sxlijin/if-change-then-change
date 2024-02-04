use pretty_assertions::assert_eq;
use test_log::test;

mod framework;

mod two_files {}
#[test]
fn both_changed_both_added_lines_in_if_change() -> anyhow::Result<()> {
    // both a.sh and b.sh changed
    //   a.sh and b.sh contain if-change-then-change
    //   a.sh and b.sh changed in if-change block
    let run =
        framework::run_tool("tests/data/2-files/both-changed-both-added-lines-in-if-change.diff")?;

    assert_eq!(run.stdout, "");
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn both_changed_both_removed_lines_in_if_change() -> anyhow::Result<()> {
    // both a.sh and b.sh changed
    //   a.sh and b.sh contain if-change-then-change
    //   a.sh and b.sh changed in if-change block
    let run = framework::run_tool(
        "tests/data/2-files/both-changed-both-removed-lines-in-if-change.diff",
    )?;

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
    let run = framework::run_tool("tests/data/2-files/both-changed-one-in-if-change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/2-files/b.sh:3-5 - expected change here due to change in tests/data/2-files/a.sh:2-5
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
tests/data/one-file-missing-if-change/d.sh - expected an if-change-then-change in this file that matches tests/data/one-file-missing-if-change/c.sh:2-5
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn one_changed_in_if_change() -> anyhow::Result<()> {
    // a.sh changed in if-change, then-change points at b.sh
    // b.sh contains an if-change-then-change pointing back at a.sh
    let run = framework::run_tool("tests/data/2-files/one-changed-in-if-change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/2-files/b.sh:3-5 - expected change here due to change in tests/data/2-files/a.sh:2-5
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
tests/data/one-file-missing-if-change/d.sh - expected an if-change-then-change in this file that matches tests/data/one-file-missing-if-change/c.sh:2-5
tests/data/one-file-missing-if-change/d.sh - expected change here due to change in tests/data/one-file-missing-if-change/c.sh:2-5
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn file_with_2_blocks___both_blocks_changed() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/file-with-2-blocks/a.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/file-with-2-blocks/b1.sh:2-4 - expected change here due to change in tests/data/file-with-2-blocks/a.sh:2-5
tests/data/file-with-2-blocks/b2.sh:2-4 - expected change here due to change in tests/data/file-with-2-blocks/a.sh:7-10
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn file_with_2_blocks___both_blocks_missing_changes() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/file-with-2-blocks/b.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/file-with-2-blocks/a.sh:2-5 - expected change here due to change in tests/data/file-with-2-blocks/b1.sh:2-4
tests/data/file-with-2-blocks/a.sh:7-10 - expected change here due to change in tests/data/file-with-2-blocks/b2.sh:2-4
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn file_with_2_blocks___1st_block_changed_2nd_block_missing_change() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/file-with-2-blocks/a-and-b2.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/file-with-2-blocks/a.sh:7-10 - expected change here due to change in tests/data/file-with-2-blocks/b2.sh:2-4
tests/data/file-with-2-blocks/b1.sh:2-4 - expected change here due to change in tests/data/file-with-2-blocks/a.sh:2-5
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn file_with_2_blocks___1st_block_missing_change_2nd_block_changed() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/file-with-2-blocks/a-and-b1.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/file-with-2-blocks/a.sh:2-5 - expected change here due to change in tests/data/file-with-2-blocks/b1.sh:2-4
tests/data/file-with-2-blocks/b2.sh:2-4 - expected change here due to change in tests/data/file-with-2-blocks/a.sh:7-10
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
stdin - diff references file that does not exist: 'nonexistent.sh'
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn then_change_references_nonexistent_file() -> anyhow::Result<()> {
    let run = framework::run_tool(
        "tests/data/path-validation/then-change-references-nonexistent-file.diff",
    )?;

    assert_eq!(
            run.stdout,
            "\
tests/data/path-validation/z.sh:7 - then-change references file that does not exist: 'nonexistent.cfg'
"
        );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn invalid_before_after_paths() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/path-validation/invalid-before-after-paths.diff")?;

    assert_eq!(
            run.stdout,
            "\
stdin - invalid git diff: expected a/before.path -> b/after.path, but got 'a/invalid-before1.txt' -> 'invalid-after1.txt'
stdin - invalid git diff: expected a/before.path -> b/after.path, but got 'invalid-before0.txt' -> 'b/invalid-after0.txt'
"
        );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
#[ignore]
fn new_file() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/diff-has-path-changes/e-new-file.diff")?;

    assert_eq!(run.stdout, "");
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
#[ignore]
fn deleted_file() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/diff-has-path-changes/f-deleted-file.diff")?;

    assert_eq!(
        run.stdout,
        "\
should complain that f1.sh ictc is now orphaned
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
#[ignore]
fn renamed_file() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/diff-has-path-changes/g-renamed-file.diff")?;

    assert_eq!(
        run.stdout,
        "\
g1.sh:5 - then-change points at g2.sh, but g2.sh was deleted
"
    );
    // may need spectral for this
    assert!(!run.stdout.contains("g3.sh:5 - g1.sh was not modified"));
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
#[ignore]
fn copied_file() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/diff-has-path-changes/h-copied-file.diff")?;

    assert_eq!(
        run.stdout,
        "\
still need to decide what'll go here
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn three_files() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/3-files/change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/3-files/build.sh:2-7 - expected change here due to change in tests/data/3-files/release.sh:2-7
tests/data/3-files/push.sh:2-7 - expected change here due to change in tests/data/3-files/release.sh:2-7
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
#[ignore]
fn three_files_chain() -> anyhow::Result<()> {
    // "chain" because push.sh <-> release.sh, push.sh -> build.sh, build.sh -> release.sh
    let run = framework::run_tool("tests/data/3-files-chain/change.diff")?;

    assert_eq!(
        run.stdout,
        "\
still need to decide what'll go here
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn three_files_incomplete() -> anyhow::Result<()> {
    // "incomplete" because 2 of the 3 files do not point at all of the other files
    let run = framework::run_tool("tests/data/3-files-incomplete/change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/3-files-incomplete/build.sh:2-6 - expected change here due to change in tests/data/3-files-incomplete/push.sh:2-7
tests/data/3-files-incomplete/release.sh:2-6 - expected change here due to change in tests/data/3-files-incomplete/push.sh:2-7
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn five_files() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/5-files/change.diff")?;

    assert_eq!(
        run.stdout,
        "\
tests/data/5-files/push.sh:2-10 - expected change here due to change in tests/data/5-files/build.sh:2-10
tests/data/5-files/release-prod.sh:2-10 - expected change here due to change in tests/data/5-files/build.sh:2-10
tests/data/5-files/release-staging.sh:2-10 - expected change here due to change in tests/data/5-files/build.sh:2-10
tests/data/5-files/release-stress.sh:2-10 - expected change here due to change in tests/data/5-files/build.sh:2-10
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

// TODO- add test for one file with two ICTC blocks
// TODO- add test case for LFS diff
// TODO- add handling for // and -- comment delimiters, also leading spaces
// TODO- validate that diffs match the current state of the file
// TODO- add tests for malformed ictc blocks
// TODO- implement ordering for diagnostics
