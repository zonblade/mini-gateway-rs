# mini-router

run and gun dynamically configurable minimalistic proxy router, based on pingora

## Architecture

![img](assets/architecture.png)

The architecture of the mini-router is currently simple and straightforward, with support for Dragonfly and Redis. Looking ahead, there are plans to expand its capabilities to include support for:​
- Kafka
- RabbitMQ
- Custom Message Queues

This planned expansion aims to enhance the mini-router's versatility and effectiveness in diverse messaging environments.

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
