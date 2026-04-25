//!
//! This plugin creates an egui window in xplane
//! Endless possibilities...
//!

extern crate xplm_egui;

use xplm_egui::{
    debugln,
    egui_window::{App, EguiWindow},
    geometry::ScreenRect,
    menu::{ActionItem, Menu, MenuClickHandler},
    plugin::{Plugin, PluginInfo},
    xplane_plugin,
};

struct EguiPlugin {
    _menu: Menu,
}

#[derive(Default)]
struct EguiApp {
    count: i32,
    text: String,

    cols: i32,
    rows: i32,
    cell_clicked: Option<(i32, i32)>,
}

impl App for EguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _: &xplm_egui::window::Window) {
        egui::CentralPanel::default_margins().show_inside(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                ui.heading("Hello from egui!");

                if ui.button(format!("Click {}", self.count)).clicked() {
                    self.count += 1;
                }

                ui.text_edit_singleline(&mut self.text);
                ui.label(format!("Text: {}", self.text));

                ui.horizontal_wrapped(|ui| {
                    if ui.button("Add column").clicked() {
                        self.cols += 1;
                        self.rows = self.rows.max(1);
                    }
                    if ui.button("Add row").clicked() {
                        self.rows += 1;
                        self.cols = self.cols.max(1);
                    }
                    if let Some((x, y)) = self.cell_clicked {
                        ui.strong(format!("{x}-{y}"));
                    }
                });

                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    egui::Grid::new("buttons grid").show(ui, |ui| {
                        for r in 0..self.rows {
                            for c in 0..self.cols {
                                if ui.button(format!("{c}-{r}")).clicked() {
                                    self.cell_clicked = Some((c, r));
                                    log::info!("Cell {c}-{r} clicked");
                                }
                            }
                            ui.end_row();
                        }
                    });
                });
            });
        });
    }
}

impl Plugin for EguiPlugin {
    type Error = anyhow::Error;

    fn start() -> anyhow::Result<Self> {
        if let Err(why) = init_log() {
            debugln!("Failed to initialize logger: {why}");
        }

        // Window size
        let width = 200;
        let height = 200;

        // Screen to position the window on
        let screen = xplm_egui::display::get_screen_bounds_global();

        // rect centered on the screen with the specified width and height
        let window_geometry = ScreenRect::zero()
            .translate(screen.center().to_vector())
            .inflate(width, height);

        let window = EguiWindow::new(EguiApp::default(), window_geometry)?;

        // EguiWindow is a wrapper around a regular xplane window, so all the same apis can be used

        // Anything except \0
        window.set_title("Egui Window")?;

        // Minimum 100x100, no maximum
        window.set_resizing_limits(100, 100, None, None);

        // Menu boilerplate
        let plugins_submenu = Menu::new("Egui Example")?;
        plugins_submenu.add_child(ActionItem::new(
            "Show egui window",
            ActionHandlerImpl(window),
        )?);
        plugins_submenu.add_to_plugins_menu();

        // The menu needs to be part of the plugin struct, or it will immediately get dropped and will not appear
        Ok(EguiPlugin {
            _menu: plugins_submenu,
        })
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("Rust Egui Plugin"),
            signature: String::from("ch.pcaracal.xplm-egui.examples.egui"),
            description: String::from("A plugin written in Rust that creates an egui window"),
        }
    }
}

xplane_plugin!(EguiPlugin);

struct ActionHandlerImpl(EguiWindow);

impl MenuClickHandler for ActionHandlerImpl {
    fn item_clicked(&mut self, _: &ActionItem) {
        let vis = !self.0.visible();
        self.0.set_visible(vis);
        log::info!("Egui window visible: {vis}");
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
