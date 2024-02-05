mod diagnostic;
mod if_change_then_change2;

use crate::diagnostic::{Diagnostic, DiagnosticPosition};
use anyhow::Result;
use if_change_then_change2::FileNodeParseError;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Read;
use std::ops::Range;

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
                        // TODO- $lines should reference the lines of the diff
                        start_line: None,
                        end_line: None,
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

    // To discover and parse all the if-change-then-change blocks relevant to this change, we do a
    // BFS starting from every path present in the diff, and then move on to every then-change
    // referenced in each file we read.
    let file_nodes_by_path = {
        let mut ret = HashMap::new();
        let mut search = diffs_by_post_diff_path
            .keys()
            .map(|path| {
                (
                    Diagnostic {
                        path: "stdin".to_string(),
                        // TODO- for files we're reading because they were in the diff,
                        //       start_line should be the line in the diff
                        start_line: None,
                        end_line: None,
                        // TODO- read_to_string can fail for other reasons (e.g.
                        // $path is a dir, $path does not allow reads)
                        message: format!("diff references file that does not exist: '{}'", path),
                    },
                    path.clone(),
                )
            })
            .collect::<VecDeque<(Diagnostic, String)>>();

        loop {
            let Some((diagnostic_if_read_fails, path)) = search.pop_front() else {
                break;
            };

            // $path entries come from one of two sources: either it is a path present in the input
            // diffs, or it is a then-change path in one of the former paths. In the first case,
            // this is where we do the file-exists validation; in the second case, we check
            // `Path::exists` before attempting to read the file here.
            let Ok(file_contents) = std::fs::read_to_string(&path) else {
                // TODO- in what cases does the post-diff path not exist?
                // TODO- if a file is deleted, the post-diff path is... /dev/null?
                diagnostics.push(diagnostic_if_read_fails);
                continue;
            };
            match if_change_then_change2::FileNode::from_str(&path, &file_contents) {
                Err(error) => {
                    diagnostics.extend(error.diagnostics);
                }
                Ok(mut file_node) => {
                    for block in file_node.blocks.iter_mut() {
                        block.then_change = block
                            .then_change
                            .drain(..)
                            .filter(|(then_change_lineno, then_change_key)| {
                                if diffs_by_post_diff_path.contains_key(&then_change_key.path) {
                                    return true;
                                }
                                if block.key.path == then_change_key.path {
                                    // We silently ignore self-referential then-change entries.
                                    return false;
                                }
                                if !std::path::Path::new(&then_change_key.path).exists() {
                                    diagnostics.push(Diagnostic {
                                        path: block.key.path.clone(),
                                        start_line: Some(*then_change_lineno),
                                        end_line: None,
                                        message: format!(
                                            "then-change references file that does not exist: '{}'",
                                            then_change_key.path
                                        ),
                                    });
                                    return false;
                                }
                                if !ret.contains_key(&then_change_key.path) {
                                    search.push_back((
                                        Diagnostic {
                                            path: block.key.path.clone(),
                                            start_line: Some(*then_change_lineno),
                                            end_line: None,
                                            message: format!(
                                                "then-change references file that could not be read: '{}'",
                                                then_change_key.path
                                            ),
                                        },
                                        then_change_key.path.clone(),
                                    ));
                                }
                                true
                            })
                            .collect();
                    }
                    ret.insert(path.clone(), file_node);
                }
            };
        }

        ret
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
    let modified_blocks_by_path = {
        let mut modified_blocks_by_path = HashMap::new();

        for (path, file_node) in file_nodes_by_path.iter() {
            let Some(&diff) = diffs_by_post_diff_path.get(path) else {
                continue;
            };

            let mut modified_blocks = Vec::new();

            for ictc_block in file_node.blocks.iter() {
                let mut intersects_any_hunk = false;
                for hunk in diff.hunks() {
                    // TODO- we can skip hunks with no intersection
                    let mut in_ictc_block = false;
                    for line in hunk.lines() {
                        // TODO- is this algo sound? are there ways that can break this approach w in_ictc_block?
                        if let Some(lineno) = line.target_line_no {
                            // target_line_no is 1-indexed
                            in_ictc_block = ictc_block.content_range().contains(&(lineno - 1));
                        }
                        if in_ictc_block && (line.is_added() || line.is_removed()) {
                            intersects_any_hunk = true;
                        }
                    }
                }
                if intersects_any_hunk {
                    modified_blocks.push(ictc_block.clone());
                }
            }

            if !modified_blocks.is_empty() {
                modified_blocks_by_path.insert(
                    path.clone(),
                    if_change_then_change2::FileNode::new(modified_blocks),
                );
            }
        }

        modified_blocks_by_path
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
    for ictc_block in modified_blocks_by_path
        .values()
        .flat_map(|file_node| file_node.blocks.iter())
    {
        for (_, then_change_key) in ictc_block.then_change.iter() {
            if let Some(then_change_file_node) = modified_blocks_by_path.get(&then_change_key.path)
            {
                if then_change_file_node
                    .get_corresponding_block(ictc_block)
                    .is_some()
                {
                    continue;
                }
            }

            let mut block_range = None;
            if let Some(ictc_blocks) = file_nodes_by_path.get(&then_change_key.path) {
                if let Some(ictc_block) = ictc_blocks.get_corresponding_block(&ictc_block) {
                    block_range = Some(ictc_block.content_range());
                }
            }
            if block_range.is_none() {
                diagnostics.push(Diagnostic {
                    path: then_change_key.path.clone(),
                    start_line: block_range.as_ref().map(|range| range.start),
                    end_line: block_range.as_ref().map(|range| range.end),
                    message: format!(
                        "expected an if-change-then-change in this file that matches {}",
                        DiagnosticPosition {
                            path: &ictc_block.key.path,
                            start_line: Some(ictc_block.content_range().start),
                            end_line: Some(ictc_block.content_range().end),
                        },
                    ),
                });
            }

            if block_range.is_some() || !diffs_by_post_diff_path.contains_key(&then_change_key.path)
            {
                diagnostics.push(Diagnostic {
                    path: then_change_key.path.clone(),
                    start_line: block_range.as_ref().map(|range| range.start),
                    end_line: block_range.as_ref().map(|range| range.end),
                    message: format!(
                        "expected change here due to change in {}",
                        DiagnosticPosition {
                            path: &ictc_block.key.path,
                            start_line: Some(ictc_block.content_range().start),
                            end_line: Some(ictc_block.content_range().end),
                        },
                    ),
                });
            }
        }
    }

    diagnostics.sort();

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
