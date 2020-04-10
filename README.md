# Simple image uploader server in rust
This repo contains an example of a simple asynchronous http server written in rust using [`tokio`](https://github.com/tokio-rs/tokio) and [`warp` framework](https://github.com/seanmonstar/warp). In order to run the server you only need `docker` and `docker-compose`.
There are separate dockerfiles for development and production mode since they have different workflow and requirements.
This project uses `ffi` bindings to [`stb` C library](https://github.com/nothings/stb) and has quite a bit of `unsafe` rust code.

## Development mode
Development mode uses `cargo-watch`, recompiling files when they are saved inside a container.
```shell
docker-compose -f docker-compose.yml -f docker-compose-dev.yml up
```
## Production mode
Production mode image is using multistage docker building technique, reducing the size of final image.
```shell
docker-compose -f docker-compose.yml up
```

## API
The server starts at `localhost:3000` and has just one method `/upload_image`, which accepts `post` requests with either `multipart/form-data` encoded files, or a `json` array, containing `base64`-encoded images:
```json
[
  {
    "filename": "pic.png",
    "data": "iVBORw0..."
  }
]
```

The server also serves images at `/img`, and static page at `/images`. The better way to do this is by using separate container (for example `nginx`) for serving static files and reverse-proxying api requests to this server.

## Testing
There are no kinds of automated tests here, instead go at [`/`](http://localhost:3000/) to send a request, go at
[`/images`](http://localhost:3000/images) to see uploaded images.
