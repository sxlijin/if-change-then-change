use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Read;
use std::ops::Range;

mod if_change_then_change;

struct DiagnosticPosition<'a> {
    path: &'a String,
    // 0-indexed, inclusive-exclusive
    lines: Option<&'a Range<usize>>,
}

impl<'a> fmt::Display for DiagnosticPosition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(Range {
            start: start_line,
            end: end_line,
        }) = self.lines
        {
            // We _could_ just always show "a.sh:4-4" when the line range only consists of one line, but
            // "a.sh:4" is much more obvious at first glance; c.f. the GH permalink format.
            if start_line + 1 == *end_line {
                write!(f, "{}:{}", self.path, start_line + 1)
            } else {
                write!(f, "{}:{}-{}", self.path, start_line + 1, end_line)
            }
        } else {
            write!(f, "{}", self.path)
        }
    }
}

// Diagnostics should always be tied to the location where we want the user to
// make a change, i.e. if a.sh contains a "if change ... then change b.sh", a.sh
// has been changed but b.sh has not, then the diagnostic should be tied to b.sh.
struct Diagnostic {
    path: String,
    // 0-indexed, inclusive-exclusive
    lines: Option<Range<usize>>,
    message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}",
            DiagnosticPosition {
                path: &self.path,
                lines: self.lines.as_ref()
            },
            self.message
        )
    }
}

