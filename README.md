# Substack RSS filter

Serves to remove premium posts from substack RSS feeds

## Usage

### Accessing the filtered feed
```
127.0.0.1:PORT/filter/www.examplesubstackwebsite.com/feed?API_KEY=API_KEY
```

### Running the server
You set the API_KEY either as the environment variable SRF_API_KEY, or pass at runtime with --api-key  
```
Usage: substack-rss-filter [OPTIONS] --api-key <API_KEY>

Options:
  -p, --port <PORT>        Sets a port to expose the web server on [default: 3000]
      --api-key <API_KEY>  Sets the api key to authenticate against [env: SRF_API_KEY=]
  -h, --help               Print help
  -V, --version            Print version
```
Recommended to generate the API_KEY with openssl
```bash
openssl rand -hex 32
```

## Building

You can build with either cargo or nix
```bash
cargo build --release
```
```bash
nix build
```

## Filtering mechanism

At the moment literally just checks if content contains `Read more` so may overfit.