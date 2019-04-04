"""
Playlist Formatter Tool
Akseli Lukkarila
2018
"""
import csv
import os
import sys
import time
import platform

from datetime import datetime, timedelta
from timeit import default_timer as timer

import colorama
import openpyxl
import requests
from titlecase import titlecase

from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.support.ui import Select, WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

from PyQt5.Qt import PYQT_VERSION_STR, QSizePolicy
from PyQt5.QtCore import Qt, QT_VERSION_STR
from PyQt5.QtGui import QIcon, QColor, QPalette, QKeySequence, QFont
from PyQt5.QtWidgets import (QApplication, QWidget, QFileDialog, QStyle, QTreeWidgetItem, QHeaderView,
                            QMainWindow, QMenuBar, QAbstractItemView, QGridLayout, QAction, QMessageBox,
                            QDesktopWidget, QPushButton, QListWidget, QFontDialog, QLineEdit, QLabel, QTreeWidget)

# ==================================================================================

class PlaylistFormatter:
    """Reads a playlist textfile and creates correctly formatted csv or excel"""
    def __init__(self):
        self.playlistFile = None
        self.playlistDate = None
        self.playlistName = ""
        self.filepath = ""
        self.filename = ""
        self.filetype = ""
        self.playlist = []
        self.driver = None
        if platform.system().lower() == "darwin": # MacOS
            self.driverPath = "/Users/Dropbox/CODE/webdriver/chromedriver"
        else:
            self.driverPath = "D:/Dropbox/CODE/webdriver/chromedriver.exe"

    # ------------------------------------------------------------------------------

    def readPlaylist(self, filename):
        if not os.path.isfile(filename):
            raise RuntimeError("File does not exist.")
        
        print("reading playlist {}\n".format(getColor(filename, colorama.Fore.YELLOW)))
        self.filepath, self.filename = os.path.split(filename)
        self.filename, self.filetype = os.path.splitext(self.filename)
        self.filetype = self.filetype.strip().lower()
        if self.filetype == ".csv":
            self._readCSV(filename)

        elif self.filetype == ".txt":
            self._readTXT(filename)

        elif self.filetype in (".xlsx", ".xlsm", ".xltx", ".xltm"):
            self._readXLS(filename)

        else:
            raise RuntimeError("Unsupported filetype \"{}\".".format(self.filetype))

        self.printPlaylist()

    # ------------------------------------------------------------------------------

    def _readCSV(self, filename):
        try:
            with open(filename) as csvFile:
                playlistData = csv.DictReader(csvFile)

                previousTime = timedelta()
                playlist = []
                playlistIndex = 0
                for index, rowData in enumerate(playlistData):
                    if index == 0:
                        self.playlistName = rowData["name"]
                        self.playlistDate = rowData["start time"].split(",")[0]
                        continue

                    timeString = rowData["start time"].replace(".", ":").strip().split(" ")[0]
                    rowData["start time"] = datetime.strptime(timeString, "%H:%M:%S")

                    if index == 1:
                        startTime = rowData["start time"]

                    title = rowData["name"]
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

                    playTime = rowData["start time"] - startTime
                    songData = {"artist": titlecase(rowData["artist"]), 
                                "song": titlecase(title),
                                "time": playTime,
                                "playtime": playTime - previousTime,
                                "starttime": rowData["start time"]}

                    if songData["playtime"] < timedelta(seconds=60):
                        songData["playtime"] = timedelta(seconds=60)

                    # sum duplicate song playtimes
                    if playlistIndex and playlist[playlistIndex-1]["song"] == songData["song"] and playlist[playlistIndex-1]["artist"] == songData["artist"]:
                        playlist[playlistIndex-1]["playtime"] += songData["playtime"]

                    else:
                        playlist.append(songData)
                        playlistIndex += 1
                        previousTime = playTime

                for i in range(1, len(playlist)):
                    playlist[i-1]["playtime"] = playlist[i]["playtime"]

                self.playlist = playlist
                self.playlistFile = filename

        except Exception:
            errorType, errorValue, _ = sys.exc_info()
            raise RuntimeError("Error reading CSV:\n{}: {}".format(str(errorType), str(errorValue)))

    # ------------------------------------------------------------------------------

    def _readXLS(self, filename):
        # TODO
        pass

    # ------------------------------------------------------------------------------

    def _readTXT(self, filename):
        # TODO
        pass

    # ------------------------------------------------------------------------------

    def exportCSV(self, filename = None):
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        outFilename = filename if filename else self.filename
        if not outFilename.endswith(".csv"):
            outFilename += ".csv"

        outFile = os.path.join(self.filepath, outFilename)
        with open(outFile, "w", newline = "") as csvFile:
            csvWriter = csv.writer(csvFile, delimiter = ",")
            csvWriter.writerow(["Artist", "", "Song", "Time", "Playtime", "Start time"])
            for row in self.playlist:
                csvWriter.writerow([row["artist"],
                                    "-",
                                    row["song"],
                                    str(row["time"]).split(", ")[-1],
                                    str(row["playtime"]).split(", ")[-1],
                                    row["starttime"].strftime("%H:%M:%S")])

    # ------------------------------------------------------------------------------

    def printPlaylist(self):
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        widthArtist = max(len(row["artist"]) for row in self.playlist)
        widthTitle  = max(len(row["song"])  for row in self.playlist)
        heading = "{:<{widthArtist}s} {:<{widthTitle}s}   {:9s} {:9s} {:9s}".format(
                    "ARTIST", 
                    "SONG", 
                    "TIME", 
                    "PLAYTIME",
                    "STARTTIME", 
                    widthArtist = widthArtist + 2, 
                    widthTitle = widthTitle)
        printBold(heading)
        printColor("".join(["-"] * len(heading)), colorama.Fore.LIGHTBLACK_EX)

        for row in self.playlist:
            print("{:<{widthArtist}s} - {:<{widthTitle}s}   {}   {}   {}".format(
                    row["artist"], 
                    row["song"],
                    colorama.Fore.YELLOW + str(row["time"]).split(", ")[-1],
                    colorama.Fore.GREEN + str(row["playtime"]).split(", ")[-1], 
                    colorama.Fore.BLUE + row["starttime"].strftime("%H:%M:%S"), 
                    widthArtist = widthArtist,
                    widthTitle = widthTitle) + 
                    colorama.Style.RESET_ALL)  

        printColor("".join(["-"] * len(heading)) + "\n", colorama.Fore.LIGHTBLACK_EX)

    # ------------------------------------------------------------------------------

    def formatPlaylist(self):
        """
        Return formatted playlist for printing.
            Returns (str): list of formatted song strings
        """
        playlist = []
        if not self.playlist:
            raise RuntimeError("No playlist. Read a playlist first!")

        widthArtist = max(len(row["artist"]) for row in self.playlist)
        widthTitle  = max(len(row["song"])  for row in self.playlist)

        for row in self.playlist:
            playlist.append("{:<{widthArtist}s} - {:<{widthTitle}s}   {}".format(
                    row["artist"],
                    row["song"],
                    str(row["time"]).split(", ")[-1],
                    widthArtist = widthArtist,
                    widthTitle = widthTitle))

        return playlist

    # ------------------------------------------------------------------------------

    def fillBasso(self, show, startIndex = 0):
        """Fill radioshow playlist to Bassoradio database using Selenium"""
        printBold("Uploading playlist to dj.basso.fi...", colorama.Fore.RED)
        startTime = timer()

        if len(self.playlist) <= startIndex:
            print("Index not valid.")
            return

        self.openBassoDriver(show)

        print("\nFilling playlist for show:")
        printColor(show, colorama.Fore.CYAN)

        # input song data
        printColor("\nAdding songs...", colorama.Fore.MAGENTA)
        for index, row in enumerate(self.playlist[startIndex:]):
            inputIndex = 0
            print("  {:d}: {:s} - {:s}".format(index + 1, row["artist"], row["song"]))
            while True:
                # increase index so we don't send the first letter multiple times when trying again
                inputIndex += 1
                try:
                    time.sleep(0.5)
                    findTrack = WebDriverWait(self.driver, 10).until(EC.presence_of_element_located((By.ID, "find-track-textfield")))
                    findTrack.send_keys(row["artist"][:inputIndex])

                    WebDriverWait(self.driver, 10).until(EC.presence_of_element_located((By.ID, "new-track-entry-form")))

                    artist = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.artist']")))
                    time.sleep(0.5)
                    artist.send_keys(row["artist"][inputIndex:])

                    song = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.title']")))
                    song.send_keys(row["song"])

                    mins = row["playtime"].seconds // 60
                    minutes = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.minutes']")))
                    minutes.send_keys(mins)

                    secs = row["playtime"].seconds % 60
                    seconds = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable((By.CSS_SELECTOR, "[ng-model*='newTrack.seconds']")))
                    seconds.send_keys(secs)

                    save = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable((By.XPATH, "//input[@type='button' and @value='Tallenna uusi biisi']")))
                    save.click()

                    submitButton = WebDriverWait(self.driver, 10).until(EC.element_to_be_clickable((By.XPATH, "//input[@type='submit' and @value='Lisää biisilistaan']")))
                    submitButton.click()

                except Exception as e:
                    printColor(str(e), colorama.Fore.RED)
                    continue
                else:
                    break

        printColor("Done in {:.2f} seconds!".format(timer() - startTime), colorama.Fore.GREEN)

    # ------------------------------------------------------------------------------

    def openBassoDriver(self, show):
        if not self.driver:
            # open webdriver if not already open
            self.driver = webdriver.Chrome(executable_path = self.driverPath)

        self.driver.get("Basso website here...")

        # clear current show
        self.driver.find_element_by_id("broadcast-title-clear").click()

        # select correct show
        select = Select(self.driver.find_element_by_css_selector("[ng-model*='play.broadcast']"))
        select.select_by_visible_text(show)

    # ------------------------------------------------------------------------------

    def getShowString(self, date, showName):
        return "{} 20:00-22:00 LIVE {}".format(date, showName)


