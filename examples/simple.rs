use codeview::*;
use std::cell::*;
use std::fs::File;
use std::io::Write;

extern crate object;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let obj = RefCell::new(object::write::Object::new(
        object::BinaryFormat::Coff,
        object::Architecture::X86_64,
        object::Endianness::Little,
    ));
    let mut cv = Codeview::<String>::new();

    let leaf_current_dir = cv.add_leaf(Leaf::StringId(leaf::StringId {
        substr: LeafId::null(),
        content: "D:\\test".to_string(),
    }));

    let leaf_build_tool = cv.add_leaf(Leaf::StringId(leaf::StringId {
        substr: LeafId::null(),
        content: "cl.exe".to_string(),
    }));

    let leaf_source_file = cv.add_leaf(Leaf::StringId(leaf::StringId {
        substr: LeafId::null(),
        content: "main.cpp".to_string(),
    }));

    let leaf_program_database_file = cv.add_leaf(Leaf::StringId(leaf::StringId {
        substr: LeafId::null(),
        content: "D:\\test\\vc140.pdb".to_string(),
    }));

    let leaf_command_args_sub = cv.add_leaf(Leaf::StringId(leaf::StringId {
        substr: LeafId::null(),
        content: "-Z7".to_string(),
    }));

    let leaf_command_args_sub_list = cv.add_leaf(Leaf::SubstrList(leaf::SubstrList {
        strings: vec![leaf_command_args_sub],
    }));

    let leaf_command_args = cv.add_leaf(Leaf::StringId(leaf::StringId {
        substr: leaf_command_args_sub_list,
        content: "-I\"a\" -I\"b\"".to_string(),
    }));

    let leaf_build_info = cv.add_leaf(Leaf::BuildInfo(leaf::BuildInfo::new(
        leaf_current_dir,
        leaf_build_tool,
        leaf_source_file,
        leaf_program_database_file,
        leaf_command_args,
    )));

    cv.add_subsection(Subsection::Symbols(vec![Symbol::BuildInfo(
        symbol::BuildInfo {
            leaf: leaf_build_info,
        },
    )]));

    cv.write(|name| {
        let mut obj = obj.borrow_mut();
        let segment = obj.segment_name(object::write::StandardSegment::Debug);
        let section = obj.add_section(segment.into(), name.into(), object::SectionKind::Debug);
        obj.append_section_data(section, &[], 4);

        struct Writer<'a> {
            obj: RefMut<'a, object::write::Object>,
            section: object::write::SectionId,
        }
        impl<'a> Writer<'a> {
            fn add_reloc(
                &mut self,
                reloc: &str,
                kind: object::RelocationKind,
                size: u8,
            ) -> std::result::Result<(), object::write::Error> {
                let symbol = self
                    .obj
                    .symbol_id(reloc.as_bytes())
                    .expect("Undefiend symbol");
                self.obj.add_relocation(
                    self.section,
                    object::write::Relocation {
                        offset: 0,
                        size: size * 8,
                        kind,
                        encoding: object::RelocationEncoding::Generic,
                        symbol,
                        addend: 0,
                    },
                )
            }
        }
        impl<'a> SectionWrite<String> for Writer<'a> {
            type Error = object::write::Error;

            fn write(&mut self, data: &[u8]) -> std::result::Result<(), Self::Error> {
                self.obj.append_section_data(self.section, data, 1);
                Ok(())
            }

            fn write_rva(&mut self, reloc: &String) -> std::result::Result<(), Self::Error> {
                self.add_reloc(reloc, object::RelocationKind::ImageOffset, 4)?;
                self.obj.append_section_data(self.section, &[0; 4], 1);
                Ok(())
            }

            fn write_section(&mut self, reloc: &String) -> std::result::Result<(), Self::Error> {
                self.add_reloc(reloc, object::RelocationKind::SectionIndex, 2)?;
                self.obj.append_section_data(self.section, &[0; 2], 1);
                Ok(())
            }

            fn write_secrel(&mut self, reloc: &String) -> std::result::Result<(), Self::Error> {
                self.add_reloc(reloc, object::RelocationKind::SectionOffset, 4)?;
                self.obj.append_section_data(self.section, &[0; 4], 1);
                Ok(())
            }
        }

        Writer { obj, section }
    })?;

    let mut obj_file = File::create("output.obj")?;
    obj_file.write_all(&obj.borrow().write()?)?;
    Ok(())
}
