export BUILD_PATH="target/opt/hello-world"
# if-change
export VERSION="0.3.1-alpha"
# then-change
#   tests/data/5-files/build.sh
#   tests/data/5-files/push.sh
#   tests/data/5-files/release-prod.sh
#   tests/data/5-files/release-staging.sh
#   tests/data/5-files/release-stress.sh
# end-change

echo "set up build env step 1 of 3"
echo "set up build env step 2 of 3"
echo "set up build env step 3 of 3"

echo "flushing the build cache"
echo "fetching 3rd-party dependencies"
echo "running codegen"
echo "building all binaries and libraries"
echo "packaging all targets into a single container"

echo "success - container is now available as hello-world@$VERSION"