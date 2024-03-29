# Reference

This document is a quick reference to the maker-panel specification language.

## Syntax

Maker-panel specifications are similar to code, being interpreted top-down,
and similar in construction to modern high-level languages.

### Creating features

Features (explained below) are declared similar to functions in other
programming languages, taking the rough form:

```
FEATURE_NAME<PARAMETERS>(OPTIONAL_SURFACE_FEATURES)
```

For example:

```
R<5>()
```

creates:

 * A feature `R` (which corresponds to a _rectangle_ feature)
 * With the parameter `5` (for rectangle features, this means a width and height
   of `5` units)
 * And no inner surface features.

Surface features are slightly different, as they can only appear within a
non-surface feature. Updating the example above to have the rectangle contain
the drill hit surface feature, would give us:

```
R<5>(h)
```

(_h_ being the shorthand for a _hole_ AKA drill hit).

### Comments

All comments start with a `#` and extend for the remainder of the line.

### Variables

**Feature variables**

Features can be saved to a variable and later created multiple times. You define
a variable using the `let` statement:

```
let top = R<7>(h);
```

_Defines a variable named 'top', which is a 7x7 square containing a h feature._

You can use a variable wherever you would use a feature, by writing its name prefixed by
a `$` character. For example:

```
[2]$top
```

_Manifests the 'top' twice (as part of an array)._

**Number variables**

You can also save the value of a CEL expression to a variable and use it later.

```
let diameter = !{26}
let radius = !{diameter / 2}

C<$radius>
```

_In the above example, we save 26 to 'diameter', which we then use in another
CEL expression to compute 'radius'. We then use radius as the input to a circle
feature._


## Features

The different elements that make up a spec are called _features_. If features
are specified alongside one another, they are combined: features that detail
the geometry of the panel are unioned (ie: two overlapping squares produce a
larger rectangle with dimensions no greater than the extremes of either rectangle),
and surface features are applied to the panel in the order in which they appear.

In general, it is an error if your shapes don't overlap to form a single,
combined shape. You can change this behavior with the `-c` flag, which will compute
the convex hull of all the shapes (hence forming a single shape).

### Geometry

_Geometric features_ make up the overall shape of the panel, and are the most
common. Almost all shapes can be represented by combining different, simpler
shapes, such as rectangles, circles, and triangles.

#### Rectangles

Form                                                                        | Example                                                | Meaning
--------------------------------------------------------------------------- | ------------------------------------------------------ | ------------
`R<dimension>()`                                                            | `R<5>()`                                               | Creates a rectangle with a width and height of 5 units.
`R<@(x, y), dimension>()`                                                   | `R<@(1, 2), 5>()`                                      | Creates a rectangle with a width and height of 5 units, centered on (1, 2).
`R<width, height>()` <br> `R<size = (width, height)>()`                     | `R<3, 5>()` <br> `R<size = (3,5)>()`                   | Creates a rectangle with a width of 3 units and a height of 5 units.
`R<@(x, y), width, height>()` <br> `R<@(x, y), size = (width, height)>()`   | `R<@(1, 2), 3, 5>()` <br> `R<@(1, 2), size = (3,5)>()` | Creates a rectangle with a width of 3 units and a height of 5 units, centered on (1, 2).

#### Circles

Form                                                                                         | Example                                                                      | Meaning
-------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- | -----------
`C<radius>()` <br> `C<radius = radius>()` <br> `C<r = radius>()`                             | `C<5>()` <br> `C<radius = 5>()` <br> `C<r = 5>()`                            | Creates a circle with a radius of 5 units.
`C<@(x, y), radius>()` <br> `C<@(x, y), radius = radius>()` <br> `C<@(x, y), r = radius>()`  | `C<@(2, 3), 5>()` <br> `C<@(2, 3), radius = 5>()` <br> `C<@(2, 3), r = 5>()` | Creates a circle with a radius of 5 units. The center of the circle is positioned at (2, 3).


#### Triangles

Form                                                      | Example                              | Meaning
--------------------------------------------------------- | ------------------------------------ | ------------
`T<dimension>()`                                          | `T<5>()`                             | Creates a triangle with a width and height of 5 units.
`T<width, height>()` <br> `T<size = (width, height)>()`   | `T<3, 5>()` <br> `R<size = (3,5)>()` | Creates a triangle with a width of 3 units and a height of 5 units.


#### Right-angle mount

Creates a cut-out suitable for bolting another panel to the side at right angles, using M3 fasteners.

Form                                | Example                              | Meaning
----------------------------------- | ------------------------------------ | ------------
`mount_cut<length>`                 | `mount_cut<8>()`                     | Creates an upwards-facing mount cutout, with a depth of 8mm.
`mount_cut_left<length>`            | `mount_cut_left<8>()`                | Creates a left-facing mount cutout, with a depth of 8mm.
`mount_cut_right<length>`           | `mount_cut_right<8>()`               | Creates a right-facing mount cutout, with a depth of 8mm.


### Surface

Surfaces features can **only** be specified within the parentheses of a geometry feature.
Surface features are typically centered in the feature that contains them.

#### Drill hits / holes

A circular drill through the panel, with metal plating like a through-hole solder joint.

