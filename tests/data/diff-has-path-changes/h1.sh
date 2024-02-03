export LOG="catalog-parser=info;library-parser=debug;"
# if-change
export BOOKSHOP_CATALOGS="s3://bookshop-catalogs/"
export LIBRARY_CATALOGS="s3://library-catalogs/"
# then-change tests/data/diff-has-path-changes/h2.sh
export SERVICE_CONFIG="catalog-service/config.json"
