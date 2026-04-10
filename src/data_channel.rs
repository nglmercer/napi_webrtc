use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::sync::Arc;
use webrtc::data_channel::RTCDataChannel;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit as InternalRTCDataChannelInit;
use crate::api::RUNTIME;

#[napi(object)]
pub struct RTCDataChannelInit {
    pub ordered: Option<bool>,
    pub max_packet_life_time: Option<u16>,
    pub max_retransmits: Option<u16>,
    pub protocol: Option<String>,
    pub negotiated: Option<u16>,
}

impl From<RTCDataChannelInit> for InternalRTCDataChannelInit {
    fn from(init: RTCDataChannelInit) -> Self {
        InternalRTCDataChannelInit {
            ordered: init.ordered,
            max_packet_life_time: init.max_packet_life_time,
            max_retransmits: init.max_retransmits,
            protocol: init.protocol,
            negotiated: init.negotiated,
        }
    }
}

#[napi]
pub struct DataChannel {
    pub(crate) inner: Arc<RTCDataChannel>,
}

#[napi]
impl DataChannel {
    pub(crate) fn new(dc: Arc<RTCDataChannel>) -> Self {
        DataChannel { inner: dc }
    }

    #[napi]
    pub fn on_open(&self, callback: ThreadsafeFunction<()>) -> Result<()> {
        let _guard = RUNTIME.enter();
        let callback = Arc::new(callback);
        self.inner.on_open(Box::new(move || {
            let callback = callback.clone();
            Box::pin(async move {
                let _ = callback.call(Ok(()), ThreadsafeFunctionCallMode::NonBlocking);
            })
        }));
        Ok(())
    }

    #[napi]
    pub fn on_message(&self, #[napi(ts_arg_type = "(err: any, data: string | Buffer) => void")] callback: ThreadsafeFunction<Either<String, Buffer>>) -> Result<()> {
        let _guard = RUNTIME.enter();
        let callback = Arc::new(callback);
        self.inner.on_message(Box::new(move |msg: DataChannelMessage| {
            let callback = callback.clone();
            Box::pin(async move {
                if msg.is_string {
                    let text = String::from_utf8_lossy(&msg.data).to_string();
                    let _ = callback.call(Ok(Either::A(text)), ThreadsafeFunctionCallMode::NonBlocking);
                } else {
                    let buffer = msg.data.to_vec();
                    let _ = callback.call(Ok(Either::B(Buffer::from(buffer))), ThreadsafeFunctionCallMode::NonBlocking);
                }
            })
        }));
        Ok(())
    }

    #[napi]
    pub async fn send(&self, data: String) -> Result<()> {
        {
            let _guard = RUNTIME.enter();
            self.inner.send_text(data)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to send text: {}", e)))?;
        Ok(())
    }

    #[napi]
    pub async fn send_buffer(&self, data: Buffer) -> Result<()> {
        let bytes = bytes::Bytes::copy_from_slice(&data);
        {
            let _guard = RUNTIME.enter();
            self.inner.send(&bytes)
        }.await
            .map_err(|e| Error::from_reason(format!("Failed to send binary: {}", e)))?;
        Ok(())
    }

    #[napi]
    pub fn label(&self) -> String {
        self.inner.label().to_string()
    }
}
