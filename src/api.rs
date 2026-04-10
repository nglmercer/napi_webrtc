use napi::bindgen_prelude::*;
use napi_derive::napi;
use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;
use crate::peer_connection::PeerConnection;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

pub static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

#[napi(js_name = "WebRTCAPI")]
pub struct WebRTCAPI {
    api: webrtc::api::API,
}

#[napi]
impl WebRTCAPI {
    #[napi(constructor)]
    pub fn new() -> Self {
        let api = APIBuilder::new().build();
        WebRTCAPI { api }
    }

    #[napi]
    pub async fn create_peer_connection(&self) -> Result<PeerConnection> {
        let config = RTCConfiguration::default();
        let pc = {
            let _guard = RUNTIME.enter();
            self.api.new_peer_connection(config)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to create peer connection: {}", e)))?;
        
        Ok(PeerConnection::new(pc))
    }
}
