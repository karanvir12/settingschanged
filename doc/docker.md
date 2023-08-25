# Using Docker

## The easiest way

The easiest/faster option to run peer in Docker is to use the latest release images. These are small images that use the latest official release of the peer binary, pulled from our package repository.

**_Following examples are running on  chain and without SSL. They can be used to quick start and learn how peer needs to be configured. Please find out how to secure your node, if you want to operate it on the internet. Do not expose RPC and WS ports, if they are not correctly configured._**

Let's first check the version we have. The first time you run this command, the peer docker image will be downloaded. This takes a bit of time and bandwidth, be patient:

```bash
docker run --rm -it parity/peer:latest --version
```

You can also pass any argument/flag that peer supports:

```bash
docker run --rm -it parity/peer:latest --chain  --name "peerDocker"
```

## Examples

Once you are done experimenting and picking the best node name :) you can start peer as daemon, exposes the peer ports and mount a volume that will keep your blockchain data locally. Make sure that you set the ownership of your local directory to the peer user that is used by the container. Set user id 1000 and group id 1000, by running `chown 1000.1000 /my/local/folder -R` if you use a bind mount.

To start a peer node on default rpc port 9933 and default p2p port 30333 use the following command. If you want to connect to rpc port 9933, then must add peer startup parameter: `--rpc-external`.

```bash
docker run -d -p 30333:30333 -p 9933:9933 -v /my/local/folder:/peer parity/peer:latest --chain  --rpc-external --rpc-cors all
```

Additionally if you want to have custom node name you can add the `--name "YourName"` at the end

```bash
docker run -d -p 30333:30333 -p 9933:9933 -v /my/local/folder:/peer parity/peer:latest --chain  --rpc-external --rpc-cors all --name "peerDocker"
```

If you also want to expose the webservice port 9944 use the following command:

```bash
docker run -d -p 30333:30333 -p 9933:9933 -p 9944:9944 -v /my/local/folder:/peer parity/peer:latest --chain  --ws-external --rpc-external --rpc-cors all --name "peerDocker"
```

## Using Docker compose

You can use the following docker-compose.yml file:

```bash
version: '2'

services:
  peer:
    container_name: peer
    image: parity/peer
    ports:
      - 30333:30333 # p2p port
      - 9933:9933 # rpc port
      - 9944:9944 # ws port
    volumes:
      - /my/local/folder:/peer
    command: [
      "--name", "peerDocker",
      "--ws-external",
      "--rpc-external",
      "--rpc-cors", "all"
    ]
```

With following docker-compose.yml you can set up a node and use peer-js-apps as the front end on port 80. After starting the node use a browser and enter your Docker host IP in the URL field: _<http://[YOUR_DOCKER_HOST_IP>_

```bash
version: '2'

services:
  peer:
    container_name: peer
    image: parity/peer
    ports:
      - 30333:30333 # p2p port
      - 9933:9933 # rpc port
      - 9944:9944 # ws port
    command: [
      "--name", "peerDocker",
      "--ws-external",
      "--rpc-external",
      "--rpc-cors", "all"
    ]

  peerui:
    container_name: peerui
    image: jacogr/peer-js-apps
    environment:
      - WS_URL=ws://[YOUR_DOCKER_HOST_IP]:9944
    ports:
      - 80:80
```

## Limiting Resources

Chain syncing will utilize all available memory and CPU power your server has to offer, which can lead to crashing.

If running on a low resource VPS, use `--memory` and `--cpus` to limit the resources used. E.g. To allow a maximum of 512MB memory and 50% of 1 CPU, use `--cpus=".5" --memory="512m"`. Read more about limiting a container's resources [here](https://docs.docker.com/config/containers/resource_constraints).

Start a shell session with the daemon:

```bash
docker exec -it $(docker ps -q) bash;
```

Check the current version:

```bash
peer --version
```

## Build your own image

To get up and running with the smallest footprint on your system, you may use the peer Docker image.
You can build it yourself (it takes a while...) in the shell session of the daemon:

```bash
cd scripts/ci/dockerfiles/peer
./build.sh
```

## Reporting issues

If you run into issues with peer when using docker, please run the following command
(replace the tag with the appropriate one if you do not use latest):

```bash
docker run --rm -it parity/peer:latest --version
```

This will show you the peer version as well as the git commit ref that was used to build your container.
Just paste that in the issue you create.
