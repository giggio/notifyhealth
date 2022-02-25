# Notifyhealth

[![Docker Stars](https://img.shields.io/docker/stars/giggio/notifyhealth.svg)](https://hub.docker.com/r/giggio/notifyhealth/)
[![Docker Pulls](https://img.shields.io/docker/pulls/giggio/notifyhealth.svg)](https://hub.docker.com/r/giggio/notifyhealth/)

This app checks if a container is running and healthy or stopped (and shouldn't) and notifies accordingly.

This can be run on Linux for AMD64 and ARMv7.

## Upstream Links

* Docker Registry @ [giggio/notifyhealth](https://hub.docker.com/r/giggio/notifyhealth/)
* GitHub @ [giggio/notifyhealth](https://github.com/giggio/notifyhealth)

## Quick Start

To run it as a daemon run it like this:

````bash
docker run --name notifyhealth -d -v /var/run/docker.sock:/var/run/docker.sock giggio/notifyhealth --label <label> [options]
````

To run it only once and view the output directly in the terminal, run it like this:

````bash
docker run --name notifyhealth --rm -ti -v /var/run/docker.sock:/var/run/docker.sock giggio/notifyhealth --label <label> print
````

To run it on Windows:

````powershell
docker run --name notifyhealth --rm -ti -v \\.\pipe\docker_engine:\\.\pipe\docker_engine giggio/notifyhealth --label <label> print
````

### Detailed commands

TBD.

## Contributing

Questions, comments, bug reports, and pull requests are all welcome.  Submit them at
[the project on GitHub](https://github.com/giggio/notifyhealth/).

Bug reports that include steps-to-reproduce (including code) are the
best. Even better, make them in the form of pull requests.

## Author

[Giovanni Bassi](https://github.com/giggio)

## License

Licensed under the MIT license.
