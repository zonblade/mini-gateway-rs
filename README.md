# Mini Gateway

<p align="center">
<img style="height:100%; width: 512px;" src="https://raw.githubusercontent.com/zonblade/mini-gateway-rs/main/assets/logo.png"/></br>
<span style="font-size:32px;">mini gateway</span></br>
<span style="font-size:16px;">A very fast yet easy to control Gatway!</span>
</p>

## Architecture

![img](assets/architecture.gif)

The architecture of the mini-router is currently simple and straightforward.

> Incoming traffic enters from the internet and is first secured by the TLS/WSS proxy, which decrypts the data and passes it as TCP/HTTP/WS traffic (while you can disable the TLS if not needed). This standardized traffic is then processed by the Gateway, which dynamically determines whether to pass the request through a scripting plugin or directly to backend services. In parallel, a background mechanism constantly monitors for configuration changes. When updates are detected—via the Gateway API and an external update service—these are reloaded in memory to ensure the Gateway operates with the latest settings, all without interrupting the flow.

## Sub-repositories

Each sub-repository within this project is designed to be standalone. This means that they are independent modules and should be run separately. Ensure that you configure and execute each sub-repo according to its specific requirements and purpose. Refer to the documentation within each sub-repo for detailed setup and usage instructions.

### Sub-repository Overview

| Sub-repository | Description                                                                 | Notes                                                                                     |
|----------------|-----------------------------------------------------------------------------|-------------------------------------------------------------------------------------------|
| `router-core`  | Core proxy service responsible for handling traffic routing and forwarding. | Requires Redis/DragonflyDB for communication. Must run in a private/secure network.       |
| `router-api`   | Provides an API interface for managing and configuring the router.          | Intended for internal use only. Not designed for external consumption.                    |
| `router-cli`   | Command-line interface for interacting with and managing the router.        | Useful for quick configuration and debugging.                                             |
| `router-gui`   | Graphical user interface for managing the router.                          | Designed for internal use. Should not be exposed to public networks.                      |

## Connectivity and Network Requirements

- All of the applications are interconnected using Redis or DragonflyDB as the communication backbone. Ensure that a compatible instance of Redis or DragonflyDB is properly configured and accessible to all sub-repositories.
- The system must operate within a private and secure network. Exposing the applications or the database to public networks is not recommended and could lead to security vulnerabilities.
- The API or GUI provided by the system is intended strictly for internal use and is not designed for external consumption. Ensure that access is restricted to authorized users within the secure network.
