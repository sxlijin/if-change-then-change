export LOG="thumbnail=debug;video=info;"
# if-change
export THUMBNAIL_BUCKET="s3://video-thumbnails/"
export VIDEO_BUCKET="s3://video-service/"
# then-change tests/data/basic/d.sh
export VIDEO_CONFIG="video-service/config.json"
