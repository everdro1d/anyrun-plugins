# A bookmark launcher via anyrun

define bookmarks in a file like so:
```
# somewhere/bookmarks.txt

# format
[tag] <NAME>, <URL>

# ex:
[personal] YouTube, https://www.youtube.com/
[work] Rust Crates, https://crates.io/
```

Default Configuration:
```
// <Anyrun config dir>/bookmarks-launcher.ron
Config(
  prefix: ":b",
  // Filepath to look for a bookmarks file
  bookmarks_file: "~/bookmarks.txt",
)
