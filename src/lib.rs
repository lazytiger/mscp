use crate::options::Options;
use clap::Parser;
use std::io::SeekFrom;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::task::JoinSet;

mod logger;
mod options;
mod types;

pub async fn run() -> types::Result<()> {
    let options = Options::parse();
    logger::setup_logger(options.log_file.as_str(), options.log_level)?;
    let md = tokio::fs::metadata(options.source.as_str()).await?;
    if !md.is_file() {
        log::error!("`{}` is not a file", options.source);
        return Ok(());
    }
    let size = md.len();
    {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(options.destination.as_str())
            .await?;
        file.set_len(size).await?;
        file.flush().await?;
    }

    let segments = options.segments as u64;
    let segment_size = size / segments;
    let mut set: JoinSet<types::Result<u64>> = JoinSet::new();
    let begin = Instant::now();
    let buffer_size = options.buffer_size.max(1024);
    for i in 0..segments {
        let source = options.source.clone();
        let destination = options.destination.clone();
        set.spawn(async move {
            let mut source = tokio::fs::OpenOptions::new()
                .read(true)
                .open(source.as_str())
                .await?;
            source.seek(SeekFrom::Start(i * segment_size)).await?;
            let mut destination = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(destination.as_str())
                .await?;
            destination.seek(SeekFrom::Start(i * segment_size)).await?;
            let mut buf = vec![0u8; buffer_size];
            let mut total = if i == segments - 1 {
                size - i * segment_size
            } else {
                segment_size
            } as usize;
            log::debug!("copy segment {i} now");
            while total > 0 {
                let data = if total >= buf.len() {
                    buf.as_mut_slice()
                } else {
                    &mut buf.as_mut_slice()[..total]
                };
                source.read_exact(data).await?;
                destination.write_all(data).await?;
                total -= data.len();
            }
            destination.flush().await?;
            Ok(i)
        });
    }
    while let Some(res) = set.join_next().await {
        let i = res.unwrap()?;
        log::debug!("segment {i} finished");
    }
    let elapsed = begin.elapsed().as_secs_f64();
    log::info!(
        "copy from `{}` to `{}` finished, cost {}s, speed is {}MB/s",
        options.source,
        options.destination,
        elapsed,
        size as f64 / 1024.0 / 1024.0 / elapsed
    );
    Ok(())
}
