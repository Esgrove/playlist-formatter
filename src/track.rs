use std::fmt;
use std::ops::{Add, AddAssign};
use std::string::String;

use chrono::{NaiveDateTime, TimeDelta};

/// Represents one played track.
#[derive(Debug, Clone)]
pub struct Track {
    pub artist: String,
    pub title: String,
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub play_time: Option<TimeDelta>,
}

impl Track {
    /// Create a simple track with only artist name and song title.
    #[must_use]
    pub const fn new(artist: String, title: String) -> Self {
        Self {
            artist,
            title,
            start_time: None,
            end_time: None,
            play_time: None,
        }
    }

    /// Create a track with full information including start and play time.
    #[must_use]
    pub const fn new_with_time(
        artist: String,
        title: String,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
        play_time: Option<TimeDelta>,
    ) -> Self {
        Self {
            artist,
            title,
            start_time,
            end_time,
            play_time,
        }
    }

    /// Get the number of characters the artist name has.
    #[must_use]
    pub fn artist_length(&self) -> usize {
        // .len() counts bytes, not chars
        self.artist.chars().count()
    }

    /// Get the number of characters the song title has.
    #[must_use]
    pub fn title_length(&self) -> usize {
        self.title.chars().count()
    }
}

impl Add<Option<TimeDelta>> for Track {
    type Output = Self;
    fn add(self, duration: Option<TimeDelta>) -> Self {
        let play_time = match self.play_time {
            Some(time) => match duration {
                None => Some(time),
                Some(d) => Some(time + d),
            },
            None => duration,
        };
        Self {
            artist: self.artist,
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
            play_time,
        }
    }
}

impl AddAssign<TimeDelta> for Track {
    fn add_assign(&mut self, duration: TimeDelta) {
        if let Some(time) = self.play_time {
            self.play_time = Some(time + duration);
        } else {
            self.play_time = Some(duration);
        }
    }
}

impl AddAssign<Option<TimeDelta>> for Track {
    fn add_assign(&mut self, duration: Option<TimeDelta>) {
        if let Some(d) = duration {
            if let Some(time) = self.play_time {
                self.play_time = Some(time + d);
            } else {
                self.play_time = Some(d);
            }
        }
    }
}

impl Add<TimeDelta> for Track {
    type Output = Self;
    fn add(self, duration: TimeDelta) -> Self {
        Self {
            artist: self.artist,
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
            play_time: if let Some(time) = self.play_time {
                Some(time + duration)
            } else {
                Some(duration)
            },
        }
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.artist == other.artist && self.title == other.title
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.artist, self.title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeDelta};

    #[test]
    fn new_track() {
        let track = Track::new("Artist".to_string(), "Title".to_string());
        assert_eq!(track.artist, "Artist");
        assert_eq!(track.title, "Title");
        assert!(track.start_time.is_none());
        assert!(track.end_time.is_none());
        assert!(track.play_time.is_none());
    }

    #[test]
    fn new_track_with_time() {
        let start_time = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let end_time = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(13, 0, 0)
            .unwrap();
        let play_time = TimeDelta::try_hours(1).unwrap();
        let track = Track::new_with_time(
            "Artist".to_string(),
            "Title".to_string(),
            Some(start_time),
            Some(end_time),
            Some(play_time),
        );
        assert_eq!(track.start_time.unwrap(), start_time);
        assert_eq!(track.end_time.unwrap(), end_time);
        assert_eq!(track.play_time.unwrap(), play_time);
    }

    #[test]
    fn equals() {
        let start_time = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let end_time = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(13, 0, 0)
            .unwrap();
        let play_time = TimeDelta::try_minutes(2).unwrap();
        let track1 = Track::new_with_time(
            "Some Artist".to_string(),
            "Song Title (Remix)".to_string(),
            Some(start_time),
            Some(end_time),
            Some(play_time),
        );
        let track2 = Track::new("Some Artist".to_string(), "Song Title (Remix)".to_string());
        assert_eq!(track1, track2);
    }

    #[test]
    fn lengths() {
        let track = Track::new("Artist".to_string(), "Title".to_string());
        assert_eq!(track.artist_length(), 6);
        assert_eq!(track.title_length(), 5);
    }

    #[test]
    fn add_duration_to_track_without_initial_play_time() {
        let track = Track::new("Artist".to_string(), "Title".to_string());
        let duration = TimeDelta::try_minutes(5);
        let result = track + duration;
        assert_eq!(result.play_time.unwrap(), TimeDelta::try_minutes(5).unwrap());
    }

    #[test]
    fn add_duration_to_track_with_initial_play_time() {
        let start_time = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(12, 0, 0);
        let end_time = start_time.unwrap() + TimeDelta::try_minutes(30).unwrap();
        let track = Track::new_with_time(
            "Artist".to_string(),
            "Title".to_string(),
            start_time,
            Some(end_time),
            Some(TimeDelta::try_minutes(30).unwrap()),
        );
        let duration = TimeDelta::try_minutes(15);
        let result = track + duration;
        assert_eq!(result.play_time, TimeDelta::try_minutes(45));
    }

    #[test]
    fn add_none_to_track() {
        let mut track = Track::new("Artist".to_string(), "Title".to_string());
        track += None;
        assert!(track.play_time.is_none());
    }

    #[test]
    fn add_some_duration_to_track_without_initial_play_time() {
        let mut track = Track::new("Artist".to_string(), "Title".to_string());
        let duration = TimeDelta::try_minutes(10);
        track += duration;
        assert_eq!(track.play_time, TimeDelta::try_minutes(10));
    }

    #[test]
    fn add_some_duration_to_track_with_initial_play_time() {
        let start_time = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap().and_hms_opt(12, 0, 0);
        let end_time = start_time.unwrap() + TimeDelta::try_hours(1).unwrap();
        let mut track = Track::new_with_time(
            "Artist".to_string(),
            "Title".to_string(),
            start_time,
            Some(end_time),
            TimeDelta::try_hours(1),
        );
        let duration = Some(TimeDelta::try_minutes(30).unwrap());
        track += duration;
        assert_eq!(track.play_time, TimeDelta::try_minutes(90));
    }
}
