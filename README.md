# ConfigServer

> Disclaimer, this is a work in progress project

## About

ConfigServer is an attempt to rewrite the [Sping-Cloud-ConfigServer](https://docs.spring.io/spring-cloud-config/docs/current/reference/html/)in Rust (mainly for learning purposes
)

## Supported features

- Multi Git Repository Support
- Per Repository access control (using basic Auth)
- Encryption of sensitive values

## Configuration

On startup, `configserver` will lookup for a configuration file named `configserver.yml` with the following lookup order

- $CONFIGSEVER_CFG - Full path to a yaml configuration file (can be named as you like)
- $CONFIGSEVER_HOME/configserver.yml
- cwd - file named `configserver.yml`

```yaml
name: "Test Config Server" # Name of the configuration server instance
encryption_key: "76bXqgz3HOb13TN&oE8jqvMbkXMD#q4cv0hvQ" # Key used to encrypt sensitive values (see below)
network:
  host: 127.0.0.1 # Hostname on which configserver will listen for inbound connections
  port: 8080 # Port on which configserver will listen for inbound connections
repositories: # You can define multiple repositories which will be cloned at startup
  - name: fswatcher # Name of the repository
    url: https://github.com/fredjeck/fswatcher.git # Repository URL
    user_name: user # Not used for now
    password: pwd # Not used for now
    refresh_interval: 120000 # Interval in milliseconds between two refreshes (pull) of the repository
    credentials: # Define here accounts which will be allowed to access the repository via the config server rest interface
      - user_name: admin
        password: admin
```

## Accessing Repository data

Once started, `configserver` will clone the configured repositories and expose their files via HTTP.
URLs are constructed as following :
```http://host/repository_name/path/file```

Where
- `repository_name` is the name defined in the configuration file
- `path` and `file` is a full path to a file form the repository root

If for a given repository credentials were defined in the configuration, access will require identification via Basic Auth.

## Encrypting sensitive data

You can encrypt sensitive data using the `/encrypt` endpoint and by sending in the payload the value to encrypt for instance
```bash
curl --location --request POST 'http://localhost:8080/encrypt' \
--header 'Content-Type: text/plain' \
--data-raw 'Hello World'
```

will return `vZ1HNpBIGQy9Mwat3oQIVQ==`

You can then inject this value into your repository files using the `{enc:}` syntax:

```database.password={enc:vZ1HNpBIGQy9Mwat3oQIVQ==}```

ConfigServer will automatically decrypt the value when the file is requested.