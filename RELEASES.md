# Version 0.1.20

* Background task API ([#214](https://github.com/neon-bindings/neon/pull/214)).
* Fixes to Windows builds ([#221](https://github.com/neon-bindings/neon/pull/221), [#227](https://github.com/neon-bindings/neon/pull/227)), thanks to [@hone](https://github.com/hone)'s tenacious troubleshooting.

# Version 0.1.19

* TypeScript upgrade fixes ([neon-bindings/neon-cli#62](https://github.com/neon-bindings/neon-cli/pull/62), [neon-bindings/neon-cli#65](https://github.com/neon-bindings/neon-cli/pull/65)).

# Version 0.1.18

* CLI bugfix ([neon-bindings/neon-cli#59](https://github.com/neon-bindings/neon-cli/pull/59)).
* JsArrayBuffer ([#210](https://github.com/neon-bindings/neon/pull/210)).

# Version 0.1.17

* CLI bugfix ([neon-bindings/neon-cli#57](https://github.com/neon-bindings/neon-cli/pull/57)).

# Version 0.1.16

* CLI bugfix ([neon-bindings/neon-cli#56](https://github.com/neon-bindings/neon-cli/pull/56)).

# Version 0.1.15 (2017-05-21)

* Better Electron support in CLI's build process.
* Better support for Electron via the artifacts file ([neon-bindings/neon-cli#52](https://github.com/neon-bindings/neon-cli/pull/52)).

# Version 0.1.14 (2017-04-02)

* Ensure failing tests break the build ([#191](https://github.com/neon-bindings/neon/pull/191))
* Catch Rust panics and convert them to JS exceptions ([#192](https://github.com/neon-bindings/neon/pull/192))
* Implement `Error` for `Throw` ([#201](https://github.com/neon-bindings/neon/pull/191))
* Clean up the CLI and allow `neon build` to optionally take module names ([neon-bindings/neon-cli#48](https://github.com/neon-bindings/neon-cli/pull/48)).

# Version 0.1.13 (2017-02-17)

* More robust build scripts for neon-runtime, fixing Homebrew node installations (see [#189](https://github.com/neon-bindings/neon/pull/189))

# Version 0.1.12 (2017-02-16)

* [Optimized rooting protocol](https://github.com/neon-bindings/neon/commit/cef41584d9978eda2d59866a077cfe7c7d3fa46e)
* [Eliminate rustc warnings](https://github.com/neon-bindings/neon/pull/107)
* Lots of internal API docs
* Windows support! :tada:
* [Renamed `neon-sys` to `neon-runtime`](https://github.com/neon-bindings/neon/issues/169)
* Depend on `neon-build` as a build dependency (see [neon-bindings/neon-cli#46](https://github.com/neon-bindings/neon-cli/issues/46)).

# Version 0.1.11 (2016-08-08)

* [Exposed `This` trait](https://github.com/neon-bindings/neon/issues/101) to allow user-level abstractions involving `FunctionCall`
* Bump version to match Neon so they can be kept in sync from now on.
* Generate a `build.rs` to make Windows work (see [neon-bindings/neon-cli#42](https://github.com/neon-bindings/neon-cli/pull/42) and [neon-bindings/neon-cli#44](https://github.com/neon-bindings/neon-cli/issues/44)).

# Version 0.1.10 (2016-05-11)

* Added `JsError` API with support for throwing [all](https://github.com/neon-bindings/neon/issues/65) [standard](https://github.com/neon-bindings/neon/issues/66) [error](https://github.com/neon-bindings/neon/issues/67) [types](https://github.com/neon-bindings/neon/issues/74)
* [Test harness and CI integration](https://github.com/neon-bindings/neon/issues/80)!! :tada: :tada: :tada:
* API to [call JS functions from Rust](https://github.com/neon-bindings/neon/issues/60)
* API to [new JS functions from Rust](https://github.com/neon-bindings/neon/issues/61)
* Added [generalized `as_slice` and `as_mut_slice` methods](https://github.com/neon-bindings/neon/issues/64) to `CSlice` API.
* Fixed a [soundness issue](https://github.com/neon-bindings/neon/issues/64) with Locks.

## Incompatible Changes

* The `JsTypeError` type is gone, and replaced by the more general `JsError` type.
* `neon::js::error::JsTypeError::throw(msg)` is now `neon::js::error::JsError::throw(neon::js::error::kind::TypeError, msg)`
