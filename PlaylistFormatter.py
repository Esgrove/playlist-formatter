"""
Playlist Formatter Tool
Akseli Lukkarila
2018
"""
import csv
import os
import platform
import time
from datetime import datetime, timedelta
from timeit import default_timer as timer

import colorama
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.support.ui import Select, WebDriverWait
from titlecase import titlecase

from colorprint import Color, get_color, print_color, print_bold


class PlaylistFormatter:
    """Reads a playlist text file and creates correctly formatted csv or excel."""

    def __init__(self):
        self.playlist_file = None
        self.playlist_date = None
        self.playlist_name = ""
        self.filepath = ""
        self.filename = ""
        self.filetype = ""
        self.playlist = []
        self.driver = None
        if platform.system().lower() == "darwin":  # MacOS
            self.driverPath = "/usr/local/bin/chromedriver"
        else:
            self.driverPath = "C:\\ProgramData\\chocolatey\\bin\\chromedriver.exe"

    def read_playlist(self, filename):
        if not os.path.isfile(filename):
            raise RuntimeError("File does not exist.")

        print(f"reading playlist {get_color(filename, Color.yellow)}\n")
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

        self.print_playlist()

    def _read_csv(self, filename):
        with open(filename) as csvFile:
            playlist_data = csv.DictReader(csvFile)

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
                if " - " in title:
                    title = title.replace(" - ", " (") + ")"

                title = title.replace("(Clean)", "").replace("(clean)", "")
                title = title.replace("(Dirty)", "").replace("(dirty)", "")
                title = title.replace("(Original Mix)", "").replace("(original Mix)", "")
                title = title.replace("(Dirty-", "(").replace("(dirty-", "(")
                title = title.replace("(Clean-", "(").replace("(clean-", "(")
                title = title.replace(" )", ")")
                title = title.replace("( ", "(")

                # split at all whitespace chars and recombine -> remove extra spaces and linebreaks...
                title = " ".join(title.split())

                play_time = row_data["start time"] - start_time
                song_data = {"artist": titlecase(row_data["artist"]),
                             "song": titlecase(title),
                             "time": play_time,
                             "playtime": play_time - previous_time,
                             "starttime": row_data["start time"]}

                if song_data["playtime"] < timedelta(seconds=60):
                    song_data["playtime"] = timedelta(seconds=60)

                # sum duplicate song playtimes
                if playlist_index and playlist[playlist_index - 1]["song"] == song_data["song"] and \
                        playlist[playlist_index - 1]["artist"] == song_data["artist"]:
                    playlist[playlist_index - 1]["playtime"] += song_data["playtime"]

                else:
                    playlist.append(song_data)
                    playlist_index += 1
                    previous_time = play_time

            for i in range(1, len(playlist)):
                playlist[i - 1]["playtime"] = playlist[i]["playtime"]

            self.playlist = playlist
            self.playlist_file = filename

    def _read_xls(self, filename):
        # TODO
        raise NotImplementedError

    def _read_txt(self, filename):
        # TODO
        raise NotImplementedError

    def export_csv(self, filename=None):
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        out_filename = filename if filename else self.filename
        if not out_filename.endswith(".csv"):
            out_filename += ".csv"

        out_file = os.path.join(self.filepath, out_filename)
        with open(out_file, "w", newline="") as csvFile:
            csv_writer = csv.writer(csvFile, delimiter=",")
            csv_writer.writerow(["Artist", "", "Song", "Time", "Playtime", "Start time"])
            for row in self.playlist:
                csv_writer.writerow([row["artist"],
                                     "-",
                                     row["song"],
                                     str(row["time"]).split(", ")[-1],
                                     str(row["playtime"]).split(", ")[-1],
                                     row["starttime"].strftime("%H:%M:%S")])

    def print_playlist(self):
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        width_artist = max(len(row["artist"]) for row in self.playlist)
        width_title = max(len(row["song"]) for row in self.playlist)
        heading = "{:<{width_artist}s} {:<{width_title}s}   {:9s} {:9s} {:9s}".format(
            "ARTIST",
            "SONG",
            "TIME",
            "PLAYTIME",
            "STARTTIME",
            width_artist=width_artist + 2,
            width_title=width_title)
        print_bold(heading)
        print_color("".join(["-"] * len(heading)))

        for row in self.playlist:
            print("{:<{width_artist}s} - {:<{width_title}s}   {}   {}   {}".format(
                row["artist"],
                row["song"],
                Color.yellow + str(row["time"]).split(", ")[-1],
                Color.green + str(row["playtime"]).split(", ")[-1],
                Color.blue + row["starttime"].strftime("%H:%M:%S"),
                width_artist=width_artist,
                width_title=width_title) +
                  colorama.Style.RESET_ALL)

        print_color("".join(["-"] * len(heading)) + "\n")

    def format_playlist(self):
        """
        Return formatted playlist for printing.
            Returns (str): list of formatted song strings
        """
        playlist = []
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        width_artist = max(len(row["artist"]) for row in self.playlist)
        width_title = max(len(row["song"]) for row in self.playlist)

        for row in self.playlist:
            playlist.append("{:<{widthArtist}s} - {:<{widthTitle}s}   {}".format(
                row["artist"],
                row["song"],
                str(row["time"]).split(", ")[-1],
                widthArtist=width_artist,
                widthTitle=width_title))

        return playlist

    def fill_basso(self, show, start_index=0):
        """Fill radioshow playlist to Bassoradio database using Selenium."""
        print_bold("Uploading playlist to dj.basso.fi...", Color.red)
        start_time = timer()

        if len(self.playlist) <= start_index:
            print("Index not valid.")
            return

        self.open_basso_driver(show)

        print("\nFilling playlist for show:")
        print_color(show, Color.cyan)

        # input song data
        print_color("\nAdding songs...", Color.magenta)
        for index, row in enumerate(self.playlist[start_index:]):
            input_index = 0
            print("  {:d}: {:s} - {:s}".format(index + 1, row["artist"], row["song"]))
            while True:
                # increase index so we don't send the first letter multiple times when trying again
                input_index += 1
                try:
                    time.sleep(0.5)
                    find_track = WebDriverWait(self.driver, 10).until(
                        EC.presence_of_element_located((By.ID, "find-track-textfield")))
                    find_track.send_keys(row["artist"][:input_index])

                    WebDriverWait(self.driver, 10).until(
                        EC.presence_of_element_located((By.ID, "new-track-entry-form")))

                    artist = WebDriverWait(self.driver, 10).until(
                        EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.artist']")))
                    time.sleep(0.5)
                    artist.send_keys(row["artist"][input_index:])

                    song = WebDriverWait(self.driver, 10).until(
                        EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.title']")))
                    song.send_keys(row["song"])

                    mins = row["playtime"].seconds // 60
                    minutes = WebDriverWait(self.driver, 10).until(
                        EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.minutes']")))
                    minutes.send_keys(mins)

                    secs = row["playtime"].seconds % 60
                    seconds = WebDriverWait(self.driver, 10).until(
                        EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.seconds']")))
                    seconds.send_keys(secs)

                    save = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable(
                        (By.XPATH, "//input[@type='button' and @value='Tallenna uusi biisi']")))
                    save.click()

                    submit_button = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable(
                        (By.XPATH, "//input[@type='submit' and @value='Lisää biisilistaan']")))
                    submit_button.click()

                except Exception as e:
                    print_color(str(e), Color.red)
                    continue
                else:
                    break

        print_color(f"Done in {timer() - start_time:.2f} seconds!", Color.green)

    def open_basso_driver(self, show):
        if not self.driver:
            # open webdriver if not already open
            self.driver = webdriver.Chrome(executable_path=self.driverPath)

        self.driver.get("Basso website here...")

        # clear current show
        self.driver.find_element_by_id("broadcast-title-clear").click()

        # select correct show
        select = Select(self.driver.find_element_by_css_selector("[ng-model*='play.broadcast']"))
        select.select_by_visible_text(show)

    @staticmethod
    def get_show_string(date, show_name):
        return f"{date} 20:00-22:00 LIVE {show_name}"
