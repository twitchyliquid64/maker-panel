let base = R<7.5>;
let mount_v = R<1.5, 2>(msp<1.5, 2>);
let mount_h = R<2, 1.5>(msp<2, 1.5>);

wrap ($base) with {
  top align interior => $mount_v,
  left align interior => $mount_h,

  right  align exterior => [2; R]R<7.5>(h),
  bottom align exterior => [2; D]R<7.5>(h),
}
