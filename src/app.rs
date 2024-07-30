use eframe::egui;
use egui::Align2;
use serde::{Deserialize, Serialize};
use crate::{Elabs, Voice};

pub const APP_KEY: &str = "please_speak";

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct TtsApp {
    api_key: String,
    #[serde(skip)]
    elabs: Elabs,

    text: String,

    voice: Voice,
    #[serde(skip)]
    voices: Vec<Voice>,

    #[serde(skip)]
    settings_modal: bool,

    #[serde(skip)]
    api_error_manager: ErrorManager,
    #[serde(skip)]
    elabs_error_manager: ErrorManager,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
struct ErrorManager {
    #[serde(skip)]
    error_rx: async_channel::Receiver<String>,
    #[serde(skip)]
    last_error: Option<String>,
    #[serde(skip)]
    modal_open: bool,
    #[serde(skip)]
    name: String,
}

impl Default for ErrorManager {
    fn default() -> Self {
        let (_error_tx, error_rx) = async_channel::unbounded();
        Self {
            error_rx,
            last_error: None,
            modal_open: false,
            name: "Error".to_owned(),
        }
    }
}

impl ErrorManager {
    fn new(name: String, error_rx: async_channel::Receiver<String>) -> Self {
        Self {
            error_rx,
            last_error: None,
            modal_open: false,
            name,
        }
    }

    fn update(&mut self, ctx: &egui::Context) {
        if let Ok(error) = self.error_rx.try_recv() {
            println!("[{}]: Error: {:?}", self.name, error);
            self.last_error = Some(error);
            self.modal_open = true;
        }

        if let Some(error) = &self.last_error {
            egui::Window::new(&self.name)
                .resizable(false)
                .pivot(Align2::CENTER_CENTER)
                .open(&mut self.modal_open)
                .show(ctx, |ui| {
                    ui.label(format!("Error: {:?}", error));
                });

            if !self.modal_open {
                self.last_error = None;
            }
        }
    }
}

impl Default for TtsApp {
    fn default() -> Self {
        let (api_error_tx, api_error_rx) = async_channel::unbounded();
        let (elabs_error_tx, elabs_error_rx) = async_channel::unbounded();
        Self {
            api_key: "your_api_key".to_owned(),
            elabs: Elabs::new(api_error_tx, elabs_error_tx),

            text: "Hello World!".to_owned(),
            voice: Voice::default(),
            voices: Vec::new(),

            settings_modal: false,
            api_error_manager: ErrorManager::new("Api error".to_string(), api_error_rx),
            elabs_error_manager: ErrorManager::new("Elabs error".to_string(), elabs_error_rx),
        }
    }
}

impl TtsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, APP_KEY).unwrap_or_default();
        }

        TtsApp::default()
    }

    pub fn init(&mut self) {
        self.elabs.init(self.api_key.clone());

        if !self.elabs.connected() {
            return ;
        }

        self.load_api_resources();
    }

    pub fn load_api_resources(&mut self) {
        if !self.elabs.connected() {
            return
        }

        self.voices = self.elabs.get_voices_sync(true).unwrap_or_default();
    }
}

impl eframe::App for TtsApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Settings").clicked() {
                            self.settings_modal = true;
                            ui.close_menu();
                        }

                        ui.separator();

                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Please Speak - Powered by Elenlabs");

            if false {
                ui.label("Please enter your API Key to get started.");
                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    ui.text_edit_singleline(&mut self.api_key);
                    if ui.button("Submit").clicked() {
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add_sized([ui.available_size().x, 150.], egui::TextEdit::multiline(&mut self.text));
                });

                egui::ComboBox::from_label("Select a voice")
                    .selected_text(format!("Voice: {}", self.voice.get_voice_name()))
                    .show_ui(ui, |ui| {
                        for voice in &self.voices {
                            ui.selectable_value(&mut self.voice, voice.clone(), voice.get_voice_name());
                        }
                    });
                ui.end_row();

                if ui.button("Speak").clicked() {
                    // let client = self.eleven_labs_client.clone();
                    // let text = self.text.clone();
                    // let voice = self.voice.clone();
                    // let tx = self.speech_tx.clone();
                    // let handle = self.rt_handle.clone();
                    //
                    // handle.spawn(async move {
                    //     if let Some(client) = client {
                    //         let body = TextToSpeechBody::new(&text, Model::ElevenMultilingualV2);
                    //         let endpoint = TextToSpeech::new(voice.voice_id, body);
                    //
                    //         match client.hit(endpoint).await {
                    //             Ok(speech) => {
                    //                 let _ = play(speech);
                    //                 let _ = tx.send("Speech played successfully".to_string()).await;
                    //             }
                    //             Err(e) => {
                    //                 eprintln!("Error: {:?}", e);
                    //                 let _ = tx.send(format!("Error: {:?}", e)).await;
                    //             }
                    //         }
                    //     }
                    // });
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        if self.settings_modal {
            egui::Window::new("Set API Key")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter your API Key:");
                    ui.text_edit_singleline(&mut self.api_key);

                    if ui.button("Done").clicked() {
                        self.settings_modal = false;
                        self.elabs.init(self.api_key.clone());
                        self.load_api_resources();
                    }
                });
        }

        self.api_error_manager.update(ctx);
        self.elabs_error_manager.update(ctx);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(" | ");
        ui.add(egui::github_link_file!(
            "https://github.com/malallai/please_speak/blob/main/",
            "Source code."
        ));
    });
}
