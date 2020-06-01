#![allow(redundant_semicolons)]

use crate::error::Error;
use crate::section_write::SectionWrite;
use crate::struct_macro::*;
use std::convert::*;

/// An identifier for a leaf record.
#[derive(Debug, Clone)]
pub struct LeafId(pub(crate) u32);

impl LeafId {
    pub fn null() -> LeafId {
        LeafId(0)
    }
}

writable_transparent!(LeafId);

record! {
    /// Build information.
    [BuildInfo = 0x1603]
    ///
    (len(args)): u16,
    /// Arguments for build information. Point to `Leaf::StringId`.
    args: Vec<LeafId>,
}

impl BuildInfo {
    /// Create a new `BuildInfo` instance with common argument convention.
    pub fn new(
        current_dir: LeafId,
        build_tool: LeafId,
        source_file: LeafId,
        program_database_file: LeafId,
        command_args: LeafId,
    ) -> BuildInfo {
        BuildInfo {
            args: vec![
                current_dir,
                build_tool,
                source_file,
                program_database_file,
                command_args,
            ],
        }
    }
}

record! {
    /// Substring list.
    [SubstrList = 0x1604]
    ///
    (len(strings)): u32,
    /// Substrings. Point to `Leaf::StringId`
    strings: Vec<LeafId>,
}

record! {
    /// String ID.
    [StringId = 0x1605]
    /// Substring list. Point to `Leaf::SubstrList`.
    substr: LeafId,
    /// String content.
    content: String,
}

all_records! {
    /// A record ("leaf") in the CodeView type section.
    #[derive(Debug, Clone)]
    pub enum Leaf {
        BuildInfo,
        SubstrList,
        StringId,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::section_write::*;

    fn test_leaf(leaf: Leaf, data: &[u8]) {
        let mut sink = SectionSink::<()>::new();
        write(&leaf, &mut sink).unwrap();
        assert_eq!(sink.data, data);
    }

    #[test]
    fn leaves() {
        test_leaf(
            Leaf::BuildInfo(BuildInfo {
                args: vec![LeafId(1), LeafId(0x22), LeafId(0x3344)],
            }),
            &[
                16, 0, 0x03, 0x16, 3, 0, 1, 0, 0, 0, 0x22, 0, 0, 0, 0x44, 0x33, 0, 0,
            ],
        );

        test_leaf(
            Leaf::SubstrList(SubstrList {
                strings: vec![LeafId(1), LeafId(0x22), LeafId(0x3344)],
            }),
            &[
                18, 0, 0x04, 0x16, 3, 0, 0, 0, 1, 0, 0, 0, 0x22, 0, 0, 0, 0x44, 0x33, 0, 0,
            ],
        );

        test_leaf(
            Leaf::StringId(StringId {
                substr: LeafId(0x1122_3344),
                content: "hello".to_string(),
            }),
            &[
                12, 0, 0x05, 0x16, 0x44, 0x33, 0x22, 0x11, b'h', b'e', b'l', b'l', b'o', 0,
            ],
        );
    }
}
