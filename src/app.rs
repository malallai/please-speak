use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use eframe::egui;
use elevenlabs_rs::{Bytes};
use elevenlabs_rs::utils::{play, save};
use serde::{Deserialize, Serialize};
use crate::{Elabs, ErrorManager, Voice};

pub const APP_KEY: &str = "please_speak";

pub struct TtsApp {
    configuration: Configuration,

    elabs: Elabs,
    voices: Vec<Voice>,
    last_generated: Option<Bytes>,
    last_generated_file_name: String,
    last_generated_file_path: String,

    settings_modal: bool,

    api_error_manager: ErrorManager,
    elabs_error_manager: ErrorManager,

    voices_loading_rx: Receiver<Vec<Voice>>,
    voices_loading_tx: Sender<Vec<Voice>>,
    voices_loading: bool,

    generate_loading_rx: Receiver<Bytes>,
    generate_loading_tx: Sender<Bytes>,
    generate_loading: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct Configuration {
    api_key: String,
    text: String,
    voice: Voice,
    save_to: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            api_key: "".to_owned(),
            text: "Hello World!".to_owned(),
            voice: Voice::default(),
            save_to: "".to_owned(),
        }
    }
}

impl TtsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (api_error_tx, api_error_rx) = async_channel::unbounded();
        let (elabs_error_tx, elabs_error_rx) = async_channel::unbounded();

        let (voices_loading_tx, voices_loading_rx) = channel();
        let (generate_loading_tx, generate_loading_rx) = channel();

        let mut configuration: Configuration = Configuration::default();
        if let Some(storage) = cc.storage {
            configuration = eframe::get_value(storage, APP_KEY).unwrap_or_default();
        }

        let elabs = Elabs::new(api_error_tx, elabs_error_tx);
        Self {
            configuration,
            elabs,
            voices: Vec::new(),
            last_generated: None,
            last_generated_file_name: "".to_string(),
            last_generated_file_path: "".to_string(),
            settings_modal: false,
            api_error_manager: ErrorManager::new("Api error".to_string(), api_error_rx),
            elabs_error_manager: ErrorManager::new("Elabs error".to_string(), elabs_error_rx),

            voices_loading_rx,
            voices_loading_tx,
            voices_loading: false,

            generate_loading_rx,
            generate_loading_tx,
            generate_loading: false,
        }
    }

    pub fn init(&mut self) {
        self.elabs.init(self.configuration.api_key.clone());

        if !self.elabs.connected() {
            return;
        }

        self.load_api_resources();
        self.security_checks();
    }

    pub fn load_api_resources(&mut self) {
        if !self.elabs.connected() {
            return
        }

        self.voices_loading = true;

        let app = Arc::new(self);
        let elabs = app.elabs.clone();
        let tx = app.voices_loading_tx.clone();
        std::thread::spawn(move || {
            let voices = elabs.run_sync(|elabs| {
                elabs.get_voices(false)
            });

            if let Some(voices) = voices {
                tx.send(voices).unwrap()
            }
        });
    }

    pub fn security_checks(&mut self) {
        if self.configuration.save_to.is_empty() {
            self.configuration.save_to = std::env::temp_dir().to_str().unwrap().to_string();
        }

        let path = Path::new(&self.configuration.save_to);
        if !path.exists() {
            fs::create_dir_all(path).unwrap();
        }
    }

    pub fn generate(&mut self) {
        if !self.elabs.connected() {
            return
        }

        self.generate_loading = true;

        let app = Arc::new(self);

        let elabs = app.elabs.clone();
        let tx = app.generate_loading_tx.clone();

        let text = app.configuration.text.clone();
        let voice = app.configuration.voice.clone();

        std::thread::spawn(move || {
            let voices = elabs.run_sync(|elabs| {
                elabs.generate_speak(text, voice, true)
            });

            if let Some(voices) = voices {
                tx.send(voices).unwrap()
            }
        });
    }
}

impl eframe::App for TtsApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, &self.configuration);
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

            if self.configuration.api_key.is_empty() || !self.elabs.connected() {
                ui.label("Please enter your API Key to get started.");
                ui.horizontal(|ui| {
                    ui.label("API Key:");
                    ui.text_edit_singleline(&mut self.configuration.api_key);
                    if ui.button("Submit").clicked() {
                        self.elabs.init(self.configuration.api_key.clone());
                        self.load_api_resources();
                        self.security_checks();
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    ui.add_sized([ui.available_size().x, 150.], egui::TextEdit::multiline(&mut self.configuration.text));
                });

                if self.voices_loading {
                    ui.horizontal(|ui| {
                        ui.label("Loading voices...");
                        ui.spinner();
                    });
                } else {
                    egui::ComboBox::from_label("Select a voice")
                        .selected_text(format!("Voice: {}", self.configuration.voice.get_voice_name()))
                        .show_ui(ui, |ui| {
                            for voice in &self.voices {
                                ui.selectable_value(&mut self.configuration.voice, voice.clone(), voice.get_voice_name());
                            }
                        });
                }
                ui.end_row();

                ui.horizontal(|ui| {
                    if ui.button("Generate").clicked() {
                        self.generate();
                    }
                    if self.generate_loading {
                        ui.label("Generating...");
                        ui.spinner();
                    }
                });

                ui.end_row();

                if self.last_generated.is_some() {
                    ui.label(self.last_generated_file_path.clone());
                    ui.horizontal(|ui| {
                        if ui.button("Play for me").clicked() {
                            play(self.last_generated.as_ref().unwrap().clone());
                        }

                        if ui.button("Send to Soundboard").clicked() {

                        }

                        if ui.button("Save").clicked() {
                            self.last_generated_file_path = format!("{}/{}", self.configuration.save_to, &self.last_generated_file_name);
                            println!("Saving to: {}", self.last_generated_file_path);
                            save(&self.last_generated_file_path, self.last_generated.as_ref().unwrap().clone()).unwrap();
                        }
                    });
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
                    let falsified_key = "*".repeat(self.configuration.api_key.clone().len());
                    ui.text_edit_singleline(&mut falsified_key.to_string());

                    ui.separator();

                    ui.label("Save to:");
                    ui.text_edit_singleline(&mut self.configuration.save_to);

                    if ui.button("Done").clicked() {
                        self.settings_modal = false;
                        self.elabs.init(self.configuration.api_key.clone());
                        self.load_api_resources();
                        self.security_checks();
                    }
                });
        }

        self.api_error_manager.update(ctx);
        self.elabs_error_manager.update(ctx);

        if let Ok(voices) = self.voices_loading_rx.try_recv() {
            self.voices = voices;
            self.voices_loading = false;
        }

        if let Ok(bytes) = self.generate_loading_rx.try_recv() {
            self.generate_loading = false;
            self.last_generated = Some(bytes.clone());

            self.last_generated_file_name = format!("{}_{}.wav", self.configuration.voice.get_voice_name(), chrono::Local::now().format("%Y-%m-%d_%H-%M-%S-%3f"));
            self.last_generated_file_path = format!("/tmp/{}", &self.last_generated_file_name);
            save(&self.last_generated_file_path, bytes).unwrap();
        }
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
