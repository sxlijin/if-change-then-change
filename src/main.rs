use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Read;
use std::ops::Range;

mod if_change_then_change;

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
        let path_and_lineno = {
            if let Some(Range {
                start: start_line,
                end: end_line,
            }) = self.lines
            {
                // We _could_ just always show "a.sh:4-4" when the line range only consists of one line, but
                // "a.sh:4" is much more obvious at first glance; c.f. the GH permalink format.
                if start_line + 1 == end_line {
                    format!("{}:{}", self.path, start_line + 1)
                } else {
                    format!("{}:{}-{}", self.path, start_line + 1, end_line)
                }
            } else {
                self.path.clone()
            }
        };

        write!(f, "{} - {}", path_and_lineno, self.message)
    }
}

fn run() -> Result<()> {
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
        .map(|patched_file| {
            let post_diff_path = {
                if is_git_diff
                    && patched_file.source_file.starts_with("a/")
                    && patched_file.target_file.starts_with("b/")
                {
                    // To strip "a/" and "b/" when it's a "diff --git", we have to do this ourselves: unidiff's PatchedFile
                    // does not expose metadata about the type of unified diff (that is, --git vs not) and also uses the
                    // source file by default for `patched_file.path()`
                    patched_file.target_file[2..].to_string()
                } else {
                    patched_file.target_file.to_string()
                }
            };
            (post_diff_path, patched_file)
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

        // TODO- is doing 1 layer of BFS sufficient to discover all paths containing if-change-then-change blocks?
        let mut then_change_paths = Vec::new();

        for path in diffs_by_post_diff_path.keys() {
            let ictc_by_block_name = if_change_then_change::IfChangeThenChange::from_str(
                path,
                &std::fs::read_to_string(path)?,
            );
            for ictc in ictc_by_block_name.values() {
                for then_change_key in ictc.then_change.iter() {
                    if !diffs_by_post_diff_path.contains_key(&then_change_key.path) {
                        then_change_paths.push(then_change_key.path.clone());
                    }
                }
            }
            ictc_by_blockname_by_path.insert(path.clone(), ictc_by_block_name);
        }

        for path in then_change_paths {
            let ictc_by_block_name = if_change_then_change::IfChangeThenChange::from_str(
                &path,
                &std::fs::read_to_string(&path)?,
            );
            ictc_by_blockname_by_path.insert(path, ictc_by_block_name);
        }

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
    let diagnostics = {
        let mut diagnostics = Vec::new();

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

                if block_range.is_some()
                    || !diffs_by_post_diff_path.contains_key(&then_change_key.path)
                {
                    diagnostics.push(Diagnostic {
                        path: then_change_key.path.clone(),
                        lines: block_range,
                        message: format!(
                            "expected change here due to if-change in {}",
                            ictc_block.key.path
                        ),
                    });
                }
            }
        }

        diagnostics
    };

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
