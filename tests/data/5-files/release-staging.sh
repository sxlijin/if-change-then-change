export AWS_PROFILE=staging
# if-change
export VERSION="0.3.1-alpha"
# then-change
#   tests/data/5-files/build.sh
#   tests/data/5-files/push.sh
#   tests/data/5-files/release-prod.sh
#   tests/data/5-files/release-stress.sh
# end-change

echo "checking AWS credentials to ensure they're valid"
echo "checking k8s credentials to ensure they're valid"
echo "warn if not all changes have been committed"

echo "generating k8s assets using hello-world@$VERSION"
echo "applying new k8s assets to $AWS_PROFILE"