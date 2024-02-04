export LOG="thumbnail=debug;metadata=info;"
# if-change
export METADATA_BUCKET="s3://image-metadata/"
# then-change tests/data/file-with-2-blocks/b1.sh
export OS="ubuntu-18.04"
# if-change
export THUMBNAIL_BUCKET="s3://image-thumbnails/"
# then-change tests/data/file-with-2-blocks/b2.sh
export SERVICE_CONFIG="image-service/config.json"
