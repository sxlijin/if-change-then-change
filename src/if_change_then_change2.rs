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

    pub fn from_str(path: &str, s: &str) -> anyhow::Result<FileNode> {
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
            let line_type = LineType::NotComment;

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
    // interior contents of after-if-change to before-then-change
    interior_range: Range<usize>,
    // if-change-lineno thru end-change
    block_range: Range<usize>,
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
