use crate::cell::ton_hash::vec_ton_hash_serde_b64;
use crate::cell::ton_hash::TonHash;
use crate::clients::tonlib::tl::stack::TvmStackEntry;
use crate::clients::tonlib::tl::types::{
    AccountAddress, BlockId, BlockIdExt, BlocksAccountTransactionId, InternalTransactionId, Options,
    SmcLibraryQueryExt, SmcMethodId,
};
use crate::clients::tonlib::tl::Base64Standard;
use crate::types::ton_address::TonAddress;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "@type", rename_all = "camelCase")]
pub enum TLRequest {
    // tonlib_api.tl, line 216
    LiteServerInfo {
        now: i64,
        version: i32,
        capabilities: i64,
    },

    // tonlib_api.tl, line 238
    Init {
        options: Options,
    },

    // tonlib_api.tl, line 261
    #[serde(rename = "raw.sendMessageReturnHash")]
    RawSendMessageReturnHash {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
    },

    // tonlib_api.tl, line 265
    #[serde(rename = "sync")]
    Sync {},

    //tonlib_api.tl, line 266
    #[serde(rename = "raw.getAccountState")]
    RawGetAccountState {
        account_address: TonAddress,
    },

    //tonlib_api.tl, line 267
    #[serde(rename = "raw.getAccountStateByTransaction")]
    RawGetAccountStateByTransaction {
        account_address: TonAddress,
        transaction_id: InternalTransactionId,
    },

    // tonlib_api.tl, line 268
    #[serde(rename = "raw.getTransactions")]
    RawGetTransactions {
        account_address: AccountAddress,
        from_transaction_id: InternalTransactionId,
    },

    // tonlib_api.tl, line 269
    #[serde(rename = "raw.getTransactionsV2")]
    RawGetTransactionsV2 {
        account_address: AccountAddress,
        from_transaction_id: InternalTransactionId,
        count: u32,
        try_decode_messages: bool,
    },

    // tonlib_api.tl, line 270
    #[serde(rename = "raw.sendMessage")]
    RawSendMessage {
        #[serde(with = "Base64Standard")]
        body: Vec<u8>,
    },

    // tonlib_api.tl, line 288
    #[serde(rename = "getAccountState")]
    GetAccountState {
        account_address: AccountAddress,
    },

    // tonlib_api.tl, line 294
    #[serde(rename = "getConfigParam")]
    GetConfigParam {
        mode: u32,
        param: u32,
    },

    // tonlib_api.tl, line 295
    #[serde(rename = "getConfigAll")]
    GetConfigAll {
        mode: u32,
    },

    // tonlib_api.tl, line 306
    #[serde(rename = "smc.load")]
    SmcLoad {
        account_address: AccountAddress,
    },

    // tonlib_api.tl, line 307
    #[serde(rename = "smc.loadByTransaction")]
    SmcLoadByTransaction {
        account_address: AccountAddress,
        transaction_id: InternalTransactionId,
    },

    // tonlib_api.tl, line 308
    #[serde(rename = "smc.forget")]
    SmcForget {
        id: i64,
    },

    // tonlib_api.tl, line 309
    #[serde(rename = "smc.getCode")]
    SmcGetCode {
        id: i64,
    },

    // tonlib_api.tl, line 310
    #[serde(rename = "smc.getData")]
    SmcGetData {
        id: i64,
    },

    // tonlib_api.tl, line 311
    #[serde(rename = "smc.getState")]
    SmcGetState {
        id: i64,
    },

    // tonlib_api.tl, line 312
    #[serde(rename = "smc.runGetMethod")]
    SmcRunGetMethod {
        id: i64,
        method: SmcMethodId,
        stack: Vec<TvmStackEntry>,
    },

    // tonlib_api.tl, line 314
    #[serde(rename = "smc.getLibraries")]
    SmcGetLibraries {
        #[serde(with = "vec_ton_hash_serde_b64")]
        library_list: Vec<TonHash>,
    },

    // tonlib_api.tl, line 315
    #[serde(rename = "smc.getLibrariesExt")]
    SmcGetLibrariesExt {
        list: Vec<SmcLibraryQueryExt>,
    },

    // tonlib_api.tl, line 316
    #[serde(rename = "blocks.getMasterchainInfo")]
    BlocksGetMasterchainInfo {},

    // tonlib_api.tl, line 327
    #[serde(rename = "blocks.getShards")]
    BlocksGetShards {
        id: BlockIdExt,
    },

    // tonlib_api.tl, line 328
    #[serde(rename = "blocks.lookupBlock")]
    BlocksLookupBlock {
        mode: i32,
        id: BlockId,
        lt: i64,
        utime: i32,
    },

    // tonlib_api.tl, line 329
    #[serde(rename = "blocks.getTransactions")]
    BlocksGetTransactions {
        id: BlockIdExt,
        mode: u32,
        count: u32,
        after: BlocksAccountTransactionId,
    },

    // tonlib_api.tl, line 330
    #[serde(rename = "blocks.getTransactionsExt")]
    BlocksGetTransactionsExt {
        id: BlockIdExt,
        mode: u32,
        count: u32,
        after: BlocksAccountTransactionId,
    },

    // tonlib_api.tl, line 331
    #[serde(rename = "blocks.getBlockHeader")]
    GetBlockHeader {
        id: BlockIdExt,
    },

    // tonlib_ai.tl, line 342
    #[serde(rename = "liteServer.getInfo")]
    LiteServerGetInfo {},

    // tonlib_api.tl, line 352
    SetLogVerbosityLevel {
        new_verbosity_level: u32,
    },
    // tonlib_api.tl, line 355
    GetLogVerbosityLevel {},
}
