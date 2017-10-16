# dupr

A duplicate file finder written in Rust.

 - compares files using [XXHash](https://github.com/Cyan4973/xxHash)
 - does not consider hardlinks to the same file as duplicates
 - includes zero length files by default

[XXHash](https://github.com/Cyan4973/xxHash) is a fast, non-cryptographic hash
function. In theory collisions are possible and while I haven't noticed any I'll
be adding an additional verification step that uses a cryptographic hash or does
a byte by byte comparison.
