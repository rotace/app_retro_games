// Core library for retro games runners

pub mod games;
pub mod noise;
pub mod traits;

pub use games::RetroGames;
pub use traits::*; // RenderTarget, GameCore, InputState, tick_frame を再エクスポート

#[derive(Debug)]
pub enum NoiseError {
    GenerationError(String),
    RenderError(String),
}

impl std::fmt::Display for NoiseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoiseError::GenerationError(msg) => write!(f, "Noise generation error: {}", msg),
            NoiseError::RenderError(msg) => write!(f, "Noise render error: {}", msg),
        }
    }
}

impl std::error::Error for NoiseError {}
