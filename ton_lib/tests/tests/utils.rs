use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Root};
use log4rs::Config;
use std::fs::{exists, File};
use std::io::Read;
use std::sync::Once;
use ton_lib::net_config::{TON_NET_CONF_MAINNET, TON_NET_CONF_TESTNET};
static LOG: Once = Once::new();

pub fn get_net_conf(mainnet: bool) -> anyhow::Result<String> {
    let mut net_conf = match mainnet {
        true => TON_NET_CONF_MAINNET.to_string(),
        false => TON_NET_CONF_TESTNET.to_string(),
    };

    if let Ok(path) = std::env::var("TON_NET_CONF_PATH") {
        if exists(&path)? {
            let mut new_conf = String::new();
            let mut file = File::open(&path)?;
            file.read_to_string(&mut new_conf)?;
            net_conf = new_conf;
            log::info!("Using TON_NET_CONF from {path}")
        } else {
            log::warn!("env_var TON_NET_CONF_PATH is set, but path {path} is not available");
        }
    }
    Ok(net_conf)
}

pub(crate) fn init_logging() {
    LOG.call_once(|| {
        let stderr = ConsoleAppender::builder()
            .target(Target::Stderr)
            .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
                "{d(%Y-%m-%d %H:%M:%S%.6f)} {T:>15.15} {h({l:>5.5})} {t}:{L} - {m}{n}",
            )))
            .build();

        let config = Config::builder()
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .build(Root::builder().appender("stderr").build(LevelFilter::Info))
            .unwrap();

        log4rs::init_config(config).unwrap();
    })
}
