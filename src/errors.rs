use async_channel::Receiver;
use eframe::egui;
use egui::Align2;

#[derive(Clone)]
pub struct ErrorManager {
    error_rx: Receiver<String>,
    last_error: Option<String>,
    modal_open: bool,
    name: String,
}

impl ErrorManager {
    pub(crate) fn new(name: String, error_rx: Receiver<String>) -> Self {
        Self {
            error_rx,
            last_error: None,
            modal_open: false,
            name,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
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