use rodio::{Device, DeviceTrait};
use rodio::cpal::traits::HostTrait;
use serde::{Deserialize, Serialize};
use crate::TtsApp;

#[derive(Clone, Serialize, Deserialize)]
pub struct PSDevice {
    pub device_name: String,
}

impl PSDevice {
    pub fn new(device: Device) -> Self {
        Self {
            device_name: device.name().unwrap(),
        }
    }

    pub fn get_device_name(&self) -> &str {
        &self.device_name
    }

    pub fn get_device(&self) -> Device {
        TtsApp::get_devices().iter().find(|d| d.name().unwrap() == self.device_name).unwrap().clone()
    }
}

impl PartialEq for PSDevice {
    fn eq(&self, other: &Self) -> bool {
        self.device_name == other.device_name
    }
}
