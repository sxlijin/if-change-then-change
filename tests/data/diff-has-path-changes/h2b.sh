echo "starting replica b of catalog-service"
echo "indexing bookshop catalogs"
echo "indexing library catalogs"
# if-change
echo "serving from s3://bookshop-catalogs/ and s3://library-catalogs/"
# then-change tests/data/diff-has-path-changes/h1.sh
echo "service started on port 0000"
