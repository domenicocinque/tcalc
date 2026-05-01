# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/domenicocinque/tcalc/compare/tcalc_core-v0.1.1...tcalc_core-v0.2.0) - 2026-05-01

### Added

- *(core)* support holiday calendars
- *(core)* add working day durations

### Fixed

- *(parser)* reject trailing tokens
- *(parser)* validate date and time fields

### Other

- *(core)* move calendar logic out of evaluator
- *(core)* hide internal modules
- *(core)* add property-based coverage

## [0.1.1](https://github.com/domenicocinque/tcalc/compare/tcalc_core-v0.1.0...tcalc_core-v0.1.1) - 2025-12-08

### Other

- copy readme to cli and core

## [0.1.0](https://github.com/domenicocinque/tcalc/releases/tag/tcalc_core-v0.1.0) - 2025-12-08

### Added

- add time - time
- improve formatting
- add year duration
- add web

### Fixed

- am/pm logic

### Other

- add release workflow
- adjust paths for ci deploy
- separate cli and core
