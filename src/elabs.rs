use std::future::Future;
use async_channel::Sender;
use elevenlabs_rs::{Bytes, ElevenLabsClient, Model, TextToSpeech, TextToSpeechBody};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct Elabs {
    eleven_labs_client: Option<ElevenLabsClient>,
    connected: bool,
    api_error_tx: Sender<String>,
    elabs_error_tx: Sender<String>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct Voice {
    voice_id: String,
    voice_name: String,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            voice_id: "2EiwWnXFnvU5JabPnv8n".to_string(),
            voice_name: "Clyde".to_string(),
        }
    }
}

impl Voice {
    pub fn get_voice_id(&self) -> &str {
        &self.voice_id
    }

    pub fn get_voice_name(&self) -> &str {
        &self.voice_name
    }
}

impl Elabs {
    pub fn new(api_error_tx: Sender<String>, elabs_error_tx: Sender<String>) -> Self {
        Self {
            eleven_labs_client: None,
            connected: false,
            api_error_tx,
            elabs_error_tx,
        }
    }

    pub fn run_sync<'a, F, Fut>(&'a self, method: F) -> Fut::Output
    where
        F: FnOnce(&'a Elabs) -> Fut,
        Fut: Future,
    {
        let rt = Runtime::new().unwrap();
        rt.block_on(method(self))
    }

    pub fn init(&mut self, api_key: String) {
        self.eleven_labs_client = Some(ElevenLabsClient::new(api_key));
        if self.run_sync(|elabs| {
            elabs.get_voices(false)
        }).is_some() {
            self.connected = true;
            return;
        }

        self.run_sync(|elabs| {
            elabs.capture_error("ElevenLabsClient not initialized, please set your API key on the settings page")
        });

        self.connected = false;
    }

    pub fn connected(&self) -> bool {
        self.connected
    }

    pub async fn capture_error(&self, error: &str) {
        let _ = self.elabs_error_tx.send(error.to_string()).await;
    }

    pub async fn get_voices(&self, raise: bool) -> Option<Vec<Voice>> {
        if let Some(client) = &self.eleven_labs_client {
            match client.hit(elevenlabs_rs::GetVoices).await {
                Ok(result) => Some(
                    result
                        .get_voices()
                        .iter()
                        .map(|voice| Voice {
                            voice_id: voice.get_voice_id().to_string(),
                            voice_name: voice.get_name().to_string(),
                        }).collect()
                ),
                Err(e) => {
                    if raise {
                        let _ = self.api_error_tx.send(format!("API Error: {:?}", e)).await;
                    }
                    None
                }
            }
        } else {
            let _ = self.elabs_error_tx.send("ElevenLabsClient not initialized".to_string()).await;
            None
        }
    }

    pub async fn generate_speak(&self, text: String, voice: Voice, raise: bool) -> Option<Bytes> {
        if let Some(client) = &self.eleven_labs_client {
            let body = TextToSpeechBody::new(text.as_str(), Model::ElevenMultilingualV2);
            let endpoint = TextToSpeech::new(voice.get_voice_id(), body);

            match client.hit(endpoint).await {
                Ok(bytes) => Some(bytes),
                Err(e) => {
                    if raise {
                        let _ = self.api_error_tx.send(format!("API Error: {:?}", e)).await;
                    }
                    None
                }
            }
        } else {
            let _ = self.elabs_error_tx.send("ElevenLabsClient not initialized".to_string()).await;
            None
        }
    }

}
