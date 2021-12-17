mod about_window;
use about_window::AboutWindow;
use eframe::{egui, epi};

/// It derives Deserialize/Serialize so it can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
/// New fields are are given default values when deserializing old state.
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    value: f32,

    #[cfg_attr(feature = "persistence", serde(skip))]
    about_window: AboutWindow,
}

/// Implement trait to create default window.
impl Default for App {
    /// Create default window.
    fn default() -> Self {
        App {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            about_window: AboutWindow::default(),
        }
    }
}

/// Define function for running app natively and on web.
impl epi::App for App {
    /// Provides the name of the window.
    fn name(&self) -> &str {
        "Online Radio"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per
    /// second.  Put your widgets into a `SidePanel`, `TopPanel`,
    /// `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let Self {
            label,
            value,
            about_window,
        } = self;

        // Show the about window when the menu item is pressed.
        about_window.update(ctx, frame);

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                // Add a menu bar category for the current file/page.
                egui::menu::menu(ui, "File", |ui| {
                    // Add a menu item for quitting the application.
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                // Add a menu bar category for showing iformation about the app.
                egui::menu::menu(ui, "Help", |ui| {
                    // Add a menu item for shoowing the information.
                    if ui.button("About").clicked() {
                        // Toggle the window on and off.
                        self.about_window.is_open = !self.about_window.is_open;
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(label);
            });

            ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                *value += 1.0;
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and
            // SidePanel's

            ui.heading("Online Radio");
            egui::warn_if_debug_build(ui);
        });
    }
}
