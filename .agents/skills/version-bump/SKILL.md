---
name: version-bump
description: Determines appropriate semantic version bumps based on changes. Use when deciding version numbers, evaluating breaking changes, or planning releases. Triggers on terms like "version", "semver", "breaking change", "major/minor/patch".
---

# Semantic Versioning Skill

This skill helps determine appropriate version bumps following [Semantic Versioning](https://semver.org/).

## Version Format

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Breaking changes
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, backwards compatible

_This format is referenced as `X.Y.Z` in this document._

## Version Bump Decision Tree

### MAJOR (X.0.0) - Breaking Changes

Bump MAJOR when you make incompatible API changes:

- Removed public classes or methods
- Changed existing class namespaces
- Changed method signatures (parameters, return types)
- Changed default behavior that breaks existing usage

### MINOR (0.X.0) - New Features

Bump MINOR when you add functionality in a backwards compatible manner:

- New classes or methods

### PATCH (0.0.X) - Bug Fixes

Bump PATCH when you make backwards compatible bug fixes:

- Fix incorrect behavior
- Fix crashes or errors
- Performance improvements (no API changes)
- Internal refactoring (no behavior changes)
- Documentation fixes

## Quick Reference

| Change Type                      | Version Bump |
|----------------------------------|--------------|
| Breaking API change              | MAJOR        |
| Removed feature                  | MAJOR        |
| New command/feature              | MINOR        |
| New CLI flag                     | MINOR        |
| New provider/integration         | MINOR        |
| Bug fix                          | PATCH        |
| Performance fix                  | PATCH        |
| Documentation only               | PATCH        |
| Refactoring (no behavior change) | PATCH        |

## Pre-1.0 Versioning

For versions < 1.0.0 (like this project):
- MINOR can include breaking changes
- PATCH is for bug fixes and small features
- More flexibility before reaching stability

## Instructions

1. Review all changes since last release:
   `git log --oneline "$(git describe --tags --abbrev=0 2>/dev/null || git rev-list --max-parents=0 HEAD)"..HEAD`
   The fallback covers the first release, when `git describe` has no tag to find
   and would otherwise fail.
2. Check for breaking changes:
   - Removed or renamed public APIs?
   - Changed default behaviors?
   - Incompatible configuration changes?
3. If breaking changes exist -> MAJOR bump
4. If new features exist -> MINOR bump
5. If only fixes/refactoring -> PATCH bump

## Preparing the Release

When `CHANGELOG.md` has a non-empty `## [Unreleased]` section, prepare the
release on a branch rather than committing to `main`:

1. Create the branch from `main`: `chore/prepare-release-X.Y.Z`
2. Make the changelog and version commits there (see *Version Update Locations* below)
3. Push the branch and open a pull request
4. Stop here and wait for the pull request to be merged

## Version Update Locations

When bumping version, update `CHANGELOG.md`:

1. Rename the accumulated `## [Unreleased]` heading to `## [X.Y.Z] - YYYY-MM-DD`
2. Add a fresh, empty `## [Unreleased]` heading above it, so the next change has
   somewhere to go
3. Reconcile the section's contents with what actually shipped — entries written
   while the work was in flight often still call a feature deferred that landed
   later in the same cycle
4. Add a `[X.Y.Z]: https://github.com/<org>/<repo>/releases/tag/X.Y.Z` link
   definition at the bottom of CHANGELOG.md
5. Repoint the `[Unreleased]` link at the version just cut —
   `.../compare/X.Y.Z...main`. It is the one comparison link the changelog
   keeps, and leaving it behind makes the empty `[Unreleased]` section link to
   a diff full of released work
6. Update `Cargo.toml` version.

## Release Process

Tag only when `## [Unreleased]` is empty — the top of the changelog is then the
version to release. Otherwise, create a pull request as described in
*Preparing the Release*.

1. Check whether the tag already exists and which commit it points at.
   Resolve it before comparing: `git rev-parse X.Y.Z^{commit}` for a local tag,
   or `git ls-remote origin 'refs/tags/X.Y.Z*'` for the remote, where the `^{}`
   line is the peeled commit of an annotated tag. Compare that commit with the
   commit being released.
   - No tag: continue to step 2.
   - Remote tag at the release commit: a prior run already created and pushed
     it, so skip to step 4. `--verify-tag` checks the remote tag, not a local
     one.
   - Local tag at the release commit, not yet on the remote: a prior run
     created the signed tag but did not push it — skip to step 3 to push it.
   - Tag pointing anywhere else: stop. Never move or force an existing tag.
2. Create a signed tag with no `v` prefix: `git tag -s X.Y.Z -m "X.Y.Z"`
3. Push the tag: `git push origin X.Y.Z`
4. Create the GitHub release from the changelog section for that version:

   ```sh
   awk -v ver="X.Y.Z" '
     $0 ~ "^## \\[" ver "\\]" { inside=1; next }
     inside && (/^## \[/ || /^\[[^]]+\]: /) { inside=0 }
     inside { print }
   ' CHANGELOG.md > /tmp/release-notes.md

   url=$(sed -n 's/^\[X\.Y\.Z\]: //p' CHANGELOG.md)
   [ -n "$url" ] && printf '\n**Full Changelog**: %s\n' "$url" >> /tmp/release-notes.md

   gh release create X.Y.Z --title X.Y.Z --notes-file /tmp/release-notes.md --verify-tag
   ```

5. Verify the release exists and is marked latest: `gh release list`

Release notes are the changelog section verbatim, so the changelog stays the single source
of truth. `--verify-tag` fails the command rather than creating a tag that was never pushed.
