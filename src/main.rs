use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::ops::Range;

mod if_change_then_change;

// Diagnostics should always be tied to the location where we want the user to
// make a change, i.e. if a.sh contains a "if change ... then change b.sh", a.sh
// has been changed but b.sh has not, then the diagnostic should be tied to b.sh.
struct Diagnostic {
    path: String,
    // 0-indexed, inclusive
    start_line: i32,
    // 0-indexed, exclusive
    end_line: i32,
    message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{} - {}",
            self.path, self.start_line, self.end_line, self.message
        )
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
        .map(|patched_file| (&patched_file.target_file, patched_file))
        .collect::<HashMap<&String, &unidiff::PatchedFile>>();

    let mut ictc_blocks_by_path = HashMap::new();

    for post_diff_path in diffs_by_post_diff_path.keys() {
        let actual_path = post_diff_path.replace("b/", "");
        ictc_blocks_by_path.insert(
            actual_path.clone(),
            if_change_then_change::IfChangeThenChange::from_str(
                &actual_path,
                &std::fs::read_to_string(&actual_path)?,
            ),
        );
    }

    let ictc_blocks_by_path = ictc_blocks_by_path;

    let mut diagnostics = Vec::new();

    for (ifchange_path, ictc_blocks) in ictc_blocks_by_path {
        for (_, ictc_block) in ictc_blocks {
            for then_change_key in ictc_block.then_change {
                // DO NOT LAND- need to actually compute intersection of diff and ictc blocks
                diagnostics.push(Diagnostic {
                    path: then_change_key.path,
                    start_line: 0,
                    end_line: 0,
                    message: format!("expected change here due to if-change in {}", ifchange_path),
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
