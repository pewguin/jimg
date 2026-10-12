let sigma = 1.5;
let sigma_wide_coef = 1.3;

let gray = source.grayscale();
let dog = gray.gaussian(sigma) - gray.gaussian(sigma * sigma_wide_coef);

source - dog.smoothstep(0.008, 0.025)
