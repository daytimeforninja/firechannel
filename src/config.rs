use serde::{Deserialize, Serialize};

/// Tummo practice configuration
#[derive(Debug, Clone, Copy)]
pub struct TummoConfig {
    /// Duration for each inhale in nine-round breathing
    pub inhale_duration: f32,
    /// Duration for each exhale in nine-round breathing
    pub exhale_duration: f32,
    /// Duration to hold breath in vase breathing (beginner-friendly default)
    pub vase_hold_duration: f32,
}

impl Default for TummoConfig {
    fn default() -> Self {
        Self {
            inhale_duration: 6.0,
            exhale_duration: 8.0,
            vase_hold_duration: 15.0,
        }
    }
}

/// Persistent settings saved to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentSettings {
    pub inhale_duration: f32,
    pub exhale_duration: f32,
    pub vase_hold_duration: f32,
    pub session_duration: diaframe::SessionDuration,
    #[serde(flatten)]
    pub audio: diaframe::AudioSettings,
    #[serde(flatten)]
    pub stats: diaframe::PracticeStats,
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self {
            inhale_duration: 6.0,
            exhale_duration: 8.0,
            vase_hold_duration: 15.0,
            session_duration: diaframe::SessionDuration::default(),
            audio: diaframe::AudioSettings::default(),
            stats: diaframe::PracticeStats::default(),
        }
    }
}

impl PersistentSettings {
    /// Load settings from disk, or return defaults
    pub fn load() -> Self {
        diaframe::config_path("com", "firechannel", "firechannel")
            .map(|path| diaframe::load_json::<Self>(&path))
            .unwrap_or_default()
    }

    /// Save settings to disk
    pub fn save(&self) {
        if let Some(path) = diaframe::config_path("com", "firechannel", "firechannel") {
            diaframe::save_json(&path, self);
        }
    }
}
