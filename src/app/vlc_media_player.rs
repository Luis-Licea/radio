// Use atomic data types for exchanging data between the main thread and player
// thread thread-safely.
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
// Use atomically-reference-counted shared pointer for sharing immutable data
// across threads thread-safely. In combination with atomics, immutable data can
// be made mutable thread-safely.
use std::sync::Arc;
/// Use threads from the standard library to create a separate thread for the
/// music player. The player needs its own thread or else it will block the GUI.
use std::thread::{self};
/// Use VLC as the media player.
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
        }
    }

    /// Play media from a valid URL or media resource location.
    pub fn play_url(&mut self, url_to_play: String) {
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

        // Use threads from the standard library to create a separate thread for the
        // music player. The player needs its own thread or else it will block the GUI.
        thread::spawn(move || {
            // Create a VLC instance.
            let instance = Instance::new().unwrap();

            // Create media player using the VLC instance.
            let media_player = MediaPlayer::new(&instance).unwrap();

            // Create media from URL.
            let media = Media::new_location(&instance, &url_to_play).unwrap();

            // TODO: Return metadata.

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

    pub fn toggle_play_and_get_is_playing(&mut self) -> bool {
        // Get the new playing state by applying NOT to the previous state.
        let toggled_state = !self.is_playing.load(Ordering::Relaxed);
        // Store the new playing state.
        self.is_playing.store(toggled_state, Ordering::Relaxed);
        // Return the new playing state.
        toggled_state
    }

    pub fn set_volume(&mut self, volume: i32) {
        // Validate that the volume is between 0 and 100.
        let volume = VLCMediaPlayer::validate_volume(volume);
        // Set the volume target.
        self.volume.store(volume, Ordering::SeqCst);
        // Flag for a neededed volume change.
        self.is_volume_changed.store(true, Ordering::SeqCst);
    }

    /// Returns the volme if it is between 0 and 100 and panics otherwise.
    fn validate_volume(volume: i32) -> i32 {
        match volume {
            // Validate that the volume is between 0 and 100.
            0..=100 => volume,
            // Panic if the volume is not between 0 and 100.
            _ => panic!("The given volume {:} is not between 0 and 100.", volume),
        }
    }
}
