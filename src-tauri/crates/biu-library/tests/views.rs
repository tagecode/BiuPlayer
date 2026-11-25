use biu_library::Database;

#[test]
fn genres_years_and_folders_group_and_filter() {
    let mut db = Database::open_in_memory().unwrap();
    let tx = db.connection_mut().transaction().unwrap();
    tx.execute(
        "INSERT INTO folders(path, added_at) VALUES ('/a', '2026-01-01'), ('/b', '2026-01-01')",
        [],
    )
    .unwrap();
    for (i, (genre, year, folder)) in [("Rock", 1999, 1), ("Rock", 2001, 1), ("Jazz", 1999, 2)]
        .iter()
        .enumerate()
    {
        tx.execute(
            "INSERT INTO tracks(folder_id, path, title, genre, year, added_at, updated_at, search_text)
             VALUES (?1, ?2, ?3, ?4, ?5, '2026-01-01', '2026-01-01', '')",
            rusqlite::params![
                folder,
                format!("/{}/{i}.mp3", if *folder == 1 { "a" } else { "b" }),
                format!("Track {i}"),
                genre,
                year
            ],
        )
        .unwrap();
    }
    tx.commit().unwrap();

    let genres = db.list_genres().unwrap();
    assert_eq!(
        genres,
        vec![("Jazz".to_string(), 1), ("Rock".to_string(), 2)]
    );

    let years = db.list_years().unwrap();
    assert_eq!(years, vec![(2001, 1), (1999, 2)]);

    let folders = db.list_folders().unwrap();
    assert_eq!(folders.len(), 2);
    assert!(folders
        .iter()
        .any(|(id, path, count)| *id == 1 && path == "/a" && *count == 2));

    assert_eq!(db.browse_tracks("genre", "Rock").unwrap().len(), 2);
    assert_eq!(db.browse_tracks("year", "1999").unwrap().len(), 2);
    assert_eq!(db.browse_tracks("folder", "2").unwrap().len(), 1);
    assert!(db.browse_tracks("bogus", "x").is_err());
}

#[test]
fn search_treats_operator_words_as_terms() {
    let mut db = Database::open_in_memory().unwrap();
    let tx = db.connection_mut().transaction().unwrap();
    tx.execute(
        "INSERT INTO folders(path, added_at) VALUES ('/a', '2026-01-01')",
        [],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO tracks(folder_id, path, title, added_at, updated_at, search_text)
         VALUES (1, '/a/1.mp3', 'Rock and Roll', '2026-01-01', '2026-01-01', 'Rock and Roll')",
        [],
    )
    .unwrap();
    tx.commit().unwrap();

    assert_eq!(db.search("rock AND roll", 10).unwrap().len(), 1);
    assert_eq!(db.search("NOT", 10).unwrap().len(), 0);
}
