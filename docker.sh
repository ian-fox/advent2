IMAGE_NAME=advent2

# Build the image if it doesn't exist
docker image inspect "$IMAGE_NAME" &>/dev/null || docker build -t "$IMAGE_NAME" .

# Launch the environment
docker run --privileged --rm -it -v $(pwd):/build "$IMAGE_NAME"
