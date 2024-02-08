use chrono::{Duration, NaiveDateTime};

use std::fmt;
use std::ops::{Add, AddAssign};
use std::string::String;

/// Represents one played track
#[derive(Debug, Clone)]
pub struct Track {
    pub artist: String,
    pub title: String,
    pub start_time: Option<NaiveDateTime>,
    pub end_time: Option<NaiveDateTime>,
    pub play_time: Option<Duration>,
}

impl Track {
    /// Create a simple track with only artist name and song title
    pub fn new(artist: String, title: String) -> Track {
        Track {
            artist,
            title,
            start_time: None,
            end_time: None,
            play_time: None,
        }
    }

    /// Create a track with full information including start and play time.
    pub fn new_with_time(
        artist: String,
        title: String,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
        play_time: Option<Duration>,
    ) -> Track {
        Track {
            artist,
            title,
            start_time,
            end_time,
            play_time,
        }
    }

    /// Get the number of characters the artist name has.
    pub fn artist_length(&self) -> usize {
        // .len() counts bytes, not chars
        self.artist.chars().count()
    }

    /// Get the number of characters the song title has.
    pub fn title_length(&self) -> usize {
        self.title.chars().count()
    }
}

impl Add<Option<Duration>> for Track {
    type Output = Track;
    fn add(self, duration: Option<Duration>) -> Track {
        let play_time = match self.play_time {
            Some(time) => match duration {
                None => Some(time),
                Some(d) => Some(time + d),
            },
            None => duration,
        };
        Track {
            artist: self.artist,
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
            play_time,
        }
    }
}

impl AddAssign<Duration> for Track {
    fn add_assign(&mut self, duration: Duration) {
        if let Some(time) = self.play_time {
            self.play_time = Some(time + duration)
        } else {
            self.play_time = Some(duration)
        }
    }
}

impl AddAssign<Option<Duration>> for Track {
    fn add_assign(&mut self, duration: Option<Duration>) {
        if let Some(d) = duration {
            if let Some(time) = self.play_time {
                self.play_time = Some(time + d)
            } else {
                self.play_time = Some(d)
            }
        }
    }
}

impl Add<Duration> for Track {
    type Output = Track;
    fn add(self, duration: Duration) -> Track {
        Track {
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
