### Git usage

This project follows [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary) loosely.
In addition, you should name your branch like `<name>/feature-x`

Example commits:

- `feat: profile support`
- `fix(db): insert query`

### Notes for the server

- If you have docker available you can `cd server && docker build . -t airshipper` and `cd server && docker-compose up` to make the server available. However make sure to checkout [docker-compose.yml](docker-compose.yml) to verify your setup is correct and you've set the env vars.

### Agreement

By contributing to this project you accept your code being licensed under [GPLv3](LICENSE) and distributed indefinitely.
