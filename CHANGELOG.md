## 0.2.0

Add Cargo features.

**BREAKING CHANGE**: `TestServer` functionality is now behind the `server`
feature which is not enabled by default. 

To use this add the feature to your `Cargo.toml` dependencies:
  ```toml
  tux = { version = "0.2.0", features = ["server"] }
  ```

All changes:

- Each crate functionality now has a corresponding feature:
  `diff`, `exec`, `server`, `temp`, `testdata`, `text`
- All features except for `server` are enabled by default.
- Going forward, features that are too specific and costly to build will
  be disabled by default.
- Improve crate documentation.

## 0.1.1

Documentation changes

## 0.1.0

Initial release
