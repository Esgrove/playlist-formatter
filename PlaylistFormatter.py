"""
Playlist Formatter Tool
Akseli Lukkarila
2018
"""
import csv
import os
import sys
from datetime import datetime, timedelta
from enum import Enum, auto
from pathlib import Path
from typing import Optional

import chardet
import colorama
from titlecase import titlecase

from colorprint import Color, get_color, print_bold, print_color


class PlaylistType(Enum):
    SERATO = auto()
    REKORDBOX = auto()


class Track:
    def __init__(
        self,
        artist: str,
        title: str,
        relative_time: datetime = None,
        start_time: datetime = None,
        play_time: datetime = None,
    ):
        self.artist = " ".join(artist.strip().split())
        self.title = " ".join(title.strip().split())
        self.time = relative_time
        self.start_time = start_time
        self.play_time = play_time


class PlaylistFormatter:
    """Reads a playlist text file and creates correctly formatted csv or excel."""

    def __init__(self):
        self.filename = ""
        self.filepath = ""
        self.filetype = ""
        self.playlist: list[Track] = []
        self.playlist_date = None
        self.playlist_file = None
        self.playlist_name = ""
        self.playlist_type: Optional[PlaylistType] = None

    def read_playlist(self, filename: str):
        if not os.path.isfile(filename):
            raise RuntimeError("File does not exist.")

        print(f"Reading playlist: {get_color(filename, Color.yellow)}")
        self.filepath, self.filename = os.path.split(filename)
        self.filename, self.filetype = os.path.splitext(self.filename)
        self.filetype = self.filetype.strip().lower()
        if self.filetype == ".csv":
            self._read_csv(filename)
        elif self.filetype == ".txt":
            self._read_txt(filename)
        elif self.filetype in (".xlsx", ".xlsm", ".xltx", ".xltm"):
            self._read_xls(filename)
        else:
            raise RuntimeError(f"Unsupported filetype '{self.filetype}'!")

    @staticmethod
    def _format_title(title: str) -> str:
        """Format song title."""

        title = title.replace(" (Clean)", "").replace(" (clean)", "")
        title = title.replace(" (Dirty)", "").replace(" (dirty)", "")
        title = title.replace(" (Original Mix)", "").replace(" (original mix)", "")
        title = title.replace(" (Dirty-", " (").replace(" (dirty-", " (")
        title = title.replace(" (Clean-", " (").replace(" (clean-", " (")

        # TODO: clean this up
        if " - " in title:
            dash_index = title.index(" - ")
            if " (" in title and ")" in title:
                parenthesis_start = title.index(" (")
                parenthesis_end = title.index(")")
                if parenthesis_start < dash_index < parenthesis_end:
                    title = title.replace(" - ", " ")
                else:
                    # If there are more parentheses in the title,
                    # insert the closing parenthesis before the existing one
                    if " (" in title[dash_index:] and ")" in title[dash_index:]:
                        opening_parenthesis_index = dash_index + title[dash_index:].index(" (")
                        title_before = title[:opening_parenthesis_index]
                        title_after = title[opening_parenthesis_index:]
                        title = title_before + ")" + title_after
                    else:
                        title = title + ")"

                    title = title.replace(" - ", " (")
            else:
                title = title.replace(" - ", " (")
                title = title + ")"

        # split at all whitespace chars and recombine -> remove extra spaces and linebreaks...
        title = " ".join(title.split())

        return title

    def _read_csv(self, filename):
        with open(filename) as csv_file:
            playlist_data = csv.DictReader(csv_file)

            previous_time = timedelta()
            playlist = []
            playlist_index = 0
            for index, row_data in enumerate(playlist_data):
                if index == 0 and "name" in row_data and "start time" in row_data:
                    # info row
                    self.playlist_name = row_data["name"]
                    self.playlist_date = row_data["start time"].split(",")[0]
                    continue

                time_string = row_data["start time"].replace(".", ":").strip().split(" ")[0]
                row_data["start time"] = datetime.strptime(time_string, "%H:%M:%S")

                if index == 1:
                    start_time = row_data["start time"]

                title = row_data["name"]
                title = self._format_title(title)

                play_time = row_data["start time"] - start_time
                song_data = {
                    "artist": titlecase(row_data["artist"]),
                    "song": titlecase(title),
                    "time": play_time,
                    "playtime": play_time - previous_time,
                    "starttime": row_data["start time"],
                }

                if song_data["playtime"] < timedelta(seconds=60):
                    song_data["playtime"] = timedelta(seconds=60)

                # sum duplicate song play times
                if (
                    playlist_index
                    and playlist[playlist_index - 1]["song"] == song_data["song"]
                    and playlist[playlist_index - 1]["artist"] == song_data["artist"]
                ):
                    playlist[playlist_index - 1]["playtime"] += song_data["playtime"]
                else:
                    playlist.append(song_data)
                    playlist_index += 1
                    previous_time = play_time

            for i in range(1, len(playlist)):
                playlist[i - 1]["playtime"] = playlist[i]["playtime"]

            self.playlist = playlist
            self.playlist_file = filename
            self.playlist_type = PlaylistType.SERATO

    def _read_xls(self, filename: str):
        # TODO
        raise NotImplementedError

    def _read_txt(self, filename: str):
        filepath = Path(filename)
        if not filepath.exists():
            sys.exit(f"File does not exist: {filename}")

        raw_data = filepath.read_bytes()
        detection = chardet.detect(raw_data)
        encoding = detection["encoding"]
        confidence = detection["confidence"]
        print(f"Encoding: {encoding} ({confidence})")
        with open(filename, encoding=encoding) as txt_file:
            lines = txt_file.readlines()

        if not lines:
            raise RuntimeError(f"File is empty: {filename}")

        # Rekordbox txt
        if lines[0].startswith("#"):
            self.playlist_type = PlaylistType.REKORDBOX
            playlist = []
            # Rekordbox output:
            # Track, Title, Artist, BPM, Time, Key, Genre, Date Added
            for row in lines[1:]:
                data = row.split("\t")
                artist = data[2]
                title = self._format_title(data[1])
                track = {"artist": artist, "song": title}
                # skip duplicate tracks
                if len(playlist) < 1 or playlist[-1] != track:
                    playlist.append(track)
        else:
            raise NotImplementedError

        self.playlist = playlist
        self.playlist_file = filename

    def export_csv(self, filename=None):
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        out_filename = filename if filename else self.filename
        if not out_filename.endswith(".csv"):
            out_filename += ".csv"

        print(f"Exporting as: {get_color(out_filename, Color.green)}")

        out_file = os.path.join(self.filepath, out_filename)
        with open(out_file, "w", newline="") as csv_file:
            csv_writer = csv.writer(csv_file, delimiter=",")
            if self.playlist_type == PlaylistType.REKORDBOX:
                csv_writer.writerow(["Artist", "", "Song"])
                for row in self.playlist:
                    csv_writer.writerow(
                        [
                            row["artist"],
                            "-",
                            row["song"],
                        ]
                    )
            else:
                csv_writer.writerow(["Artist", "", "Song", "Time", "Playtime", "Start time"])
                for row in self.playlist:
                    csv_writer.writerow(
                        [
                            row["artist"],
                            "-",
                            row["song"],
                            str(row["time"]).split(", ")[-1],
                            str(row["playtime"]).split(", ")[-1],
                            row["starttime"].strftime("%H:%M:%S"),
                        ]
                    )

    def print_playlist(self):
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        total_tracks = len(self.playlist)
        print(
            f"Printing playlist: {self.playlist_name if self.playlist_name else self.filename} "
            f"({get_color(self.playlist_type.name, Color.cyan)})\n"
            f"Total tracks: {total_tracks}"
        )

        width_artist = max(len(row["artist"]) for row in self.playlist)
        width_title = max(len(row["song"]) for row in self.playlist)

        if self.playlist_type == PlaylistType.REKORDBOX:
            heading = "{:<{width_artist}s} {:<{width_title}s}".format(
                "ARTIST",
                "SONG",
                width_artist=width_artist + 2,
                width_title=width_title,
            )
            print_bold(heading)
            print_color("".join(["-"] * len(heading)))

            for number, track in enumerate(self.playlist, 1):
                print(
                    "{:>{width_number}d}: {:<{width_artist}s} - {:<{width_title}s}".format(
                        number,
                        track["artist"],
                        track["song"],
                        width_number=len(str(total_tracks)),
                        width_artist=width_artist,
                        width_title=width_title,
                    )
                )
        else:
            heading = "{:<{width_artist}s} {:<{width_title}s}   {:9s} {:9s} {:9s}".format(
                "ARTIST",
                "SONG",
                "TIME",
                "PLAYTIME",
                "STARTTIME",
                width_artist=width_artist + 2,
                width_title=width_title,
            )
            print_bold(heading)
            print_color("".join(["-"] * len(heading)))

            for row in self.playlist:
                print(
                    "{:<{width_artist}s} - {:<{width_title}s}   {}   {}   {}".format(
                        row["artist"],
                        row["song"],
                        Color.yellow + str(row["time"]).split(", ")[-1],
                        Color.green + str(row["playtime"]).split(", ")[-1],
                        Color.blue + row["starttime"].strftime("%H:%M:%S"),
                        width_artist=width_artist,
                        width_title=width_title,
                    )
                    + colorama.Style.RESET_ALL
                )

        print_color("".join(["-"] * len(heading)))

    def format_playlist(self) -> str:
        """
        Return formatted playlist for printing.
        Returns a list of formatted song strings.
        """
        playlist = []
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        width_artist = max(len(row["artist"]) for row in self.playlist)
        width_title = max(len(row["song"]) for row in self.playlist)

        for row in self.playlist:
            playlist.append(
                "{:<{widthArtist}s} - {:<{widthTitle}s}   {}".format(
                    row["artist"],
                    row["song"],
                    str(row["time"]).split(", ")[-1],
                    widthArtist=width_artist,
                    widthTitle=width_title,
                )
            )

        return playlist
