use crate::diagnostic::Diagnostic;
use std::fmt;
use std::ops::Range;

use derive_builder::Builder;

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

struct Parser<'a> {
    input_path: &'a str,
    input_content: &'a str,

    block_nodes: Vec<BlockNode>,
    errors: Vec<Diagnostic>,
    parse_state: ParseState,
}

impl<'a> Parser<'a> {
    fn new(path: &'a str, s: &'a str) -> Parser<'a> {
        Parser {
            input_path: path,
            input_content: s,
            block_nodes: Vec::new(),
            errors: Vec::new(),
            parse_state: ParseState::NoOp,
        }
    }

    fn record_error(&mut self, lineno: usize, message: &str) {
        self.errors.push(Diagnostic {
            path: self.input_path.to_string(),
            lines: Some(lineno..lineno + 1),
            message: message.to_string(),
        })
    }

    fn parse(mut self) -> Result<Vec<BlockNode>, Vec<Diagnostic>> {
        for (i, line) in self.input_content.lines().enumerate() {
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

            match self.parse_state {
                ParseState::NoOp => match line_type {
                    LineType::NotComment | LineType::Comment => {}
                    LineType::IfChange => {
                        let mut builder = BlockNodeBuilder::default();
                        builder.if_change_lineno(i);

                        self.parse_state = ParseState::IfChange(builder);
                    }
                    LineType::ThenChangeInline => {
                        self.record_error(i, "");
                    }
                    LineType::ThenChangeBlockStart => {
                        self.record_error(i, "");
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        self.record_error(i, "");
                    }
                },
                ParseState::IfChange(ref mut builder) => match line_type {
                    LineType::NotComment | LineType::Comment => {
                        // do nothing
                    }
                    LineType::IfChange => {
                        self.record_error(i, "");
                    }
                    LineType::ThenChangeInline => {
                        builder.then_change_push((i, line.to_string()));

                        match builder.build() {
                            Ok(block_node) => self.block_nodes.push(block_node),
                            Err(_) => self.record_error(
                                i,
                                "internal error: failed to parse if-change-then-change",
                            ),
                        }

                        self.parse_state = ParseState::NoOp;
                    }
                    LineType::ThenChangeBlockStart => {
                        self.parse_state = ParseState::ThenChange(builder.clone());
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        self.record_error(i, "");
                    }
                },
                ParseState::ThenChange(ref mut builder) => match line_type {
                    LineType::NotComment => {
                        self.record_error(i, "");
                        self.parse_state = ParseState::NoOp;
                    }
                    LineType::Comment => {
                        builder.then_change_push((i, line.to_string()));
                    }
                    LineType::IfChange => {
                        self.record_error(i, "");
                    }
                    LineType::ThenChangeInline => {
                        self.record_error(i, "");
                    }
                    LineType::ThenChangeBlockStart => {
                        self.record_error(i, "");
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        match builder.build() {
                            Ok(block_node) => self.block_nodes.push(block_node),
                            Err(_) => self.record_error(
                                i,
                                "internal error: failed to parse if-change-then-change",
                            ),
                        }

                        self.parse_state = ParseState::NoOp;
                    }
                },
            }
        }

        match self.parse_state {
            ParseState::NoOp => {}
            _ => {
                self.record_error(self.input_content.lines().count(), "");
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        log::debug!("errors are empty");

        Ok(self.block_nodes)
    }

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

#[derive(Debug)]
pub struct FileNodeParseError {
    pub diagnostics: Vec<Diagnostic>,
}

impl fmt::Display for FileNodeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in self.diagnostics.iter() {
            write!(f, "{}\n", diagnostic)?
        }

        fmt::Result::Ok(())
    }
}

impl std::error::Error for FileNodeParseError {}

// Represents all if-change-then-change nodes found within a single file.
#[derive(Debug)]
pub struct FileNode {
    pub blocks: Vec<BlockNode>,
}

impl FileNode {
    pub fn new(blocks: Vec<BlockNode>) -> FileNode {
        FileNode { blocks: blocks }
    }

    pub fn from_str(path: &str, s: &str) -> Result<FileNode, FileNodeParseError> {
        match Parser::new(path, s).parse() {
            Ok(block_nodes) => Ok(FileNode::new(block_nodes)),
            Err(errors) => Err(FileNodeParseError {
                diagnostics: errors,
            }),
        }
    }
}

#[derive(Builder, Clone, Debug, PartialEq, Eq)]
pub struct BlockNode {
    // content_range is if_change_lineno to end of then_change_linenos
    if_change_lineno: usize,
    // pairs of lineno, then_change_path
    #[builder(setter(each(name = "then_change_push")))]
    then_change: Vec<(usize, String)>,
    //then_change_lineno: usize,
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
        )?;
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
        )?;
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
        )?;
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
