#![allow(redundant_semicolons)]

use crate::error::Error;
use crate::leaf::LeafId;
use crate::section_write::SectionWrite;
use crate::struct_macro::*;
use std::convert::*;

record! {
    /// Build information.
    [BuildInfo = 0x114C]
    /// Leaf index of a `Leaf::BuildInfo`.
    leaf: LeafId,
}

all_records! {
    /// A symbol record.
    #[derive(Debug, Clone)]
    pub enum Symbol<Reloc> {
        BuildInfo,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::section_write::*;

    fn test_symbol(symbol: Symbol<()>, data: &[u8]) {
        let mut sink = SectionSink::<()>::new();
        write(&symbol, &mut sink).unwrap();
        assert_eq!(sink.data, data);
    }

    #[test]
    fn symbols() {
        test_symbol(
            Symbol::BuildInfo(BuildInfo {
                leaf: LeafId(0x4455_6677),
            }),
            &[6, 0, 0x4C, 0x11, 0x77, 0x66, 0x55, 0x44],
        );
    }
}
