# Development workflow

1. Feature is being developed on a feature branch `<github_name>/<feature_name>` or fork.
2. If CI passes and feature is complete it will be merged in `master` with adjusted Changelog.
3. Once `master` collects enough features the version will be bumped then
4. new release with attached artifacts from latest merge will be created.

# Dot Releases

1. new release branch is created r*.*.*
2. Cherry Pick fixes from `master` or PR into the release branch.
3. create new release.
