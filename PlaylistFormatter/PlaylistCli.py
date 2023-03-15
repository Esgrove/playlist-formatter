"""
Playlist Formatter CLI
Akseli Lukkarila
2018-2023
"""
import sys

from PlaylistFormatter.PlaylistFormatter import PlaylistFormatter


# TODO: proper CLI handling
def run_cli():
    args = sys.argv[1:]
    filename = args[0]
    outfile = args[1] if len(args) >= 2 else None

    formatter = PlaylistFormatter()
    formatter.read_playlist(filename)
    formatter.print_playlist()
    formatter.export_csv(outfile)
