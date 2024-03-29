use pretty_assertions::assert_eq;
use test_log::test;

mod framework;

#[test]
fn comment_formats_and_indentation_levels() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/formatting/if-change.diff")?;

    assert_eq!(run.stdout, "\
tests/data/formatting/if-change.foo:6 - then-change references file that does not exist: 'then-change-inline1.foo'
tests/data/formatting/if-change.foo:13 - then-change references file that does not exist: 'then-change-inline2.foo'
tests/data/formatting/if-change.foo:18 - then-change references file that does not exist: 'then-change-inline3.foo'
tests/data/formatting/if-change.foo:24 - then-change references file that does not exist: 'then-change-inline4.foo'
tests/data/formatting/if-change.foo:28 - then-change references file that does not exist: 'then-change-inline5.foo'
tests/data/formatting/if-change.foo:32 - then-change references file that does not exist: 'then-change-inline6.foo'
tests/data/formatting/if-change.foo:42 - then-change references file that does not exist: 'then-change-inline7.foo'
tests/data/formatting/if-change.foo:48 - then-change references file that does not exist: 'then-change-block1.foo'
tests/data/formatting/if-change.foo:55 - then-change references file that does not exist: 'then-change-block2a.foo'
tests/data/formatting/if-change.foo:56 - then-change references file that does not exist: 'then-change-block2b.foo'
tests/data/formatting/if-change.foo:57 - then-change references file that does not exist: 'then-change-block2c.foo'
tests/data/formatting/if-change.foo:66 - then-change references file that does not exist: 'then-change-block3.foo'
tests/data/formatting/if-change.foo:73 - then-change references file that does not exist: 'then-change-block4.foo'
tests/data/formatting/if-change.foo:79 - then-change references file that does not exist: 'then-change-block5.foo'
tests/data/formatting/if-change.foo:87 - then-change references file that does not exist: 'then-change-block6a.foo'
tests/data/formatting/if-change.foo:88 - then-change references file that does not exist: 'then-change-block6b.foo'
tests/data/formatting/if-change.foo:97 - then-change references file that does not exist: 'then-change-block7.foo'
");
    assert_eq!(run.exit_code, 0);

    Ok(())
}

