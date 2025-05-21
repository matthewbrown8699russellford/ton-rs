use crate::cell::boc::BOC;
use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::meta::cell_meta::CellMeta;
use crate::cell::meta::cell_type::CellType;
use crate::cell::ton_cell::{TonCell, TonCellRef};
use crate::cell::ton_hash::TonHash;
use crate::errors::TonlibError;
use crate::types::tlb::tlb_type::TLBType;
use std::sync::Arc;

impl TLBType for TonCell {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> {
        let bits_left = parser.data_bits_left()?;
        if parser.cell.data_bits_len == bits_left && parser.next_ref_pos == 0 {
            let _data = parser.read_bits(bits_left)?; // drain data
            parser.next_ref_pos = parser.cell.refs.len(); // drain refs
            Ok(parser.cell.clone())
        } else {
            let data = parser.read_bits(bits_left)?;
            let refs = Vec::from(&parser.cell.refs[parser.next_ref_pos..]);
            parser.next_ref_pos = parser.cell.refs.len(); // drain refs
            let meta = CellMeta::new(CellType::Ordinary, &data, bits_left, &refs)?;
            Ok(Self {
                meta,
                data,
                data_bits_len: bits_left,
                refs,
            })
        }
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> { builder.write_cell(self) }

    fn cell_hash(&self) -> Result<TonHash, TonlibError> { Ok(self.hash().clone()) }

    fn from_boc(boc: &[u8]) -> Result<Self, TonlibError> {
        // optimization - doesn't copy Cell, just takes ownership
        // unwrap is safe - no one own Reference expect this function
        Ok(Arc::try_unwrap(BOC::from_bytes(boc)?.single_root()?.0).unwrap())
    }

    fn to_cell(&self) -> Result<TonCell, TonlibError> { Ok(self.clone()) }

    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TonlibError> {
        BOC::new(self.clone().into_ref()).to_bytes(add_crc32)
    }
    fn cell_type(&self) -> CellType { self.meta.cell_type }
}

impl TLBType for TonCellRef {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> { parser.read_next_ref().cloned() }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> { builder.write_ref(self.clone()) }
    fn cell_hash(&self) -> Result<TonHash, TonlibError> { Ok(self.hash().clone()) }
    /// Inconsistent with read(): extract value from BOC root, not from the first child
    fn from_boc(boc: &[u8]) -> Result<Self, TonlibError> { BOC::from_bytes(boc)?.single_root() }

    fn to_cell_ref(&self) -> Result<TonCellRef, TonlibError> { Ok(self.clone()) }
    /// Inconsistent with write(): write value to BOC root, not to the first child
    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TonlibError> {
        BOC::new(self.clone()).to_bytes(add_crc32)
    }
    fn cell_type(&self) -> CellType { self.meta.cell_type }
}

impl TLBType for TonHash {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> {
        TonHash::from_vec(parser.read_bits(TonHash::BITS_LEN)?)
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> {
        builder.write_bits(self.as_slice(), TonHash::BITS_LEN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_lib_macros::TLBDerive;

    #[test]
    fn test_tlb_cell() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        builder.write_num(&3u32, 32)?;
        let cell = builder.build()?;
        let parsed = TonCell::from_cell(&cell)?;
        assert_eq!(cell, parsed);
        Ok(())
    }

    #[test]
    fn test_tlb_cell_ref() -> anyhow::Result<()> {
        let mut ref_builder = TonCell::builder();
        ref_builder.write_num(&3u32, 32)?;
        let cell_ref = ref_builder.build()?.into_ref();

        let mut cell_builder = TonCell::builder();
        cell_builder.write_ref(cell_ref.clone())?;
        let cell = cell_builder.build()?;
        let parsed = TonCellRef::from_cell(&cell)?;

        assert_eq!(cell_ref, parsed);
        Ok(())
    }

    #[test]
    fn test_tlb_cell_boc() -> anyhow::Result<()> {
        let mut cell = TonCell::builder();
        cell.write_num(&3u32, 32)?;
        let cell_ref = cell.build()?.into_ref();
        let boc = cell_ref.to_boc()?;
        let parsed_ref = TonCellRef::from_boc(&boc)?;
        assert_eq!(cell_ref, parsed_ref);

        let parsed_cell = TonCell::from_boc(&boc)?;
        assert_eq!(parsed_cell.data, cell_ref.data);
        Ok(())
    }

    #[test]
    fn test_tlb_cell_boc_library() -> anyhow::Result<()> {
        let lib_hex = "b5ee9c720101010100230008420257de63d28e4d3608e0c02d437a7b50ef5f28f36a4821a047fd663ce63f4597ec";
        let lib_cell = TonCell::from_boc_hex(lib_hex)?;
        assert_eq!(lib_cell.meta.cell_type, CellType::Library);
        assert_eq!(lib_cell.to_boc_hex()?, lib_hex);

        let lib_cell_ref = TonCellRef::from_boc_hex(lib_hex)?;
        assert_eq!(lib_cell.meta.cell_type, CellType::Library);
        assert_eq!(lib_cell.to_boc_hex()?, lib_hex);

        // now library is a second cell
        let mut cell = TonCell::builder();
        cell.write_ref(lib_cell_ref.clone())?;
        let lib_child_hex = cell.build()?.to_boc_hex()?;

        let lib_child_cell = TonCell::from_boc_hex(&lib_child_hex)?;
        assert_eq!(lib_child_cell.meta.cell_type, CellType::Ordinary);
        assert_eq!(lib_child_cell.refs[0].meta.cell_type, CellType::Library);
        assert_eq!(lib_child_cell.to_boc_hex()?, lib_child_hex);

        let lib_child_cell_ref = TonCellRef::from_boc_hex(&lib_child_hex)?;
        assert_eq!(lib_child_cell_ref.meta.cell_type, CellType::Ordinary);
        assert_eq!(lib_child_cell_ref.refs[0].meta.cell_type, CellType::Library);
        assert_eq!(lib_child_cell_ref.to_boc_hex()?, lib_child_hex);

        // using extra tlb-object
        #[derive(Debug, PartialEq, TLBDerive)]
        struct TestStruct {
            cell: TonCellRef,
        }
        let test_struct = TestStruct {
            cell: lib_cell.clone().into_ref(),
        };
        let struct_hex = test_struct.to_boc_hex()?;
        let parsed_struct = TestStruct::from_boc_hex(&struct_hex)?;
        assert_eq!(test_struct, parsed_struct);
        let parsed_cell = TonCell::from_boc_hex(&struct_hex)?;
        assert_eq!(parsed_cell.meta.cell_type, CellType::Ordinary);
        assert_eq!(parsed_cell.refs[0].meta.cell_type, CellType::Library);
        Ok(())
    }
}
