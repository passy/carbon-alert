# carbon-alert

> Not *really* meant for public use.

Using [carbonintensity.org.uk](https://carbonintensity.org.uk/) to push events onto
an MQTT bus.

## Usage

```
cp config.ron.example config.ron
$VISUAL config.ron
cargo run ./config.ron
```

## Docker

Available as [`passy/carbon-alert`](https://hub.docker.com/repository/docker/passy/carbon-alert).

```
docker run --rm -v $PWD:/config passy/carbon-alert /config/config.ron
```
