let screw_holes = column center {
  [12] R<7.5>(h)
  [11] R<7.5>(h)
  [12] R<7.5>(h)
  [11] R<7.5>(h)
  [12] R<7.5>(h)
};

# To keep things neat, we define the internal geometry in
# a variable (screw_holes) and reference it in wrap().
wrap ($screw_holes) with {
  top-0.5 => C<2>,
  top+0.5 => C<2>,
  bottom-0.5 => C<2>,
  bottom+0.5 => C<2>,
  bottom+0.417 => R<3>(smiley),
}
