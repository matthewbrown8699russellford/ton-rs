pub mod common_msg_info;

use super::msg_address::{MsgAddress, MsgAddressExt, MsgAddressInt};
use crate::cell::ton_cell::TonCell;
use crate::cell::ton_hash::TonHash;
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::coins::Coins;
use crate::types::tlb::block_tlb::state_init::StateInit;
use crate::types::tlb::primitives::either::EitherRef;
use crate::types::tlb::primitives::either::EitherRefLayout::ToRef;
use crate::types::tlb::TLB;
use common_msg_info::CommonMsgInfo;
use ton_lib_macros::TLBDerive;

// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L157
#[derive(Debug, Clone, PartialEq, TLBDerive)]
pub struct Msg {
    pub info: CommonMsgInfo,
    pub init: Option<EitherRef<StateInit>>,
    pub body: EitherRef<TonCell>,
}

impl Msg {
    pub fn new(info: CommonMsgInfo, body: TonCell) -> Self {
        Self {
            info,
            init: None,
            body: EitherRef::new_with_layout(body, ToRef),
        }
    }

    pub fn src(&self) -> MsgAddress {
        match &self.info {
            CommonMsgInfo::Int(info) => info.src.clone(),
            CommonMsgInfo::ExtIn(info) => info.src.clone().into(),
            CommonMsgInfo::ExtOut(info) => info.src.clone().into(),
        }
    }

    pub fn dst(&self) -> MsgAddress {
        match &self.info {
            CommonMsgInfo::Int(info) => info.dest.clone(),
            CommonMsgInfo::ExtIn(info) => info.dest.clone().into(),
            CommonMsgInfo::ExtOut(info) => info.dest.clone().into(),
        }
    }

    pub fn created_at(&self) -> Option<u32> {
        match &self.info {
            CommonMsgInfo::Int(info) => Some(info.created_at),
            CommonMsgInfo::ExtIn(_) => None,
            CommonMsgInfo::ExtOut(info) => Some(info.created_at),
        }
    }

    pub fn created_lt(&self) -> Option<u64> {
        match &self.info {
            CommonMsgInfo::Int(info) => Some(info.created_lt),
            CommonMsgInfo::ExtIn(_) => None,
            CommonMsgInfo::ExtOut(info) => Some(info.created_lt),
        }
    }

    pub fn state_init(&self) -> Option<&StateInit> { self.init.as_ref().map(|init| &init.value) }

