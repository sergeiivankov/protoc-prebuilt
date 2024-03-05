# Changelog

## 0.3.0 - 2024-03-06

- Add authorization in GitHub API usage to avoid API limits
- Using custom protobuf installation
- Using HTTP proxy from environment variables
- Add check `protoc` binary file exists, run it with "--version" argument and compare returned version with required
- Add `VersionCheck` and `ForcePath` lib `Error` variants

Thanks:

- [Frank Laub](https://github.com/flaub)
- [scx1332](https://github.com/scx1332)
- [Someone](https://github.com/SomeoneSerge)
- [Vincent Yang](https://github.com/soloist-v)

## 0.2.0 - 2023-05-30

- Replace `reqwest` dependency by `ureq`
- Wrap request library error variant by `Box`
- Fix `clippy` warnings

Thanks:

- [Emil Ernerfeldt](https://github.com/emilk)

## 0.1.0 - 2023-02-22

Initial version