export AWS_PROFILE=production
# if-change
export VERSION="0.3.1-alpha"
# then-change
#   tests/data/3-files-incomplete/push.sh
# end-change

echo "checking AWS credentials to ensure they're valid"
echo "checking k8s credentials to ensure they're valid"
echo "sanity-check: verifying that all changes have been committed"

echo "generating k8s assets using hello-world@$VERSION"
echo "applying new k8s assets to $AWS_PROFILE"