use std::os::raw::c_void;

use crate::plugin::messages::Message;

/// Accessing and communicating with other plugins
pub mod management;

/// Inter-plugin messaging
pub mod messages;

/// Items used by the xplane_plugin! macro, which must be public
#[doc(hidden)]
pub mod internal;

/// Information about a plugin
pub struct PluginInfo {
    /// The plugin name
    pub name: String,
    /// The plugin's signature, in reverse DNS format
    pub signature: String,
    /// A description of the plugin
    pub description: String,
}

impl PluginInfo {
    pub fn new(
        name: impl Into<String>,
        signature: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            signature: signature.into(),
            description: description.into(),
        }
    }
}

/// The trait that all plugins should implement
pub trait Plugin: Sized {
    /// The error type that a plugin may encounter when starting up or enabling
    type Error: std::fmt::Display;

    /// Called when X-Plane loads this plugin
    ///
    /// On success, returns a plugin object
    fn start() -> Result<Self, Self::Error>;
    /// Called when the plugin is enabled
    ///
    /// If this function returns an Err, the plugin will remain disabled.
    ///
    /// The default implementation returns Ok(()).
    fn enable(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    /// Called when the plugin is disabled
    ///
    /// The default implementation does nothing.
    fn disable(&mut self) {}

    /// Returns information on this plugin
    fn info(&self) -> PluginInfo;

    #[allow(unused_variables)]
    /// Called when the plugin receives a message
    ///
    /// The default implementation does nothing.
    fn receive_message(&mut self, from: i32, message: Message, param: *mut c_void) {}
}

pub fn reload_plugins() {
    unsafe { xplm_sys::XPLMReloadPlugins() }
}