    pub fn hash_normalized(&self) -> Result<TonHash, TonlibError> {
        match &self.info {
            CommonMsgInfo::ExtIn(_) => {
                let mut msg_normalized = self.clone();
                let CommonMsgInfo::ExtIn(info) = &mut msg_normalized.info else {
                    unreachable!()
                };
                info.src = MsgAddressExt::NONE;
                match &mut info.dest {
                    MsgAddressInt::Std(addr) => addr.anycast = None,
                    MsgAddressInt::Var(addr) => addr.anycast = None,
                }
                info.import_fee = Coins::zero();
                msg_normalized.init = None;
                msg_normalized.body.layout = ToRef;
                msg_normalized.cell_hash()
            }
            _ => self.cell_hash(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::ton_cell::TonCell;
    use crate::types::tlb::block_tlb::coins::{Coins, CurrencyCollection};
    use crate::types::tlb::block_tlb::msg::common_msg_info::CommonMsgInfoExtIn;
    use crate::types::tlb::block_tlb::msg_address::{Anycast, MsgAddressExtern, MsgAddressIntStd};
    use crate::types::tlb::block_tlb::var_len::VarLenBits;
    use crate::types::tlb::TLB;
    use crate::types::ton_address::TonAddress;
    use std::str::FromStr;
    use tokio_test::assert_ok;

    #[test]
    fn test_common_msg_info_int() -> anyhow::Result<()> {
        let msg_cell = TonCell::from_boc_hex("b5ee9c720101010100580000ab69fe00000000000000000000000000000000000000000000000000000000000000013fccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccd3050ec744000000617bc90dda80cf41ab8e40")?;
        let parsed_msg = Msg::from_cell(&msg_cell)?;
        assert!(parsed_msg.init.is_none());
        assert_eq!(parsed_msg.body.value.data_bits_len, 0); // quite useless assert, but let it be here

        let info = match parsed_msg.info.clone() {
            CommonMsgInfo::Int(info) => info,
            _ => panic!("Expected CommonMsgInfo::Int"),
        };
        assert!(info.ihr_disabled);
        assert!(info.bounce);
        assert!(!info.bounced);

        let expected_src = TonAddress::from_str("Ef8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADAU")?;
        let expected_dest = TonAddress::from_str("Ef8zMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzM0vF")?;
        assert_eq!(TonAddress::from_msg_address(info.src)?, expected_src);
        assert_eq!(TonAddress::from_msg_address(info.dest)?, expected_dest);
        assert_eq!(info.value, CurrencyCollection::new(3242439121u32));
        assert_eq!(info.ihr_fee, Coins::new(0u32));
        assert_eq!(info.fwd_fee, Coins::new(0u32));
        assert_eq!(info.created_lt, 53592141000000);
        assert_eq!(info.created_at, 1738593735u32);

        let serialized = parsed_msg.to_cell()?;
        let parsed_back = assert_ok!(Msg::from_cell(&serialized));
        assert_eq!(parsed_back, parsed_msg);
        Ok(())
    }

    #[test]
    fn test_ext_in_msg_info() -> anyhow::Result<()> {
        let ext_in_msg_info = CommonMsgInfoExtIn::from_boc_hex(
            "b5ee9c7201010101002500004588010319f77e4d761f956e78f9c9fd45f1e914b7ffab9b5c1ea514858979c1560dee10",
        )?;
        let expected_dst = TonAddress::from_str("EQCBjPu_JrsPyrc8fOT-ovj0ilv_1c2uD1KKQsS84KsG90PM")?;
        let dst = TonAddress::from_msg_address(ext_in_msg_info.dest.clone())?;
        assert_eq!(dst, expected_dst);
        assert_eq!(ext_in_msg_info.import_fee, Coins::new(0u32));

        let cell = ext_in_msg_info.to_cell()?;
        let parsed = CommonMsgInfoExtIn::from_cell(&cell)?;
        assert_eq!(parsed, ext_in_msg_info);
        Ok(())
    }

    // reproducing https://github.com/tonkeeper/tongo/blob/5c0ce694d72b7024bcb62b3d0dcd008940a75419/tlb/messages_test.go#L23C1-L80C2
    #[test]
    fn test_ext_in_msg_hash_normalized() -> anyhow::Result<()> {
        let msg_info = CommonMsgInfo::ExtIn(CommonMsgInfoExtIn {
            src: MsgAddressExt::Extern(MsgAddressExtern {
                address: VarLenBits::new(vec![1, 2, 3], 16),
            }),
            dest: MsgAddressInt::Std(MsgAddressIntStd {
                anycast: Some(Anycast::new(16, vec![9, 12])),
                workchain: -1,
                address: TonHash::from_str("adfd5f1d28db13e50591d5c76a976c15d8ab6cad90554748ab254871390d9334")?,
            }),
            import_fee: Coins::new(12364u128),
        });
        let mut body_value_builder = TonCell::builder();
        body_value_builder.write_num(&200u32, 32)?;
        let body_value = body_value_builder.build()?;

        let msg = Msg {
            info: msg_info,
            init: Some(EitherRef {
                value: StateInit::new(TonCell::EMPTY.into_ref(), TonCell::EMPTY.into_ref()),
                layout: ToRef,
            }),
            body: EitherRef {
                value: body_value,
                layout: ToRef,
            },
        };
        let hash_norm = msg.hash_normalized()?;
        assert_eq!(hash_norm, TonHash::from_str("dfacc0b48826e33a5a127ee1def710a449d8ce79def7c19f43e57b7996e870df")?);

        Ok(())
    }
}
