use clap::{Parser, Subcommand, ValueEnum};

/// Control Shure MV7 Microphone Settings
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(subcommand_required = true, arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// Top-Level subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display all current settings of the connected MV7
    Status,
    /// Mute/unmute the microphone
    Mute {
        #[arg(value_enum)]
        state: MuteState,
    },
    /// Indicate whether the mic is "near or far" to your mouth
    Position {
        #[arg(value_enum)]
        state: PositionState,
    },
    /// Slight Vocal EQ/Filters
    Tone {
        #[arg(value_enum)]
        state: ToneState,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum MuteState {
    /// Mutes the microphone
    On,
    /// Unmutes the microphone
    Off,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum PositionState {
    Near,
    Far,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ToneState {
    Dark,
    Natural,
    Bright,
}
