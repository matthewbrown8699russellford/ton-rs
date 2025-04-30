use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::errors::TonlibError;
use crate::types::tlb::adapters::dict::dict_key_adapters::DictKeyAdapter;
use crate::types::tlb::adapters::dict::dict_val_adapters::DictValAdapter;
use crate::types::tlb::adapters::Dict;
use std::collections::HashMap;
use std::hash::Hash;

/// Write present marker (0|1 bit) to root cell, and then Dict data to first ref cell.
/// Usage: `#[tlb_derive(adapter = "TLBDict::<DictKeyAdapterTonHash, DictValAdapterTLB, _, _>::new(256)")]` instead
pub struct TLBDict<KA: DictKeyAdapter<K>, VA: DictValAdapter<V>, K, V>(Dict<KA, VA, K, V>);

impl<KA, VA, K, V> TLBDict<KA, VA, K, V>
where
    KA: DictKeyAdapter<K>,
    VA: DictValAdapter<V>,
    K: Eq + Hash + Ord,
{
    pub fn new(key_bits_len: u32) -> Self { Self(Dict::new(key_bits_len)) }

    pub fn read(&self, parser: &mut CellParser) -> Result<HashMap<K, V>, TonlibError> {
        if !parser.read_bit()? {
            return Ok(HashMap::new());
        }
        self.0.read(&mut CellParser::new(parser.read_next_ref()?))
    }

    pub fn write(&self, builder: &mut CellBuilder, data: &HashMap<K, V>) -> Result<(), TonlibError> {
        if data.is_empty() {
            builder.write_bit(false)?;
            return Ok(());
        }
        builder.write_bit(true)?;
        let mut dict_data_builder = CellBuilder::new();
        self.0.write(&mut dict_data_builder, data)?;
        builder.write_ref(dict_data_builder.build()?.into_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ton_cell::TonCell;
    use crate::types::tlb::adapters::dict::dict_key_adapters::DictKeyAdapterInto;
    use crate::types::tlb::adapters::dict::dict_val_adapters::DictValAdapterNum;
    use crate::types::tlb::tlb_type::TLBType;
    use num_bigint::BigUint;
    use std::collections::HashMap;

    #[test]
    fn test_dict_blockchain_data() -> anyhow::Result<()> {
        let expected_data = HashMap::from([
            (0u8, BigUint::from(25965603044000000000u128)),
            (1, BigUint::from(5173255344000000000u64)),
            (2, BigUint::from(344883687000000000u64)),
        ]);
        let boc_hex = "b5ee9c7241010601005a000119c70d3ca5000d99b931ea4e8cc0010201cd020302012004050027400000000000000000000001325178b51d9180200026000000000000000000000168585a65986be8000026000000000000000000000047cb18538782e000353c80b9";
        let dict_cell = TonCell::from_boc_hex(boc_hex)?;
        let mut parser = CellParser::new(&dict_cell);
        let some_data = parser.read_bits(96)?;

        let parsed_data = TLBDict::<DictKeyAdapterInto, DictValAdapterNum<150>, _, _>::new(8).read(&mut parser)?;
        assert_eq!(expected_data, parsed_data);
        let mut builder = CellBuilder::new();
        builder.write_bits(&some_data, 96)?;
        TLBDict::<DictKeyAdapterInto, DictValAdapterNum<150>, _, _>::new(8).write(&mut builder, &expected_data)?;
        let constructed_cell = builder.build()?;
        assert_eq!(dict_cell, constructed_cell);
        Ok(())
    }

    #[test]
    fn test_dict_key_bits_len_bigger_than_key() -> anyhow::Result<()> {
        let data = HashMap::from([
            (0u16, BigUint::from(4u32)),
            (1, BigUint::from(5u32)),
            (2, BigUint::from(6u32)),
            (10u16, BigUint::from(7u32)),
            (127, BigUint::from(8u32)),
        ]);

        for key_len_bits in [8u32, 16, 32, 64, 111] {
            let mut builder = CellBuilder::new();
            TLBDict::<DictKeyAdapterInto, DictValAdapterNum<150>, _, _>::new(key_len_bits)
                .write(&mut builder, &data)?;
            let dict_cell = builder.build()?;
            let mut parser = CellParser::new(&dict_cell);
            let parsed =
                TLBDict::<DictKeyAdapterInto, DictValAdapterNum<150>, _, _>::new(key_len_bits).read(&mut parser)?;
            assert_eq!(data, parsed, "key_len_bits: {}", key_len_bits);
        }
        Ok(())
    }
}
