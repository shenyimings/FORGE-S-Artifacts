#!/bin/sh

# This script builds a Docker image and runs Docker container with the environment required to deploy and run Sallar Token program.
# It includes tools like Rust, Solana CLI, Solana test validator and Anchor that are required for this purpose.
#
# Prerequisites:
# - Docker installed and running correctly

SOLANA_VERSION="1.14.20"
ANCHOR_VERSION="0.27.0"
IMAGE_TAG="sallar:1.0"
CONTAINER_NAME="sallar"

build_docker_container () {
  echo "Building Docker image with Sallar..."
  set -e
  docker build --build-arg ANCHOR_VERSION=$ANCHOR_VERSION --build-arg SOLANA_VERSION=$SOLANA_VERSION -t $IMAGE_TAG -f Dockerfile ../
  set +e

  echo "Sallar Docker image built successfuly"
}

start_docker_container_foreground_mode () {
  while true; do
    read -p "Do you want to automatically start test Solana validator and deploy Sallar in the container? (y/n) " answer
    case $answer in 
      [yY]* ) docker run -p 8899:8899 --name $CONTAINER_NAME --rm -it $IMAGE_TAG bash -c "(solana-test-validator -q &) && sleep 10 && anchor deploy && echo 'Sallar deployed and available on port: 8899.' && echo 'Once you exit this terminal, the container will be automatically stopped and removed. Use background mode to prevent this behaviour.' && bash"; exit;;
      [nN]* ) docker run -p 8899:8899 --name $CONTAINER_NAME --rm -it $IMAGE_TAG; exit;;
      * ) echo "Invalid answer, provide 'y' for yes or 'n' for no";;
    esac
  done
}

start_docker_container_background_mode () {
  BACKGROUND_CONTAINER_SUCCESS_MESSAGE="Container up and running in the background"

  while true; do
    read -p "Do you want to automatically start test Solana validator and deploy Sallar in the container? (y/n) " answer
    case $answer in 
      [yY]* ) docker run -d -p 8899:8899 --name $CONTAINER_NAME $IMAGE_TAG bash -c "(solana-test-validator -q &) && sleep 10 && anchor deploy && echo 'Sallar deployed and available on port: 8899.' && echo $BACKGROUND_CONTAINER_SUCCESS_MESSAGE && sleep infinity"; break;;
      [nN]* ) docker run -d -p 8899:8899 --name $CONTAINER_NAME $IMAGE_TAG bash -c "echo $BACKGROUND_CONTAINER_SUCCESS_MESSAGE && sleep infinity"; break;;
      * ) echo "Invalid answer, provide 'y' for yes or 'n' for no";;
    esac
  done

  echo "Waiting for the container to be up and running..."
  while true; do
    if docker logs $CONTAINER_NAME | grep "$BACKGROUND_CONTAINER_SUCCESS_MESSAGE"; then
        break
    fi
    sleep 1
  done

  docker logs $CONTAINER_NAME

  echo "You can attach terminal to the container using the following command: docker exec -it $CONTAINER_NAME bash"
}

start_docker_container () {
  while true; do
    read -p "Do you want to start Docker container? (y/n) " answer
    case $answer in 
      [yY]* ) break;;
      [nN]* ) exit;;
      * ) echo "Invalid answer, provide 'y' for yes or 'n' for no";;
    esac
  done

  while true; do
  read -p "Do you want to start Docker container in the background or in the foreground? (b/f) " answer
  case $answer in 
    [fF]* ) start_docker_container_foreground_mode; exit;;
    [bB]* ) start_docker_container_background_mode; exit;;
    * ) echo "Invalid answer, provide 'b' for background or 'f' for foreground";;
  esac
done
}

build_docker_container
start_docker_container
