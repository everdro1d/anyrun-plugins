# A bookmark launcher via anyrun

Define bookmarks in a file like so:
```
# .../somewhere/<bookmarks file>

# format
<URL> [tag] <NAME>

# ex:
https://www.youtube.com/ [personal] YouTube
https://crates.io/ [work] Rust Crates
```

If a bookmark entry is missing a tag, it will set the tag to unknown.

Default Configuration:
```
// <Anyrun config dir>/bookmarks-launcher.ron
Config(
  prefix: ":b",
  // Filepath to look for a bookmarks file
  bookmarks_file: "~/bookmarks.txt",
)