#[test]
fn malformed_syntax() -> anyhow::Result<()> {
    let run = framework::run_tool("tests/data/malformed/if-change.diff")?;

    // TODO- if-change.foo should not be in this list
    assert_eq!(
        run.stdout,
        "\
tests/data/malformed/if-change-then-end-change.foo:2 - if-change must be closed by a then-change, but found no such then-change
tests/data/malformed/if-change-then-end-change.foo:5 - end-change must close an if-change and then-change, but found no then-change to close (found if-change on line 2)
tests/data/malformed/if-change-then-if-change.foo:2 - if-change must be closed by a then-change, but found no such then-change
tests/data/malformed/if-change-then-if-change.foo:4 - if-change may not be nested in another if-change
tests/data/malformed/nested-if-change.foo:2 - if-change must be closed by a then-change, but found no such then-change
tests/data/malformed/nested-if-change.foo:4 - if-change may not be nested in another if-change
tests/data/malformed/nested-if-change.foo:7 - then-change must close an if-change, but found no if-change to close
tests/data/malformed/orphaned-end-change.foo:4 - end-change must close an if-change and then-change, but found neither
tests/data/malformed/orphaned-then-change-block-terminated.foo:4 - then-change must close an if-change, but found no if-change to close
tests/data/malformed/orphaned-then-change-block-unterminated.foo:4 - then-change must be closed by an end-change, but found no such end-change
tests/data/malformed/orphaned-then-change-block-unterminated.foo:4 - then-change must close an if-change, but found no if-change to close
tests/data/malformed/orphaned-then-change-inline.foo:4 - then-change must close an if-change, but found no if-change to close
tests/data/malformed/then-change-into-if-change.foo:4 - then-change must be closed by an end-change, but found no such end-change
tests/data/malformed/then-change-into-if-change.foo:5 - if-change must be closed by a then-change, but found no such then-change
tests/data/malformed/then-change-into-if-change.foo:7 - end-change must close an if-change and then-change, but found no then-change to close (found if-change on line 5)
tests/data/malformed/then-change-into-invalid-paths.foo:4 - then-change references file that does not exist: 'then-change-nonexistent.foo'
tests/data/malformed/then-change-into-invalid-paths.foo:5 - then-change references file that does not exist: 'end-change-nonexistent.foo'
tests/data/malformed/then-change-into-invalid-paths.foo:6 - then-change references file that does not exist: 'empty string is invalid path'
tests/data/malformed/then-change-into-invalid-paths.foo:7 - then-change does not reference a valid path
tests/data/malformed/then-change-into-invalid-paths.foo:8 - then-change references file that does not exist: 'space-only is invalid path'
tests/data/malformed/then-change-into-invalid-paths.foo:9 - then-change does not reference a valid path
tests/data/malformed/then-change-into-invalid-paths.foo:10 - then-change references file that does not exist: 'tab-only is invalid path'
tests/data/malformed/then-change-into-invalid-paths.foo:11 - then-change does not reference a valid path
tests/data/malformed/then-change-into-invalid-paths.foo:12 - then-change references file that does not exist: 'punctuation is invalid path'
tests/data/malformed/then-change-into-invalid-paths.foo:13 - then-change does not reference a valid path
tests/data/malformed/then-change-into-then-change-block.foo:4 - then-change must be closed by an end-change, but found no such end-change
tests/data/malformed/then-change-into-then-change-block.foo:6 - then-change must close an if-change, but found no if-change to close
tests/data/malformed/then-change-into-then-change-inline.foo:4 - then-change must be closed by an end-change, but found no such end-change
tests/data/malformed/then-change-into-then-change-inline.foo:5 - then-change must close an if-change, but found no if-change to close
tests/data/malformed/unterminated-if-change.foo:2 - if-change must be closed by a then-change, but found no such then-change
tests/data/malformed/unterminated-then-change.foo:6 - then-change must be closed by an end-change, but found no such end-change
"
    );
    assert_eq!(run.exit_code, 0);

    Ok(())
}

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
fn renamed_file_no_changes() -> anyhow::Result<()> {
    let run =
        framework::run_tool("tests/data/diff-has-path-changes/g-renamed-file-no-changes.diff")?;

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
fn unidiff_test() -> anyhow::Result<()> {
    let diff =
        std::fs::read_to_string("tests/data/diff-has-path-changes/g-renamed-file-no-changes.diff")?;

    let mut patch_set = unidiff::PatchSet::new();
    patch_set.parse(diff).ok().expect("Error parsing diff");

    for patched_file in patch_set {
        // do something with patched_file
        log::info!("patched_file: {}", patched_file.source_file);
        log::info!("patched_file: {:?}", patched_file.source_timestamp);
        log::info!("patched_file: {}", patched_file.target_file);
        log::info!("patched_file: {:?}", patched_file.target_timestamp);
        for hunk in patched_file {
            // do something with hunk
            log::info!("hunk: {}", hunk.section_header);
            for line in hunk {
                // do something with line
                log::info!("line: {}", line.diff_line_no);
            }
        }
    }

    Err(anyhow::anyhow!("induce failure"))
}

#[test]
#[ignore]
fn renamed_file_with_changes() -> anyhow::Result<()> {
    let run =
        framework::run_tool("tests/data/diff-has-path-changes/g-renamed-file-with-changes.diff")?;

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
fn copied_file_no_changes() -> anyhow::Result<()> {
    let run =
        framework::run_tool("tests/data/diff-has-path-changes/h-copied-file-no-changes.diff")?;

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
#[ignore]
fn copied_file_with_changes() -> anyhow::Result<()> {
    let run =
        framework::run_tool("tests/data/diff-has-path-changes/h-copied-file-with-changes.diff")?;

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

// TODO- add test case for LFS diff
// TODO- validate that diffs match the current state of the file
// TODO- add malformed/then-change-into-invalid-paths handling
