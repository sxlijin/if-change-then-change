export AWS_PROFILE=production
# if-change
export VERSION="0.3.1-alpha"
# then-change
#   tests/data/5-files/build.sh
#   tests/data/5-files/push.sh
#   tests/data/5-files/release-prod.sh
#   tests/data/5-files/release-staging.sh
#   tests/data/5-files/release-stress.sh
# end-change

echo "checking AWS credentials to ensure they're valid"
echo "sanity-check: verifying that all changes have been committed"

echo "creating a new image push destination on-demand if necessary"

echo "pushing hello-world@$VERSION to $AWS_PROFILE"