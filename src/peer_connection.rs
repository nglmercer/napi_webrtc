use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::sync::Arc;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit as RTCIceCandidateInitInternal;
use webrtc::data_channel::RTCDataChannel;
use crate::data_channel::DataChannel;
use crate::ice::RTCIceCandidateInit;
use crate::api::RUNTIME;

#[napi]
pub struct PeerConnection {
    pub(crate) inner: Arc<RTCPeerConnection>,
}

#[napi]
impl PeerConnection {
    pub(crate) fn new(pc: RTCPeerConnection) -> Self {
        PeerConnection {
            inner: Arc::new(pc),
        }
    }

    #[napi]
    pub fn on_ice_candidate(&self, callback: ThreadsafeFunction<Option<String>>) -> Result<()> {
        let _guard = RUNTIME.enter();
        let callback = Arc::new(callback);
        self.inner.on_ice_candidate(Box::new(move |c| {
            let callback = callback.clone();
            Box::pin(async move {
                if let Some(candidate) = c {
                    let json_result = {
                        let _guard = RUNTIME.enter();
                        candidate.to_json()
                    };
                    match json_result {
                        Ok(internal_init) => {
                            let init = RTCIceCandidateInit {
                                candidate: internal_init.candidate,
                                sdp_mid: internal_init.sdp_mid,
                                sdp_mline_index: internal_init.sdp_mline_index,
                                username_fragment: internal_init.username_fragment,
                            };
                            if let Ok(json) = serde_json::to_string(&init) {
                                let _ = callback.call(Ok(Some(json)), ThreadsafeFunctionCallMode::NonBlocking);
                            }
                        }
                        Err(_) => {}
                    }
                } else {
                    let _ = callback.call(Ok(None), ThreadsafeFunctionCallMode::NonBlocking);
                }
            })
        }));
        Ok(())
    }

    #[napi]
    pub fn on_data_channel(&self, callback: ThreadsafeFunction<DataChannel>) -> Result<()> {
        let _guard = RUNTIME.enter();
        let callback = Arc::new(callback);
        self.inner.on_data_channel(Box::new(move |dc: Arc<RTCDataChannel>| {
            let callback = callback.clone();
            Box::pin(async move {
                let channel = DataChannel::new(dc);
                let _ = callback.call(Ok(channel), ThreadsafeFunctionCallMode::NonBlocking);
            })
        }));
        Ok(())
    }

    #[napi]
    pub async fn create_offer(&self) -> Result<String> {
        let offer = {
            let _guard = RUNTIME.enter();
            self.inner.create_offer(None)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to create offer: {}", e)))?;
        
        // Sets the LocalDescription, and starts our UDP listeners
        {
            let _guard = RUNTIME.enter();
            self.inner.set_local_description(offer.clone())
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to set local description: {}", e)))?;

        Ok(offer.sdp)
    }

    #[napi]
    pub async fn create_answer(&self) -> Result<String> {
        let answer = {
            let _guard = RUNTIME.enter();
            self.inner.create_answer(None)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to create answer: {}", e)))?;
        
        // Sets the LocalDescription, and starts our UDP listeners
        {
            let _guard = RUNTIME.enter();
            self.inner.set_local_description(answer.clone())
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to set local description: {}", e)))?;

        Ok(answer.sdp)
    }

    #[napi]
    pub async fn set_remote_description(&self, sdp: String, sdp_type: String) -> Result<()> {
        let mut desc = RTCSessionDescription::default();
        desc.sdp = sdp;
        desc.sdp_type = match sdp_type.to_lowercase().as_str() {
            "offer" => RTCSdpType::Offer,
            "answer" => RTCSdpType::Answer,
            "pranswer" => RTCSdpType::Pranswer,
            "rollback" => RTCSdpType::Rollback,
            _ => return Err(Error::from_reason(format!("Invalid SDP type: {}", sdp_type))),
        };
        {
            let _guard = RUNTIME.enter();
            self.inner.set_remote_description(desc)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to set remote description: {}", e)))?;
        Ok(())
    }

    #[napi]
    pub async fn add_ice_candidate(&self, candidate: RTCIceCandidateInit) -> Result<()> {
        let internal = RTCIceCandidateInitInternal {
            candidate: candidate.candidate,
            sdp_mid: candidate.sdp_mid,
            sdp_mline_index: candidate.sdp_mline_index,
            username_fragment: candidate.username_fragment,
        };
        {
            let _guard = RUNTIME.enter();
            self.inner.add_ice_candidate(internal)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to add ice candidate: {}", e)))?;
        Ok(())
    }

    #[napi]
    pub async fn create_data_channel(&self, label: String) -> Result<DataChannel> {
        let dc = {
            let _guard = RUNTIME.enter();
            self.inner.create_data_channel(&label, None)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to create data channel: {}", e)))?;
        Ok(DataChannel::new(dc))
    }

    #[napi]
    pub async fn close(&self) -> Result<()> {
        {
            let _guard = RUNTIME.enter();
            self.inner.close()
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to close peer connection: {}", e)))?;
        Ok(())
    }
}
