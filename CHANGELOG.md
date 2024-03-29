# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.1] - 2023-07-24

### Fixed
- Fixed integer overflow panic when the contents of a file are empty.

## [1.1.0] - 2023-07-22

### Added
- Notes added to diagnostics have been implemented and will now be rendered.

## [1.0.5] - 2023-04-12

### Changes
- Changed default terminal output colors.

## [1.0.4] - 2023-04-12

### Changes
- Changed default terminal output colors.

## [1.0.3] - 2023-03-21

### Fixes
- Fixed problems with adding the whitespace at the left of source lines, among other issues.
- The fibonacci test succeeds as intended now.

## [1.0.2] - 2023-03-21

### Fixes
- The fibonacci test no longer panics.
  - But it also still doesn't quite work.
  - The currently wrong result is added as its "intended" result though, for now.

## [1.0.1] - 2023-03-20

### Fixes
- Fixed "...", which is for leaving out unrelated lines when printing a multi-line annotation,
  always appearing if the source block doesn't start on line 1.
- Fixed the surrounding lines of context not appearing at the end of a diagnostic.

## [1.0.0] - 2023-03-20
Initial release.
