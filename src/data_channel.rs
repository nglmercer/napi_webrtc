use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::sync::Arc;
use webrtc::data_channel::RTCDataChannel;
use webrtc::data_channel::data_channel_message::DataChannelMessage;

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
        let callback = Arc::new(callback);
        self.inner.on_open(Box::new(move || {
            let callback = callback.clone();
            Box::pin(async move {
                let _ = callback.call((), ThreadsafeFunctionCallMode::NonBlocking);
            })
        }));
        Ok(())
    }

    #[napi]
    pub fn on_message(&self, #[napi(ts_arg_type = "(data: string) => void")] callback: ThreadsafeFunction<String>) -> Result<()> {
        let callback = Arc::new(callback);
        self.inner.on_message(Box::new(move |msg: DataChannelMessage| {
            let callback = callback.clone();
            Box::pin(async move {
                let text = String::from_utf8_lossy(&msg.data).to_string();
                let _ = callback.call(text, ThreadsafeFunctionCallMode::NonBlocking);
            })
        }));
        Ok(())
    }

    #[napi]
    pub async fn send(&self, data: String) -> Result<()> {
        self.inner.send_text(data).await
            .map_err(|e| Error::from_reason(format!("Failed to send text: {}", e)))?;
        Ok(())
    }

    #[napi]
    pub fn label(&self) -> String {
        self.inner.label().to_string()
    }
}
