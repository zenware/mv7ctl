use clap::Parser;

mod cli;
use cli::{Cli, Commands, MuteState};

mod mv7;
use mv7::MV7Device;
//use rusb::{Context, Result};

fn main() -> rusb::Result<()> {
    let args = Cli::parse();

    println!("Args {:?}", args);
    let mut mv7 = MV7Device::open()?;

    match args.command {
        Commands::Status => {
            todo!();
        }
        Commands::Mute { state } => match state {
            MuteState::On => {
                mv7.set_mute(true)?;
            }
            MuteState::Off => {
                mv7.set_mute(false)?;
            }
        },
        Commands::Position { state: _ } => {
            todo!();
        }
        Commands::Tone { state: _ } => {
            todo!();
        }
    }
    Ok(())
}
