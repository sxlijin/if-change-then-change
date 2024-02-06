echo "indexing bookshop catalogs"
echo "indexing library catalogs"
echo "moving indexes to replica C"
# if-change
echo "serving from s3://bookshop-catalogs/ and s3://library-catalogs/"
# then-change tests/data/diff-has-path-changes/h1.sh
echo "service started on port 0001"
