# protoc-prebuilt

[![crates.io][crates-io-shields]][crates-io]
[![docs.rs][docs-rs-shields]][docs-rs]
[![license][license-shields]][license]

[crates-io]: https://crates.io/crates/protoc-prebuilt
[crates-io-shields]: https://img.shields.io/crates/v/protoc-prebuilt
[docs-rs]: https://docs.rs/protoc-prebuilt
[docs-rs-shields]: https://img.shields.io/docsrs/protoc-prebuilt
[license]: https://github.com/sergeiivankov/protoc-prebuilt/blob/main/license
[license-shields]: https://img.shields.io/github/license/sergeiivankov/protoc-prebuilt

Protobuf compiler `protoc` pre-built binaries installer.

Installed binaries stored in `OUT_DIR` of the crate using the library.

## Usage

Library export `init` function which takes `version` parameter. Version parameter should be a tag name from protobuf repository without `v` prefix, for example, "21.12" or "22.0-rc3" (see [protobuf repository tags](https://github.com/protocolbuffers/protobuf/tags)). Function return a tuple contains paths to `protoc` binary and `include` directory.

In next examples provided `build.rs` script content for different generators. For example, we have next simplified project structure with protobuf files:
```text
src/
    proto/
          apple.proto
          orange.proto
build.rs
```

With [prost-build](https://crates.io/crates/prost-build)

```rust,no_run
use prost_build::compile_protos;
use protoc_prebuilt::init;
use std::env::set_var;

fn main() {
  let (protoc_bin, _) = init("22.0").unwrap();
  set_var("PROTOC", protoc_bin);

  compile_protos(
    &["src/proto/apple.proto", "src/proto/orange.proto"],
    &["src/proto"]
  ).unwrap();
}
```

With [protobuf-codegen](https://crates.io/crates/protobuf-codegen)

```rust,no_run
use protobuf_codegen::Codegen;
use protoc_prebuilt::init;
use std::env::set_var;

fn main() {
  let (protoc_bin, _) = init("22.0").unwrap();

  Codegen::new()
    .protoc()
    .protoc_path(&protoc_bin)
    .includes(&["src/proto"])
    .inputs(&["src/proto/apple.proto", "src/proto/orange.proto"])
    .cargo_out_dir("proto")
    .run_from_script();
}
```

## GitHub API limits

To avoid GitHub API limits library add `Authorization` header to requests to API with `GITHUB_TOKEN` environment variable content.

To prevent this behavior, set `PROTOC_PREBUILT_NOT_ADD_GITHUB_TOKEN` environment variable to any value reduced to `true` (see `var_bool` function in sources).

To force this library to use autorization token from another environment variable, set its name to `PROTOC_PREBUILT_GITHUB_TOKEN_ENV_NAME` environment variable.

## Comparison with analogues

- [protoc-bin-vendored](https://crates.io/crates/protoc-bin-vendored) store pre-built protobuf compiler in dependencies crates, so you can't use latest or specify version of compiler, if it's not provide by crate author
- [protobuf-src](https://crates.io/crates/protobuf-src) build protobuf compiler from sources, it not support `windows` target and compilers versions hardcoded, so you can't use specify version