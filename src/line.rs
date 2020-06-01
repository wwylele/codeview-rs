use crate::file::FileId;

/// A subsection that records line number information.
#[derive(Debug, Clone)]
pub struct Lines<Reloc> {
    /// Code address.
    pub address: Reloc,

    /// Blocks of lines.
    pub blocks: Vec<Block>,
    // TODO: flags
}

/// A group of line number information sourced from the same file.
#[derive(Debug, Clone)]
pub struct Block {
    /// Source file ID.
    pub file: FileId,

    /// Line numbers.
    pub lines: Vec<Line>,
}

/// A line number entry.
#[derive(Debug, Clone)]
pub struct Line {
    /// Offset relative to the address specified in `Lines::address`.
    pub offset: u32,

    /// Line where the statement/expression starts.
    pub line_start: u32,

    /// Delta to line where statement/expression ends.
    pub line_delta: Option<u32>,

    /// Whether the line is a statement or an expression.
    pub is_statement: bool,
}
