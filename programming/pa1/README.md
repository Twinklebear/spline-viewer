Will Usher - Programming Assignment 1
-

## Compilation

To compile the program you will need the Rust compiler and its Cargo toolchain
(included with Rust). You can find a download for the most recent version of Rust
at [rust-lang.org](https://www.rust-lang.org/en-US/downloads.html) along with
more information about the language.

Once Rust is installed both the `rustc` and `cargo` commands should be in your path.
To build the project in release mode cd into the project directory (with the `Cargo.toml` file)
and run `cargo build --release`. This will take a few minutes as some dependencies are downloaded
and compiled locally. After compiling the project you can run it with `cargo run --release`
or directly find the binary under `./target/release/bezier`.

## Running and Loading Curves

To run the program and specify some curves or data files to load on the command line you can
run through cargo with arguments to the program following a second --

```
cargo run --release -- <list of .dat and .crv files>
```

or directly:

```
./target/release/bezier <list of .data and .crv files>
```

You can also pass -h as an argument to print the program options.

## Controls

- Left click somewhere on the scene to add a new control point to the active curve,
if you hold left click after adding you can continue dragging the new point.

- Left click and drag an existing control point to move it around.

- Shift + Left click on a control point to remove it.

- Right click and drag to pan the camera around

- Scroll to zoom in and out.

- The UI can be used to add/remove curves, select the active curve,
toggle drawing of the curve, control points and polygon along
with the curve and control polygon colors.

- To add a curve scroll to the bottom of the curve list to find the add curve button,
this new curve will have 0 control points initially and will be selected automatically.

- To change the curve color you can enter the values manually or left click and drag
left/right on the r/g/b color boxes to change the value.

## Method for adding points

I chose to find the nearest segment on the control polygon to determine which existing
control points a new point should be inserted in between. This works well and is simple
to implement, though there can be some cases of ambiguity when the distance
to two segments is the same. Some care is needed when finding the "nearest" point on
the segment to treat them as actual segments instead of infinite lines so I clamp the
position on the line we find our distance too to be on the segment.

To determine if we're appending or prepending a point I check if it's the first or
last segment and if the point we projected to on the line is at the end or start
of the segment accordingly. For example if we find the nearest point is the end
of the last segment, we're appending a new control point.

## Curve generation method

I didn't implement the subdivision method for rendering the curves, I take
a fixed number of samples along t and draw lines between these points. As
a result the inner piece of the treble clef looks pretty bad if you tug it around some.

