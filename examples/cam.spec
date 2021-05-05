# Wrap a rectangle with circles of the same size to produce
# a tubular shape.
wrap(R<20>) with {
  left => C<10>,
  right => C<10>,

  # Place a smiley 8 units down from the center, just for lolz.
  angle(90) +8 => R<3>(smiley),

  # Cut-out a cam shape from the center.
  center => negative {
    wrap(R<20, 5>) with {
      left => C<2.5>,
      right => C<2.5>,
    }
  },
}
