use std::time::Instant;

use biu_library::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let count = std::env::args()
        .nth(1)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10_000);
    let root = std::env::temp_dir().join(format!("biu-scan-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;

    let create_start = Instant::now();
    for i in 0..count {
        std::fs::write(root.join(format!("track-{i:05}.mp3")), [])?;
    }
    println!(
        "generated {count} files in {} ms",
        create_start.elapsed().as_millis()
    );

    let db_path = root.join("bench.db");
    let mut db = Database::open(&db_path)?;
    let folder_id = db.add_folder(&root)?;

    let scan_start = Instant::now();
    let report = db.scan_folder(folder_id, &root)?;
    let scan_ms = scan_start.elapsed().as_millis();

    let query_start = Instant::now();
    let hits = db.search("track 05000", 20)?;
    let query_ms = query_start.elapsed().as_millis();

    println!(
        "scan: seen={} inserted={} skipped={} elapsed={}ms tracks={}",
        report.files_seen,
        report.inserted,
        report.skipped,
        scan_ms,
        db.track_count()?
    );
    println!("fts: hits={} elapsed={query_ms}ms", hits.len());

    let rescan_start = Instant::now();
    let rescan = db.scan_folder(folder_id, &root)?;
    println!(
        "rescan: seen={} skipped={} elapsed={}ms",
        rescan.files_seen,
        rescan.skipped,
        rescan_start.elapsed().as_millis()
    );

    std::fs::remove_dir_all(&root)?;
    Ok(())
}
