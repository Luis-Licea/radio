use eframe::{egui, epi};

/// The About window shows information about the application, such as creator
/// names, source code links, and technologies used.
/// It derives Deserialize/Serialize so it can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
/// New fields are are given default values when deserializing old state.
// #[cfg_attr(feature = "persistence", serde(default))]
pub struct AboutWindow {
    /// Wether the window is open or closed.
    pub is_open: bool,
}

/// Implement trait to create default window.
impl Default for AboutWindow {
    /// Create default window.
    fn default() -> Self {
        AboutWindow {
            // Set the window closed by default.
            is_open: false,
        }
    }
}

/// Define function for running app natively and on web.
impl epi::App for AboutWindow {
    /// Provides the name of the window.
    fn name(&self) -> &str {
        "About"
    }

    /// Called each time the UI needs repainting
    fn update(&mut self, ctx: &eframe::egui::CtxRef, _frame: &mut eframe::epi::Frame<'_>) {
        // Create an About window. The window only pops up when the About menu
        // itme is pressed.
        egui::Window::new(self.name())
            .open(&mut self.is_open)
            .show(ctx, |ui| {
                // Display the name of the application.
                ui.vertical_centered(|ui| {
                    ui.heading("ℹ Online Radio");
                });

                // Display the name of the creators.
                ui.label("🔨 Created by Luis David Licea Torres.");

                // Display the source code link.
                ui.horizontal(|ui| {
                    // Remove the horizontal spacing so that labels and
                    // hyperlinks are next to each other.
                    ui.spacing_mut().item_spacing.x = 0.0;

                    ui.label(" Source code available at ");
                    ui.hyperlink_to(
                        "github.com/Luis-Licea/radio",
                        "https://github.com/Luis-Licea/radio",
                    );
                    ui.label(".");
                });

                // Display the techonologies used to create the application.
                // Powered by ...
                ui.horizontal(|ui| {
                    // Remove the horizontal spacing so that labels and
                    // hyperlinks are next to each other.
                    ui.spacing_mut().item_spacing.x = 0.0;

                    ui.label("🔥 Powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/eframe");
                    ui.label(".");
                });
            });
    }
}
