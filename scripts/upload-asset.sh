#!/usr/bin/env bash
#
# Upload artifacts to GitHub Actions.
#
# Based on: https://gist.github.com/schell/2fe896953b6728cc3c5d8d5f9f3a17a3
#
# Requires curl and jq on PATH

# Args:
#   token: GitHub API user token
#   repo: GitHub username/reponame
#   tag: Name of the tag for which to create a release
#   description: Release description
create_release() {
    # Args
    token=$1
    repo=$2
    tag=$3
    description=$4
    echo "Creating release:"
    echo "  repo=$repo"
    echo "  tag=$tag"
    echo ""

    # Create release
    http_code=$(
        curl -s -o create.json -w '%{http_code}' \
            --header "Accept: application/vnd.github.v3+json" \
            --header "Authorization: Bearer $token" \
            --header "Content-Type:application/json" \
            "https://api.github.com/repos/$repo/releases" \
            -d '{"tag_name":"'"$tag"'","name":"'"${tag/v/Version }"'","draft":true,"body":"'"${description/\"/\\\"}"'"}'
    )
    if [ "$http_code" == "201" ]; then
        echo "Release for tag $tag created."
    else
        echo "Asset upload failed with code '$http_code'."
        return 1
    fi
}

# Args:
#   token: GitHub API user token
#   repo: GitHub username/reponame
#   tag: Name of the tag for which to upload the assets
#   file: Path to the asset file to upload
#   name: Name to use for the uploaded asset
upload_release_file() {
    # Args
    token=$1
    repo=$2
    tag=$3
    file=$4
    name=$5
    echo "Uploading:"
    echo "  repo=$repo"
    echo "  tag=$tag"
    echo "  file=$file"
    echo "  name=$name"
    echo ""

    # Determine upload URL of latest draft release for the specified tag
    upload_url=$(
        curl -s \
        --header "Accept: application/vnd.github.v3+json" \
        --header "Authorization: Bearer $token" \
        "https://api.github.com/repos/$repo/releases" \
        | jq -r '[.[] | select(.tag_name == "'"$tag"'" and .draft)][0].upload_url' \
        | cut -d"{" -f'1'
    )
    echo "Determined upload URL: $upload_url"
    http_code=$(
        curl -s -o upload.json -w '%{http_code}' \
            --request POST \
            --header "Accept: application/vnd.github.v3+json" \
            --header "Authorization: Bearer $token" \
            --header "Content-Type: application/octet-stream" \
            --data-binary "@$file" "$upload_url?name=$name"
    )
    if [ "$http_code" == "201" ]; then
        echo "Asset $name uploaded:"
        jq -r .browser_download_url upload.json
    else
        echo "Asset upload failed with code '$http_code':"
        cat upload.json
        return 1
    fi
}
