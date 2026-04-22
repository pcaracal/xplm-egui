extern crate xplm_egui;

use xplm_egui::plugin::{Plugin, PluginInfo};
use xplm_egui::{debugln, xplane_plugin};

struct MinimalPlugin;

impl Plugin for MinimalPlugin {
    type Error = std::convert::Infallible;

    fn start() -> Result<Self, Self::Error> {
        // The following message should be visible in the developer console and the Log.txt file
        debugln!("Hello, World! From the Minimal Rust Plugin");
        Ok(MinimalPlugin)
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("Minimal Rust Plugin"),
            signature: String::from("org.samcrow.xplm.examples.minimal"),
            description: String::from("A plugin written in Rust"),
        }
    }
}

xplane_plugin!(MinimalPlugin);
