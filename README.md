# to-absolute-rs

Simple rust library to get absolute path for a existing path. This library is
almost same with `std::fs::canonicalize`, but removes some unusual prefix (e.g.
`\\?\`) on Windows.
