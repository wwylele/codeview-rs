use crate::line::Lines;
use crate::symbol::Symbol;

/// A subsection of the CodeView symbol section.
#[derive(Debug, Clone)]
pub enum Subsection<Reloc> {
    /// A subsection containing symbol records,
    Symbols(Vec<Symbol<Reloc>>),

    /// A subsection containing line records,
    Lines(Lines<Reloc>),
}
