# Development workflow

1. Feature is being developed on a feature branch `<github_name>/<feature_name>`
2. If CI passes and feature is complete it will be merged in `staging`
3. Once `staging` collects enough features the version will be bumped then
4. `staging` branch will be merged in `master` where CI:
   - creates artifacts for all targets
   - publishes new release with changelog
