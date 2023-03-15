# Playlist Tool

Python app for auto-formatting DJ playlists for my own and fellow Bassoradio djs use. 
PyQt5 GUI and command line interfaces.

Used to process a raw playlist file: 
text is formatted and title cased properly and play times are calculated from timestamps, 
and then exported back to a file. 
The formatted playlist will work well for example with Mixcloud.

Currently, works directly only for csv-files exported from the Serato DJ Pro software, 
though txt and excel support is coming at some point (probably when I would need those filetypes for the first time).

~~Also has the option to autofill the imported playlist to Bassoradio's database, 
which saves me a lot of time and manual work when I don't have to input every song manually through the not so great web interface. 
Implemented in a somewhat hacky way with _Selenium_, as I could not get it working directly with HTTP posts using the _requests_ package.~~

### Looks like:
![alt text](https://github.com/Esgrove/playlistTool/blob/master/playlistformatter.png)
