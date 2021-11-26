# Hyperion

Hyperion is a single node process orchestrator which facitilates inter-communication between processes via an internal event bus.

## Features

- Ensures process stay in healthy state
- Can load binaries from remote location as well as from local host OS
- Hyperion child process (aka wodules) can publish data which can be subscribed by other wodules.
- Provides a gRPC interface which can list all the running wodules, add a new wodule, delete a wodule, watch for logs and watch for data.
