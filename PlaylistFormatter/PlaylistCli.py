"""
Playlist Formatter CLI
Akseli Lukkarila
2018-2023
"""

from PlaylistFormatter.PlaylistFormatter import PlaylistFormatter


# TODO: proper CLI handling
def run_cli(args: list):
    """Run playlist formatting for file given as an argument."""
    if not args:
        raise RuntimeError("No arguments given")

    filename = args[0].strip()
    outfile = args[1].strip() if len(args) >= 2 else None

    formatter = PlaylistFormatter()
    formatter.read_playlist(filename)
    formatter.print_playlist()
    formatter.export_csv(outfile)
