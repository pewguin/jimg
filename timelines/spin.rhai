let backspin_t = smoothstep(0.0, 0.05, t);
let spin_t = smoothstep(0.05, 0.85, t);
let scale_t = smoothstep(0.1, 0.8, t);

let lowres = source.resize(300, 400);
lowres.scale(1.0 - 0.9 * scale_t, 1.0 + 2.0 * scale_t)
      .rotate(-20.0 * backspin_t + 7220.0 * spin_t)
