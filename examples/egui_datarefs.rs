//!
//! Show some datarefs in an egui window
//!

extern crate xplm_egui;

use egui::Widget;
use uom::{
    fmt::DisplayStyle,
    si::{angle::degree, length::foot, pressure::hectopascal, velocity::knot},
};
use xplm_egui::{
    data::typed::TypedDataRead,
    egui_window::{App, EguiWindow},
    geometry::ScreenRect,
    menu::{ActionItem, Menu, MenuClickHandler},
    plugin::{Plugin, PluginInfo},
    xplane_plugin,
};

struct EguiDatarefsPlugin {
    _menu: Menu,
}

xplm_egui::uom_typed_dataref!(
    name: angle,
    type: uom::si::angle::Angle,
    unit: uom::si::angle::degree,
    range: -360.0..=360.0
);

xplm_egui::uom_typed_dataref!(
    name: velocity_mps,
    type: uom::si::velocity::Velocity,
    unit: uom::si::velocity::meter_per_second,
    range: f64::MIN..f64::MAX
);

xplm_egui::uom_typed_dataref!(
    name: velocity_kt,
    type: uom::si::velocity::Velocity,
    unit: uom::si::velocity::knot,
    range: f64::MIN..f64::MAX
);

xplm_egui::uom_typed_dataref!(
    name: length,
    type: uom::si::length::Length,
    unit: uom::si::length::meter,
    range: f64::MIN..f64::MAX
);

xplm_egui::uom_typed_dataref!(
    name: pressure,
    type: uom::si::pressure::Pressure,
    unit: uom::si::pressure::pascal,
    range: f64::MIN..f64::MAX
);

struct Datarefs {
    latitude: angle::DataRef<f64>,
    longitude: angle::DataRef<f64>,
    elevation: length::DataRef<f64>,
    ground_speed: velocity_mps::DataRef<f32>,
    ias: velocity_kt::DataRef<f32>,
    tas: velocity_mps::DataRef<f32>,
    mag_heading: angle::DataRef<f32>,
    true_heading: angle::DataRef<f32>,
    qnh: pressure::DataRef<f32>,
}

#[derive(Default, Clone, Copy)]
struct Data {
    latitude: uom::si::f64::Angle,
    longitude: uom::si::f64::Angle,
    elevation: uom::si::f64::Length,
    ground_speed: uom::si::f32::Velocity,
    tas: uom::si::f32::Velocity,
    ias: uom::si::f32::Velocity,
    mag_heading: uom::si::f32::Angle,
    true_heading: uom::si::f32::Angle,
    qnh: uom::si::f32::Pressure,
}