fn run() -> Result<()> {
    let mut diagnostics = Vec::new();

    let (patch_set, is_git_diff) = {
        let mut input = String::new();

        std::io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read stdin");

        let is_git_diff = input.starts_with("diff --git");

        let mut patch_set = unidiff::PatchSet::new();
        patch_set.parse(input).ok().expect("Error parsing diff");

        (patch_set, is_git_diff)
    };

    // We want to key this map by the path at HEAD corresponding to a given diff
    let diffs_by_post_diff_path = patch_set
        .files()
        .iter()
        .inspect(|patched_file| {
            log::info!("patched file in diff: {}", patched_file.target_file);
        })
        .filter_map(|patched_file| {
            if is_git_diff {
                let source_path_valid = patched_file.source_file.starts_with("a/") || patched_file.source_file == "/dev/null";
                let target_path_valid = patched_file.target_file.starts_with("b/") || patched_file.target_file == "/dev/null";

                // Do some light git diff validation. There are only two cases where the source file and target file are not
                // prefixed with "a/" and "b/" respectively: when a file has been added (source file is /dev/null) and when
                // a file has been deleted (target file is /dev/null).
                if !source_path_valid || !target_path_valid {
                    diagnostics.push(Diagnostic {
                        path: "stdin".to_string(),
                        // TODO- $lines should reference the line where the thenchange comes from
                        lines: None,
                        message: format!(
                            "invalid git diff: expected a/before.path -> b/after.path, but got '{}' -> '{}'",
                            patched_file.source_file,
                            patched_file.target_file,
                        ),
                    });
                    return None;
                }

                if patched_file.target_file.starts_with("b/") {
                    // In a "diff --git", the pre-diff and post-diff paths are prefixed with "a/" and "b/". We have
                    // to strip these prefixes ourselves, because unidiff::PatchedFile does not expose metadata about
                    // whether or not it represents a "diff --git" or normal diff. (PatchedFile.path() does do some
                    // stripping here, but it uses the source file and is poorly implemented.)
                    Some((patched_file.target_file[2..].to_string(), patched_file))
                } else {
                    // We don't index deleted files in diffs_by_post_diff_path, because we can't read a deleted file
                    // (after we build this hashmap, the next thing we do is parse if-change-then-change blocks out
                    // of all files changed in the diff).
                    None
                }
            } else {
                if patched_file.target_file == "/dev/null" {
                    return None;
                }

                Some((patched_file.target_file.clone(), patched_file))
            }
        })
        .collect::<HashMap<String, &unidiff::PatchedFile>>();

    // To discover and parse all the if-change-then-change blocks relevant to this change, we do a two-step search:
    //
    //   1. find all if-change-then-change blocks in the paths present in the diff
    //   2. for every ictc block, also parse every path in a then-change block
    //
    // i.e. we do a BFS 1 layer deep. This is sufficient for well-formed paths, but unclear if it's sufficient for more complex forms.
    let ictc_by_blockname_by_path = {
        let mut ictc_by_blockname_by_path = HashMap::new();

        for path in diffs_by_post_diff_path.keys() {
            let Ok(file_contents) = std::fs::read_to_string(path) else {
                diagnostics.push(Diagnostic {
                    // TODO- in what cases does the post-diff path not exist?
                    // TODO- if a file is deleted, the post-diff path is... /dev/null?
                    path: "stdin".to_string(),
                    // TODO- $lines should reference the line where the thenchange comes from
                    lines: None,
                    message: format!("diff references file that does not exist: '{}'", path),
                });
                continue;
            };
            let ictc_by_block_name =
                if_change_then_change::IfChangeThenChange::from_str(path, &file_contents);
            ictc_by_blockname_by_path.insert(path.clone(), ictc_by_block_name);
        }

        // TODO- is doing 1 layer of BFS sufficient to discover all paths containing if-change-then-change blocks?
        let mut more_ictcs = Vec::new();
        for ictc_by_blockname in ictc_by_blockname_by_path.values_mut() {
            for ictc in ictc_by_blockname.values_mut() {
                ictc.then_change = ictc
                    .then_change
                    .drain(..)
                    .filter(|then_change_key| {
                        if diffs_by_post_diff_path.contains_key(&then_change_key.path) {
                            return true;
                        }
                        let Ok(file_contents) = std::fs::read_to_string(&then_change_key.path)
                        else {
                            diagnostics.push(Diagnostic {
                                path: ictc.key.path.clone(),
                                // TODO- $lines should reference the line where the thenchange comes from
                                lines: Some(ictc.content_range.clone()),
                                message: format!(
                                    "then-change references file that does not exist: '{}'",
                                    then_change_key.path
                                ),
                            });
                            return false;
                        };
                        let ictc_by_block_name =
                            if_change_then_change::IfChangeThenChange::from_str(
                                &then_change_key.path,
                                &file_contents,
                            );
                        more_ictcs.push((then_change_key.path.clone(), ictc_by_block_name));
                        true
                    })
                    .collect();
            }
        }
        ictc_by_blockname_by_path.extend(more_ictcs);

        ictc_by_blockname_by_path
    };

    // Before we can generate diagnostics, we also need to know, for each
    // if-change-then-change block, whether or not its contents were modified.
    //
    // for every ictc-block
    //   find all intersecting patch hunks
    //   for each intersecting patch hunk
    //     check if the intersection in the ictc-block contains added/removed lines in the hunk
    //     (hunks have both added/removed lines and also context lines)
    //     if so, mark the block as "modified"
    let modified_blocks_by_key = {
        let mut modified_blocks_by_key = HashSet::new();

        for ictc_by_block_name in ictc_by_blockname_by_path.values() {
            for ictc_block in ictc_by_block_name.values() {
                let Some(&target_diff) = diffs_by_post_diff_path.get(&ictc_block.key.path) else {
                    continue;
                };

                for hunk in target_diff.hunks() {
                    // TODO- we can skip hunks with no intersection
                    let mut in_ictc_block = false;
                    for line in hunk.lines() {
                        // TODO- is this algo sound? are there ways that can break this approach w in_ictc_block?
                        if let Some(lineno) = line.target_line_no {
                            // target_line_no is 1-indexed
                            in_ictc_block = ictc_block.content_range.contains(&(lineno - 1));
                        }
                        if in_ictc_block && (line.is_added() || line.is_removed()) {
                            modified_blocks_by_key.insert(ictc_block.key.clone());
                        }
                    }
                }
            }
        }

        modified_blocks_by_key
    };

    // Now that we know which if-change-then-change blocks have and have not been modified in the
    // current diff, we can actually build diagnostics
    //
    // for every ictc-block
    //   if the ifchange block is in the "modified block" set
    //     for every thenchange block
    //       if the thenchange block exists in the "modified block" set
    //         do nothing
    //       else
    //         add diagnostic
    for ictc_block in ictc_by_blockname_by_path
        .values()
        .flat_map(|ictc_by_block_name| ictc_by_block_name.values())
    {
        if !modified_blocks_by_key.contains(&ictc_block.key) {
            continue;
        }

        for then_change_key in ictc_block.then_change.iter() {
            if modified_blocks_by_key.contains(then_change_key) {
                continue;
            }

            let mut block_range = None;
            if let Some(ictc_blocks) = ictc_by_blockname_by_path.get(&then_change_key.path) {
                if let Some(ictc_block) = ictc_blocks.get(&then_change_key.block_name) {
                    block_range = Some(ictc_block.content_range.clone());
                }
            }
            if block_range.is_none() {
                diagnostics.push(Diagnostic {
                    path: then_change_key.path.clone(),
                    lines: None,
                    message: format!(
                        "expected if-change-then-change in this file due to if-change in {}",
                        ictc_block.key.path
                    ),
                });
            }

            if block_range.is_some() || !diffs_by_post_diff_path.contains_key(&then_change_key.path)
            {
                diagnostics.push(Diagnostic {
                    path: then_change_key.path.clone(),
                    lines: block_range,
                    message: format!(
                        "expected change here due to change in {}",
                        DiagnosticPosition {
                            path: &ictc_block.key.path,
                            lines: Some(&ictc_block.content_range),
                        },
                    ),
                });
            }
        }
    }

    for diagnostic in diagnostics {
        println!("{}", diagnostic);
    }

    Ok(())
}

fn main() {
    env_logger::init();

    log::info!("Starting to-be-named");

    match run() {
        Ok(_) => (),
        Err(err) => {
            log::error!("{}", err);
            std::process::exit(1);
        }
    }
}
