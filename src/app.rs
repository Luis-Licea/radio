mod about_window;
use about_window::AboutWindow;
use eframe::egui;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use web_sys::HtmlAudioElement;

/// Enumerate the user interface languages.
/// Debug and PartialEq are needed to print and use enums.
#[derive(Debug, PartialEq)]
/// It derives Deserialize/Serialize so it can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
enum Language {
    English,
    Spanish,
    Russian,
}

/// The dtata associated to a radio station (url, name, etc).
// Deriving the deserialization and serialization features is done by the
// serde_json dependency. These derivations allow JSON text to be converted into
// a Station struct.
#[derive(Deserialize, Debug)]
pub struct Station {
    pub stationuuid: String,
    pub name: String,
    pub url: String,
    pub url_resolved: String,
    pub homepage: String,
    pub favicon: String,
    pub tags: String,
    pub country: String,
    pub state: String,
    pub language: String,
    pub votes: i32,
    pub lastchangetime: String,
    pub codec: String,
    pub bitrate: u32,
    pub lastcheckoktime: String,
    pub clicktimestamp: String,
    pub clickcount: u32,
    pub clicktrend: i32,
}

/// The download status.
enum Download {
    /// No downloads done or in progress.
    None,
    /// The download is in progress.
    InProgress,
    /// The download is done and the data is stored in the response, unless the
    /// donwnload resulted in an error.
    Done(Result<ehttp::Response, ehttp::Error>),
}

/// It derives Deserialize/Serialize so it can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
/// New fields are are given default values when deserializing old state.
#[cfg_attr(feature = "persistence", serde(default))]
pub struct App {
    // The name of the window.
    name: String,

    /// Stores stations as they are retrieved from the database.
    /// Opt-out of serialization for downloads.
    #[cfg_attr(feature = "persistence", serde(skip))]
    download: Arc<Mutex<Download>>,

    /// The list of stations that was retrieved from the database.
    /// Opt-out of serialization for stations.
    #[cfg_attr(feature = "persistence", serde(skip))]
    stations: Arc<Mutex<Vec<Station>>>,

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

