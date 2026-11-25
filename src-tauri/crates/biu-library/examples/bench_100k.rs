use std::time::Instant;

use biu_library::Database;

fn pct(values: &[f64], p: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let index = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[index]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join(format!("biu-bench-100k-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let mut db = Database::open(&path)?;

    let started = Instant::now();
    let tx = db.connection_mut().transaction()?;
    tx.execute(
        "INSERT INTO folders(path, added_at) VALUES (?1, '2026-01-01')",
        ["/bench"],
    )?;
    for artist in 0..1000 {
        tx.execute(
            "INSERT INTO artists(name) VALUES (?1)",
            [format!("Artist {artist}")],
        )?;
    }
    for album in 0..5000 {
        tx.execute(
            "INSERT INTO albums(title, artist_id, year) VALUES (?1, ?2, ?3)",
            rusqlite::params![
                format!("Album {album}"),
                (album % 1000) as i64 + 1,
                1990 + (album % 35)
            ],
        )?;
    }
    for i in 0..100_000i64 {
        let title = format!("Track {i:06}");
        let artist = (i % 1000) + 1;
        let album = (i % 5000) + 1;
        let genre = ["Pop", "Rock", "Jazz", "Electronic", "Classical"][i as usize % 5];
        let year = 1990 + (i % 35);
        let search_text = format!(
            "{title} Artist {} Album {} qing tian qt",
            i % 1000,
            i % 5000
        );
        tx.execute(
            "INSERT INTO tracks(folder_id, path, title, artist_id, album_id, genre, year,
                                duration_ms, format, added_at, updated_at, search_text)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'mp3', '2026-01-01', '2026-01-01', ?8)",
            rusqlite::params![
                format!("/bench/{i:06}.mp3"),
                title,
                artist,
                album,
                genre,
                year,
                180_000 + (i % 120) * 1000,
                search_text
            ],
        )?;
    }
    tx.commit()?;
    let insert_ms = started.elapsed().as_secs_f64() * 1000.0;

    let count = db.track_count()?;
    let mut elapsed = Vec::new();
    for i in (0..100).map(|n| n * 937) {
        let query = format!("track {:06}", i % 100_000);
        let t = Instant::now();
        let hits = db.search(&query, 50)?;
        elapsed.push(t.elapsed().as_secs_f64() * 1000.0);
        if hits.is_empty() {
            eprintln!("warning: empty result for {query}");
        }
    }
    let p50 = pct(&elapsed, 0.5);
    let p95 = pct(&elapsed, 0.95);
    let max = elapsed.iter().cloned().fold(0.0, f64::max);
    let size = std::fs::metadata(&path)?.len();

    println!(
        "tracks={count} insert_100k_ms={insert_ms:.0} db_mb={:.1}",
        size as f64 / 1_048_576.0
    );
    println!("search_p50_ms={p50:.2} search_p95_ms={p95:.2} search_max_ms={max:.2}");

    drop(db);
    let _ = std::fs::remove_file(&path);
    Ok(())
}
