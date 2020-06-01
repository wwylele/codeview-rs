use crate::error::{wu, Error};
use crate::section_write::SectionWrite;
use std::convert::*;

pub(crate) trait Writable<Reloc> {
    fn write<W: SectionWrite<Reloc>>(&self, writer: &mut W) -> Result<(), Error<W::Error>>;
    fn size(&self) -> usize;
}

macro_rules! writable_primitive {
    ($t:ty) => {
        impl<Reloc> Writable<Reloc> for $t {
            fn write<W: SectionWrite<Reloc>>(&self, writer: &mut W) -> Result<(), Error<W::Error>> {
                wu(writer.write(&self.to_le_bytes()))?;
                Ok(())
            }

            fn size(&self) -> usize {
                std::mem::size_of::<$t>()
            }
        }
    };
}

writable_primitive!(u8);
writable_primitive!(u16);
writable_primitive!(u32);
writable_primitive!(i8);
writable_primitive!(i16);
writable_primitive!(i32);

impl<Reloc> Writable<Reloc> for String {
    fn write<W: SectionWrite<Reloc>>(&self, writer: &mut W) -> Result<(), Error<W::Error>> {
        // It is uncertain which encoding CodeView uses.
        // Maybe it is locale-dependent.
        // However, on my en-US Windows system, Chinese characters are encoded in UTF-8
        // in Visual Studio-generated CodeView data, so UTF-8 is a good bet here.
        if self.as_bytes().iter().any(|c| *c == 0) {
            return Result::Err(Error::StringError(self.clone()));
        }
        wu(writer.write(&self.as_bytes()))?;
        wu(writer.write(&[0]))?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.len() + 1
    }
}

macro_rules! writable_transparent {
    ($t:ty) => {
        impl<Reloc> Writable<Reloc> for $t {
            fn write<W: SectionWrite<Reloc>>(&self, writer: &mut W) -> Result<(), Error<W::Error>> {
                Writable::<Reloc>::write(&self.0, writer)
            }

            fn size(&self) -> usize {
                Writable::<Reloc>::size(&self.0)
            }
        }
    };
}