# ==================================================================================

class PlaylistTool(QMainWindow):
    def __init__(self):
        super().__init__()

        self.formatter = PlaylistFormatter()
        if platform.system().lower() == "darwin": # MacOS
            self.defaultPath = os.path.expanduser("~/Dropbox")
        else:
            self.defaultPath = 'D:/Dropbox'

        self.initUI()

    # ------------------------------------------------------------------------------

    def initUI(self):
        self.setWindowTitle("Esgrove's Playlist Tool")
        self.setWindowIcon(self.style().standardIcon(QStyle.SP_MediaPlay))
        self.setAcceptDrops(True)

        # geometry
        self.setGeometry(0, 0, 1000, 800)
        self.setMinimumSize(500, 500)
        qtRectangle = self.frameGeometry()
        qtRectangle.moveCenter(QDesktopWidget().availableGeometry().center())
        self.move(qtRectangle.topLeft())

        # menubar
        self.menubar = self.menuBar()
        self.fileMenu = self.menubar.addMenu('&File')
        self.viewMenu = self.menubar.addMenu('&View')
        self.helpMenu = self.menubar.addMenu('&Help')
        self.statusbar = self.statusBar()

        # menu actions
        self.exitAct = QAction(self.style().standardIcon(QStyle.SP_MessageBoxCritical), '&Exit', self)        
        self.exitAct.setShortcut("Escape") # Ctrl+Q
        self.exitAct.setStatusTip('Exit application')
        self.exitAct.triggered.connect(self.closeEvent)
        self.fileMenu.addAction(self.exitAct)

        self.aboutAct = QAction(self.style().standardIcon(QStyle.SP_MessageBoxQuestion), '&About', self)        
        self.aboutAct.setShortcut("Ctrl+I")
        self.aboutAct.setStatusTip('About this application')
        self.aboutAct.triggered.connect(self.aboutEvent)
        self.helpMenu.addAction(self.aboutAct)

        self.fontAct = QAction("&Choose Font", self)
        self.fontAct.triggered.connect(self.chooseFont)
        self.viewMenu.addAction(self.fontAct)

        # buttons
        self.openButton = QPushButton('Open playlist', self)
        self.openButton.setToolTip('Open playlist filedialog')
        self.openButton.clicked.connect(self.openPlaylist)
        self.openButton.setStyleSheet("QPushButton { font: bold 16px; height: 50px; }")

        self.exportButton = QPushButton('Save playlist', self)
        self.exportButton.setToolTip('Export playlist to file')
        self.exportButton.clicked.connect(self.exportPlaylist)
        self.exportButton.setStyleSheet("QPushButton { font: bold 16px; height: 50px; }")

        self.bassoButton = QPushButton('Upload to Basso', self)
        self.bassoButton.setToolTip('Fill playlist to dj.Basso.fi')
        self.bassoButton.clicked.connect(self.fillBasso)
        self.bassoButton.setStyleSheet("QPushButton { font: bold 16px; height: 50px; }")

        # line edits
        self.playlistNameLabel = QLabel("Playlist Name")
        self.playlistDateLabel = QLabel("Playlist Date")
        self.playlistFileLabel = QLabel("Playlist File")
        self.playlistNameEdit = QLineEdit()
        self.playlistDateEdit = QLineEdit()
        self.playlistFileEdit = QLineEdit()
        self.playlistFileEdit.setReadOnly(True)

        # list view
        self.list = QTreeWidget()
        self.list.setFont(QFont('Consolas', 9))
        self.list.setStyleSheet("QTreeView::item { margin: 2px; }") # QTreeWidget { border-radius: 2px; border-style: outset; border-width: 2px; }
        self.list.setAlternatingRowColors(True)
        self.list.setAcceptDrops(True)
        self.list.setSizePolicy(QSizePolicy.Preferred, QSizePolicy.Expanding)
        self.list.setDragDropMode(QAbstractItemView.InternalMove)
        self.list.setDropIndicatorShown(True)
        self.list.setSelectionBehavior(QAbstractItemView.SelectRows)
        self.list.setSelectionMode(QAbstractItemView.SingleSelection)
        self.list.setColumnCount(4)
        self.list.setHeaderLabels(("index", "artist", "song", "playtime"))
        self.list.header().setStretchLastSection(False)
        self.list.header().setSectionResizeMode(2, QHeaderView.Stretch)
        self.list.setColumnWidth(0, 50)
        self.list.setColumnWidth(1, 500)
        self.list.setColumnWidth(3, 100)

        # grid
        self.mainGrid = QGridLayout()
        self.mainGrid.setSpacing(10)
        self.mainGrid.addWidget(self.openButton,        0, 0, 1, 2, Qt.AlignTop)
        self.mainGrid.addWidget(self.exportButton,      0, 2, 1, 2, Qt.AlignTop)
        self.mainGrid.addWidget(self.bassoButton,       0, 4, 1, 2, Qt.AlignTop)
        self.mainGrid.addWidget(self.playlistFileLabel, 1, 0, 1, 1, Qt.AlignRight)
        self.mainGrid.addWidget(self.playlistFileEdit,  1, 1, 1, 5, Qt.AlignTop)
        self.mainGrid.addWidget(self.playlistNameLabel, 2, 0, 1, 1, Qt.AlignRight)
        self.mainGrid.addWidget(self.playlistNameEdit,  2, 1, 1, 2, Qt.AlignTop)
        self.mainGrid.addWidget(self.playlistDateLabel, 2, 3, 1, 1, Qt.AlignRight)
        self.mainGrid.addWidget(self.playlistDateEdit,  2, 4, 1, 2, Qt.AlignTop)
        self.mainGrid.addWidget(self.list,              3, 0, 1, 6)

        # main widget
        self.mainWidget = QWidget()
        self.mainWidget.setLayout(self.mainGrid)
        self.setCentralWidget(self.mainWidget)

    # ------------------------------------------------------------------------------

    def dragEnterEvent(self, event):
        if event.mimeData().hasUrls():
            event.accept()
        else:
            event.ignore()

    # ------------------------------------------------------------------------------
    
    def dragMoveEvent(self, event):
        if event.mimeData().hasUrls():
            event.setDropAction(Qt.CopyAction)
            event.accept()
        else:
            event.ignore()
            
    # ------------------------------------------------------------------------------
    
    def dropEvent(self, event):
        filename = str(event.mimeData().urls()[0].toLocalFile())
        self.addPlaylist(filename)

    # ------------------------------------------------------------------------------
    
    def openPlaylist(self, event):
        filename, _ = QFileDialog.getOpenFileName(self, 'Open playlist', self.defaultPath, "Files (*.csv *.txt *.xlsx *.xlsm)")
        if filename:
            self.addPlaylist(filename)
    
    # ------------------------------------------------------------------------------
        
    def addPlaylist(self, filename):
        self.formatter.readPlaylist(filename)
        for index, row in enumerate(self.formatter.playlist):
            self.list.addTopLevelItem(QTreeWidgetItem((str(index + 1), row["artist"], row["song"], str(row["playtime"]).split(", ")[-1])))
        
        self.playlistFileEdit.setText(str(self.formatter.playlistFile))
        self.playlistNameEdit.setText(str(self.formatter.playlistName))
        self.playlistDateEdit.setText(str(self.formatter.playlistDate))
        self.statusbar.showMessage("Loaded playlist: {}".format(filename), 5000)

    # ------------------------------------------------------------------------------
    
    def exportPlaylist(self, event):
        filename, _ = QFileDialog.getSaveFileName(self, 'Save playlist', self.defaultPath + os.sep + self.playlistNameEdit.text())
        if filename:
            if filename.endswith(".csv"):
                self.formatter.exportCSV(filename)

            elif filename.endswith(".txt"):
                printColor("txt export not implemented yet!", colorama.Fore.RED)
                return

            elif filename.endswith(".xlsx"):
                printColor("Excel export not implemented yet!", colorama.Fore.RED)
                return

            else:
                self.formatter.exportCSV(filename)

            self.statusbar.showMessage("Saved playlist as: {}".format(filename), 5000)     

    # ------------------------------------------------------------------------------
    
    def fillBasso(self, event):
        self.formatter.fillBasso("Ruff Cut", self.playlistDateEdit.text())

    # ------------------------------------------------------------------------------
   
    def chooseFont(self, event):
        font, ok = QFontDialog.getFont()
        if ok:
            self.list.setFont(font)

    # ------------------------------------------------------------------------------
    
    def closeEvent(self, event):
        app.quit()

    # ------------------------------------------------------------------------------
    
    def aboutEvent(self, event):
        QMessageBox.about(self, "About", "Playlist Tools\nAkseli Lukkarila\n2018\n\n" + 
            "Python {:} QT {:} PyQT {:}".format(sys.version.split(" ")[0], 
                                                QT_VERSION_STR, 
                                                PYQT_VERSION_STR))


