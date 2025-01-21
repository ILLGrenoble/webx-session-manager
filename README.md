# WebX Session Manager

## Description

The WebX Session Manager is used to manage Xorg and window manager processes for specific users, handling requests initiated by the WebX Router.

When A WebX Remote Desktop session request is received by the WebX Router, the user authentication and X11 session management is delegated to the WebX Session Manager. The WebX Session Manager authenticates a user using the PAM library. If authentication succeeds it will then spawn an Xorg process and start a configured window manager. It manages a single session per user.

The session request returns details of connection parameters which the WebX Router then uses to spawn a WebX Engine processes.

Communication between the WebX Router and WebX Session Manager is handled using ZeroMQ IPC sockets.

### Included in this project

This project includes:
 - The WebX Session Manager server Rust source code
 - The WebX Session Manager test client Rust source code
 - Dockerfiles to build the WebX Session Manager and package it in a Debian Package
 - Github actions to buid Debian Packages and add them to releases

## WebX Overview

WebX is a Remote Desktop technology allowing an X11 desktop to be rendered in a user's browser. It's aim is to allow a secure connection between a user's browser and a remote linux machine such that the user's desktop can be displayed and interacted with, ideally producing the effect that the remote machine is behaving as a local PC.

WebX's principal differentiation to other Remote Desktop technologies is that it manages individual windows within the display rather than treating the desktop as a single image. A couple of advantages with a window-based protocol is that window movement events are efficiently passed to clients (rather than graphically updating regions of the desktop) and similarly it avoids <em>tearing</em> render effects during the movement. WebX aims to optimise the flow of data from the window region capture, the transfer of data and client rendering.

> The full source code is openly available and the technology stack can be (relatively) easily demoed but it should be currently considered a work in progress.

