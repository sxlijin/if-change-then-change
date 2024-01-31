use anyhow::Result;
use std::collections::HashMap;
use std::io::Read;
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
struct IfChangeThenChange {
    path: String,
    block_name: String,
    if_change: Range<usize>,
    then_change: Vec<String>,
}

// single-file format
// ---
// if-change
// lorem ipsum dolor
// sit amet
// then-change a/b/c.rs

// multi-file format
// ---
// if-change
// lorem ipsum dolor
// sit amet
// then-change
//   a/b/c.rs
//   a/b/c2.rs
//   a/b/c3.rs
// fi-change

impl IfChangeThenChange {
    // TODO: errors should hand back meaningful diagnostics about the malformed ictc
    fn from_str(path: &str, s: &str) -> Vec<IfChangeThenChange> {
        let mut ret: Vec<IfChangeThenChange> = Vec::new();
        let mut curr: Option<IfChangeThenChange> = None;
        let mut errors: Vec<String> = Vec::new();

        for (i, line) in s.lines().enumerate() {
            log::debug!("Parsing line {:?}", line);
            if line.starts_with("# if-change") {
                if let Some(ictc) = curr {
                    errors.push(
                        format!("invalid if-change block starting on {:?}", ictc.if_change)
                            .to_string(),
                    );
                }
                curr = Some(IfChangeThenChange {
                    path: path.to_string(),
                    // DO NOT LAND- block name is not being parsed out right now
                    block_name: "".to_string(),
                    if_change: i..i,
                    then_change: vec![],
                });
            } else if line.starts_with("# then-change") {
                if let Some(mut ictc) = curr {
                    let if_change_range = ictc.if_change;
                    ictc.if_change = if_change_range.start..i;

                    // if this line is "then-change $filename"
                    if line.starts_with("# then-change ") {
                        // NB: little bit of a hack
                        ictc.then_change
                            .push((&line["# then-change ".len()..]).to_string());
                        ret.push(ictc);
                        curr = None;
                    } else {
                        curr = Some(ictc);
                    }
                    // otherwise, we expect the next lines until fi-change to all
                    // be in the then_change set
                } else {
                    errors.push("no if-change preceding this then-change".to_string());
                }
            } else if line.starts_with("# fi-change") {
                match curr {
                    Some(ictc) => ret.push(ictc),
                    None => errors.push(
                        "fi-change found on line ??? but does not match a preceding then-change"
                            .to_string(),
                    ),
                }
                curr = None;
            } else {
                if let Some(ictc) = &mut curr {
                    if !ictc.if_change.is_empty() {
                        ictc.then_change.push(line.to_string());
                    }
                }
            }
        }

        // DO NOT LAND- throw if errors is non-empty
        ret
    }
}

mod test {
    use crate::IfChangeThenChange;

    //#[test]
    fn basic() -> anyhow::Result<()> {
        let parsed = IfChangeThenChange::from_str(
            "if-change.foo",
            "\
lorem
// if-change
ipsum
dolor
sit
// then-change then-change.foo
amet",
        );
        assert_eq!(
            parsed,
            vec![IfChangeThenChange {
                path: "if-change.foo".to_string(),
                block_name: "".to_string(),
                if_change: 1..5,
                then_change: vec!["then-change.foo".to_string()],
            }]
        );

        Ok(())
    }
}

fn run() -> Result<()> {
    // Create a mutable String to store the user input
    let mut input = String::new();

    // Read a line from stdin and store it in the 'input' String
    std::io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read line");

    log::debug!("stdin: {}", input);


    let mut patch = unidiff::PatchSet::new();
    patch.parse(input).ok().expect("Error parsing diff");

    let mut diffs_by_post_diff_path = HashMap::new();

    for patched_file in patch.files() {
        log::debug!("patched file {}", patched_file.path());
        log::debug!(
            "diff says {} -> {}",
            patched_file.source_file, patched_file.target_file
        );
        diffs_by_post_diff_path.insert(patched_file.target_file.clone(), patched_file);
    }

    let mut ictc_blocks_by_path = HashMap::new();

    for post_diff_path in diffs_by_post_diff_path.keys() {
        let actual_path = post_diff_path.replace("b/", "");
        ictc_blocks_by_path.insert(
            actual_path.clone(),
            IfChangeThenChange::from_str(&actual_path, &std::fs::read_to_string(&actual_path)?),
        );
    }

    let ictc_blocks_by_path = ictc_blocks_by_path;

    let mut diagnostics = Vec::new();

    for (ifchange_path, ictc_blocks ) in ictc_blocks_by_path {
        for ictc_block in ictc_blocks {
            for then_change_path in ictc_block.then_change {
                // DO NOT LAND- need to actually compute intersection of diff and ictc blocks
                diagnostics.push(format!("{}:{} - expected change in this file, b/c there was a change in {}", 
                    then_change_path, 0, ifchange_path))
            }
        }
    }

    for diagnostic in diagnostics {
        println!("diagnostic: {}", diagnostic);
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