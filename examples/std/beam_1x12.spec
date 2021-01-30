let mount = R<1.5, 2>(msp<1.5, 2>);

let beam = wrap ([12] R<7.5>(h)) with {
  left align interior => $mount,
  right align interior => $mount,
};

[10; D; v-score]$beam
