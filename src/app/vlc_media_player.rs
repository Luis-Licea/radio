// The following code is only compiled when the target is NOT webassembly.

// Use atomic data types for exchanging data between the main thread and player
// thread thread-safely.
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
// Use atomically-reference-counted shared pointer for sharing immutable data
// across threads thread-safely. In combination with atomics, immutable data can
// be made mutable thread-safely.
use std::sync::Arc;
/// Use threads from the standard library to create a separate thread for the
/// music player. The player needs its own thread or else it will block the GUI.
use std::thread;
/// Use VLC media player when compiling natively.
#[cfg(not(target_arch = "wasm32"))]
use vlc::{Instance, Media, MediaPlayer, MediaPlayerAudioEx};

/// A thread-safe VLC media player with basic functionality such as stopping and
/// playing URLs, and changing the volume of the player.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
pub struct VLCMediaPlayer {
    // Flag for wether the media player should be playing.
    is_playing: Arc<AtomicBool>,
    // Flag for wether the user has changed the volume and needs to be updated.
    is_volume_changed: Arc<AtomicBool>,
    // Integer representing the volume after it was changed by the user. The
    // volume should be between 0 and 100, where 0 represents the mute state.
    volume: Arc<AtomicI32>,
    // The url corresponding to the station that will be streamed.
    url: String,
}

impl VLCMediaPlayer {
    /// Create a thread-safe VLC media player with a given volume level, where 0
    /// is mute and 100 is the max volume.
    pub fn new(volume: i32) -> Self {
        // Validate that the volume is between 0 and 100.
        let volume = VLCMediaPlayer::validate_volume(volume);

        VLCMediaPlayer {
            // The player is off by default.
            is_playing: Arc::new(AtomicBool::new(false)),
            // No volume change is needed because a default value is given.
            is_volume_changed: Arc::new(AtomicBool::new(false)),
            // The default volume is 50, where 0 is mute and 100 is the max.
            volume: Arc::new(AtomicI32::new(volume)),
            // The url that will be streamed is empty by default.
            url: "".to_string(),
        }
    }

    /// Play media from a valid URL or media resource location. Set the url
    /// before playing it.
    pub fn play_url(&mut self) {
        // NOTE: Using Arc and atomics were nedded to prevent unsafe code.

        // Atomics: exchanging data between the main/player threads thread-safely.
        // Arc: sharing immutable data across threads thread-safely. In combination
        // with atomics, immutable data can be made mutable thread-safely.

        // Make an atomically-reference-counted shared pointer for each member by
        // cloning. The player thread will take ownership of the ARC shared
        // pointers while the main thread keeps a copy of each pointer.
        let is_playing: Arc<AtomicBool> = Arc::clone(&self.is_playing);
        let is_volume_changed: Arc<AtomicBool> = Arc::clone(&self.is_volume_changed);
        let volume: Arc<AtomicI32> = Arc::clone(&self.volume);
        // Make a string slice that so that the thread can take ownership of it.
        let url = self.url.to_string();

        // Use threads from the standard library to create a separate thread for the
        // music player. The player needs its own thread or else it will block the GUI.
        thread::spawn(move || {
            // Create a VLC instance.
            let instance = Instance::new().unwrap();

            // Create media player using the VLC instance.
            let media_player = MediaPlayer::new(&instance).unwrap();

            // Create media from URL.
            let media = Media::new_location(&instance, &url).unwrap();

            // Set the URL to play.
            media_player.set_media(&media);

            // Start playing
            media_player.play().unwrap();

            // While the media player is playing:
            while is_playing.load(Ordering::Relaxed) {
                // If the volume changed:
                if is_volume_changed.load(Ordering::Relaxed) {
                    // Set the new volume.
                    media_player
                        .set_volume(volume.load(Ordering::Relaxed))
                        .unwrap();
                    // Set volume changed to false because volume is updated.
                    is_volume_changed.store(false, Ordering::Relaxed);
                }
                // Avoid busy waiting by yielding time slice to other processes. I hope
                // this helps with CPU scheduling.
                thread::yield_now();
            }
            // Stop playing the URL.
            media_player.stop();

            // Set playing state of the media player to false.
            is_playing.store(false, Ordering::Relaxed);
        });
    }

    /// Save the URL that will be played.
    pub fn set_src(&mut self, url: &str) {
        self.url = url.to_string();
    }

    /// Stop the media player.
    pub fn play(&mut self) {
        // Store the new playing state.
        self.is_playing.store(true, Ordering::Relaxed);
        self.play_url();
    }

    /// Start the media player.
    pub fn pause(&mut self) {
        // Store the new playing state.
        self.is_playing.store(false, Ordering::Relaxed);
    }

    /// Set the volume level, which should be between 0 and 100.
    pub fn set_volume(&mut self, volume: i32) {
        // Validate that the volume is between 0 and 100.
        let volume = VLCMediaPlayer::validate_volume(volume);
        // Set the volume target.
        self.volume.store(volume, Ordering::Relaxed);
        // Flag for a neededed volume change.
        self.is_volume_changed.store(true, Ordering::Relaxed);
    }

    /// Return the volme if it is between 0 and 100 and panic otherwise.
    fn validate_volume(volume: i32) -> i32 {
        match volume {
            // Validate that the volume is between 0 and 100.
            0..=100 => volume,
            // Panic if the volume is not between 0 and 100.
            _ => panic!("The given volume {:} is not between 0 and 100.", volume),
        }
    }
}
