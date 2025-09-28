use anyhow::Result;
use serde::{Deserialize, Serialize};
use mpris::{Player, PlayerFinder, PlaybackStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub position: u64,
    pub duration: u64,
    pub is_playing: bool,
    pub cover_url: Option<String>,
}

impl Default for TrackInfo {
    fn default() -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            position: 0,
            duration: 0,
            is_playing: false,
            cover_url: None,
        }
    }
}

pub struct MusicPlayer {
    current_track: TrackInfo,
    player_finder: PlayerFinder,
    current_player: Option<Player>,
    last_update: std::time::Instant,
}

impl MusicPlayer {
    pub fn new() -> Self {
        Self {
            current_track: TrackInfo::default(),
            player_finder: PlayerFinder::new().unwrap_or_else(|_| {
                // Create a dummy finder if connection fails
                PlayerFinder::new().unwrap()
            }),
            current_player: None,
            last_update: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self) {
        // Only update every 2 seconds to avoid excessive D-Bus calls
        if self.last_update.elapsed() < std::time::Duration::from_secs(2) {
            return;
        }

        // Try to find and connect to MPRIS players
        if let Err(e) = self.update_from_mpris() {
            eprintln!("Failed to update from MPRIS: {}", e);
        }

        self.last_update = std::time::Instant::now();
    }

    fn update_from_mpris(&mut self) -> Result<()> {
        // Find all available MPRIS players
        let players = self.player_finder.find_all()?;
        
        // If no players available, clear current player
        if players.is_empty() {
            self.current_player = None;
            self.current_track = TrackInfo::default();
            return Ok(());
        }
        
        // If current player is no longer available, switch to first available player
        if let Some(ref current_player) = self.current_player {
            if !players.iter().any(|p| p.identity() == current_player.identity()) {
                self.current_player = Some(players.into_iter().next().unwrap());
            }
        } else {
            // No current player, select the first available one
            self.current_player = Some(players.into_iter().next().unwrap());
        }
        
        // Update track info for current player
        if self.current_player.is_some() {
            let player = self.current_player.take().unwrap();
            let track_info = self.get_track_info_from_player(&player)?;
            self.current_track = track_info;
            self.current_player = Some(player);
        }
        
        Ok(())
    }

    fn get_track_info_from_player(&self, player: &Player) -> Result<TrackInfo> {
        // Get playback status
        let playback_status = player.get_playback_status()?;
        
        // Get metadata
        let metadata = player.get_metadata()?;
        
        // Get position
        let position = player.get_position().unwrap_or(std::time::Duration::from_secs(0));
        
        // Create track info
        Ok(TrackInfo {
            title: metadata.title().map(|s| s.to_string()),
            artist: metadata.artists().and_then(|artists| artists.first().map(|s| s.to_string())),
            album: metadata.album_name().map(|s| s.to_string()),
            position: (position.as_micros() / 1_000_000) as u64, // Convert from microseconds to seconds
            duration: metadata.length().map(|d| (d.as_micros() / 1_000_000) as u64).unwrap_or(0),
            is_playing: playback_status == PlaybackStatus::Playing,
            cover_url: metadata.art_url().map(|s| s.to_string()),
        })
    }

    pub fn get_current_track(&self) -> &TrackInfo {
        &self.current_track
    }
    
    pub fn get_current_player(&self) -> Option<&Player> {
        self.current_player.as_ref()
    }
    
    pub fn is_connected(&self) -> bool {
        self.current_player.is_some()
    }
    
    pub fn get_available_players(&self) -> Result<Vec<String>> {
        let players = self.player_finder.find_all()?;
        Ok(players.into_iter().map(|p| p.identity().to_string()).collect())
    }

    pub fn toggle_play_pause(&mut self) {
        if let Some(ref player) = self.current_player {
            if let Err(e) = player.play_pause() {
                eprintln!("Failed to send PlayPause command: {}", e);
            } else {
                println!("Sent PlayPause command to {}", player.identity());
            }
        }
    }

    pub fn next(&mut self) {
        if let Some(ref player) = self.current_player {
            if let Err(e) = player.next() {
                eprintln!("Failed to send Next command: {}", e);
            } else {
                println!("Sent Next command to {}", player.identity());
            }
        }
    }

    pub fn previous(&mut self) {
        if let Some(ref player) = self.current_player {
            if let Err(e) = player.previous() {
                eprintln!("Failed to send Previous command: {}", e);
            } else {
                println!("Sent Previous command to {}", player.identity());
            }
        }
    }
}