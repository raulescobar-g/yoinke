* small example ["game"](https://niklasei.github.io/bevy_game_template/) (*warning: biased; e.g., split into a lot of plugins and using `bevy_kira_audio` for sound*)
* easy setup for running the web build using [trunk] (`trunk serve`) 
* run the native version with `cargo run`
* workflow for GitHub actions creating releases for Windows, Linux, macOS, and Web (Wasm) ready for distribution
    * push a tag in the form of `v[0-9]+.[0-9]+.[0-9]+*` (e.g. `v1.1.42`) to trigger the flow


 3. [Update the icons as described below](#updating-the-icons)
 4. Start coding :tada:
    * Start the native app: `cargo run`
    * Start the web build: `trunk serve`
       * requires [trunk]: `cargo install --locked trunk`
       * requires `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
       * this will serve your app on `8080` and automatically rebuild + reload it after code changes

You should keep the `credits` directory up to date. The release workflow automatically includes the directory in every build.

### Updating the icons
 1. Replace `build/windows/icon.ico` (used for windows executable and as favicon for the web-builds)
 2. Replace `build/macos/icon_1024x1024.png` with a `1024` times `1024` pixel png icon and run `create_icns.sh` (make sure to run the script inside the `macos` directory) - _Warning: sadly this seems to require a mac..._




#### TODOS:
1. make 3d player with camera
   - get the little editor thing up
   - pass player pos to terrain gen

2. make basic terrain that autogens and autodeletes
   - sphere
   - terrain gen
   - better terrain
   - skybox of stars

3. physics for player
   - another file for physics 
   - modular and good code
      - gravity
      - rigid body
      - mass for gravity
   - gravity
   - basic collisions
   - toggle flying

4. make better terrain
   - perlin layering
   - mountains
   - water
   - caves?

5. make sphere for planets
6. fix physics for planet stuff
7. make player model
8. better terrain for worlds
9. lighting
10. ...reevaulate

