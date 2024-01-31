use anyhow::Result;
use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct BlockKey {
    pub path: String,
    pub block_name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IfChangeThenChange {
    pub key: BlockKey,
    pub if_change: Range<usize>,
    pub then_change: Vec<BlockKey>,
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
    pub fn from_str(path: &str, s: &str) -> Vec<IfChangeThenChange> {
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
                    key: BlockKey {
                        path: path.to_string(),
                        // TODO- implement block name parsing
                        block_name: "".to_string(),
                    },
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
                        ictc.then_change.push(BlockKey {
                            path: (&line["# then-change ".len()..]).to_string(),
                            // TODO- implement block name parsing
                            block_name: "".to_string(),
                        });
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
                        ictc.then_change.push(BlockKey {
                            path: line.to_string(),
                            // TODO- implement block name parsing
                            block_name: "".to_string(),
                        });
                    }
                }
            }
        }

        // DO NOT LAND- throw if errors is non-empty
        ret
    }
}

mod test {
    use crate::if_change_then_change::{BlockKey, IfChangeThenChange};

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
                key: BlockKey {
                    path: "if-change.foo".to_string(),
                    block_name: "".to_string(),
                },
                if_change: 1..5,
                then_change: vec![BlockKey {
                    path: "then-change.foo".to_string(),
                    block_name: "".to_string(),
                }],
            }]
        );

        Ok(())
    }
}
