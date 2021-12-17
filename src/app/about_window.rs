use eframe::{egui, epi};

/// The About window shows information about the application, such as creator
/// names, source links, and technologies used.
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
            is_open: true,
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
    fn update(&mut self, ctx: &eframe::egui::CtxRef,
        _frame: &mut eframe::epi::Frame<'_>) {

        // Create an About window. The window only pops up when the About menu
        // itme is pressed.
        egui::Window::new(self.name()).open(&mut self.is_open).show(ctx, |ui| {

            // Display the name of the application.
            ui.vertical_centered(|ui| {
                ui.heading("Online Radio");
            });

            // Display the name of the creators.
            ui.label("Created by Luis David Licea Torres.");

            // Display the techonologies used to create the application.
            // Powered by ...
            ui.horizontal(|ui| {

                // Remove the horizontal spacing so that labels and
                // hyperlinks are next to each other.
                ui.spacing_mut().item_spacing.x = 0.0;

                ui.label("Powered by ");
                ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                ui.label(" and ");
                ui.hyperlink_to("eframe",
                "https://github.com/emilk/egui/tree/master/eframe");
                ui.label(".");
            });
        });
    }
}