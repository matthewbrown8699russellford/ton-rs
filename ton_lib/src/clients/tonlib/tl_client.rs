use crate::bc_constants::{TON_MASTERCHAIN_ID, TON_SHARD_FULL};
use crate::cell::ton_hash::TonHash;
use crate::clients::tonlib::clients_impl::TLConnection;
use crate::clients::tonlib::tl_api::tl_request::TLRequest;
use crate::clients::tonlib::tl_api::tl_response::TLResponse;
use crate::clients::tonlib::tl_api::tl_types::{
    TLAccountAddress, TLBlockId, TLBlockIdExt, TLBlocksAccountTxId, TLBlocksHeader, TLBlocksMCInfo, TLBlocksShards,
    TLBlocksTxs, TLConfigInfo, TLFullAccountState, TLRawFullAccountState, TLRawTxs, TLSmcLibraryResult, TLTxId,
};
use crate::errors::TonlibError;
use crate::types::ton_address::TonAddress;
use async_trait::async_trait;

#[macro_export]
macro_rules! unwrap_tl_response {
    ($result:expr, $variant:ident) => {
        match $result {
            TLResponse::$variant(inner) => Ok(inner),
            _ => Err(TonlibError::TLClientWrongResponse(stringify!($variant).to_string(), format!("{:?}", $result))),
        }
    };
}