impl<Reloc, T: Writable<Reloc>> Writable<Reloc> for Vec<T> {
    fn write<W: SectionWrite<Reloc>>(&self, writer: &mut W) -> Result<(), Error<W::Error>> {
        for element in self {
            element.write(writer)?;
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.iter().map(|e| e.size()).sum()
    }
}

pub(crate) trait WritableRecord<Reloc>: Writable<Reloc> {
    fn type_id(&self) -> u16;
}

macro_rules! record_struct {
    ($(#[$outer:meta])*
    [$name:ident]
    [{$(#[$inner1:meta])*} $m1:ident : $t1:ty , $({$(#[$inner:meta])*} $m:tt : $t:ty,)*]
    [$({$(#[$inner2:meta])*} $m2:ident : $t2:ty,)*]) => {
        record_struct!($(#[$outer])*
            [$name]
            [$({$(#[$inner])*} $m : $t,)*]
            [$({$(#[$inner2])*} $m2 : $t2,)* {$(#[$inner1])*} $m1 : $t1,]);
    };

    ($(#[$outer:meta])*
    [$name:ident]
    [{$(#[$inner1:meta])*} $m1:tt : $t1:ty , $({$(#[$inner:meta])*} $m:tt : $t:ty,)*]
    [$({$(#[$inner2:meta])*} $m2:ty : $t2:ty,)*]) => {
        record_struct!($(#[$outer])*
            [$name]
            [$({$(#[$inner])*} $m : $t,)*]
            [$({$(#[$inner2])*} $m2 : $t2,)*]);
    };

    ($(#[$outer:meta])*
    [$name:ident]
    []
    [$({$(#[$inner2:meta])*} $m2:ident : $t2:ty,)*]) => {
        $(#[$outer])*
        #[derive(Debug, Clone)]
        pub struct $name {
            $($(#[$inner2])* pub $m2 : $t2,)*
        }
    };
}

macro_rules! record_write {
    ([$self:ident, $writer:ident, $reloc:ident]
    [$m1:ident : $t1:ty , $($m:tt : $t:ty,)*]
    [$($s:stmt)*]) => {
        record_write!([$self, $writer, $reloc] [$($m : $t,)*] [$($s)*
            Writable::<$reloc>::write(&$self.$m1, $writer)?;
        ])
    };

    ([$self:ident, $writer:ident, $reloc:ident]
    [(len($m1:ident)) : $t1:ty , $($m:tt : $t:ty,)*]
    [$($s:stmt)*]) => {
        record_write!([$self, $writer, $reloc] [$($m : $t,)*] [$($s)*
            Writable::<$reloc>::write(&<$t1>::try_from($self.$m1.len())?, $writer)?;
        ])
    };

    ([$self:ident, $writer:ident, $reloc:ident]
    []
    [$($s:stmt)*]) => {
        $($s)*
    };
}

macro_rules! record_size {
    ([$self:ident, $reloc:ident]
    [$m1:ident : $t1:ty , $($m:tt : $t:ty,)*]
    [$($s:expr)*]) => {
        record_size!([$self, $reloc]
            [$($m : $t,)*]
            [$($s)* Writable::<$reloc>::size(&$self.$m1) ])
    };

    ([$self:ident, $reloc:ident]
    [(len($m1:ident)) : $t1:ty , $($m:tt : $t:ty,)*]
    [$($s:expr)*]) => {
        record_size!([$self, $reloc]
            [$($m : $t,)*]
            [$($s)* std::mem::size_of::<$t1>() ])
    };

    ([$self:ident, $reloc:ident]
    []
    [$($s:expr)*]) => {
        $($s +)* 0
    };
}

macro_rules! record {
    ( $(#[$outer:meta])*
    [ $name:ident = $type_id:literal ]
    $(#[doc=$ds:literal] $m:tt : $t:ty,)* ) => {
        record_struct!($(#[$outer])* [$name] [$({#[doc=$ds]} $m : $t,)*] []);

        impl<Reloc> Writable<Reloc> for $name{
            fn write<W: SectionWrite<Reloc>>(&self, writer: &mut W) -> Result<(), Error<W::Error>> {
                record_write!([self, writer, Reloc] [$($m : $t,)*] [] );
                Ok(())
            }

            fn size(&self) -> usize {
                record_size!([self, Reloc] [$($m : $t,)*] [])
            }
        }

        impl<Reloc> WritableRecord<Reloc> for $name {
            fn type_id(&self) -> u16 {
                $type_id
            }
        }
    };
}

pub(crate) fn write_record<Reloc, T: WritableRecord<Reloc>, W: SectionWrite<Reloc>>(
    record: &T,
    writer: &mut W,
) -> Result<(), Error<W::Error>> {
    u16::try_from(record.size() + 2)?.write(writer)?;
    record.type_id().write(writer)?;
    record.write(writer)
}

macro_rules! all_records {
    ($(#[$outer:meta])* pub enum $name:ident$(<$reloc:ident>)? {$($t:ident,)*}) => {
        $(#[$outer])*
        pub enum $name$(<$reloc>)? {
            $( $t($t), )*
            $(Phantom(std::marker::PhantomData<$reloc>),)?
        }

        pub(crate) fn write<Reloc, W: SectionWrite<Reloc>>(
            record: &$name$(<$reloc>)?,
            writer: &mut W
        ) -> Result<(), Error<W::Error>> {
            match record {
                $( $name::$t(s) => write_record(s, writer) ,)*
                _ => panic!()
            }
        }

        #[allow(dead_code)]
        pub(crate) fn size<Reloc>(record: &$name$(<$reloc>)?) -> usize {
            match record {
                $( $name::$t(s) => Writable::<Reloc>::size(s) + 4 ,)*
                _ => panic!()
            }
        }
    };
}
