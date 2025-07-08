[//]: # (SERVER_HOST=0.0.0.0  # The IP address where your server is hosted.)

[//]: # (SERVER_PORT=       # The port number on which your server will listen.)

[//]: # (ALLOWED_IPS=0.0.0.0 # The IP addresses where your service can connect)

[//]: # ()

[//]: # (DATABASE_URL=)

[//]: # (MAX_CONNECTIONS_LIMIT=)

[//]: # (MIN_CONNECTIONS_LIMIT=)

[//]: # (MAX_CONNECTIONS=)

[//]: # (MIN_CONNECTIONS=)

[//]: # (CONNECT_TIMEOUT= # seconds)

[//]: # (ACQUIRE_TIMEOUT=  # in seconds)

[//]: # (IDLE_TIMEOUT=    # in seconds)

[//]: # (MAX_LIFETIME=     # in seconds)

[//]: # ()

[//]: # ()

[//]: # (REDIS_URL=redis://  # The URL for your Redis instance, including the host and port.)

[//]: # (REDIS_ENABLED= #false or true)

[//]: # ()

[//]: # (AWS_REGION=  # The AWS region where your resources are located.)

[//]: # (CLOUDWATCH_AWS_ACCESS_KEY=  # Your AWS access key for authentication.)

[//]: # (CLOUDWATCH_AWS_SECRET_KEY=  # Your AWS secret key for authentication.)

[//]: # (CLOUDWATCH_AWS_REGION=  # The region for AWS CloudWatch.)

[//]: # (AWS_LOG_GROUP=  # The name of the log group in AWS CloudWatch.)

[//]: # (LOG_TO_CLOUDWATCH= # `true` for enable cloud watch and `false` to disable, In case of false it behaves like println!)

[//]: # ()

[//]: # ()

[//]: # (#Access Token)

[//]: # (ACCESS_TOKEN=)

[//]: # ()

[//]: # ()

[//]: # (#RPC)

[//]: # (RPC_URL=http://)

[//]: # (RPC_USER=)

[//]: # (RPC_PASSWORD=)

[//]: # ()

[//]: # (LITECOIN_RPC_URL=)

[//]: # (LITECOIN_RPC_USER=)

[//]: # (LITECOIN_RPC_PASSWORD=)

[//]: # ()

[//]: # (DOGECOIN_RPC_URL=)

[//]: # (DOGECOIN_RPC_USER=)

[//]: # (DOGECOIN_RPC_PASSWORD=)

[//]: # ()

[//]: # (COIN_MARKET_CAP_API_KEY=)

[//]: # (COIN_MARKET_CAP_BASE_URL=)

[//]: # (RATE_UPDATE_INTERVAL_IN_SECONDS=)

[//]: # ()

[//]: # (#Invoice)

[//]: # (INVOICE_EXPIRY_TIME=)

[//]: # ()

[//]: # (#RUST ENV)

[//]: # (RUST_LOG=Debug)

# `.env` Configuration Example

- This document outlines an example configuration for environment variables used in a project. These settings are
  crucial for connecting to various services like databases, cache systems, and external APIs.

### Server Settings

- Configure the server's host address and port. These are critical for defining on which IP address and port your server
  will listen for incoming connections.

```sh
SERVER_HOST=127.0.0.1   # The IP address where your server is hosted.
SERVER_PORT=8080        # The port number on which your server will listen.

# The IP addresses allowed by your service. If omitted or not found in DB/Redis, all IPs are blocked.
# If you include "0.0.0.0", any IP connecting is treated as "Unknown" with a smaller rate limit.
ALLOWED_IPS=127.0.0.1,127.0.0.2
```

### PostgreSQL Database Connection String

```sh
#Ensure these environment variables are set in your .env file:

DATABASE_URL_WRITE=postgresql://username:password@hostname/database_name
DATABASE_URL_READ=postgresql://username:password@hostname/database_name
Example: postgresql://postgres:password@localhost/my_database

### Request Rate Limiting

MAX_CONNECTIONS_LIMIT=50
MIN_CONNECTIONS_LIMIT=5
MAX_CONNECTIONS=25
MIN_CONNECTIONS=5
CONNECT_TIMEOUT=10  # seconds
ACQUIRE_TIMEOUT=10  # in seconds
IDLE_TIMEOUT=20     # in seconds
MAX_LIFETIME=60     # in seconds
```

### Redis Configuration

- Configuration for connecting to a Redis instance. Redis is often used for caching and session storage.

```sh
REDIS_URL=redis://host:port  # The URL for your Redis instance, including the host and port.
REDIS_ENABLED=true #false or true
```

### AWS CloudWatch Configuration

- Settings for AWS CloudWatch, which is used for monitoring and logging. It requires AWS credentials and configuration
  for
  the region and log group.

```sh
AWS_REGION=ap-south-1  # The AWS region where your resources are located.
CLOUDWATCH_AWS_ACCESS_KEY=your_access_key  # Your AWS access key for authentication.
CLOUDWATCH_AWS_SECRET_KEY=your_secret_key  # Your AWS secret key for authentication.
CLOUDWATCH_AWS_REGION=ap-south-1  # The region for AWS CloudWatch.
AWS_LOG_GROUP=log_group_name  # The name of the log group in AWS CloudWatch.
LOG_TO_CLOUDWATCH=false # `true` for enable cloud watch and `false` to disable, In case of false it behaves like println!
```

### Access Token

- An access token is often used for authentication and authorization. It should be kept secure and not shared publicly.

```sh
ACCESS_TOKEN=your_access_token
```

### RPC Configuration

- Remote Procedure Call (RPC) settings for connecting to external services. These settings are often used for
- interacting with blockchain nodes or other services.

```sh
RPC_URL=http://rpc_host:rpc_port
RPC_USER=rpc_username
RPC_PASSWORD=rpc_password
```

### Litecoin RPC Configuration

- Configuration for connecting to a Litecoin node via RPC.

```sh
LITECOIN_RPC_URL=http://rpc_host:rpc_port
LITECOIN_RPC_USER=rpc_username
LITECOIN_RPC_PASSWORD=rpc_password
```

### Dogecoin RPC Configuration

- Configuration for connecting to a Dogecoin node via RPC.

```sh
DOGECOIN_RPC_URL=http://rpc_host:rpc_port
DOGECOIN_RPC_USER=rpc_username
DOGECOIN_RPC_PASSWORD=rpc_password
```

### CoinMarketCap API Configuration

- Settings for connecting to the CoinMarketCap API, which provides cryptocurrency market data.

```sh
COIN_MARKET_CAP_API_KEY=your_api_key
COIN_MARKET_CAP_BASE_URL=https://pro-api.coinmarketcap.com
RATE_UPDATE_INTERVAL_IN_SECONDS=60
```

### Invoice Configuration

- Configuration for generating and managing invoices. This includes settings like the expiry time for invoices.

```sh
INVOICE_EXPIRY_TIME=3600  # Invoice expiry time in seconds
```

### Rust Environment Configuration

- Settings for the Rust environment, including logging levels.

```sh
RUST_LOG=Debug
```
