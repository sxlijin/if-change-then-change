use std::ops::Range;

// Represents all if-change-then-change nodes found within a single file.
#[derive(Debug)]
pub struct FileNode {
    pub blocks: Vec<BlockNode>,
}

impl FileNode {
    pub fn new(blocks: Vec<BlockNode>) -> FileNode {
        FileNode { blocks: blocks }
    }

    pub fn get_corresponding_block(&self, src_block: &BlockNode) -> Option<&BlockNode> {
        // Linear search is fast enough for our purposes. It's very unlikely that a file will
        // have enough ICTC blocks for linear search to be slow (working around this would
        // require indexing the ICTC blocks, which is hard in Rust because that means
        // self-referential structs).
        for dst_block in self.blocks.iter() {
            if dst_block.then_change.contains(&src_block.key) {
                return Some(&dst_block);
            }
        }
        None
    }

    // TODO: errors should hand back meaningful diagnostics about the malformed ictc
    pub fn from_str(path: &str, s: &str) -> anyhow::Result<FileNode> {
        let mut ret = Vec::new();
        let mut curr: Option<BlockNode> = None;
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
                curr = Some(BlockNode {
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
            } else if line.starts_with("# end-change") {
                match curr {
                    Some(ictc) => {
                        ret.push(ictc);
                    }
                    None => errors.push(
                        "end-change found on line ??? but does not match a preceding then-change"
                            .to_string(),
                    ),
                }
                curr = None;
            } else {
                if let Some(ictc) = &mut curr {
                    if !ictc.content_range.is_empty() {
                        ictc.then_change.push(BlockKey {
                            // TODO- this needs to be more robust
                            path: line["# ".len()..].trim_start_matches(" ").to_string(),
                            // TODO- implement block name parsing
                            block_name: "".to_string(),
                        });
                    }
                }
            }
        }

        // DO NOT LAND- throw if errors is non-empty
        Ok(FileNode::new(ret))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BlockKey {
    pub path: String,
    pub block_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockNode {
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

impl BlockNode {}

#[cfg(test)]
mod test {
    use crate::if_change_then_change::*;
    use anyhow::anyhow;
    use spectral::prelude::*;

    #[test]
    fn then_change_one_file() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
lorem
# if-change
ipsum
dolor
sit
# then-change then-change.foo
amet
",
        )?;
        assert_that!(parsed.blocks).has_length(1);
        assert_that!(parsed.blocks).is_equal_to(vec![BlockNode {
            key: BlockKey {
                path: "if-change.foo".to_string(),
                block_name: "".to_string(),
            },
            content_range: 2..5,
            then_change: vec![BlockKey {
                path: "then-change.foo".to_string(),
                block_name: "".to_string(),
            }],
        }]);

        Ok(())
    }

    #[test]
    fn then_change_two_files() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
lorem
# if-change
ipsum
dolor
sit
# then-change
#   then-change1.foo
#   then-change2.foo
# end-change
amet",
        )?;
        assert_that!(parsed.blocks).has_length(1);
        assert_that!(parsed.blocks).is_equal_to(vec![BlockNode {
            key: BlockKey {
                path: "if-change.foo".to_string(),
                block_name: "".to_string(),
            },
            content_range: 2..5,
            then_change: vec![
                BlockKey {
                    path: "then-change1.foo".to_string(),
                    block_name: "".to_string(),
                },
                BlockKey {
                    path: "then-change2.foo".to_string(),
                    block_name: "".to_string(),
                },
            ],
        }]);

        Ok(())
    }

