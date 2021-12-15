# Online Radio

This project is powered by [eframe](https://github.com/emilk/egui/tree/master/eframe) and [egui](https://github.com/emilk/egui/). See the original [template](https://github.com/emilk/eframe_template/).

The application compiles natively or for the web, and is sharable thru Github Pages.

## Getting started

`src/app.rs` contains a simple example app.

Use the latest version of stable rust by running `rustup update`

### Testing locally

`cargo run --release`

On Linux you need to first run:

`sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev`

On Fedora Rawhide you need to run:

`dnf install clang clang-devel clang-tools-extra speech-dispatcher-devel libxkbcommon-devel pkg-config openssl-devel`

### Compiling for the web

Compile the app to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and publish it as a web page. There are a few simple scripts that help you with this:

``` sh
./setup_web.sh
./build_web.sh
./start_server.sh
open http://127.0.0.1:8080/
```

* `setup_web.sh` installs the tools required to build for web
* `build_web.sh` compiles your code to wasm and puts it in the `docs/` folder (see below)
* `start_server.sh` starts a local HTTP server so you can test before you publish
* Open http://127.0.0.1:8080/ in a web browser to view

The finished web application is found in the `docs/` folder so it is easily shareable thru [GitHub Pages](https://docs.github.com/en/free-pro-team@latest/github/working-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site). It consists of three files:

* `index.html`: A few lines of HTML, CSS and JS that loads your app. **Edit this to replace with the name of the crate.**
* `your_crate_bg.wasm`: What the Rust code compiles to.
* `your_crate.js`: Auto-generated binding between Rust and JS.

