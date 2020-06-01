use crate::error::{wu, Error};
use crate::leaf::{self, Leaf, LeafId};
use crate::section_write::SectionWrite;
use crate::subsection::Subsection;
use crate::symbol;
use std::convert::*;

/// CodeView information for an object.
///
/// `Reloc` can be any type that carries relocation symbol information.
#[derive(Debug, Default)]
pub struct Codeview<Reloc> {
    leafs: Vec<Leaf>,
    subsections: Vec<Subsection<Reloc>>,
}

impl<Reloc> Codeview<Reloc> {
    /// Create a new `Codeview` instance.
    pub fn new() -> Codeview<Reloc> {
        Codeview {
            leafs: vec![],
            subsections: vec![],
        }
    }

    pub fn add_leaf(&mut self, leaf: Leaf) -> LeafId {
        self.leafs.push(leaf);
        LeafId(
            (self.leafs.len() - 1 + 0x1000)
                .try_into()
                .expect("Too many leaves"),
        )
    }

    pub fn add_subsection(&mut self, subsection: Subsection<Reloc>) {
        self.subsections.push(subsection);
    }

    /// Write CodeView information to object sections.
    pub fn write<W, F>(&self, mut writer_factory: F) -> Result<(), Error<W::Error>>
    where
        W: SectionWrite<Reloc>,
        F: FnMut(&str) -> W,
    {
        let mut type_section = writer_factory(".debug$T");
        wu(type_section.write(&4u32.to_le_bytes()))?;

        for leaf in &self.leafs {
            leaf::write(&leaf, &mut type_section)?;
        }

        drop(type_section);

        let mut symbol_section = writer_factory(".debug$S");
        wu(symbol_section.write(&4u32.to_le_bytes()))?;

        for subsection in &self.subsections {
            match subsection {
                Subsection::Symbols(symbols) => {
                    let len = symbols.iter().map(symbol::size::<Reloc>).sum::<usize>();
                    wu(write_subsection_header(
                        &mut symbol_section,
                        0xF1,
                        len.try_into()?,
                    ))?;
                    for symbol in symbols {
                        symbol::write(&symbol, &mut symbol_section)?;
                    }
                    wu(symbol_section.write(&[0; 3][0..(4 - len % 4) % 4]))?;
                }
                _ => unimplemented!(),
            }
        }

        Ok(())
    }
}

fn write_subsection_header<Reloc, W: SectionWrite<Reloc>>(
    writer: &mut W,
    subsection_type: u32,
    len: u32,
) -> Result<(), W::Error> {
    writer.write(&subsection_type.to_le_bytes())?;
    writer.write(&len.to_le_bytes())?;
    Ok(())
}
