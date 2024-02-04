use std::ops::Range;

use derive_builder::Builder;

// Represents all if-change-then-change nodes found within a single file.
#[derive(Debug)]
pub struct FileNode {
    pub blocks: Vec<BlockNode>,
}

impl FileNode {
    pub fn new(blocks: Vec<BlockNode>) -> FileNode {
        FileNode { blocks: blocks }
    }

    pub fn from_str(path: &str, s: &str) -> Result<FileNode, Vec<usize>> {
        enum ParseState {
            NoOp,
            IfChange(BlockNodeBuilder),
            ThenChange(BlockNodeBuilder),
        }

        enum LineType {
            NotComment,
            Comment,
            IfChange,
            ThenChangeInline,
            ThenChangeBlockStart,
            EndChangeAkaThenChangeBlockEnd,
        }

        let mut ret = Vec::new();
        let mut errors: Vec<usize> = Vec::new();
        let mut parse_state = ParseState::NoOp;

        for (i, line) in s.lines().enumerate() {
            let line_type = if line == "# if-change" {
                LineType::IfChange
            } else if line == "# then-change" {
                LineType::ThenChangeBlockStart
            } else if line.starts_with("# then-change") {
                LineType::ThenChangeInline
            } else if line == "# end-change" {
                LineType::EndChangeAkaThenChangeBlockEnd
            } else if line.starts_with("#") {
                LineType::Comment
            } else {
                LineType::NotComment
            };

            match parse_state {
                ParseState::NoOp => match line_type {
                    LineType::NotComment | LineType::Comment => {}
                    LineType::IfChange => {
                        let mut builder = BlockNodeBuilder::default();
                        builder.if_change_lineno(i);

                        parse_state = ParseState::IfChange(builder);
                    }
                    LineType::ThenChangeInline => {
                        errors.push(i);
                    }
                    LineType::ThenChangeBlockStart => {
                        errors.push(i);
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        errors.push(i);
                    }
                },
                ParseState::IfChange(ref mut builder) => match line_type {
                    LineType::NotComment | LineType::Comment => {
                        // do nothing
                    }
                    LineType::IfChange => {
                        errors.push(i);
                    }
                    LineType::ThenChangeInline => {
                        builder.then_change_push((i, line.to_string()));

                        match builder.build() {
                            Ok(block_node) => ret.push(block_node),
                            Err(_) => errors.push(i),
                        }

                        parse_state = ParseState::NoOp;
                    }
                    LineType::ThenChangeBlockStart => {
                        parse_state = ParseState::ThenChange(builder.clone());
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        errors.push(i);
                    }
                },
                ParseState::ThenChange(ref mut builder) => match line_type {
                    LineType::NotComment => {
                        errors.push(i);
                        parse_state = ParseState::NoOp;
                    }
                    LineType::Comment => {
                        builder.then_change_push((i, line.to_string()));
                    }
                    LineType::IfChange => {
                        errors.push(i);
                    }
                    LineType::ThenChangeInline => {
                        errors.push(i);
                    }
                    LineType::ThenChangeBlockStart => {
                        errors.push(i);
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        match builder.build() {
                            Ok(block_node) => ret.push(block_node),
                            Err(_) => errors.push(i),
                        }

                        parse_state = ParseState::NoOp;
                    }
                },
            }
        }

        match parse_state {
            ParseState::NoOp => {}
            _ => {
                errors.push(s.lines().count());
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        log::debug!("errors are empty");

        Ok(FileNode::new(ret))

        /*
        enum ParseState {
            NOOP
            IF_CHANGE
            THEN_CHANGE
        }

        enum LineType {
            NOT_COMMENT
            COMMENT
            IF_CHANGE
            THEN_CHANGE_INLINE
            THEN_CHANGE_BLOCK_START
            END_CHANGE / THEN_CHANGE_BLOCK_END
        }

        for i, line in file {
          line_type = determine_line_type(line)

          match parse_state {
            NOOP => {
                match line_type {
                    NOT_COMMENT => do nothing
                    COMMENT => do nothing
                    IF_CHANGE => {
                        current_ictc_block = starts here
                        parse_state = IF_CHANGE
                    }
                    THEN_CHANGE_INLINE => {
                        record error
                    }
                    THEN_CHANGE_BLOCK_START => {
                        record error
                    }
                    END_CHANGE_AKA_THEN_CHANGE_BLOCK_END => {
                        record error
                    }
                }
            }

            IF_CHANGE => {
                match line_type {
                    NOT_COMMENT => do nothing
                    COMMENT => do nothing
                    IF_CHANGE => {
                        record error
                    }
                    THEN_CHANGE_INLINE => {
                        terminate current ictc block
                        parse_state = NOOP
                    }
                    THEN_CHANGE_BLOCK_START => {
                        switch current ictc block to accumulate thenchange
                        parse_state = THEN_CHANGE
                    }
                    END_CHANGE_AKA_THEN_CHANGE_BLOCK_END => {
                        record error
                    }
                }
            }

            THEN_CHANGE => {
                match line_type {
                    NOT_COMMENT => {
                        record error
                    }
                    COMMENT => {
                        add current line to current ictc then-change
                    }
                    IF_CHANGE => {
                        record error
                    }
                    THEN_CHANGE_INLINE => {
                        record error
                    }
                    THEN_CHANGE_BLOCK_START => {
                        record error
                    }
                    END_CHANGE_AKA_THEN_CHANGE_BLOCK_END => {
                        terminate current ictc block
                        parse_state = NOOP
                    }
                }
            }
          }
        }

        if parse_state != NOOP {
            record error - we should've terminated
        }
         */
    }
}

#[derive(Builder, Clone, Debug, PartialEq, Eq)]
pub struct BlockNode {
    // content_range is if_change_lineno to end of then_change_linenos
    if_change_lineno: usize,
    // pairs of lineno, then_change_path
    #[builder(setter(each(name = "then_change_push")))]
    then_change: Vec<(usize, String)>,
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

#[cfg(test)]
mod test {
    use crate::if_change_then_change2::*;
    use anyhow::anyhow;
    use spectral::prelude::*;
    use test_log::test;

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
        )
        .unwrap();
        assert_that!(parsed.blocks).has_length(1);

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
        )
        .unwrap();
        assert_that!(parsed.blocks).has_length(1);

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
        )
        .unwrap();
        assert_that!(parsed.blocks).has_length(1);

        Ok(())
    }

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
        log::error!("if change not closed");
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
}
