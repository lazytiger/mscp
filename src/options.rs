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

    /// number of threads to run copy command.
    #[arg(short = 't', long, default_value = "8")]
    pub thread_count: usize,

    /// the size of each segment, unit is MB.
    #[arg(short = 'S', long)]
    pub segment_size: Option<u64>,

    /// the source file path
    #[arg(short = 's', long)]
    pub source: String,

    /// the destination file path
    #[arg(short = 'd', long)]
    pub destination: String,

    /// the buffer size for copy routine, unity is kB
    #[arg(short = 'b', long, default_value = "8")]
    pub buffer_size: usize,
}