#[async_trait]
pub trait TLClient: Send + Sync + Clone + 'static {
    async fn get_connection(&self) -> Result<&TLConnection, TonlibError>;

    async fn exec(&self, req: &TLRequest) -> Result<TLResponse, TonlibError> {
        self.get_connection().await?.exec_impl(req).await
    }

    async fn get_mc_info(&self) -> Result<TLBlocksMCInfo, TonlibError> {
        let req = TLRequest::BlocksGetMCInfo {};
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksMCInfo)
    }

    /// * `mode`: Lookup mode: `1` - by `block_id.seqno`, `2` - by `lt`, `4` - by `utime`.
    async fn lookup_block(
        &self,
        mode: i32,
        block_id: TLBlockId,
        lt: i64,
        utime: i32,
    ) -> Result<TLBlockIdExt, TonlibError> {
        let req = TLRequest::BlocksLookupBlock {
            mode,
            id: block_id,
            lt,
            utime,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLBlockIdExt)
    }

    async fn lookup_mc_block(&self, seqno: i32) -> Result<TLBlockIdExt, TonlibError> {
        let block_id = TLBlockId {
            workchain: TON_MASTERCHAIN_ID,
            shard: TON_SHARD_FULL as i64,
            seqno,
        };
        self.lookup_block(1, block_id, 0, 0).await
    }

    async fn get_account_state_raw(&self, address: TonAddress) -> Result<TLRawFullAccountState, TonlibError> {
        let req = TLRequest::RawGetAccountState {
            account_address: TLAccountAddress {
                account_address: address.to_hex(),
            },
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawFullAccountState)
    }

    async fn get_account_state_raw_by_tx(
        &self,
        address: TonAddress,
        tx_id: TLTxId,
    ) -> Result<TLRawFullAccountState, TonlibError> {
        let req = TLRequest::RawGetAccountStateByTx {
            account_address: TLAccountAddress {
                account_address: address.to_hex(),
            },
            transaction_id: tx_id,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawFullAccountState)
    }

    async fn get_account_state(&self, address: &TonAddress) -> Result<TLFullAccountState, TonlibError> {
        let req = TLRequest::GetAccountState {
            account_address: TLAccountAddress {
                account_address: address.to_hex(),
            },
        };
        unwrap_tl_response!(self.exec(&req).await?, TLFullAccountState)
    }

    async fn get_txs_raw(&self, address: &TonAddress, from_tx_id: &TLTxId) -> Result<TLRawTxs, TonlibError> {
        let req = TLRequest::RawGetTxs {
            account_address: TLAccountAddress {
                account_address: address.to_hex(),
            },
            from_transaction_id: from_tx_id.clone(),
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawTxs)
    }

    async fn get_txs_raw_v2(
        &self,
        address: &TonAddress,
        from_tx: TLTxId,
        count: usize,
        try_decode_msg: bool,
    ) -> Result<TLRawTxs, TonlibError> {
        if count > 16 {
            return Err(TonlibError::TLInvalidArgs(format!(
                "get_raw_transactions_v2: count <= 16 supported, got {count}"
            )));
        }
        let req = TLRequest::RawGetTxsV2 {
            account_address: TLAccountAddress {
                account_address: address.to_hex(),
            },
            from_transaction_id: from_tx.clone(),
            count: count as u32,
            try_decode_messages: try_decode_msg,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLRawTxs)
    }

    async fn send_msg(&self, body: Vec<u8>) -> Result<TonHash, TonlibError> {
        let req = TLRequest::RawSendMsgReturnHash { body };
        let rsp = unwrap_tl_response!(self.exec(&req).await?, TLRawExtMessageInfo)?;
        TonHash::from_vec(rsp.hash)
    }

    async fn sync(&self) -> Result<TLBlockIdExt, TonlibError> {
        let req = TLRequest::Sync {};
        unwrap_tl_response!(self.exec(&req).await?, TLBlockIdExt)
    }

    // async fn smc_load(
    //     &self,
    //     account_address: &TonAddress,
    // ) -> Result<LoadedSmcState, TonClientError> {
    //     let func = TonFunction::SmcLoad {
    //         account_address: AccountAddress {
    //             account_address: account_address.to_hex(),
    //         },
    //     };
    //     let (conn, result) = self.invoke_on_connection(&func).await?;
    //     match result {
    //         TonResult::SmcInfo(smc_info) => Ok(LoadedSmcState {
    //             conn,
    //             id: smc_info.id,
    //         }),
    //         r => Err(TonClientError::unexpected_ton_result(
    //             TonResultDiscriminants::SmcInfo,
    //             r,
    //         )),
    //     }
    // }
    // async fn smc_load_by_transaction(
    //     &self,
    //     address: &TonAddress,
    //     tx_id: &InternalTransactionId,
    // ) -> Result<LoadedSmcState, TonClientError> {
    //     let func = TonFunction::SmcLoadByTransaction {
    //         account_address: AccountAddress {
    //             account_address: address.to_hex(),
    //         },
    //         transaction_id: tx_id.clone(),
    //     };
    //     let (conn, result) = self.invoke_on_connection(&func).await?;
    //     match result {
    //         TonResult::SmcInfo(smc_info) => Ok(LoadedSmcState {
    //             conn,
    //             id: smc_info.id,
    //         }),
    //         r => Err(TonClientError::unexpected_ton_result(
    //             TonResultDiscriminants::SmcInfo,
    //             r,
    //         )),
    //     }
    // }
    //
    async fn get_libs(&self, lib_ids: Vec<TonHash>) -> Result<TLSmcLibraryResult, TonlibError> {
        let req = TLRequest::SmcGetLibraries { library_list: lib_ids };
        unwrap_tl_response!(self.exec(&req).await?, TLSmcLibraryResult)
    }
    //
    // async fn smc_get_libraries_ext(
    //     &self,
    //     list: &[SmcLibraryQueryExt],
    // ) -> Result<SmcLibraryResultExt, TonClientError> {
    //     let func = TonFunction::SmcGetLibrariesExt {
    //         list: list.to_vec(),
    //     };
    //     let result = self.invoke(&func).await?;
    //     match result {
    //         TonResult::SmcLibraryResultExt(r) => Ok(r),
    //         r => Err(TonClientError::unexpected_ton_result(
    //             TonResultDiscriminants::SmcLibraryResultExt,
    //             r,
    //         )),
    //     }
    // }
    //

    //
    async fn get_block_shards(&self, block_id: TLBlockIdExt) -> Result<TLBlocksShards, TonlibError> {
        let req = TLRequest::BlocksGetShards { id: block_id.clone() };
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksShards)
    }

    /// Returns up to specified number of ids of transactions in specified block.
    ///
    /// * `block_id`: ID of the block to retrieve transactions for (either masterchain or shard).
    /// * `mode`: Use `7` to get first chunk of transactions or `7 + 128` for subsequent chunks.
    /// * `count`: Maximum mumber of transactions to retrieve.
    /// * `after`: Specify `NULL_BLOCKS_ACCOUNT_TRANSACTION_ID` to get the first chunk
    ///             or id of the last retrieved tx for subsequent chunks.
    ///
    async fn get_block_txs(
        &self,
        block_id: TLBlockIdExt,
        mode: u32,
        count: u32,
        after: TLBlocksAccountTxId,
    ) -> Result<TLBlocksTxs, TonlibError> {
        let req = TLRequest::BlocksGetTxs {
            id: block_id,
            mode,
            count,
            after,
        };
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksTxs)
    }
    //
    // async fn get_block_transactions_ext(
    //     &self,
    //     block_id: &BlockIdExt,
    //     mode: u32,
    //     count: u32,
    //     after: &BlocksAccountTransactionId,
    // ) -> Result<BlocksTransactionsExt, TonClientError> {
    //     let func = TonFunction::BlocksGetTransactionsExt {
    //         id: block_id.clone(),
    //         mode,
    //         count,
    //         after: after.clone(),
    //     };
    //     let result = self.invoke(&func).await?;
    //     match result {
    //         TonResult::BlocksTransactionsExt(result) => Ok(result),
    //         r => Err(TonClientError::unexpected_ton_result(
    //             TonResultDiscriminants::BlocksTransactionsExt,
    //             r,
    //         )),
    //     }
    // }
    //
    // async fn lite_server_get_info(&self) -> Result<LiteServerInfo, TonClientError> {
    //     let func = TonFunction::LiteServerGetInfo {};
    //     let result = self.invoke(&func).await?;
    //     match result {
    //         TonResult::LiteServerInfo(result) => Ok(result),
    //         r => Err(TonClientError::unexpected_ton_result(
    //             TonResultDiscriminants::LiteServerInfo,
    //             r,
    //         )),
    //     }
    // }
    //
    async fn get_block_header(&self, block_id: TLBlockIdExt) -> Result<TLBlocksHeader, TonlibError> {
        let req = TLRequest::GetBlockHeader { id: block_id };
        unwrap_tl_response!(self.exec(&req).await?, TLBlocksHeader)
    }

    async fn get_config_param(&self, mode: u32, param: u32) -> Result<TLConfigInfo, TonlibError> {
        let req = TLRequest::GetConfigParam { mode, param };
        unwrap_tl_response!(self.exec(&req).await?, TLConfigInfo)
    }
    //
    async fn get_config_all(&self, mode: u32) -> Result<TLConfigInfo, TonlibError> {
        let req = TLRequest::GetConfigAll { mode };
        unwrap_tl_response!(self.exec(&req).await?, TLConfigInfo)
    }
}