impl Datarefs {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            latitude: angle::DataRef::find("sim/flightmodel/position/latitude")?,
            longitude: angle::DataRef::find("sim/flightmodel/position/longitude")?,
            elevation: length::DataRef::find("sim/flightmodel/position/elevation")?,
            ground_speed: velocity_mps::DataRef::find("sim/flightmodel/position/groundspeed")?,
            tas: velocity_mps::DataRef::find("sim/flightmodel/position/true_airspeed")?,
            ias: velocity_kt::DataRef::find("sim/flightmodel/position/indicated_airspeed")?,
            mag_heading: angle::DataRef::find("sim/flightmodel/position/mag_psi")?,
            true_heading: angle::DataRef::find("sim/flightmodel/position/true_psi")?,
            qnh: pressure::DataRef::find("sim/weather/aircraft/qnh_pas")?,
        })
    }

    fn data(&self) -> Data {
        Data {
            latitude: self.latitude.get().unwrap_or_default(),
            longitude: self.longitude.get().unwrap_or_default(),
            elevation: self.elevation.get().unwrap_or_default(),
            ground_speed: self.ground_speed.get().unwrap_or_default(),
            tas: self.tas.get().unwrap_or_default(),
            ias: self.ias.get().unwrap_or_default(),
            mag_heading: self.mag_heading.get().unwrap_or_default(),
            true_heading: self.true_heading.get().unwrap_or_default(),
            qnh: self.qnh.get().unwrap_or_default(),
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
struct EguiApp {
    striped: bool,
    overline: bool,
    resizable: bool,
    clickable: bool,

    datarefs: Datarefs,
}

impl App for EguiApp {
    #[allow(clippy::too_many_lines)]
    fn ui(&mut self, ui: &mut egui::Ui, _: &xplm_egui::window::Window) {
        egui::CentralPanel::default_margins().show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.striped, "Striped");
                ui.checkbox(&mut self.overline, "Overline");
                ui.checkbox(&mut self.resizable, "Resizable");
                ui.checkbox(&mut self.clickable, "Clickable");
            });

            ui.vertical(|ui| {
                let data = self.datarefs.data();

                let mut table = egui_extras::TableBuilder::new(ui)
                    .striped(self.striped)
                    .resizable(self.resizable)
                    .cell_layout(
                        egui::Layout::left_to_right(egui::Align::Center).with_cross_justify(true),
                    )
                    .auto_shrink(false)
                    .column(egui_extras::Column::auto())
                    .column(egui_extras::Column::auto());

                if self.clickable {
                    table = table.sense(egui::Sense::click());
                }

                let names = [
                    "Latitude",
                    "Longitude",
                    "Elevation",
                    "Ground Speed",
                    "TAS",
                    "IAS",
                    "Mag Heading",
                    "True Heading",
                    "QNH",
                ];

                let values = [
                    format!(
                        "{:.6}",
                        data.latitude
                            .into_format_args(degree, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.6}",
                        data.longitude
                            .into_format_args(degree, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.1}",
                        data.elevation
                            .into_format_args(foot, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.2}",
                        data.ground_speed
                            .into_format_args(knot, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.2}",
                        data.tas.into_format_args(knot, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.2}",
                        data.ias.into_format_args(knot, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.1}",
                        data.mag_heading
                            .into_format_args(degree, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.1}",
                        data.true_heading
                            .into_format_args(degree, DisplayStyle::Abbreviation)
                    ),
                    format!(
                        "{:.2}",
                        data.qnh
                            .into_format_args(hectopascal, DisplayStyle::Abbreviation)
                    ),
                ];

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Value");
                        });
                    })
                    .body(|mut body| {
                        for (name, value) in names.into_iter().zip(values) {
                            body.row(18.0, |mut row| {
                                row.set_overline(self.overline);
                                row.col(|ui| {
                                    egui::Label::new(egui::RichText::new(name))
                                        .selectable(false)
                                        .ui(ui);
                                });
                                row.col(|ui| {
                                    egui::Label::new(egui::RichText::new(value))
                                        .selectable(false)
                                        .ui(ui);
                                });
                            });
                        }
                    });
            });
        });
    }
}

impl Plugin for EguiDatarefsPlugin {
    type Error = anyhow::Error;

    fn start() -> anyhow::Result<Self> {
        let datarefs = Datarefs::new()?;

        // Window size
        let width = 200;
        let height = 200;

        // Screen to position the window on
        let screen = xplm_egui::display::get_screen_bounds_global();

        // rect centered on the screen with the specified width and height
        let window_geometry = ScreenRect::zero()
            .translate(screen.center().to_vector())
            .inflate(width, height);

        let window = EguiWindow::new(
            EguiApp {
                striped: true,
                overline: true,
                resizable: true,
                clickable: true,
                datarefs,
            },
            window_geometry,
        )?;

        // EguiWindow is a wrapper around a regular xplane window, so all the same apis can be used

        // Anything except \0
        window.set_title("Egui Datarefs")?;

        // Minimum 100x100, no maximum
        window.set_resizing_limits(100, 100, None, None);

        // Menu boilerplate
        let plugins_submenu = Menu::new("Egui Datarefs Example")?;
        plugins_submenu.add_child(ActionItem::new(
            "Toggle egui window",
            ActionHandlerImpl(window),
        )?);
        plugins_submenu.add_to_plugins_menu();

        // The menu needs to be part of the plugin struct, or it will immediately get dropped and will not appear
        Ok(EguiDatarefsPlugin {
            _menu: plugins_submenu,
        })
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("Rust Egui Plugin"),
            signature: String::from("ch.pcaracal.xplm-egui.examples.egui_datarefs"),
            description: String::from(
                "A plugin written in Rust that creates an egui window to show some datarefs",
            ),
        }
    }
}

xplane_plugin!(EguiDatarefsPlugin);

struct ActionHandlerImpl(EguiWindow);

impl MenuClickHandler for ActionHandlerImpl {
    fn item_clicked(&mut self, _: &ActionItem) {
        self.0.set_visible(!self.0.visible());
    }
}
