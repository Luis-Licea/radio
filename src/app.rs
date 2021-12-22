mod about_window;
/// Use VLC media player when compiling natively.
#[cfg(not(target_arch = "wasm32"))]
mod vlc_media_player;
use about_window::AboutWindow;
use eframe::{egui, epi};
/// Use VLC media player when compiling natively.
#[cfg(not(target_arch = "wasm32"))]
use vlc_media_player::VLCMediaPlayer;
/// Use Web-sys media player when compiling webassembly.
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlAudioElement;

/// Debug and PartialEq are needed to print and use enums.
#[derive(Debug, PartialEq)]
/// It derives Deserialize/Serialize so it can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
/// Enumerate the user interface languages.
enum Language {
    English,
    Spanish,
    Russian,
}

/// It derives Deserialize/Serialize so it can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
/// New fields are are given default values when deserializing old state.
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    /// The station URL that will be streamed.
    station_url: String,

    /// The string used to search for station names.
    text_to_search: String,

    /// The volume level shown at all times.
    volume_on_slider: i32,

    /// The volume level stored before muting the volume.
    volume_before_mute: i32,

    /// The About window shown in the menu bar.
    about_window: AboutWindow,

    /// Use VLC for playing URLs when compiling natively.
    /// Opt-out of serialization for the VLC media player.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(feature = "persistence", serde(skip))]
    media_player: VLCMediaPlayer,

    /// Use Web-sys for playing URLs when compiling webassembly.
    /// Opt-out of serialization for the Web-sys media player.
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(feature = "persistence", serde(skip))]
    media_player: HtmlAudioElement,

    /// Wether an station is playing or not.
    playing_icon: char,

    /// Wether the user settings panel is open or not.
    user_settings_is_open: bool,

    /// The user interface language.
    language: Language,
}

