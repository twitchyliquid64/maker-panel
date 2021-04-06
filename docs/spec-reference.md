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

All comments start with a `#` and extend for the remainder of the line. For now,
comments within a `wrap()` feature don't work.

### Variables

Features can be saved to a variable and later created multiple times.

TODO


## Features

The different elements that make up a spec are called _features_. If features
are specified alongside one another, they are combined: features that detail
the geometry of the panel are unioned (ie: two overlapping squares produce a
larger rectangle with dimensions no greater than the extremes of either rectangle),
and surface features are applied to the panel in the order in which they appear.

### Geometry

_Geometric features_ make up the overall shape of the panel, and are the most
common. Almost all shapes can be represented by combining different, simpler
shapes, such as rectangles, circles, and triangles.

#### Rectangles

Form                                                      | Example                              | Meaning
--------------------------------------------------------- | ------------------------------------ | ------------
`R<dimension>()`                                          | `R<5>()`                             | Creates a rectangle with a width and height of 5 units.
`R<width, height>()` <br> `R<size = (width, height)>()`   | `R<3, 5>()` <br> `R<size = (3,5)>()` | Creates a rectangle with a width of 3 units and a height of 5 units.

TODO positioning

#### Circles

Form                                                              | Example                              | Meaning
----------------------------------------------------------------- | ------------------------------------ | -----------
`C<radius>()` <br> `C<radius = radius>()` <br> `C<r = radius>()`  | `C<5>()` <br> `C<radius = 5>()` <br> `C<r = 5>()` | Creates a circle with a radius of 5 units.

TODO positioning

#### Triangles

TODO

#### Right-angle mount

TODO

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

TODO

## Other language constructs

TODO
