use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

use playlist_formatter::playlist::Playlist;
use playlist_formatter::types::{FileFormat, PlaylistType};

/// Path to the `tests/files` directory.
static TEST_FILES_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("files"));

#[test]
fn test_dir() {
    assert!(TEST_FILES_DIR.is_dir());
}

#[test]
fn test_serato_csv() -> anyhow::Result<()> {
    let test_file_path = TEST_FILES_DIR.join("serato.csv");
    assert!(
        test_file_path.is_file(),
        "File does not exist: {}",
        test_file_path.display()
    );
    let playlist = Playlist::new(&test_file_path)?;
    assert_eq!(playlist.file_format, FileFormat::Csv);
    assert_eq!(playlist.name, "Serato 30.3.2023".to_string());
    assert_eq!(playlist.playlist_type, PlaylistType::Serato);
    assert_eq!(playlist.tracks.len(), 4);
    assert_eq!(
        playlist.date,
        Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 3, 30).unwrap(),
            NaiveTime::from_hms_opt(16, 4, 53).unwrap()
        ))
    );
    Ok(())
}

#[test]
fn test_serato_txt() -> anyhow::Result<()> {
    let test_file_path = TEST_FILES_DIR.join("serato.txt");
    assert!(
        test_file_path.is_file(),
        "File does not exist: {}",
        test_file_path.display()
    );
    let playlist = Playlist::new(&test_file_path)?;
    assert_eq!(playlist.file_format, FileFormat::Txt);
    assert_eq!(playlist.name, "Serato 30.3.2023".to_string());
    assert_eq!(playlist.playlist_type, PlaylistType::Serato);
    assert_eq!(playlist.tracks.len(), 4);
    assert_eq!(
        playlist.date,
        Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 3, 30).unwrap(),
            NaiveTime::from_hms_opt(16, 4, 53).unwrap()
        ))
    );
    Ok(())
}

#[test]
fn test_rekordbox_txt() -> anyhow::Result<()> {
    let test_file_path = TEST_FILES_DIR.join("rekordbox.txt");
    assert!(
        test_file_path.is_file(),
        "File does not exist: {}",
        test_file_path.display()
    );
    let playlist = Playlist::new(&test_file_path)?;
    assert_eq!(playlist.file_format, FileFormat::Txt);
    assert_eq!(playlist.name, "rekordbox".to_string());
    assert_eq!(playlist.playlist_type, PlaylistType::Rekordbox);
    assert_eq!(playlist.tracks.len(), 28);
    assert_eq!(playlist.date, None);
    Ok(())
}

#[test]
fn test_rekordbox_txt_with_date() -> anyhow::Result<()> {
    let test_file_path = TEST_FILES_DIR.join("rekordbox-2020.12.20.txt");
    assert!(
        test_file_path.is_file(),
        "File does not exist: {}",
        test_file_path.display()
    );
    let playlist = Playlist::new(&test_file_path)?;
    assert_eq!(playlist.file_format, FileFormat::Txt);
    assert_eq!(playlist.name, "rekordbox-2020.12.20".to_string());
    assert_eq!(playlist.playlist_type, PlaylistType::Rekordbox);
    assert_eq!(playlist.tracks.len(), 28);
    assert_eq!(
        playlist.date,
        Some(NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2020, 12, 20).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap()
        ))
    );
    Ok(())
}

#[test]
fn test_formatted_csv() -> anyhow::Result<()> {
    let test_file_path = TEST_FILES_DIR.join("formatted.csv");
    assert!(
        test_file_path.is_file(),
        "File does not exist: {}",
        test_file_path.display()
    );
    let playlist = Playlist::new(&test_file_path)?;
    assert_eq!(playlist.file_format, FileFormat::Csv);
    assert_eq!(playlist.playlist_type, PlaylistType::Formatted);
    assert_eq!(playlist.date, None);
    assert_eq!(playlist.tracks.len(), 24);
    Ok(())
}
