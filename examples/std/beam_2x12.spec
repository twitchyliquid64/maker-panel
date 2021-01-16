let screw_holes = column center {
  [12] R<7.5>(h)
  [11] R<7.5>(h)
};
let mount = R<1.5, 2>(msp<1.5, 2>);

wrap ($screw_holes) with {
  left align interior => $mount,
  right align interior => $mount,
}

# Not used for now
# let mounts_top = column center {
#   [11]column center {
#     R<7.5, 1.6>
#     $mount
#   }
# };
# let mounts_bottom = column center {
#   [12]column center {
#     $mount
#     R<7.5, 1.6>
#   }
# };
# top align interior => $mounts_top,
# bottom align interior => $mounts_bottom,