The WebX remote desktop stack is composed of a number of different projects:
 - [WebX Engine](https://github.com/ILLGrenoble/webx-engine) The WebX Engine is the core of WebX providing a server that connects to an X11 display obtaining window parameters and images. It listens to X11 events and forwards event data to connected clients. Remote clients similarly interact with the desktop and the actions they send to the WebX Engine are forwarded to X11.
 - [WebX Router](https://github.com/ILLGrenoble/webx-router) The WebX Router manages multiple WebX sessions on single host, routing traffic between running WebX Engines and the WebX Relay. 
 - [WebX Session Manager](https://github.com/ILLGrenoble/webx-session-manager) The WebX Session manager is used by the WebX Router to authenticate and initiate new WebX sessions. X11 displays and desktop managers are spawned when new clients successfully authenticate.
 - [WebX Relay](https://github.com/ILLGrenoble/webx-relay) The WebX Relay provides a Java library that can be integrated into the backend of a web application, providing bridge functionality between WebX host machines and client browsers. TCP sockets (using the ZMQ protocol) connect the relay to host machines and websockets connect the client browsers to the relay. The relay transports data between a specific client and corresponding WebX Router/Engine.
 - [WebX Client](https://github.com/ILLGrenoble/webx-client) The WebX Client is a javascript package (available via NPM) that provides rendering capabilities for the remote desktop and transfers user input events to the WebX Engine via the relay.

To showcase the WebX technology, a demo is available. The demo also allows for simplified testing of the WebX remote desktop stack. The projects used for the demo are:
 - [WebX Demo Server](https://github.com/ILLGrenoble/webx-demo-server) The WebX Demo Server is a simple Java backend integrating the WebX Relay. It can manage a multiuser environment using the full WebX stack, or simply connect to a single user, <em>standalone</em> WebX Engine.
 - [WebX Demo Client](https://github.com/ILLGrenoble/webx-demo-client) The WebX Demo Client provides a simple web frontend packaged with the WebX Client library. The demo includes some useful debug features that help with the development and testing of WebX.
 - [WebX Demo Deploy](https://github.com/ILLGrenoble/webx-demo-deploy) The WebX Demo Deploy project allows for a one line deployment of the demo application. The server and client are run in a docker compose stack along with an Nginx reverse proxy. This provides a very simple way of connecting to a running WebX Engine for testing purposes.

 The following projects assist in the development of WebX:
 - [WebX Dev Environment](https://github.com/ILLGrenoble/webx-dev-env) This provides a number of Docker environments that contain the necessary libraries and applications to build and run a WebX Engine in a container. Xorg and Xfce4 are both launched when the container is started. Mounting the WebX Engine source inside the container allows it to be built there too.
 - [WebX Dev Workspace](https://github.com/ILLGrenoble/webx-dev-workspace) The WebX Dev Workspace regroups the WebX Engine, WebX Router and WebX Session Manager as git submodules and provides a devcontainer environment with the necessary build and runtime tools to develop and debug all three projects in a single docker environment. Combined with the WebX Demo Deploy project it provides an ideal way of developing and testing the full WebX remote desktop stack.

## Development

The WebX Session Manager is designed to be built and run in a Linux environment and runs in connection with a WebX Router process. It authenticates users using the PAM library and spawns Xorg and window manager processes for the user. 

Development can be made directly on a linux machine providing the relevant libraries are installed or (as recommendd) development can be performed within a devcontainer.

### Building and running from source on a linux machine

The following assumes a Debian or Ubuntu development environment.

Install the following dependencies:

```
apt install curl gcc libzmq3-dev libclang-dev libpam-dev clang
```

Next, install the Rust language:

```
curl https://sh.rustup.rs -sSf > /tmp/rustup-init.sh \
    && chmod +x /tmp/rustup-init.sh \
    && sh /tmp/rustup-init.sh -y \
    && rm -rf /tmp/rustup-init.sh
```

Opening a new termminal should provide Rust's `cargo` build command, otherwise it can be located at `~/.cargo/bin/cargo`.

To compile the WebX Session Manager you need to use the latest version of Rust

```
rustup default nightly
```

To compile the WebX Session Manager (and the test client), run the command: 

```
cargo build
```

The WebX Router can either be launched in a terminal the following command:

```
./target/debug/server
```

#### WebX Session Manager configuration

The configuration file `config.yml` is used to define the logging level, IPC paths, Xorg config path, Window manager run scripts. An example file is provided (`config.example.yml`). The WebX Session Manager will search for the config file at either `config.yml` in the working directory or `/etc/webx/webx-session-manager-config.yml`. Alternatively each configuration value can be overridden by an environment variable, prefixed by WEBX_SESSION_MANAGER. For example, the `xorg: config_path:` configuration value can be overridden by the environment variable `WEBX_SESSION_MANAGER_XORG_CONFIG_PATH`.

### Building, running and debugging using the WebX Dev Workspace

The [WebX Dev Workspace](https://github.com/ILLGrenoble/webx-dev-env) combines the development of The WebX Engine, WebX Router and WebX Session Manager in a single workspace and the development and testing of all of these can be combined in a single devcontainer environment.

This is the recommended way of building, running and debuggine the WebX stack as it provides the most flexible approach to development without installing any dependencies. The environment is configured to easily run the three projects together and contains VSCode Launch Commands to debug the application.

In the devcontainer you should start by building the WebX Engine then launch the WebX Router and WebX Session Manager using the VSCode Launch Commands. The WebX Session Manager can be debugged using the standard VSCode debugger.

Please refer to the project's README for more information.

### Testing using the test client

Testing the WebX Session Manager on it's own is most easily done using the compiled `client` application.

Run the `client login` command within the webx-session-manager project, for example, assuming you are running in the Dev Workspace devcontainer you can use the pre-configured user as follows:

```
./target/debug/client login --username mario --width 19210 --height 1080
```

When prompted enter the password for the user: `mario`. The WebX Session Manager will authenticate the user and then spawn Xorg and window manager processes for the user.

You can <em>logout</em> the user and stop the spawned processes by running the command

```
./target/debug/client logout --id <session_id>
```

The <em>session_id</em> is provided by the login response.

### Running the WebX Demo to test the WebX Remote Desktop Stack

In a terminal on the host computer, the simplest way to test the WebX Session Manager and its connection to the other projects is by running the [WebX Demo Deploy](https://github.com/ILLGrenoble/webx-demo-deploy) project. This runs the WebX Demo in a docker compose stack.

To fully test the WebX Stack run the demo as follows:

```
./deploy.sh
```

In a browser open https://localhost

You need to set the host of the WebX Server: running in a local devcontainer, set this to `host.docker.internal`.

Using the WebX Dev Workspace, you can log in with any of the pre-defined users (mario, luigi, peach, toad, yoshi and bowser), the password is the same as the username.

This will send the request to the WebX Router: the WebX Session Manager will authenticate the user and run Xorg and Xfce4 for the user; WebX Router then launches the locally-built webx-engine.

## Design

The WebX Session Manager runs a server listening to client requests on a TCP socket. It provides user authentication and X11 session management functionality. 

### Socket connections

A single ZeroMQ IPC socket running the request-response pattern (`ZMQ_REP`) provides connection capabilities to the server. Each request is handled sequentially.

Requests come from other processes running on the same host (in our case this is presumed to be the [WebX Router](https://github.com/ILLGrenoble/webx-router)) which listens to requests from other hosts.

### Server requests and responses

The server supports the following requests from clients:
 - login (and session creation)
 - logout (and session destruction)
 - who (for current session information)

#### Login request

A client requests a new X11 session for a specific user. User credentials are passed in the request body along with X11 screen resolution parameters.

Authentication is provided using a standard linux PAM service that verifies the username and password. 

Once authenticated, the server will determine if an X11 session is already running or not. 

If a new X11 session is required the server will:
 - Spawn a configured `Xorg` processes using the UID and GID of the user with the desired screen resolution. A unique DISPLAY environment variable is selected (a simple counter from 60) and the X11 server is secured using the XAUTHORITY environment variable.
 - Spawn a configured window manager. This is typically a script to start the desired manager. By default a script is included to start Xfce4. The environment variables necessary to connect to the X11 server (DISPLAY, XAUTHORITY) are passed to the window management script which runs using the user's UID and GID.

Each session has its own unique sessionId.

Each Xorg and window manager has it's own log files generated for debugging purposes.

If the process is successful (or if an existing session is already open) the response is sent including:
 - session Id
 - username
 - user UID
 - DISPLAY and XAUTHORITY environment variable values
 - xorg process id
 - window manager process id

### Logout request

To stop the xorg and window manage processes, a user can request to <em>logout</em>. The logout request includes the session Id generated by the login request.

The WebX Session Manager kills the xorg and window manager processes associated to the session Id.

### Who request

The who request will simply return a list of current sessions. The response details of each session is identical to that produces by the login request.

