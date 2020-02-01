### Git usage
This project follows [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#summary) loosely.
In addition, you should name your branch like `<name>/feature-x`

Example commits:
- ``feat: profile support``
- ``fix(db): insert query``

### Notes for the server
- Requires Postgres to be running.
- If you have docker available you can ``docker build . -t airshipper`` and ``docker-compose up`` to make the server available. However make sure to checkout [docker-compose.yml](docker-compose.yml) to verify your setup is correct.

### Notes on testing
- use the ``test_all.ps1`` script to run all tests.
- If you run linux feel free to contribute linux equivalent scripts.

### Agreement
By contributing to this project you accept your code being licensed under [GPLv3](LICENSE) and distributed indefinitely.