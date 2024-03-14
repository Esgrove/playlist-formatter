use std::path::PathBuf;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use lazy_static::lazy_static;

use crate::playlist::Playlist;
use crate::utils::{FileFormat, PlaylistType};

lazy_static! {
    /// Path to the `tests/files` directory.
    static ref TEST_FILES_DIR: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("files");
}

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
