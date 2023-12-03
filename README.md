# dirigera - Manage your IKEA devices

**This crate is under development and can change at any time! Do not use just
yet!**

> [!NOTE]\
> I only have a few IKEA devices so most of the endpoints I've been trying has
> been for lights. I also don't use scenes extensively but tried to set up and
> configure all available ones to research the API.
> If you have any other devices and want to contribute to the code that would be
> much appreciated!

## Setup

To communicate with the Dirigera device you need to know its IP address and
obtain a token. I can't help you find the IP but usually you can see this by
looking at your routers device list.

Once you figured that out, run the `generate-token` binary to generate a file
named `config.toml` that will store the device's IP address and the obtained
token.

```sh
cargo run --bin generate-token --features binary <your-ip-address>
```

When you have a valid configuration file you can use the default trait for the
`Hub` to generate an instance that will call the configured IP address with the
configured token.

```rust
let hub = dirigera::hub::Hub::default();
```

> **NOTE** Since the configuration file depends on toml support for this is
> hidden behind a feature flag called `config`. To skip using toml simply use
> the `new` constructor and pass IP and token.

### Configuration file

If you want to create the configuration file manually, this is what it looks
like:

```toml
ip-address = 192.168.1.101
token = "abc123..."
```

## Usage

See [examples](examples) for examples on how to use this crate.

### Manual testing

Just use the token you got and your favourite HTTP client.

```sh
› http --verify=no \
  "https://[ip]:8443/v1/devices" \
  "Authorization: Bearer $TOKEN"
```

## Acknowledgement

- [Leggin/dirigera](https://github.com/Leggin/dirigera) for the inspiration and
  most of the information of the API, including token generation.