    #[test]
    fn then_change_two_files_and_self() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
lorem
# if-change
ipsum
dolor
sit
# then-change
#   if-change.foo
#   then-change1.foo
#   then-change2.foo
# end-change
amet",
        )?;
        assert_that!(parsed.blocks).has_length(1);
        assert_that!(parsed.blocks).is_equal_to(vec![BlockNode {
            key: BlockKey {
                path: "if-change.foo".to_string(),
                block_name: "".to_string(),
            },
            content_range: 2..5,
            then_change: vec![
                BlockKey {
                    path: "if-change.foo".to_string(),
                    block_name: "".to_string(),
                },
                BlockKey {
                    path: "then-change1.foo".to_string(),
                    block_name: "".to_string(),
                },
                BlockKey {
                    path: "then-change2.foo".to_string(),
                    block_name: "".to_string(),
                },
            ],
        }]);

        Ok(())
    }

    //     #[test]
    //     fn handles_all_indentation_levels() -> anyhow::Result<()> {
    //         let parsed = BlockNode::from_str(
    //             "if-change.foo",
    //             "\
    // lorem
    // # if-change
    // ipsum
    // dolor
    // sit
    // # then-change then-change1.foo
    // amet

    //     # if-change
    //     consectetur
    //     adipiscing
    //     elit
    //     # then-change then-change2.foo

    // // IDK if I like allowing mismatched indentation levels to match up
    // // with each other, but this is easier to implement than asserting
    // // that comment formats must match (plus, I don't see the value in
    // // adding handling for mismatches)
    //         # if-change
    //     sed
    //     do
    //     eiusmod
    //     tempor
    //     incididunt
    //     # then-change then-change3.foo
    // ut
    // labore
    // ",
    //         );
    //         assert_that!(parsed).has_length(1);
    //         assert_that!(parsed).contains_entry(
    //             "".to_string(),
    //             BlockNode {
    //                 key: BlockKey {
    //                     path: "if-change.foo".to_string(),
    //                     block_name: "".to_string(),
    //                 },
    //                 content_range: 2..5,
    //                 then_change: vec![BlockKey {
    //                     path: "then-change.foo".to_string(),
    //                     block_name: "".to_string(),
    //                 }],
    //             },
    //         );

    //         Ok(())
    //     }

    //     #[test]
    //     fn handles_all_comment_formats() -> anyhow::Result<()> {
    //         let parsed = BlockNode::from_str(
    //             "if-change.foo",
    //             "\
    // lorem
    // # if-change
    // ipsum
    // dolor
    // sit
    // # then-change then-change1.foo
    // amet

    // // if-change
    // consectetur
    // adipiscing
    // elit
    // // then-change then-change2.foo

    // sed
    // do
    // -- if-change
    // eiusmod
    // tempor
    // -- then-change then-change3.foo
    // incididunt
    // ut

    // // IDK if I like allowing mismatched comment formats to line up
    // // with each other, but this is easier to implement than asserting
    // // that comment formats must match (plus, I don't see the value in
    // // adding handling for mismatches)
    // -- if-change
    // labore
    // et
    // dolore
    // magna
    // aliqua
    // // then-change then-change4.foo
    // ",
    //         );
    //         assert_that!(parsed).has_length(1);
    //         assert_that!(parsed).contains_entry(
    //             "".to_string(),
    //             BlockNode {
    //                 key: BlockKey {
    //                     path: "if-change.foo".to_string(),
    //                     block_name: "".to_string(),
    //                 },
    //                 content_range: 2..5,
    //                 then_change: vec![BlockKey {
    //                     path: "then-change.foo".to_string(),
    //                     block_name: "".to_string(),
    //                 }],
    //             },
    //         );

    //         Ok(())
    //     }

    #[test]
    fn error_when_then_change_not_closed() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
lorem
# if-change
ipsum
dolor
sit
# then-change
#   then-change.foo
amet
then-change is not closed
",
        );
        assert_that!(parsed).is_err();

        Ok(())
    }

    #[test]
    fn error_when_if_change_not_closed() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
lorem
# if-change
ipsum
# if-change
dolor
sit
# then-change then-change.foo
amet
then-change is not closed
",
        );
        assert_that!(parsed).is_err();

        Ok(())
    }

    #[test]
    fn rangemap_test() -> anyhow::Result<()> {
        use rangemap::RangeSet;

        let mut rangeset = RangeSet::<i32>::new();

        rangeset.insert(5..9);

        assert!(rangeset.contains(&5i32));
        assert!(!rangeset.contains(&9i32));
        assert!(rangeset.overlaps(&(8..8)));
        assert!(!rangeset.overlaps(&(9..9)));

        Ok(())
    }
}