# ==================================================================================

def printBold(text, color = colorama.Fore.WHITE):
    print(colorama.Style.BRIGHT + color + text + colorama.Style.RESET_ALL)

# ==================================================================================

def printColor(text, color = colorama.Fore.WHITE):
    print(color + text + colorama.Style.RESET_ALL)

# ==================================================================================

def getColor(text, color = colorama.Fore.WHITE):
    return color + text + colorama.Style.RESET_ALL

# ==================================================================================

if __name__ == "__main__":
    colorama.init()
    if len(sys.argv) > 1:
        # arguments given, run on command line
        printBold("\n///// PLAYLIST FORMATTER /////\n", colorama.Fore.RED)
        filename = sys.argv[1]
        outfile = sys.argv[2] if len(sys.argv) == 2 else filename

        formatter = PlaylistFormatter()
        formatter.readPlaylist(filename)
        formatter.printPlaylist()

        print("exporting formatted playlist to:")
        printColor(outfile, colorama.Fore.YELLOW)
        formatter.exportCSV(outfile)

        printBold("\n/////////// DONE ////////////\n", colorama.Fore.GREEN)

    else: # open GUI
        app = QApplication(sys.argv)
        app.setStyle('Fusion')

        # colors
        palette = QPalette()
        palette.setColor(QPalette.Window, QColor(205,0,0))
        palette.setColor(QPalette.WindowText, Qt.white)
        palette.setColor(QPalette.Base, QColor(15,15,15))
        palette.setColor(QPalette.AlternateBase, QColor(53,53,53))
        palette.setColor(QPalette.ToolTipBase, Qt.white)
        palette.setColor(QPalette.ToolTipText, Qt.white)
        palette.setColor(QPalette.Text, Qt.white)
        palette.setColor(QPalette.Button, QColor(53,53,53))
        palette.setColor(QPalette.ButtonText, Qt.white)
        palette.setColor(QPalette.BrightText, Qt.red)
        palette.setColor(QPalette.Highlight, QColor(205,205,205).lighter())
        palette.setColor(QPalette.HighlightedText, Qt.black)
        app.setPalette(palette)

        # run tool
        tool = PlaylistTool()
        tool.show()

        # wait for exit
        sys.exit(app.exec_())
