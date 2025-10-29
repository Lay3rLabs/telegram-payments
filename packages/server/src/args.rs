use clap::Parser;

#[derive(Clone, Parser)]
#[allow(clippy::large_enum_variant)]
pub struct ServerArgs {
    #[arg(long)]
    pub port: u16,
}
