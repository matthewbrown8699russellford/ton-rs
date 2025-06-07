use crate::errors::TonlibError;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use std::ffi::CString;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct EmulatorBCConfig(Arc<CString>);

impl Deref for EmulatorBCConfig {
    type Target = CString;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Arc<CString>> for EmulatorBCConfig {
    fn from(config: Arc<CString>) -> Self { Self(config) }
}

impl EmulatorBCConfig {
    pub fn from_boc(config_boc: &[u8]) -> Result<Self, TonlibError> {
        Self::from_boc_base64(&STANDARD.encode(config_boc))
    }
    pub fn from_boc_hex(config_boc_hex: &str) -> Result<Self, TonlibError> {
        Self::from_boc_base64(&STANDARD.encode(hex::decode(config_boc_hex)?))
    }
    pub fn from_boc_base64(config_boc_base64: &str) -> Result<Self, TonlibError> {
        Ok(Self(Arc::new(CString::new(config_boc_base64)?)))
    }
}
