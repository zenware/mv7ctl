use clap::Parser;

mod cli;
use cli::{Cli, Commands, MuteState};

mod mv7;
use mv7::{MV7Device, MicPosition};

use crate::cli::PositionState;
//use rusb::{Context, Result};

fn main() -> rusb::Result<()> {
    let args = Cli::parse();

    let mut mv7 = MV7Device::open()?;
    match args.command {
        Commands::Reset => {
            mv7.reset_device()?;
            // Since the device handle becomes invalid after this we exit early
            // to make sure we don't attempt to use it further.
            std::process::exit(0);
        }
        Commands::Status => mv7.status()?,
        Commands::Mute { state } => match state {
            MuteState::On => {
                mv7.set_mute(true)?;
            }
            MuteState::Off => {
                mv7.set_mute(false)?;
            }
        },
        Commands::Position { state } => match state {
            PositionState::Near => mv7.set_mic_position(MicPosition::Near)?,
            PositionState::Far => mv7.set_mic_position(MicPosition::Far)?,
        },
        Commands::Tone { state: _ } => {
            todo!();
        }
    }
    Ok(())
}
