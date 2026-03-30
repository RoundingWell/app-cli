# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-03-30

### Added

- Version check on startup notifies when a newer release is available
- Automatic self-update via `rw update`

## [0.3.1] - 2026-03-30

### Fixed

- Clinician prepare refers to team name not abbreviation

## [0.3.0] - 2026-03-30

### Changed

- Auth cache files are now stored as `auth/{profile}.json` (previously `auth/{org}-{stage}.json`)

### Added

- One-time migration on first run automatically renames existing auth files to the new format

## [0.2.0] - 2026-03-30

### Added

- Config directory can now be set at runtime
- Clinician commands: `enable`, `disable`, `assign`, and `prepare`
- Basic auth commands: `set`

## [0.1.0] - 2026-03-29

### Added

- Initial release of the `rw` CLI tool

[Unreleased]: https://github.com/RoundingWell/app-cli/compare/0.4.0...HEAD
[0.4.0]: https://github.com/RoundingWell/app-cli/compare/0.3.1...0.4.0
[0.3.1]: https://github.com/RoundingWell/app-cli/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/RoundingWell/app-cli/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/RoundingWell/app-cli/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/RoundingWell/app-cli/releases/tag/0.1.0