    /// Opt-out of serialization for the Web-sys media player.
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
impl App {
    /// Create default window.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Initial media player volume.
        let volume = 50;
        App {
            /// Name the application (the main window).
            name: "Online Radio".to_owned(),
            /// Initially there are no downloads.
            download: Arc::new(Mutex::new(Download::None)),

            // Initially the list of stations is empty.
            stations: Arc::new(Mutex::new(Vec::new())),

            /// By default play a dubstep station.
            station_url: "https://ice5.somafm.com/dubstep-128-mp3".to_owned(),

            /// Initially there is no text to search.
            text_to_search: "".to_owned(),

            /// Set the initial slider volume.
            volume_on_slider: volume,

            /// Set the initial volume before muting.
            volume_before_mute: volume,

            /// Creates a default About window.
            about_window: AboutWindow::default(),

            /// Use Web-sys for playing URLs when compiling webassembly.
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
impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per
    /// second.  Put your widgets into a `SidePanel`, `TopPanel`,
    /// `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self {
            name,
            download,
            stations,
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
                ui.menu_button("File", |ui| {
                    // Add a menu item for quitting the application.
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                // Add a menu bar category for showing iformation about the app.
                ui.menu_button("Help", |ui| {
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
                ui.heading(name.to_string());

                // Create a flag that triggers the download of station from the
                // database.
                let mut trigger_fetch = false;

                // Add magnifying glass that triggers radio station search.
                trigger_fetch |= ui.button("🔍").clicked();

                // Calculate the button width. This will be used for spacing.
                let button_width = ui.spacing().interact_size.x;
                // Calculate the available width. This will be used for spacing.
                let width = ui.available_width();
                // Add a search bar to search for stations. Adjust search bar
                // width based on available wdith.
                let search = ui.add(
                    egui::TextEdit::singleline(text_to_search)
                        .desired_width(width - button_width * 1.6)
                        .hint_text("Search…"),
                );

                // The search bar triggers a radio station search whenever the
                // user presses "Enter".
                trigger_fetch |= search.lost_focus() && ui.input().key_pressed(egui::Key::Enter);

                if trigger_fetch {
                    // Search stations by name.
                    // TODO: Use post method to specify more than one parameter.
                    // TODO: Randomly choose a radio browser server to distribute load.
                    let request = ehttp::Request::get(format!(
                        "https://de1.api.radio-browser.info/json/stations/byname/{}?limit=100",
                        text_to_search
                    ));

                    // Create a copy of the download that will be moved to another thread.
                    let download_store = download.clone();

                    // Set the download in progress.
                    *download_store.lock().unwrap() = Download::InProgress;
                    // Fetch the request, and when done, process the response.
                    ehttp::fetch(request, move |response| {
                        // Set the download as done, and store the response.
                        *download_store.lock().unwrap() = Download::Done(response);
                    });
                }

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

        // Create a bottom pannel. The top/bottom/side panels must be drawn
        // before the central panel.
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            // Display artist and song name.
            ui.label("Artist Name - Song Name");

            // Separate the artist and song names from the buttons.
            ui.separator();

            ui.horizontal(|ui| {
                // Toggle play/pause when the play/pause icon is clicked.
                if ui.button(playing_icon.to_string()).clicked() {
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
                    // Web-sys takes volme as a float in the range 0.0 to 1.0.
                    media_player.set_volume(*volume_on_slider as f64 / 100.0);
                }

                // Display a volume slider, and change the volume when the
                // slider is clicked or dragged.
                if ui
                    .add(egui::Slider::new(volume_on_slider, 0..=100).show_value(false))
                    .is_pointer_button_down_on()
                {
                    // Web-sys takes volme as a float in the range 0.0 to 1.0.
                    media_player.set_volume(*volume_on_slider as f64 / 100.0);
                }

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

            // Create a download guard.
            let mut download_inner = download.lock().unwrap();

            // Match the donwload state.
            match &*download_inner {
                // If no download, do nothing.
                Download::None => {}
                // If download in progress, show message.
                Download::InProgress => {
                    ui.label("Retrieving stations…");
                }
                // If the HTTP response is OK, process the text.
                Download::Done(Ok(response)) => match response.text() {
                    // If there is text, try to convert it into a vector of stations.
                    Some(text) => match serde_json::from_str::<Vec<Station>>(&text) {
                        // If the conversion is ok, save the vector of stations.
                        Ok(stations_vector) => {
                            // Store the stations.
                            // TODO: Remove stations with same names and urls.
                            *stations.lock().unwrap() = stations_vector;

                            // Show there are no more downloads.
                            *download_inner = Download::None;
                        }
                        // If the conversion produced an error, show the error message.
                        Err(e) => {
                            ui.label(e.to_string());
                        }
                    },
                    // If there is no text, show a message.
                    None => {
                        ui.label("No stations.");
                    }
                },
                // If the HTTP response had an error, show error message.
                Download::Done(Err(err)) => {
                    ui.label(err);
                }
            }

            // Add a scroll area so the user can scroll through the stations.
            egui::ScrollArea::vertical()
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    // Add a grid where the stations will be placed.
                    egui::Grid::new("stations")
                        .striped(true)
                        .min_col_width(200.0)
                        .show(ui, |ui| {
                            // For every URL in the vector:
                            for station in &*stations.lock().unwrap() {
                                // Create a group of components that will represent a link to a station.
                                ui.group(|ui| {
                                    // Place the widgets horizontally.
                                    ui.horizontal(|ui| {
                                        // Add a play button for the station.
                                        if ui.button("▶").clicked() {
                                            // Update the playing icon.
                                            *playing_icon = '⏸';

                                            // Get the station URL to be streamed.
                                            *station_url = station.url_resolved.to_string();

                                            // Pass the URL to the station.
                                            media_player.set_src(station_url);

                                            // Stop the station in case it is playing.
                                            let _ = media_player.pause();

                                            // Play the station.
                                            // TODO: Allow player to play HTTP stations, not only HTTPS.
                                            let _ = media_player.play();
                                        }
                                        // Give a number to each station.
                                        ui.label(&station.name);
                                    });
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
