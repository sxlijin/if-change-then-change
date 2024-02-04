export LOG="thumbnail=debug;metadata=info;"
# if-change
export METADATA1_BUCKET="s3://image-metadata1/"
export METADATA2_BUCKET="s3://image-metadata2/"
# then-change tests/data/file-with-2-blocks/b1.sh
export OS="ubuntu-18.04"
# if-change
export THUMBNAIL1_BUCKET="s3://image-thumbnails1/"
export THUMBNAIL2_BUCKET="s3://image-thumbnails2/"
# then-change tests/data/file-with-2-blocks/b2.sh
export SERVICE_CONFIG="image-service/config.json"
