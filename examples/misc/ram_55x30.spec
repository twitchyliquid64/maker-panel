
let outline = wrap(T<60,-25>) with {
  top align exterior => R<60, 5>,
};

wrap($outline) with {
  top  align interior => mount_cut<8>
  left align interior => mount_cut_left<8>

  top-0.48 align interior => R<1.5, 2>(msp<1.5, 2>),
  top+0.48 align interior => R<1.5, 2>(msp<1.5, 2>),
}


R<@(-18, -2.5), 2>(smiley)
