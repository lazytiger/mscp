use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// file path for save log, empty means stdout
    #[arg(short = 'L', long, default_value = "")]
    pub log_file: String,

    /// log level, 0 to 4 represents TRACE, DEBUG, INFO, WARN, ERROR, others mean OFF
    #[arg(short = 'E', long, default_value = "2")]
    pub log_level: u8,

    /// number of segments to be split into.
    #[arg(short = 'S', long, default_value = "8")]
    pub segments: usize,

    /// the source file path
    #[arg(short = 's', long)]
    pub source: String,

    /// the destination file path
    #[arg(short = 'd', long)]
    pub destination: String,
}
