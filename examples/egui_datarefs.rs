//!
//! Show some datarefs in an egui window
//!

extern crate xplm_egui;

use egui_extras::{Column, TableBuilder};
use xplm_egui::{
    data::typed::{TypedDataRead, uom_util::FromUom},
    debugln,
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

struct EguiApp {
    datarefs: Datarefs,
}

impl App for EguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &xplm_egui::window::Window) {
        egui::CentralPanel::default_margins().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                let data = self.datarefs.data();

                TableBuilder::new(ui)
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .resizable(false)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.heading("Name");
                        });
                        header.col(|ui| {
                            ui.heading("Value");
                        });
                    })
                    .body(|mut body| {
                        for (name, value) in [
                            ("Latitude", format!("{:.6}°", data.latitude.degrees_f64())),
                            ("Longitude", format!("{:.6}°", data.longitude.degrees_f64())),
                            ("Elevation", format!("{:.2} ft", data.elevation.feet_f64())),
                            (
                                "Ground Speed",
                                format!("{:.2} kt", data.ground_speed.knots_f32()),
                            ),
                            ("True Airspeed", format!("{:.2} kt", data.tas.knots_f32())),
                            (
                                "Indicated Airspeed",
                                format!("{:.2} kt", data.ias.knots_f32()),
                            ),
                            (
                                "Magnetic Heading",
                                format!("{:.2}°", data.mag_heading.degrees_f32()),
                            ),
                            (
                                "True Heading",
                                format!("{:.2}°", data.true_heading.degrees_f32()),
                            ),
                            ("QNH", format!("{:.2} hPa", data.qnh.hectopascal_f32())),
                        ] {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(name);
                                });
                                row.col(|ui| {
                                    ui.label(value);
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
        if let Err(why) = init_log() {
            debugln!("Failed to initialize logger: {why}");
        }

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

        let window = EguiWindow::new(EguiApp { datarefs }, window_geometry)?;

        // EguiWindow is a wrapper around a regular xplane window, so all the same apis can be used

        // Anything except \0
        window.set_title("Egui Datarefs")?;

        // Minimum 100x100, no maximum
        window.set_resizing_limits(100, 100, None, None);

        // Menu boilerplate
        let plugins_submenu = Menu::new("Egui Test Plugin")?;
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

// log to 'X-Plane 12/Resources/plugins/xplm-egui-example/log.txt' for tail -F
fn init_log() -> anyhow::Result<()> {
    let path = std::env::current_dir()?.join("Resources/plugins/xplm-egui-example/log.txt");

    simplelog::WriteLogger::init(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        std::fs::File::create(path)?,
    )
    .map_err(Into::into)
}
