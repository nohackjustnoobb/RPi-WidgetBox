#!/bin/bash

PROJECTS=("editor" "display")
STATIC_DIR="./static"

# Remake the static directory
rm -rf "$STATIC_DIR"
mkdir -p "$STATIC_DIR"

build_and_copy() {
    local project="$1"

    if [[ -d "$project" && -f "$project/package.json" ]]; then
        echo "Building project in $project..."

        if [[ "$project" == "editor" ]]; then
            (cd "$project" && yarn install && yarn build --base="edit") || {
                echo "Build failed in $project"
                return
            }
        else
            (cd "$project" && yarn install && yarn build) || {
                echo "Build failed in $project"
                return
            }
        fi

        echo "Copying dist folder from $project to $STATIC_DIR..."
        cp -r "$project/dist" "$STATIC_DIR/$project"
    else
        echo "Project $project not found or missing package.json, skipping."
    fi
}

for project in "${PROJECTS[@]}"; do
    build_and_copy "$project"
done
