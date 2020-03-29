# Development workflow

1. Feature is being developed on a feature branch `<github_name>/<feature_name>`
2. If CI passes and feature is complete it will be merged in staging
3. If Artifacts are successfully created it will be merged in master otherwise fixes will be made.
4. Once master collects enough features the version will be bumped then
5. master branch will be merged in release where CI:
   - creates artifacts for all targets
   - publishes new release with changelog
   - uploads artifacts to my website and updates `latest` to point to new release (TBD)
   - updates `airshipper` and `airshipper-git` AUR packages. (TBD)
6. I update website to provide link to the latest version.
