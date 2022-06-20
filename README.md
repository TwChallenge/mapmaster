# mapmaster - the master of all maps
A webservice we use to simplify uploading maps and changing associated meta data.

## API Docs:
[Api docs](openapi/mapmaster.md)

Api docs were generated with this: https://mermade.github.io/widdershins/ConvertingFilesBasicCLI.html

## API keys
To give access to the api, there needs to be an API key sent with the request. Valid API keys
should be written into a file, one key per line, and the file name should be passed to mapmaster with `-a`.

## Running with docker
Make sure you put your API keys into the folder you mount to `/test`.

```sh
docker pull hardliner66/mapmaster
docker run --rm -p 80:8000 -v /srv/mapmaster/data:/data -v /srv/mapmaster/maps:/maps -v /srv/mapmaster/maps/test:/test --name mapmaster hardliner66/mapmaster
```

## Running local docker build
Make sure you put your API keys into the folder you mount to `/test`.
```sh
docker build -t mapmaster .
docker run --rm -p 80:8000 -v /srv/mapmaster/data:/data -v /srv/mapmaster/maps:/maps -v /srv/mapmaster/maps/test:/test --name mapmaster mapmaster
```

## Developer Mode

```sh
cargo run -- --dev
```