Form          | Example                              | Meaning
------------- | ------------------------------------ | ------------
`h`           | `h`                                  | Creates an M3 (3mm) drill hit.
`hDIAMETER`   | `h5`                                 | Creates an M5 (5mm) drill hit.

The specified diameter may be a decimal.

#### Metal solder points

Form          | Example                              | Meaning
------------- | ------------------------------------ | ------------
`msp`         | `msp`                                | Creates a rectangular pad with a via in it, suitable for soldering something that needs to be anchored mechanically.


TODO

### Composite

Composite features let you position features relative to other features.

#### Columns

Positions a series of features downwards, with their vertical edges touching
and justified as specified.

Form                                           | Example                         | Meaning
---------------------------------------------- | ------------------------------- | ------------
`column center { feature1 feature2 featureN }` | `column center { R<1> R<2> }`   | Positions a 1x1 rectangle on top of a 2x2 rectangle, aligned to the center.
`column left { feature1 feature2 featureN }`   | `column left { R<1> R<2> }`     | Positions a 1x1 rectangle on top of a 2x2 rectangle, aligned to the left.
`column right { feature1 feature2 featureN }`  | `column right { R<1> R<2> }`    | Positions a 1x1 rectangle on top of a 2x2 rectangle, aligned to the right.

TODO positioning

#### Pairs, tuples

A series of geometric features, laid out left-to-right with their edges touching.

Form                                | Example               | Meaning
----------------------------------- | --------------------- | ------------
`(feature1, feature2, ...)`         | `(R<5>(), R<10>())`   | Positions a 5x5 rectangle to the left of a 10x10 rectangle.

#### Arrays

A repetition of a single geometric or composite feature, extending in a given
cardinal direction.

Form                           | Example                 | Meaning
------------------------------ | ----------------------- | ------------
`[N]feature`                   | `[5]C<3.5>`             | 5 circles with a 3.5 unit radius, positioned adjacent to each other extending right.
`[N; U/D/L/R]feature`          | `[5; D]C<3.5>`          | 5 circles with a 3.5 unit radius, positioned adjacent to each other extending down.
`[N; U/D/L/R; v-score]feature` | `[5; D; v-score]C<3.5>` | As above, except an additional fabrication layer is included in the gerbers which indicates to the fab house where to v-score.

#### Wraps (edge positioning)

A wrap positions any number of features about the cardinal directions of a center
feature. The basic form looks like this:

```
wrap (SOME_CENTER_FEATURE) with {
  positioning => feature1,
  positioning => feature2,
}
```

Each section within the `{ }` braces describes how one feature which ('wraps'
the center one) should be laid out. You write these sections like this:

Form                                                 | Example                        | Meaning
---------------------------------------------------- | ------------------------------ | ------------
`top/bottom/left/right => feature,`                  | `top => R<5>,`                     | Positions a 5x5 rectangle to the center-top of its wrapping feature, overlapping halfway.
`top/bottom/left/right+offset => feature,`           | `top-0.5 => R<5>,`                 | Positions a 5x5 rectangle to the top-left of its wrapping feature, overlapping halfway.
`top/bottom/left/right align interior => feature,`   | `top align interior => R<5>,`      | Aligns a 5x5 rectangle to the center-top of its wrapping feature, completely contained within the feature it wraps.
`top/bottom/left/right align exterior => feature,`   | `top align exterior => R<5>,`      | Positions a 5x5 rectangle to the center-top of its wrapping feature, touching edges with the feature it wraps but otherwise outside of its geometry.
`min/max-top/bottom/left/right => feature,`          | `min-left align exterior => R<5>,` | Positions a 5x5 rectangle to the left of its wrapping feature, aligned across the top.
`angle(ANGLE)+offset => feature,`                    | `angle(45)+15 => R<5>,`            | Positions a 5x5 rectangle 15 units away from the centeroid of its wrapping feature, at a 45 degree angle.
`center => feature,`                                 | `center => R<5>,`                  | Positions a 5x5 rectangle at the centeroid of its wrapping feature.

Putting it all together looks like this:

```
wrap (R<30, 10>) with {
  left  => C<5>,
  right => C<5>,
}
```

_Positions two circles at either end of a rectangle._

#### Negative

A negative makes all the children features cut-outs rather than unions. So if you have
a circle in a larger circle, it will produce a ring.

```
negative {
  C<5>
}
C<10>
```

#### Rotate

A rotate construction lets you rotate the edge geometry of contained features about the origin. Note that the positioning of
inner features like screw holes is not updated, so only use this for edge geometry.

`rotate(<angle in degrees>) { <rotated geometry> }`


EG:

```
rotate(45) {
  R<5, 10>
}
```

## Other language constructs

### CEL expressions

You can read the specification for the CEL language [here](https://github.com/google/cel-spec).

You can perform simple math in maker-panel using CEL expressions. You can use
CEL expressions anywhere in `<>` where you would specify a number, by instead
writing:

```
!{my-CEL-expression}
```

For example:

```
C<!{2 + 4}>
```

You can reference any numeric variable from within your expressions. For example:

```
let some_number = !{4};
C<!{2 + some_number}>
```
