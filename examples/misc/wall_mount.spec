let screw_holes = (R<7.5>(h), R<7.5>(h), R<7.5>(h));

let top = wrap((T<-10.25, 5>, R<2, 5>, T<10.25, 5>)) with {
  bottom align exterior => R<22.5, 2>,
};

wrap($screw_holes) with {
  top align exterior => $top,
}

R<@(7.5, -7.6), 4.5>(h1)
