/// A trait for byte-oriented sinks that supports adding relocation information.
///
/// `Reloc` can be any type that carries relocation symbol information.
pub trait SectionWrite<Reloc> {
    /// Type for reporting error.
    type Error: std::error::Error + 'static;

    /// Write plain data into the sink.
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Write a relative virtual address (RVA) with relocation.
    ///
    /// Implementation should writes a 4-byte island and record a `IMAGE_REL_*_ADDR32NB`
    /// relocation on it.
    fn write_rva(&mut self, reloc: &Reloc) -> Result<(), Self::Error>;

    /// Write a section index with relocation.
    ///
    /// Implementation should writes a 2-byte island and record a `IMAGE_REL_*_SECTION`
    /// relocation on it.
    fn write_section(&mut self, reloc: &Reloc) -> Result<(), Self::Error>;

    /// Write a section-relative address with relocation.
    ///
    /// Implementation should writes a 4-byte island and record a `IMAGE_REL_*_SECREL`
    /// relocation on it.
    fn write_secrel(&mut self, reloc: &Reloc) -> Result<(), Self::Error>;
}

/// A simple section writer that collects all data and relocations.
#[derive(Debug, Clone, Default)]
pub struct SectionSink<Reloc> {
    /// Section data
    pub data: Vec<u8>,

    /// Relative virtual address relocations.
    pub reloc_rva: Vec<(usize, Reloc)>,

    /// Section index relocations.
    pub reloc_section: Vec<(usize, Reloc)>,

    /// Section-relative address relocations.
    pub reloc_secrel: Vec<(usize, Reloc)>,
}

impl<Reloc> SectionSink<Reloc> {
    /// Create a new `SectionSink` instance
    pub fn new() -> SectionSink<Reloc> {
        SectionSink {
            data: vec![],
            reloc_rva: vec![],
            reloc_section: vec![],
            reloc_secrel: vec![],
        }
    }
}

impl<Reloc: Clone> SectionWrite<Reloc> for SectionSink<Reloc> {
    type Error = std::convert::Infallible; // TODO: This should be `!`
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.data.extend_from_slice(data);
        Ok(())
    }
    fn write_rva(&mut self, reloc: &Reloc) -> Result<(), Self::Error> {
        self.reloc_rva.push((self.data.len(), reloc.clone()));
        self.data.extend_from_slice(&[0; 4]);
        Ok(())
    }
    fn write_section(&mut self, reloc: &Reloc) -> Result<(), Self::Error> {
        self.reloc_section.push((self.data.len(), reloc.clone()));
        self.data.extend_from_slice(&[0; 2]);
        Ok(())
    }
    fn write_secrel(&mut self, reloc: &Reloc) -> Result<(), Self::Error> {
        self.reloc_secrel.push((self.data.len(), reloc.clone()));
        self.data.extend_from_slice(&[0; 4]);
        Ok(())
    }
}
