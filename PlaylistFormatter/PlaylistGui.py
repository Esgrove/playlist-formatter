"""
Playlist Formatter GUI
Akseli Lukkarila
2018-2023
"""
import os
import sys

from PyQt6.QtCore import PYQT_VERSION_STR, QT_VERSION_STR, Qt
from PyQt6.QtGui import QAction, QColor, QFont, QGuiApplication, QPalette
from PyQt6.QtWidgets import (
    QAbstractItemView,
    QApplication,
    QFileDialog,
    QFontDialog,
    QGridLayout,
    QHeaderView,
    QLabel,
    QLineEdit,
    QMainWindow,
    QMessageBox,
    QPushButton,
    QSizePolicy,
    QStyle,
    QTreeWidget,
    QTreeWidgetItem,
    QWidget,
)

from PlaylistFormatter.colorprint import Color, print_color
from PlaylistFormatter.Platform import Platform
from PlaylistFormatter.PlaylistFormatter import PlaylistFormatter


class PlaylistGui(QMainWindow):
    def __init__(self):
        super().__init__()

        self.formatter = PlaylistFormatter()
        self.platform = Platform.get()
        self.defaultPath = self.platform.dropbox_path()

        self.about_act = None
        self.basso_button = None
        self.exit_act = None
        self.export_button = None
        self.file_menu = None
        self.font_act = None
        self.help_menu = None
        self.list = None
        self.main_grid = None
        self.main_widget = None
        self.menubar = None
        self.open_button = None
        self.playlist_date_edit = None
        self.playlist_date_label = None
        self.playlist_file_edit = None
        self.playlist_file_label = None
        self.playlist_name_edit = None
        self.playlist_name_label = None
        self.statusbar = None
        self.view_menu = None

        self.init_ui()

    def init_ui(self):
        self.setWindowTitle("Esgrove's Playlist Tool")
        self.setWindowIcon(self.style().standardIcon(QStyle.StandardPixmap.SP_MediaPlay))
        self.setAcceptDrops(True)

        # geometry
        self.setGeometry(0, 0, 1000, 800)
        self.setMinimumSize(500, 500)
        qt_rectangle = self.frameGeometry()
        qt_rectangle.moveCenter(QGuiApplication.primaryScreen().availableGeometry().center())
        self.move(qt_rectangle.topLeft())

        # menubar
        self.menubar = self.menuBar()
        self.file_menu = self.menubar.addMenu("&File")
        self.view_menu = self.menubar.addMenu("&View")
        self.help_menu = self.menubar.addMenu("&Help")
        self.statusbar = self.statusBar()

        # menu actions
        self.exit_act = QAction(self.style().standardIcon(QStyle.StandardPixmap.SP_MessageBoxCritical), "&Exit", self)
        self.exit_act.setShortcut("Escape")
        self.exit_act.setStatusTip("Exit application")
        self.exit_act.triggered.connect(self.closeEvent)
        self.file_menu.addAction(self.exit_act)

        self.about_act = QAction(self.style().standardIcon(QStyle.StandardPixmap.SP_MessageBoxQuestion), "&About", self)
        self.about_act.setShortcut("Ctrl+I")
        self.about_act.setStatusTip("About this application")
        self.about_act.triggered.connect(self.about_event)
        self.help_menu.addAction(self.about_act)

        self.font_act = QAction("&Choose Font", self)
        self.font_act.triggered.connect(self.choose_font)
        self.view_menu.addAction(self.font_act)

        # buttons
        self.open_button = QPushButton("Open playlist", self)
        self.open_button.setToolTip("Open playlist filedialog")
        self.open_button.clicked.connect(self.open_playlist)
        self.open_button.setStyleSheet("QPushButton { font: bold 16px; height: 50px; }")

        self.export_button = QPushButton("Save playlist", self)
        self.export_button.setToolTip("Export playlist to file")
        self.export_button.clicked.connect(self.export_playlist)
        self.export_button.setStyleSheet("QPushButton { font: bold 16px; height: 50px; }")

        self.basso_button = QPushButton("Upload to Basso", self)
        self.basso_button.setToolTip("Fill playlist to dj.Basso.fi")
        self.basso_button.clicked.connect(self.fill_basso)
        self.basso_button.setStyleSheet("QPushButton { font: bold 16px; height: 50px; }")

        # line edits
        self.playlist_name_label = QLabel("Playlist Name")
        self.playlist_date_label = QLabel("Playlist Date")
        self.playlist_file_label = QLabel("Playlist File")
        self.playlist_name_edit = QLineEdit()
        self.playlist_date_edit = QLineEdit()
        self.playlist_file_edit = QLineEdit()
        self.playlist_file_edit.setReadOnly(True)

        # list view
        self.list = QTreeWidget()
        self.list.setFont(QFont("Consolas", 9) if self.platform.is_windows() else QFont("Menlo", 9))
        self.list.setStyleSheet("QTreeView::item { margin: 2px; }")
        self.list.setAlternatingRowColors(True)
        self.list.setAcceptDrops(True)
        self.list.setSizePolicy(QSizePolicy.Policy.Preferred, QSizePolicy.Policy.Expanding)
        self.list.setDragDropMode(QAbstractItemView.DragDropMode.InternalMove)
        self.list.setDropIndicatorShown(True)
        self.list.setSelectionBehavior(QAbstractItemView.SelectionBehavior.SelectRows)
        self.list.setSelectionMode(QAbstractItemView.SelectionMode.SingleSelection)
        self.list.setColumnCount(4)
        self.list.setHeaderLabels(("index", "artist", "song", "playtime"))
        self.list.header().setStretchLastSection(False)
        self.list.header().setSectionResizeMode(2, QHeaderView.ResizeMode.Stretch)
        self.list.setColumnWidth(0, 50)
        self.list.setColumnWidth(1, 300)
        self.list.setColumnWidth(3, 100)

        # grid
        self.main_grid = QGridLayout()
        self.main_grid.setSpacing(10)
        self.main_grid.addWidget(self.open_button, 0, 0, 1, 2, Qt.AlignmentFlag.AlignTop)
        self.main_grid.addWidget(self.export_button, 0, 2, 1, 2, Qt.AlignmentFlag.AlignTop)
        self.main_grid.addWidget(self.basso_button, 0, 4, 1, 2, Qt.AlignmentFlag.AlignTop)
        self.main_grid.addWidget(self.playlist_file_label, 1, 0, 1, 1, Qt.AlignmentFlag.AlignRight)
        self.main_grid.addWidget(self.playlist_file_edit, 1, 1, 1, 5, Qt.AlignmentFlag.AlignTop)
        self.main_grid.addWidget(self.playlist_name_label, 2, 0, 1, 1, Qt.AlignmentFlag.AlignRight)
        self.main_grid.addWidget(self.playlist_name_edit, 2, 1, 1, 2, Qt.AlignmentFlag.AlignTop)
        self.main_grid.addWidget(self.playlist_date_label, 2, 3, 1, 1, Qt.AlignmentFlag.AlignRight)
        self.main_grid.addWidget(self.playlist_date_edit, 2, 4, 1, 2, Qt.AlignmentFlag.AlignTop)
        self.main_grid.addWidget(self.list, 3, 0, 1, 6)

        # main widget
        self.main_widget = QWidget()
        self.main_widget.setLayout(self.main_grid)
        self.setCentralWidget(self.main_widget)

    def about_event(self, event):
        QMessageBox.about(
            self,
            "About",
            "Playlist Tools\nAkseli Lukkarila\n2018\n\n"
            + f"Python {sys.version.split(' ')[0]} QT {QT_VERSION_STR} PyQT {PYQT_VERSION_STR}",
        )

    def add_playlist(self, filename):
        """Read playlist data and add to tree widget."""
        self.formatter.read_playlist(filename)
        for index, row in enumerate(self.formatter.playlist):
            item = QTreeWidgetItem(
                (str(index + 1), row.get("artist"), row.get("song"), str(row.get("playtime", "")).split(", ")[-1])
            )
            item.setFlags(Qt.ItemFlag.ItemIsSelectable | Qt.ItemFlag.ItemIsEditable | Qt.ItemFlag.ItemIsEnabled)
            self.list.addTopLevelItem(item)

        self.playlist_file_edit.setText(str(self.formatter.playlist_file))
        self.playlist_name_edit.setText(str(self.formatter.playlist_name))
        self.playlist_date_edit.setText(str(self.formatter.playlist_date))
        self.statusbar.showMessage(f"Loaded playlist: {filename}", 5000)

    def choose_font(self, event):
        font, ok = QFontDialog.getFont()
        if ok:
            self.list.setFont(font)

    def export_playlist(self, event):
        filename, _ = QFileDialog.getSaveFileName(
            self,
            "Save playlist",
            self.defaultPath + os.sep + self.playlist_name_edit.text(),
        )
        if filename:
            if filename.endswith(".csv"):
                self.formatter.export_csv(filename)

            elif filename.endswith(".txt"):
                print_color("txt export not implemented yet!", Color.red)
                return

            elif filename.endswith(".xlsx"):
                print_color("Excel export not implemented yet!", Color.red)
                return

            else:
                self.formatter.export_csv(filename)

            self.statusbar.showMessage(f"Saved playlist as: {filename}", 5000)

    def fill_basso(self, event):
        self.formatter.fill_basso("Ruff Cut", self.playlist_date_edit.text())

    def open_playlist(self, event):
        filename, _ = QFileDialog.getOpenFileName(
            self, "Open playlist", self.defaultPath, "Files (*.csv *.txt *.xlsx *.xlsm)"
        )
        if filename:
            self.add_playlist(filename)

    def dragEnterEvent(self, event):
        if event.mimeData().hasUrls():
            event.accept()
        else:
            event.ignore()

    def dragMoveEvent(self, event):
        if event.mimeData().hasUrls():
            event.setDropAction(Qt.DropAction.CopyAction)
            event.accept()
        else:
            event.ignore()

    def dropEvent(self, event):
        filename = str(event.mimeData().urls()[0].toLocalFile())
        self.add_playlist(filename)

    def closeEvent(self, event):
        sys.exit()


