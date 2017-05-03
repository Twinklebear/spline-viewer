# Spline Viewer

A viewer for B-spline curves and surfaces, initially written for a course on
computer aided geometric design. You can edit and create 2D B-splines
and tweak some properties of loaded 3D curves and surfaces, but can't move points on 3D objects.

## Running and Loading Curves

To run the program and specify some curves or data files to load on the command line you can
run through cargo with arguments to the program following a second `--` or run the program directly.
Examples of each JSON curve format can be found in the examples.

```
./spline-viewer <list of .json files>
```

You can also pass -h as an argument to print the program options.

## Controls

- Left click somewhere on the scene to add a new control point to the active curve,
if you hold left click after adding you can continue dragging the new point.

- Left click and drag an existing control point to move it around.

- Shift + Left click on a control point to remove it.

- Right click and drag to pan the camera around

- Scroll to zoom in and out.

- To add a curve scroll to the bottom of the curve list to find the add curve button,
this new curve will have 0 control points initially and will be selected automatically. You
can also drop a curve JSON file on the window to load it, see `examples/` for example curves.

## Screenshot

Here's what you'll see if you load all the provided examples and tweak the colors a bit.

![spline viewer](http://i.imgur.com/EzAQZyM.png)

