import sys

from PyQt5.QtCore import Qt
from PyQt5.QtGui import QColor, QPalette
from PyQt5.QtWidgets import QApplication

from PlaylistFormatter import PlaylistFormatter
from PlaylistGui import PlaylistGui
from colorprint import print_bold, print_color, Color

if __name__ == "__main__":
    if len(sys.argv) > 1:
        # arguments given, run on command line
        print_bold("\n///// PLAYLIST FORMATTER /////\n", Color.red)
        filename = sys.argv[1]
        outfile = sys.argv[2] if len(sys.argv) == 2 else filename

        formatter = PlaylistFormatter()
        formatter.read_playlist(filename)
        formatter.print_playlist()

        print("exporting formatted playlist to:")
        print_color(outfile, Color.yellow)
        formatter.export_csv(outfile)

        print_bold("\n/////////// DONE ////////////\n", Color.green)
    else:
        # open GUI
        app = QApplication(sys.argv)
        app.setStyle('Fusion')

        # colors
        palette = QPalette()
        palette.setColor(QPalette.Window, QColor(205, 0, 0))
        palette.setColor(QPalette.WindowText, Qt.white)
        palette.setColor(QPalette.Base, QColor(15, 15, 15))
        palette.setColor(QPalette.AlternateBase, QColor(53, 53, 53))
        palette.setColor(QPalette.ToolTipBase, Qt.white)
        palette.setColor(QPalette.ToolTipText, Qt.white)
        palette.setColor(QPalette.Text, Qt.white)
        palette.setColor(QPalette.Button, QColor(53, 53, 53))
        palette.setColor(QPalette.ButtonText, Qt.white)
        palette.setColor(QPalette.BrightText, Qt.red)
        palette.setColor(QPalette.Highlight, QColor(205, 205, 205).lighter())
        palette.setColor(QPalette.HighlightedText, Qt.black)
        app.setPalette(palette)

        # run tool
        tool = PlaylistGui()
        tool.show()

        # wait for exit
        sys.exit(app.exec_())
