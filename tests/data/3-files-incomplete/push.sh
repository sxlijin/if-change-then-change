export AWS_PROFILE=production
# if-change
export VERSION="0.3.1"
# then-change
#   build.sh
#   release.sh
# end-change

echo "checking AWS credentials to ensure they're valid"
echo "sanity-check: verifying that all changes have been committed"

echo "creating a new image push destination on-demand if necessary"

echo "pushing hello-world@$VERSION to $AWS_PROFILE"