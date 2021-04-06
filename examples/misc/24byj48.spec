let mounts = wrap(R<28, 3.5>) with {
  left align exterior => R<3.5>(h3.5),
  right align exterior => R<3.5>(h3.5),
};

$mounts
# C<14>
C<@(0, -8), 9>(h9)



let horizontal_mounts = wrap(
  [2]wrap(R<7.5>(h)) with { right align exterior => R<7.5> }
) with {
  right align exterior => R<7.5>(h)
};

wrap(R<@(0, -8), 37.5,32>) with {
  top align exterior => $horizontal_mounts,
  bottom align exterior => $horizontal_mounts,
}
