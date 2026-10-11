let sigma = 2.0;
let gain = 7.0;

let gray = source.grayscale();
let dog = gray.gaussian(sigma) - gray.gaussian(sigma * 1.6);

dog.multiply(gain).smoothstep(0.05, 0.1)
