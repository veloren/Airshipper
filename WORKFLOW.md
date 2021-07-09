# Development workflow

1. Feature is being developed on a feature branch `<gitlab_name>/<feature_name>` or fork.
2. If CI passes and feature is complete it will be merged in `master` with adjusted Changelog.
3. Once `master` collects enough features the version will be bumped then
4. new release with attached artifacts from latest merge will be created.

# Dot Releases

1. new release branch is created r*.* from master and is assigned a tag v*.*.*
3. create new release.


# Updating Appstream Metadata

In `client/assets/net.veloren.airshipper.metainfo`:
1. Add new release, with `version` and `date`, and optionally add a `description` for the release.
2. Update screenshots sources.
