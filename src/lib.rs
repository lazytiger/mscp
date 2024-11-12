use crate::options::Options;
use clap::Parser;
use std::io::SeekFrom;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::runtime::Handle;
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
    let thread_count = options
        .thread_count
        .min(Handle::current().metrics().num_workers());
    let segment_size = options
        .segment_size
        .map(|s| s.max(1) * 1024 * 1024)
        .unwrap_or((size - (size % thread_count as u64)) / thread_count as u64 + 1);
    let buffer_size = options.buffer_size.max(8) * 1024;
    let mut join_set = JoinSet::new();
    let mut offset = 0;

    let begin = Instant::now();
    while offset < size {
        if join_set.len() == thread_count {
            let (m, n) = join_set.join_next().await.unwrap().unwrap()?;
            log::debug!("copy segment [{}..{}] finished", m, m + n);
        }
        let source = options.source.clone();
        let destination = options.destination.clone();
        let copy_size = segment_size.min(size - offset);
        join_set.spawn(copy_segment(
            source,
            destination,
            offset,
            copy_size,
            buffer_size,
        ));
        offset += copy_size;
    }

    while let Some(res) = join_set.join_next().await {
        let (offset, copy_size) = res.unwrap()?;
        log::debug!("copy segment [{}..{}] finished", offset, offset + copy_size);
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

async fn copy_segment(
    source: String,
    destination: String,
    offset: u64,
    copy_size: u64,
    buffer_size: usize,
) -> types::Result<(u64, u64)> {
    let mut source = tokio::fs::OpenOptions::new()
        .read(true)
        .open(source.as_str())
        .await?;
    source.seek(SeekFrom::Start(offset)).await?;
    let mut destination = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(destination.as_str())
        .await?;
    destination.seek(SeekFrom::Start(offset)).await?;
    let mut buf = vec![0u8; buffer_size];
    let mut total = copy_size as usize;
    log::debug!("copy segment [{}..{}] now", offset, offset + copy_size);
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
    Ok((offset, copy_size))
}
