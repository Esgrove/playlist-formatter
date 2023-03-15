# Playlist Tool

Python app for auto-formatting DJ playlists.
Originally created for my own and fellow Bassoradio DJs use back when I was doing a show at Bassoradio.
Has both a PyQt6 GUI and a command line interface.

Used to process a raw playlist file exported from DJ softwares:
text is formatted and title cased properly and play times are calculated from timestamps,
and then exported back to a file.
The formatted playlist will work well for example with Mixcloud.

Currently supports:

- csv playlists exported from Serato DJ Pro
- txt playlists exported from Rekordbox

~~Also has the option to autofill the imported playlist to Bassoradio's database,
which saves me a lot of time and manual work when I don't have to input every song manually through the not so great web interface.
Implemented in a somewhat hacky way with _Selenium_, as I could not get it working directly with HTTP posts using the _requests_ package.~~

## Dependencies

- Python 3.11+ (due to one use of `Self` type hinting :sweat_smile:)
- [requirements.txt](./requirements.txt)

## Looks like

![alt text](https://github.com/Esgrove/playlistTool/blob/master/playlistformatter.png)