/// Implement trait to create default window.
impl Default for App {
    /// Create default window.
    fn default() -> Self {
        // Initial media player volume.
        let volume = 50;
        App {
            /// By default play a dubstep station.
            station_url: "https://ice5.somafm.com/dubstep-128-mp3".to_owned(),

            /// The text shows a hint by default.
            text_to_search: "".to_owned(),

            /// Set the initial slider volume.
            volume_on_slider: volume,

            /// Set the initial volume before muting.
            volume_before_mute: volume,

            /// Creates a default About window.
            about_window: AboutWindow::default(),

            /// Creates a VLC media player.
            /// Use VLC for playing URLs when compiling natively.
            #[cfg(not(target_arch = "wasm32"))]
            media_player: VLCMediaPlayer::new(volume),
            /// Use Web-sys for playing URLs when compiling webassembly.
            #[cfg(target_arch = "wasm32")]
            media_player: HtmlAudioElement::new().unwrap(),

            // Set the playing icon as the default icon.
            playing_icon: '▶',

            /// The user settings panel should be closed by default.
            user_settings_is_open: false,

            /// Set the default language to English.
            language: Language::English,
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
            station_url,
            text_to_search,
            volume_on_slider,
            volume_before_mute,
            about_window,
            media_player,
            playing_icon,
            user_settings_is_open,
            language,
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
                // Add theme switch in menu bar.
                egui::global_dark_light_mode_switch(ui);
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

        // Add a search panel to look for station names and access user
        // settings.
        egui::TopBottomPanel::top("search_panel").show(ctx, |ui| {
            // All the widgets should be placed in the same horizontal line, as
            // opposed to one below the other.
            ui.horizontal(|ui| {
                // Show the website name.
                ui.heading("Online Radio");
                // Add magnifying glass to decorate search bar.
                ui.heading("🔍");
                // Calculate the button width. This will be used for spacing.
                let button_width = ui.spacing().interact_size.x;
                // Calculate the available width.
                let width = ui.available_width();
                // Add a search bar to search for stations. Adjust search bar width based on available wdith.
                ui.add(
                    egui::TextEdit::singleline(text_to_search)
                        .desired_width(width - button_width * 1.6)
                        .hint_text("Search..."),
                );

                // Add a login button.
                if ui.button("👤").clicked() {
                    // This flag is used inside the central panel to draw the
                    // side panel. The side panel must be drawn inside the
                    // central body, or it will interfere with the top and
                    // bottom panels.
                    *user_settings_is_open = !*user_settings_is_open;
                }

                // Add an options button.
                if ui.button("☰").clicked() {}
            });
        });

        // TODO: Query radio stations.
        let mut url = vec![
            "https://ice5.somafm.com/dubstep-128-mp3",
            "http://stream.radioparadise.com/aac-128",
        ];

        // Create a bottom pannel. The top/bottom/side panels must be drawn
        // before the central panel.
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Toggle play/pause when the play/pause icon is clicked.
                if ui.button(*playing_icon).clicked() {
                    // Chose correct playing icon and playing state based on the icon.
                    // The logic seems reversed here, but it is really not.
                    *playing_icon = match playing_icon {
                        // If not playing, show the play button.
                        '⏸' => {
                            let _ = media_player.pause();
                            '▶'
                        }
                        // If playing, show the pause button and play the URL.
                        '▶' => {
                            media_player.set_src(station_url);
                            let _ = media_player.play();
                            '⏸'
                        }
                        // Return the same icon.
                        _ => *playing_icon,
                    }
                }

                // Chose correct volume icon based on volume level.
                let volume_icon = match volume_on_slider {
                    // If volume is 0:
                    0 => "🔇",
                    // If volume is between 1 and 30:
                    1..=30 => "🔈",
                    // If volume is between 31 and 70:
                    31..=70 => "🔉",
                    // For any other value:
                    _ => "🔊",
                };

                // Toggle volume on and off when volume icon is clicked.
                if ui.button(volume_icon).clicked() {
                    // If the volume is not mute:
                    if *volume_on_slider != 0 {
                        // Store the current volume level.
                        *volume_before_mute = *volume_on_slider;
                        // Mute the volume.
                        *volume_on_slider = 0;
                    } else {
                        // Else the volume is mute.
                        // Retrieve the last current volume level.
                        *volume_on_slider = *volume_before_mute;
                    }
                    #[cfg(target_arch = "wasm32")]
                    // Web-sys takes volme as a float in the range 0.0 to 1.0.
                    media_player.set_volume(*volume_on_slider as f64 / 100.0);
                    #[cfg(not(target_arch = "wasm32"))]
                    // VLC takes volme as an integer in the range 0 to 100.
                    media_player.set_volume(*volume_on_slider);
                }

                // Display a volume slider, and change the volume when the
                // slider is clicked or dragged.
                if ui
                    .add(egui::Slider::new(volume_on_slider, 0..=100).show_value(false))
                    .is_pointer_button_down_on()
                {
                    #[cfg(target_arch = "wasm32")]
                    // Web-sys takes volme as a float in the range 0.0 to 1.0.
                    media_player.set_volume(*volume_on_slider as f64 / 100.0);
                    #[cfg(not(target_arch = "wasm32"))]
                    // VLC takes volme as an integer in the range 0 to 100.
                    media_player.set_volume(*volume_on_slider);
                }

                // Display artist and song name.
                ui.label("Artist Name - Song Name");

                /*
                // Calculate the button width. This will be used for spacing.
                let button_width = ui.spacing().interact_size.x;
                // Calculate the available width.
                let width = ui.available_width();

                // let x = ui.spacing().interact_size.x * 1.5 + ui.spacing().slider_width
                // ui.add_space(width - 4.0 * button_width);

                // TODO: add more functionality and make consider small phone screen sizes.
                // Also, vote for each station whenever it is played.
                // Add button that copies artist and song name.
                if ui.button("📋").clicked() {
                    // Copy song title.
                }
                // Add button that adds current station to preferred stations.
                if ui.button("➕♫").clicked() {}
                if ui.button("➕🎵").clicked() {}
                if ui.button("🎶").clicked() {}
                // Add an options button.
                if ui.button("📻").clicked() {}
                */
            });
        });

        // The central panel is the region left after adding top and
        // side panels.
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::warn_if_debug_build(ui);

            // Add a scroll area so the user can scroll through the stations.
            egui::ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    // Add a grid where the stations will be placed.
                    egui::Grid::new("stations").striped(true).show(ui, |ui| {
                        // For every URL in the vector:
                        for (i, link) in url.iter_mut().enumerate() {
                            // Create a group of components that will represent a link to a station.
                            ui.group(|ui| {
                                // Place the widgets horizontally.
                                ui.horizontal(|ui| {
                                    // Add a play button for the station.
                                    if ui.button("▶").clicked() {
                                        // Update the playing icon.
                                        *playing_icon = '⏸';

                                        // Set the station URL to be streamed.
                                        *station_url = link.to_string();

                                        // Pass the URL to the station.
                                        media_player.set_src(station_url);

                                        // Play the station.
                                        let _ = media_player.play();
                                    }
                                    // Give a number to each station.
                                    ui.label(format!("Station {}", i));
                                });
                                // Show the link for each station.
                                ui.hyperlink_to(link.to_string(), link);
                            });
                            // End the grid row.
                            ui.end_row();
                        }
                    });
                });

            // If the user settings panel is open:
            if *user_settings_is_open {
                // Show the side panel:
                egui::SidePanel::right("side_panel").show(ctx, |ui| {
                    // Display the name of the panel.
                    ui.heading("User Settings");

                    // Display a combo box with available languages.
                    ui.horizontal(|ui| {
                        ui.label("Language: ");
                        egui::ComboBox::from_label("🌎")
                            // Display name of currently selected language.
                            .selected_text(format!("{:?}", language))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    language,
                                    Language::English,
                                    format!("{:?}", Language::English),
                                );
                                ui.selectable_value(
                                    language,
                                    Language::Spanish,
                                    format!("{:?}", Language::Spanish),
                                );
                                ui.selectable_value(
                                    language,
                                    Language::Russian,
                                    format!("{:?}", Language::Russian),
                                );
                            });
                    });
                });
            }
        });
    }
}
