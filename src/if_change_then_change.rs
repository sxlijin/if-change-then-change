use anyhow::anyhow;
use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BlockKey {
    pub path: String,
    pub block_name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct IfChangeThenChange {
    pub key: BlockKey,
    // TODO- make sure that this range refers to the _contents_ of the block, and excludes the block delimiters
    // i think (rn at least) that it makes sense to trigger an ictc check if and only if the contents have been
    // modified - if the then_change list has been modified, there's no need to trigger an ictc check (but we
    // should probably loop to determine whether or not the cross-file relation is well-formed)
    //
    // 0-indexed, excludes block delimiters
    pub content_range: Range<usize>,
    // TODO- we should also track line #s of if-change and then-change clauses
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
    pub fn from_str(path: &str, s: &str) -> HashMap<String, IfChangeThenChange> {
        let mut ret = HashMap::new();
        let mut curr: Option<IfChangeThenChange> = None;
        let mut errors: Vec<String> = Vec::new();

        for (i, line) in s.lines().enumerate() {
            //log::debug!("Parsing line {:?}", line);
            if line.starts_with("# if-change") {
                if let Some(ictc) = curr {
                    errors.push(
                        format!(
                            "invalid if-change block starting on {:?}",
                            ictc.content_range
                        )
                        .to_string(),
                    );
                }
                curr = Some(IfChangeThenChange {
                    key: BlockKey {
                        path: path.to_string(),
                        // TODO- implement block name parsing
                        block_name: "".to_string(),
                    },
                    content_range: i..i,
                    then_change: vec![],
                });
            } else if line.starts_with("# then-change") {
                if let Some(mut ictc) = curr {
                    let if_change_range = ictc.content_range;
                    ictc.content_range = if_change_range.start + 1..i;

                    // if this line is "then-change $filename"
                    if line.starts_with("# then-change ") {
                        // NB: little bit of a hack
                        ictc.then_change.push(BlockKey {
                            path: (&line["# then-change ".len()..]).to_string(),
                            // TODO- implement block name parsing
                            block_name: "".to_string(),
                        });
                        ret.insert(ictc.key.block_name.clone(), ictc);
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
                    Some(ictc) => {
                        ret.insert(ictc.key.block_name.clone(), ictc);
                    }
                    None => errors.push(
                        "fi-change found on line ??? but does not match a preceding then-change"
                            .to_string(),
                    ),
                }
                curr = None;
            } else {
                if let Some(ictc) = &mut curr {
                    if !ictc.content_range.is_empty() {
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
    use anyhow::anyhow;

    #[test]
    fn basic() -> anyhow::Result<()> {
        let parsed = IfChangeThenChange::from_str(
            "if-change.foo",
            "\
lorem
# if-change
ipsum
dolor
sit
# then-change then-change.foo
amet",
        );
        assert_eq!(parsed.len(), 1);
        assert_eq!(
            *parsed
                .get("")
                .ok_or(anyhow!("Did not parse an if-change-then-change block"))?,
            IfChangeThenChange {
                key: BlockKey {
                    path: "if-change.foo".to_string(),
                    block_name: "".to_string(),
                },
                content_range: 2..5,
                then_change: vec![BlockKey {
                    path: "then-change.foo".to_string(),
                    block_name: "".to_string(),
                }],
            }
        );

        Ok(())
    }

    use rangemap::RangeSet;

    #[test]
    fn rangemap_test() -> anyhow::Result<()> {
        let mut rangeset = RangeSet::<i32>::new();

        rangeset.insert(5..9);

        assert!(rangeset.contains(&5i32));
        assert!(!rangeset.contains(&9i32));
        assert!(rangeset.overlaps(&(8..8)));
        assert!(!rangeset.overlaps(&(9..9)));

        Ok(())
    }
}
