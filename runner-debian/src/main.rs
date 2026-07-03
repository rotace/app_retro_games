mod framebuffer;

use clap::{Parser, Subcommand};
use std::process;
use thiserror::Error;
use std::io::Read;
use retro_core::RenderTarget;

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Framebuffer error: {0}")]
    Framebuffer(String),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Draw Perlin noise to the framebuffer
    DrawNoise {
        #[arg(long, default_value_t = 0)]
        width: u32,

        #[arg(long, default_value_t = 0)]
        height: u32,

        #[arg(long, default_value_t = 0.0)]
        x_offset: f32,

        #[arg(long, default_value_t = 0.0)]
        y_offset: f32,

        #[arg(long, default_value_t = 0.1)]
        scale: f32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::DrawNoise {
            width,
            height,
            x_offset,
            y_offset,
            scale,
        } => {
            if let Err(e) = handle_draw_noise(width, height, x_offset, y_offset, scale) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
}

fn handle_draw_noise(
    width: u32,
    height: u32,
    x_offset: f32,
    y_offset: f32,
    scale: f32,
) -> Result<(), RunnerError> {
    println!("Drawing Perlin noise... (Args: w={}, h={}, x={}, y={}, s={})", width, height, x_offset, y_offset, scale);

    let mut framebuffer = framebuffer::Framebuffer::new("/dev/fb0")?;
    
    // If width/height are specified (non-zero), we could use them to limit drawing,
    // but for MVP, we'll use the framebuffer's actual dimensions.
    let fb_width = framebuffer.width();
    let fb_height = framebuffer.height();
    
    println!("Framebuffer dimensions: {}x{}", fb_width, fb_height);

    let mut noise_gen = retro_core::noise::NoiseGenerator::new(42, scale as f64);
    noise_gen.set_offset(x_offset as f64, y_offset as f64);

    let input = retro_core::InputState::default();
    retro_core::tick_frame(&mut noise_gen, &mut framebuffer, &input);

    println!("Perlin noise drawn successfully to framebuffer! Press Enter to exit.");
    let mut buf = [0u8];
    std::io::stdin().read_exact(&mut buf)?;

    Ok(())
}