def run_gui():
    # open GUI
    app = QApplication(sys.argv)
    app.setStyle("Fusion")

    # custom colors
    # AlternateBase: Used as the alternate background color in views with alternating row colors.
    # Base: Used mostly as the background color for text entry widgets.
    # BrightText: A text color that is very different from WindowText, and contrasts well with e.g. Dark.
    # Button: The general button background color.
    # ButtonText: A foreground color used with the Button color.
    # Text: The foreground color used with Base.
    # ToolTipBase: Used as the background color for QToolTip and QWhatsThis.
    # ToolTipText: Used as the foreground color for QToolTip and QWhatsThis.
    # Window: A general background color.
    # WindowText: A general foreground color.
    palette = QPalette()
    palette.setColor(QPalette.ColorRole.AlternateBase, QColor("#1E1E1E").lighter())
    palette.setColor(QPalette.ColorRole.Base, QColor("#1E1E1E").darker())
    palette.setColor(QPalette.ColorRole.BrightText, Qt.GlobalColor.red)
    palette.setColor(QPalette.ColorRole.Button, QColor("#1E1E1E").lighter())
    palette.setColor(QPalette.ColorRole.ButtonText, Qt.GlobalColor.white)
    palette.setColor(QPalette.ColorRole.Highlight, QColor("#CACACA"))
    palette.setColor(QPalette.ColorRole.HighlightedText, Qt.GlobalColor.darkRed)
    palette.setColor(QPalette.ColorRole.Text, Qt.GlobalColor.white)
    palette.setColor(QPalette.ColorRole.ToolTipBase, QColor("#CACACA"))
    palette.setColor(QPalette.ColorRole.ToolTipText, Qt.GlobalColor.white)
    palette.setColor(QPalette.ColorRole.Window, QColor("#1E1E1E"))
    palette.setColor(QPalette.ColorRole.WindowText, Qt.GlobalColor.white)
    app.setPalette(palette)

    # run tool
    tool = PlaylistGui()
    tool.show()

    # wait for exit
    sys.exit(app.exec())
