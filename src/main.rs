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
                // We _could_ just always do a.sh:4-4 when the line range only consists of one line, but
                // a.sh:4 is much more readable; GH permalinks are a good example of the prior art here
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
    let patch_set = {
        let mut input = String::new();

        std::io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read stdin");

        let mut patch_set = unidiff::PatchSet::new();
        patch_set.parse(input).ok().expect("Error parsing diff");

        patch_set
    };

    let diffs_by_post_diff_path = patch_set
        .files()
        .iter()
        .inspect(|patched_file| {
            for hunk in patched_file.hunks() {
                let hunk_lines = (hunk.lines()
                    .into_iter()
                    .map(|line| format!("{:?}/{}/{}", line.target_line_no, line.line_type, line.value).to_string())
                    .collect::<Vec<String>>()
                    .join("\n")).to_string();
                let target_lines = (hunk.target_lines()
                    .into_iter()
                    .map(|line| format!("{}/{}", line.line_type, line.value).to_string())
                    .collect::<Vec<String>>()
                    .join("\n")).to_string();
                // is target_length in bytes or lines? i think it's lines, but need to check
                log::debug!(
                    "patched_file {}->{} target_start={} target_length={}\n\ntarget_lines:\n{}\n\nhunk_lines:\n{}",
                    patched_file.source_file,
                    patched_file.target_file,
                    hunk.target_start,
                    hunk.target_length,
                    hunk_lines,
                    target_lines,
                );
            }
        })
        .map(|patched_file| (patched_file.target_file.replace("b/", ""), patched_file))
        .collect::<HashMap<String, &unidiff::PatchedFile>>();

    let ictc_by_block_name_by_path = {
        let mut ictc_by_block_name_by_path = HashMap::new();

        // TODO- do we need to go deeper than this?
        // BFS 1 layer deep: we shouldn't have to go further, in theory (although we may have to)

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
            ictc_by_block_name_by_path.insert(path.clone(), ictc_by_block_name);
        }

        for path in then_change_paths {
            let ictc_by_block_name = if_change_then_change::IfChangeThenChange::from_str(
                &path,
                &std::fs::read_to_string(&path)?,
            );
            ictc_by_block_name_by_path.insert(path, ictc_by_block_name);
        }

        ictc_by_block_name_by_path
    };

    let modified_blocks_by_key = {
        let mut modified_blocks_by_key = HashSet::new();

        for ictc_by_block_name in ictc_by_block_name_by_path.values() {
            for ictc_block in ictc_by_block_name.values() {
                let Some(&target_diff) = diffs_by_post_diff_path.get(&ictc_block.key.path) else {
                    continue;
                };

                for hunk in target_diff.hunks() {
                    // TODO- we can skip hunks with no intersection
                    let mut in_ictc_block = false;
                    for line in hunk.lines() {
                        // TODO- is this 1-indexed or 0-indexed lineno? might be 1-indexed
                        // TODO- is this algo sound? are there ways that can break this approach w in_ictc_block?
                        if let Some(lineno) = line.target_line_no {
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

    // for every ictc-block
    //   find all intersecting patch hunks
    //   for each intersecting patch hunk
    //     check if the intersection contains added/removed lines at all
    //     (unclear if this should only iterate thru intersection or if it should iterate thru entire hunk)
    //     if there are added/removed in the ictc-block
    //       mark the block as "modified"
    // do not mark the block as "modified"

    let mut diagnostics = Vec::new();

    for (_, ictc_by_block_name) in ictc_by_block_name_by_path.iter() {
        for (_, ictc_block) in ictc_by_block_name {
            if modified_blocks_by_key.contains(&ictc_block.key) {
                for then_change_key in ictc_block.then_change.iter() {
                    if modified_blocks_by_key.contains(then_change_key) {
                        continue;
                    }

                    let mut block_range = None;
                    if let Some(ictc_blocks) = ictc_by_block_name_by_path.get(&then_change_key.path)
                    {
                        if let Some(ictc_block) = ictc_blocks.get(&then_change_key.block_name) {
                            block_range = Some(ictc_block.content_range.clone());
                        }
                    }
                    if block_range == None {
                        diagnostics.push(Diagnostic {
                            path: then_change_key.path.clone(),
                            lines: None,
                            message: format!(
                                "expected if-change-then-change in this file due to if-change in {}",
                                ictc_block.key.path
                            ),
                        });
                    }
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
    }

    // for every ictc-block
    //   if the ifchange block is in the "modified block" set
    //     for every thenchange block
    //       if the thenchange block exists in the "modified block" set
    //         do nothing
    //       else
    //         add diagnostic

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
