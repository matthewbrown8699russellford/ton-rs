use crate::cell::build_parse::builder::CellBuilder;
use crate::cell::build_parse::parser::CellParser;
use crate::cell::ton_cell::TonCellRef;
use crate::errors::TonlibError;
use crate::types::tlb::block_tlb::out_action::{OutAction, OutActionSendMsg, OutList};
use crate::types::tlb::tlb_type::TLBType;

pub(super) fn write_up_to_4_msgs(
    dst: &mut CellBuilder,
    msgs: &[TonCellRef],
    msgs_modes: &[u8],
) -> Result<(), TonlibError> {
    validate_msgs_count(msgs, msgs_modes, 4)?;
    for (msg, mode) in msgs.iter().zip(msgs_modes.iter()) {
        mode.write(dst)?;
        msg.write(dst)?;
    }
    Ok(())
}

pub(super) fn read_up_to_4_msgs(parser: &mut CellParser) -> Result<(Vec<u8>, Vec<TonCellRef>), TonlibError> {
    let msgs_cnt = parser.cell.refs.len();
    let mut msgs_modes = Vec::with_capacity(msgs_cnt);
    let mut msgs = Vec::with_capacity(msgs_cnt);
    for _ in 0..msgs_cnt {
        msgs_modes.push(TLBType::read(parser)?);
        msgs.push(TLBType::read(parser)?);
    }
    Ok((msgs_modes, msgs))
}
pub(super) fn validate_msgs_count(msgs: &[TonCellRef], msgs_modes: &[u8], max_cnt: usize) -> Result<(), TonlibError> {
    if msgs.len() > max_cnt || msgs_modes.len() != msgs.len() {
        let err_str = format!("wrong msgs: modes_len={}, msgs_len={}, max_len={max_cnt}", msgs_modes.len(), msgs.len());
        return Err(TonlibError::CustomError(err_str));
    }
    Ok(())
}

// V5 support
// https://github.com/ton-blockchain/wallet-contract-v5/blob/88557ebc33047a95207f6e47ac8aadb102dff744/types.tlb#L26
#[derive(Debug, PartialEq, Clone)]
pub(super) struct InnerRequest {
    out_actions: Option<OutList>, // there is Option<TLBRef<OutList>>, but we don't support such description in TLBDerive
                                  // other_actions: Option<()> unsupported
}

impl TLBType for InnerRequest {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonlibError> {
        if !parser.read_bit()? {
            return Ok(Self { out_actions: None });
        }
        let out_actions = TLBType::from_cell(parser.read_next_ref()?)?;
        if parser.read_bit()? {
            return Err(TonlibError::CustomError("other_actions parsing is unsupported".to_string()));
        }
        Ok(Self {
            out_actions: Some(out_actions),
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonlibError> {
        builder.write_bit(self.out_actions.is_some())?;
        if let Some(actions) = &self.out_actions {
            builder.write_ref(actions.to_cell_ref()?)?;
        }
        builder.write_bit(false)?; // other_actions are not supported
        Ok(())
    }
}

pub(super) fn parse_inner_request(request: InnerRequest) -> Result<(Vec<TonCellRef>, Vec<u8>), TonlibError> {
    let out_list = match request.out_actions {
        Some(out_list) => out_list,
        None => return Ok((vec![], vec![])),
    };
    let mut msgs = vec![];
    let mut msgs_modes = vec![];
    for action in out_list.actions {
        if let OutAction::SendMsg(action_send_msg) = &action {
            msgs.push(action_send_msg.out_msg.clone());
            msgs_modes.push(action_send_msg.mode);
        } else {
            return Err(TonlibError::CustomError(format!("Unsupported OutAction: {action:?}")));
        }
    }

    Ok((msgs, msgs_modes))
}

pub(super) fn build_inner_request(msgs: &[TonCellRef], msgs_modes: &[u8]) -> Result<InnerRequest, TonlibError> {
    validate_msgs_count(msgs, msgs_modes, 255)?;
    // TODO suboptimal - can be done in 1 pass, but here we have 1 loop pass + recursion in OutList
    let mut actions = vec![];
    for (msg, mode) in msgs.iter().zip(msgs_modes.iter()) {
        let action = OutActionSendMsg {
            mode: *mode,
            out_msg: msg.clone(),
        };
        actions.push(OutAction::SendMsg(action));
    }

    let out_list = OutList::new(actions);

    let req = InnerRequest {
        out_actions: Some(out_list),
    };
    Ok(req)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cell::ton_cell::TonCell;
    use tokio_test::{assert_err, assert_ok};

    #[test]
    fn test_write_up_to_4_msgs() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        let msgs = vec![TonCell::EMPTY.into_ref(), TonCell::EMPTY.into_ref()];
        let msgs_modes = vec![1, 2];
        assert_ok!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

        let mut builder = TonCell::builder();
        let msgs = vec![TonCell::EMPTY.into_ref(); 4];
        let msgs_modes = vec![1, 2, 3, 4];
        assert_ok!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

        let mut builder = TonCell::builder();
        let msgs = vec![TonCell::EMPTY.into_ref(); 4];
        let msgs_modes = vec![1, 2, 3, 4, 5];
        assert_err!(write_up_to_4_msgs(&mut builder, &msgs, &msgs_modes));

        Ok(())
    }
}
