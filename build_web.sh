#!/bin/bash
set -eu

# ./setup_web.sh # <- call this first!

FOLDER_NAME="${PWD##*/}"
CRATE_NAME="$FOLDER_NAME" # Assume crate name is the same as the folder name
CRATE_NAME_SNAKE_CASE="${CRATE_NAME//-/_}" # For crates with-kebab-case.
BUILD=release

# This is required to enable the web_sys clipboard API which egui_web uses
# https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
# https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
export RUSTFLAGS=--cfg=web_sys_unstable_apis

build_project() {
    echo "Building rust…"
    # The build type, for example, release.
    local build="$1"
    # The crate name as specified in Cargo.toml.
    local crate_name="$2"
    cargo build --"$build" -p "$crate_name" --lib --target \
        wasm32-unknown-unknown
}

generate_js_bindings_for_wasm() {
    echo "Generating JS bindings for wasm…"
    # The build type, for example, release.
    local build="$1"
    # The name of the web assembly executable type.
    local target_name="$2"

    # Clear output from old stuff:
    rm -f docs/"$target_name"_bg.wasm

    wasm-bindgen "target/wasm32-unknown-unknown/$build/$target_name.wasm" \
      --out-dir docs --no-modules --no-typescript

    echo "Finished: docs/"$target_name"_bg.wasm"
}

start_browser() {
    [ "$1"  != "-r" ] && exit 0

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux, ex: Fedora
        # xdg-open http://localhost:8080/index.html
        local browser=firefox
        [ "$(pgrep $browser)" != "" ] && pkill $browser
        $browser --private-window http://127.0.0.1:8080 2> /dev/null &
    elif [[ "$OSTYPE" == "msys" ]]; then
        # Windows
        start http://localhost:8080/index.html
    else
        # Darwin/MacOS, or something else
        open http://localhost:8080/index.html
    fi
}

main() {
    build_project "$BUILD" "$CRATE_NAME"
    generate_js_bindings_for_wasm "$BUILD" "$CRATE_NAME_SNAKE_CASE"
    start_browser "$@"
}

# Program entry point.
main "$@"
