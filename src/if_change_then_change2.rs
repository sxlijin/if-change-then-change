use crate::diagnostic::Diagnostic;
use std::fmt;
use std::ops::Range;

use derive_builder::Builder;

enum ParseState {
    NoOp,
    // if-change records the line number where we switched to if-change parsing
    IfChange(usize, BlockNodeBuilder),
    // then-change records the line number where we switched to then-change parsing
    ThenChange(usize, BlockNodeBuilder),
}

enum LineType<'a> {
    // We can't distinguish between "Comment" and "NotComment" source code lines because we support
    // using block comments for if-change-then-change directives; see Parser::from_str
    SourceCode,
    IfChange,
    ThenChangeInline(&'a str),
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
            start_line: Some(lineno),
            end_line: None,
            message: message.to_string(),
        })
    }

    fn is_comment_prefix(s: &str) -> bool {
        s.chars()
            .all(|ch| ch.is_ascii_punctuation() || ch.is_ascii_whitespace())
    }

    fn line_type(line: &'a str) -> LineType<'a> {
        if let Some((pre, post)) = line.split_once("if-change") {
            if Parser::is_comment_prefix(pre)
                && post
                    .trim_end_matches(|ch: char| {
                        ch.is_ascii_punctuation() || ch.is_ascii_whitespace()
                    })
                    .chars()
                    .nth(0)
                    .map_or(true, |ch| ch.is_ascii_whitespace())
            {
                return LineType::IfChange;
            }
        }

        if let Some((pre, post)) = line.split_once("then-change") {
            if Parser::is_comment_prefix(pre) {
                let post = post.trim_end_matches(|ch: char| {
                    ch.is_ascii_punctuation() || ch.is_ascii_whitespace()
                });
                if post.is_empty() {
                    return LineType::ThenChangeBlockStart;
                }
                if post
                    .chars()
                    .nth(0)
                    .map_or(true, |ch| ch.is_ascii_whitespace())
                {
                    return LineType::ThenChangeInline(post.trim_start());
                }
            }
        }

        if let Some((pre, post)) = line.split_once("end-change") {
            if Parser::is_comment_prefix(pre) {
                return LineType::EndChangeAkaThenChangeBlockEnd;
            }
        }

        return LineType::SourceCode;
    }

    /// Parsing follows these principles:
    ///
    ///   - If the syntax looks plausible to a human, then the parser must either consider it
    ///     to be well-formed or produce an error.
    ///
    ///   - Errors must be comprehensible and actionable.
    ///
    ///   - Users should have to repeat the "run tool, identify syntax error, fix syntax error"
    ///     as few times as possible.
    ///
    ///   - We should be able to handle as many languages as possible, without having to
    ///     explicitly add support for their comment formats. (Supporting something like Brainfuck,
    ///     though, is very much a non-goal.)
    ///
    /// This has a number of implications:
    ///
    ///   - Parsing must be highly flexible, so that the parser can recognize anything that
    ///     is plausible syntax.
    ///
    ///   - As many errors should be shown at a time as possible.
    ///
    ///   - Line numbers should point to lines that make the user's syntax error obvious.
    ///
    ///     For example, if there is an unterminated then-change on line 5 of a 10-line file, the
    ///     diagnostic we return to the user should point to line 5, rather than point to EOF.
    ///     Doing the latter is easy, since upon reaching EOF you can just complain that "reached
    ///     EOF, but never terminated the then-change" - and many compilers/parsers _are_
    ///     implemented this way, because they consider error quality to be secondary to their
    ///     functionality.
    ///
    /// Based on these principles, parsing is implemented as follows:
    ///
    ///   - the parser is a line-by-line state machine
    ///   - lines are both a (1) parser state transition and (2) parser input
    ///   - to maximize error readability, lines are classified into types of input _independently_
    ///     of the current state of the state machine, and each input type must then be explicitly
    ///     handled according to the current state of the state machine
    ///
    /// ## Edge case handling
    ///
    /// 1. we can't eagerly recognize when a then-change has not been correctly closed:
    ///
    ///     ```c
    ///     // if-change
    ///     some code here
    ///     // then-change
    ///     //   then-change.foo
    ///     more code here
    ///     ```
    ///
    ///     In theory, while parsing a then-change block, we can detect when a line is not a
    ///     comment, then terminate the block there and complain to the user that the then-change
    ///     is not terminated.
    ///
    ///     However, we also allow using block comments for then-change blocks:
    ///
    ///     ```html
    ///     <!-- if-change -->
    ///     some code here
    ///     <!-- then-change -->
    ///     <!--   then-change1.foo -->
    ///     <!-- end-change -->
    ///
    ///     <!-- if-change -->
    ///     some code here
    ///     <!-- then-change
    ///             then-change2.foo
    ///          end-change -->
    ///
    ///     <!-- if-change -->
    ///     some code here
    ///     <!--
    ///         then-change
    ///             then-change3.foo
    ///         end-change
    ///     -->
    ///     ```
    ///
    ///     and because we use somewhat crude logic for identifying comments (1- we do our parsing
    ///     line-by-line, not token-by-token, and 2- we use is_ascii_punctuation and
    ///     is_ascii_whitespace to do a best-effort guess as to whether or not a token is a comment)
    ///     we can't actually recognize when the next entry in a then-change block is actually
    ///     another then-change path or just a line of code.
    ///
    /// 2. trailing punctuation in a filename is not supported
    ///
    ///     This is parsed as "then-change foo.bar", not "then-change foo.bar*":
    ///
    ///     ```c
    ///     /* if-change*/
    ///     some code here
    ///     /* then-change foo.bar**/
    ///     ```
    ///
    ///     Similarly, this is parsed as "then-change foo.bar", not "then-change foo.bar---":
    ///
    ///     ```html
    ///     <!-- if-change -->
    ///     more code here
    ///     <!--then-change foo.bar----->
    ///     ```
    ///
    ///     We do this to support maximally permissive block comment formats without having to
    ///     hardcode support for individual comment formats.
    ///     
    fn parse(mut self) -> Result<Vec<BlockNode>, Vec<Diagnostic>> {
        for (i, line) in self.input_content.lines().enumerate() {
            let line_type = Self::line_type(line);
            match self.parse_state {
                ParseState::NoOp => match line_type {
                    LineType::SourceCode => {}
                    LineType::IfChange => {
                        let mut builder = BlockNodeBuilder::default();
                        builder.key(BlockKey::new(self.input_path));
                        builder.if_change_lineno(i);

                        self.parse_state = ParseState::IfChange(i, builder);
                    }
                    LineType::ThenChangeInline(_) => {
                        self.record_error(i, "then-change must follow an if-change");
                    }
                    LineType::ThenChangeBlockStart => {
                        self.record_error(i, "then-change must follow an if-change");
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        self.record_error(i, "end-change must follow an if-change and then-change");
                    }
                },
                ParseState::IfChange(_, ref mut builder) => match line_type {
                    LineType::SourceCode => {}
                    LineType::IfChange => {
                        self.record_error(i, "if-change nesting is not allowed");
                    }
                    LineType::ThenChangeInline(then_change_path) => {
                        builder.then_change_push((i, BlockKey::new(then_change_path)));
                        builder.then_change_lineno(i);
                        builder.end_change_lineno(i);

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
                        self.parse_state =
                            ParseState::ThenChange(i, builder.then_change_lineno(i).clone());
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        self.record_error(i, "end-change must follow an if-change and then-change");
                    }
                },
                ParseState::ThenChange(_, ref mut builder) => match line_type {
                    LineType::SourceCode => {
                        builder.then_change_push((
                            i,
                            BlockKey::new(line.trim_matches(|ch: char| {
                                ch.is_ascii_punctuation() || ch.is_ascii_whitespace()
                            })),
                        ));
                    }
                    LineType::IfChange => {
                        self.record_error(i, "end-change must follow an if-change and then-change");
                    }
                    LineType::ThenChangeInline(_) => {
                        self.record_error(i, "end-change must follow an if-change and then-change");
                    }
                    LineType::ThenChangeBlockStart => {
                        self.record_error(i, "end-change must follow an if-change and then-change");
                    }
                    LineType::EndChangeAkaThenChangeBlockEnd => {
                        builder.end_change_lineno(i);

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
            ParseState::IfChange(i, _) => {
                self.record_error(i, "then-change must follow an if-change");
            }
            ParseState::ThenChange(i, _) => {
                self.record_error(i, "end-change must follow an if-change and then-change");
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

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
            write!(f, "{}\n", diagnostic)?;
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

    pub fn get_corresponding_block(&self, src_block: &BlockNode) -> Option<&BlockNode> {
        // Linear search is fast enough for our purposes. It's very unlikely that a file will
        // have enough ICTC blocks for linear search to be slow (working around this would
        // require indexing the ICTC blocks, which is hard in Rust because that means
        // self-referential structs).
        for dst_block in self.blocks.iter() {
            for (_, then_change_key) in dst_block.then_change.iter() {
                if then_change_key == &src_block.key {
                    return Some(&dst_block);
                }
            }
        }
        None
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BlockKey {
    pub path: String,
}

impl BlockKey {
    fn new(path: &str) -> BlockKey {
        BlockKey {
            path: path.to_string(),
        }
    }
}

#[derive(Builder, Clone, Debug, PartialEq, Eq)]
pub struct BlockNode {
    // BlockNode keys are NOT required to be unique per BlockNode.
    // We allow using the then-change paths to resolve a BlockNode; that is,
    // if foo.sh contains an "if-change ... then-change bar1.sh" on L4-6 and
    // "if-change ... then-change bar2.sh" on L8-10, when searching for the
    // BlockNode corresponding to bar2.sh, we can use "then-change bar2.sh"
    // to resolve to the second BlockNode.
    pub key: BlockKey,

    // pairs of (lineno, then_change_block)
    #[builder(setter(each(name = "then_change_push")))]
    pub then_change: Vec<(usize, BlockKey)>,

    // content_range is if_change_lineno to end_change_lineno + 1
    if_change_lineno: usize,
    then_change_lineno: usize,
    end_change_lineno: usize,
}

impl BlockNode {
    // The line range which we expect to see a modification in.
    //
    // It's important that this encompasses the delimiting if-change and then-change
    // directives, because that allows us to handle changes to those clauses correctly,
    // e.g. when we're wrapping existing code in if-change-then-change clauses.
    pub fn content_range(&self) -> Range<usize> {
        self.if_change_lineno..self.end_change_lineno + 1
    }
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
// end-change

#[cfg(test)]
mod test {
    use crate::if_change_then_change2::*;
    use anyhow::anyhow;
    use spectral::prelude::*;
    use test_log::test;

    #[test]
    fn then_change_well_formed() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
0 lorem
# if-change
# if-change-should-not-be-considered
3 ipsum dolor
4 sit
# then-change then-change.foo
6 amet

8 consectetur
# if-change
10 adipiscing
11 elit
# then-change
#   then-change1.foo
#   then-change2.foo
# end-change

# if-change
18 sed
19 do
20 eiusmod
# then-change
#   if-change.foo
#   then-change3.foo
#   then-change4.foo
# end-change
26 tempor
27 incididunt
",
        )?;
        assert_that!(parsed.blocks).has_length(3);
        assert_that!(parsed.blocks[0]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(5, BlockKey::new("then-change.foo"))],
            if_change_lineno: 1,
            then_change_lineno: 5,
            end_change_lineno: 5,
        });
        assert_that!(parsed.blocks[1]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![
                (13, BlockKey::new("then-change1.foo")),
                (14, BlockKey::new("then-change2.foo")),
            ],
            if_change_lineno: 9,
            then_change_lineno: 12,
            end_change_lineno: 15,
        });
        assert_that!(parsed.blocks[2]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![
                // AST node should not strip self-referential paths;
                // that happens in a higher-level context
                (22, BlockKey::new("if-change.foo")),
                (23, BlockKey::new("then-change3.foo")),
                (24, BlockKey::new("then-change4.foo")),
            ],
            if_change_lineno: 17,
            then_change_lineno: 21,
            end_change_lineno: 25,
        });

        Ok(())
    }

    #[test]
    fn handles_all_indentation_levels() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
 lorem
 # if-change
 ipsum
 dolor
 sit
 # then-change then-change1.foo
 amet

     # if-change
     consectetur
     # then-change then-change2.foo
     adipiscing

     # if-change
     elit
     # then-change
     #   then-change3.foo
     # end-change

     # if-change
     sed
     do
     # then-change
     # then-change4a.foo
     #       then-change4b.foo
     # end-change

 // IDK if I like allowing mismatched indentation levels to match up
 // with each other, but this is easier to implement than asserting
 // that comment formats must match (plus, I don't see the value in
 // adding handling for mismatches)
         # if-change
     eiusmod
     tempor
     incididunt
     # then-change then-change5.foo
 ut
 labore
 ",
        )?;
        assert_that!(parsed.blocks).has_length(5);
        assert_that!(parsed.blocks[0]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(5, BlockKey::new("then-change1.foo"))],
            if_change_lineno: 1,
            then_change_lineno: 5,
            end_change_lineno: 5,
        });
        assert_that!(parsed.blocks[1]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(10, BlockKey::new("then-change2.foo"))],
            if_change_lineno: 8,
            then_change_lineno: 10,
            end_change_lineno: 10,
        });
        assert_that!(parsed.blocks[2]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(16, BlockKey::new("then-change3.foo"))],
            if_change_lineno: 13,
            then_change_lineno: 15,
            end_change_lineno: 17,
        });
        assert_that!(parsed.blocks[3]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![
                (23, BlockKey::new("then-change4a.foo")),
                (24, BlockKey::new("then-change4b.foo")),
            ],
            if_change_lineno: 19,
            then_change_lineno: 22,
            end_change_lineno: 25,
        });
        assert_that!(parsed.blocks[4]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(35, BlockKey::new("then-change5.foo"))],
            if_change_lineno: 31,
            then_change_lineno: 35,
            end_change_lineno: 35,
        });

        Ok(())
    }

    #[test]
    fn handles_all_comment_formats_thenchange_inline() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
 lorem
 # if-change
 ipsum
 dolor
 sit
 # then-change then-change1.foo
 amet

 // if-change
 consectetur
 adipiscing
 elit
 // then-change then-change2.foo

 sed
 -- if-change
 do
 -- then-change then-change3.foo
 eiusmod

 /* if-change */
 tempor
 incididunt
 /* then-change then-change4.foo */

 <!-- if-change -->
 ut
 <!-- then-change then-change5.foo -->
 <!-- if-change-->
 labore
 et
 <!-- then-change then-change6.foo-->

 // IDK if I like allowing mismatched comment formats to line up
 // with each other, but this is easier to implement than asserting
 // that comment formats must match (plus, I don't see the value in
 // adding handling for mismatches)
 -- if-change
 dolore
 magna
 aliqua
 // then-change then-change7.foo
 ",
        )?;
        assert_that!(parsed.blocks).has_length(7);
        assert_that!(parsed.blocks[0]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(5, BlockKey::new("then-change1.foo"))],
            if_change_lineno: 1,
            then_change_lineno: 5,
            end_change_lineno: 5,
        });
        assert_that!(parsed.blocks[1]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(12, BlockKey::new("then-change2.foo"))],
            if_change_lineno: 8,
            then_change_lineno: 12,
            end_change_lineno: 12,
        });
        assert_that!(parsed.blocks[2]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(17, BlockKey::new("then-change3.foo"))],
            if_change_lineno: 15,
            then_change_lineno: 17,
            end_change_lineno: 17,
        });
        assert_that!(parsed.blocks[3]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(23, BlockKey::new("then-change4.foo"))],
            if_change_lineno: 20,
            then_change_lineno: 23,
            end_change_lineno: 23,
        });
        assert_that!(parsed.blocks[4]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(27, BlockKey::new("then-change5.foo"))],
            if_change_lineno: 25,
            then_change_lineno: 27,
            end_change_lineno: 27,
        });
        assert_that!(parsed.blocks[5]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(31, BlockKey::new("then-change6.foo"))],
            if_change_lineno: 28,
            then_change_lineno: 31,
            end_change_lineno: 31,
        });
        assert_that!(parsed.blocks[6]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(41, BlockKey::new("then-change7.foo"))],
            if_change_lineno: 37,
            then_change_lineno: 41,
            end_change_lineno: 41,
        });

        Ok(())
    }

    #[test]
    fn handles_all_comment_formats_thenchange_block() -> anyhow::Result<()> {
        let parsed = FileNode::from_str(
            "if-change.foo",
            "\
 # if-change
 lorem
 ipsum
 # then-change
 #   then-change1.foo
 # end-change

 dolor
 // if-change
 sit
 // then-change
 //   then-change2a.foo
 //   then-change2b.foo
 //   then-change2c.foo
 // end-change
 amet

/* if-change */
 consectetur
 adipiscing
 elit
 /* then-change */
 /*   then-change3.foo */
 /* end-change */

 sed
 <!-- if-change -->
 do
 <!-- then-change -->
 <!--   then-change4.foo -->
 <!-- end-change -->

 <!-- if-change -->
 no whitespace required after then-change or the then-change-path
 <!-- then-change-->
 <!--   then-change5.foo-->
 <!-- end-change-->

 <!-- if-change -->
 tempor
 incididunt
 <!--
        then-change
    then-change6a.foo
            then-change6b.foo
        end-change
 -->

 <!-- if-change -->
 ut
 labore
 et
 <!-- then-change
    then-change7.foo
        end-change -->
 ",
        )?;
        assert_that!(parsed.blocks).has_length(7);
        assert_that!(parsed.blocks[0]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(4, BlockKey::new("then-change1.foo"))],
            if_change_lineno: 0,
            then_change_lineno: 3,
            end_change_lineno: 5,
        });
        assert_that!(parsed.blocks[1]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![
                (11, BlockKey::new("then-change2a.foo")),
                (12, BlockKey::new("then-change2b.foo")),
                (13, BlockKey::new("then-change2c.foo")),
            ],
            if_change_lineno: 8,
            then_change_lineno: 10,
            end_change_lineno: 14,
        });
        assert_that!(parsed.blocks[2]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(22, BlockKey::new("then-change3.foo"))],
            if_change_lineno: 17,
            then_change_lineno: 21,
            end_change_lineno: 23,
        });
        assert_that!(parsed.blocks[3]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(29, BlockKey::new("then-change4.foo"))],
            if_change_lineno: 26,
            then_change_lineno: 28,
            end_change_lineno: 30,
        });
        assert_that!(parsed.blocks[4]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(35, BlockKey::new("then-change5.foo"))],
            if_change_lineno: 32,
            then_change_lineno: 34,
            end_change_lineno: 36,
        });
        assert_that!(parsed.blocks[5]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![
                (43, BlockKey::new("then-change6a.foo")),
                (44, BlockKey::new("then-change6b.foo")),
            ],
            if_change_lineno: 38,
            then_change_lineno: 42,
            end_change_lineno: 45,
        });
        assert_that!(parsed.blocks[6]).is_equal_to(BlockNode {
            key: BlockKey::new("if-change.foo"),
            then_change: vec![(53, BlockKey::new("then-change7.foo"))],
            if_change_lineno: 48,
            then_change_lineno: 52,
            end_change_lineno: 54,
        });

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
then-change-above is not closed
",
        );
        assert_that!(parsed).is_err();
        assert_that!(parsed.unwrap_err().to_string().as_str())
            .is_equal_to("if-change.foo:6 - end-change must follow an if-change and then-change\n");

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
then-change-above is not closed
",
        );
        assert_that!(parsed).is_err();
        assert_that!(parsed.unwrap_err().to_string().as_str())
            .is_equal_to("if-change.foo:4 - if-change nesting is not allowed\n");

        Ok(())
    }
}
