let sigma = 2.0;

let gray = source.grayscale();
let dog = gray.gaussian(sigma) - gray.gaussian(sigma * 1.6);

source - dog.smoothstep(0.01, 0.08)
