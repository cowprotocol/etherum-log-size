use std::io::Read;

use anyhow::{anyhow, Context, Result};
use plotters::{
    element::Circle,
    prelude::{BitMapBackend, ChartBuilder, IntoDrawingArea},
    series::PointSeries,
    style::{IntoFont, BLACK, WHITE},
};

fn main() {
    // let block_range = 17400000..;
    let block_range = 0..;
    let mut entries = parse_entries().unwrap();
    entries.retain(|entry| block_range.contains(&entry.block));
    entries.sort_unstable_by_key(|log| log.block);
    text(&entries);
    // plot(&entries);
}

fn text(entries: &[Entry]) {
    let mut blocks: u64 = 0;
    let mut logs: u64 = 0;
    let mut data: u64 = 0;
    let mut topics: u64 = 0;
    for entries in entries.windows(2) {
        let [a, b] = entries  else { unreachable!() };
        let blocks_ = b.block - a.block;
        blocks += blocks_;
        logs += a.log_count * blocks_;
        data += a.data_len * blocks_;
        topics += a.topic_count * blocks_;
    }
    // block number, log index, transaction index, address, topic count, data bytes count
    const OVERHEAD_PER_LOG: u64 = 8 + 8 + 8 + 20 + 1 + 8;
    let size = logs * OVERHEAD_PER_LOG + data + topics * 32;
    let average_logs = logs as f64 / blocks as f64;
    let average_size = size as f64 / blocks as f64;
    println!(
        "Extrapolating between {:.1e} entries between blocks {}..{} gives:\n{blocks:.1e} blocks\n{logs:.1e} logs, avg {average_logs:.1e}\n{size:.1e} size, avg {average_size:.1e}",
        entries.len(), entries.first().unwrap().block, entries.last().unwrap().block
    );
}

fn _plot(entries: &[Entry]) {
    let logs_data = entries.iter().map(|entry| (entry.block, entry.log_count));

    let root = BitMapBackend::new("plot.png", (500, 500)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let mut chart = ChartBuilder::on(&root)
        .caption("logs per block", ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(
            entries.first().unwrap().block..entries.last().unwrap().block,
            0u64..1000,
        )
        .unwrap();
    chart
        .configure_mesh()
        //.y_desc("logs")
        .draw()
        .unwrap();
    chart
        .draw_series(PointSeries::<_, _, Circle<_, _>, _>::new(
            logs_data, 1i32, BLACK,
        ))
        .unwrap();
    //.label("log count")
    //.legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLACK));
    /*
    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()
        .unwrap();
    */
    root.present().unwrap();
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    block: u64,
    log_count: u64,
    data_len: u64,
    topic_count: u64,
}

fn parse_entries() -> Result<Vec<Entry>> {
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open("out")
        .context("open")?;
    let mut buf = Vec::<u8>::new();
    file.read_to_end(&mut buf).context("read_to_end")?;
    std::mem::drop(file);
    const ENTRY_SIZE: usize = 8 + 8 + 8 + 8;
    if buf.len() % ENTRY_SIZE != 0 {
        return Err(anyhow!("file contents not right size"));
    }
    let entries = buf
        .chunks_exact(ENTRY_SIZE)
        .map(|chunk| Entry {
            block: u64::from_le_bytes(chunk[0..8].try_into().unwrap()),
            log_count: u64::from_le_bytes(chunk[8..16].try_into().unwrap()),
            data_len: u64::from_le_bytes(chunk[16..24].try_into().unwrap()),
            topic_count: u64::from_le_bytes(chunk[24..32].try_into().unwrap()),
        })
        .collect();
    Ok(entries)
}
