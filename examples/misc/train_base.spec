let mounts = column center {
    wrap ([3] R<7.5>) with {
      left align exterior => [3] R<7.5>(h),
      right align exterior => [3] R<7.5>(h),
    }

    wrap ([2] R<30, 7.5>(h)) with {
      left => [3] R<5>,
      right => [3] R<5>,
    }
};

#    [4] R<15, 7.5>(h)

wrap ($mounts) with {
  top-0.48 align interior => R<1.5, 2>(msp<1.5, 2>),
  top align interior => R<1.5, 2>(msp<1.5, 2>),
  top+0.48 align interior => R<1.5, 2>(msp<1.5, 2>),

  left+0.46 => C<6>(h5),
  right+0.46 => C<6>(h5),

  bottom align interior => R<2>(smiley),
}
