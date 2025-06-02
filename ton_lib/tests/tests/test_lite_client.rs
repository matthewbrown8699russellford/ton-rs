use crate::tests::utils::init_logging;
use std::str::FromStr;
use std::time::Duration;
use ton_lib::clients::lite_client::client::LiteClient;
use ton_lib::clients::lite_client::config::LiteClientConfig;
use ton_lib::clients::net_config::TonNetConfig;
use ton_lib::errors::TonlibError;
use ton_lib::types::ton_address::TonAddress;
use ton_lib::unwrap_lite_response;
use ton_liteapi::tl::request::Request;
use ton_liteapi::tl::response::Response;

#[tokio::test]
async fn test_lite_client() -> anyhow::Result<()> {
    let lite_client = make_lite_client(true).await?;

    // generic interface
    let mc_info_rsp = lite_client.exec(Request::GetMasterchainInfo, None).await?;
    let mc_info_generic = unwrap_lite_response!(mc_info_rsp, MasterchainInfo)?;
    assert_ne!(mc_info_generic.last.seqno, 0);

    // === specialized interface ===
    let mc_info = lite_client.get_mc_info().await?;
    assert_ne!(mc_info_generic.last.seqno, 0);

    let block_id = lite_client.lookup_mc_block(mc_info.last.seqno).await?;
    assert_eq!(block_id, mc_info.last);

    let usdt_addr = TonAddress::from_str("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs")?;
    let account = lite_client.get_account_state(&usdt_addr, mc_info.last.seqno, None).await?;
    assert!(account.as_account().is_some());

    Ok(())
}

#[tokio::test]
async fn test_lite_client_testnet() -> anyhow::Result<()> {
    let lite_client = make_lite_client(false).await?;
    let mc_info = lite_client.get_mc_info().await?;
    let usdt_addr = TonAddress::from_str("kQD4HpyO8ilPHHUV4CpiHMqz8F2eWyVOMH10MxTYrY3Emvmu")?;
    let account = lite_client.get_account_state(&usdt_addr, mc_info.last.seqno, None).await?;
    assert!(account.as_account().is_some());

    Ok(())
}

pub async fn make_lite_client(mainnet: bool) -> anyhow::Result<LiteClient> {
    init_logging();
    log::info!("initializing lite_client with mainnet={mainnet}...");
    let mut config = LiteClientConfig::new(&TonNetConfig::get_json(mainnet))?;
    config.retry_count = 20;
    config.retry_waiting = Duration::from_millis(200);
    Ok(LiteClient::new(config)?)
}
