# chompy-evaluation

This contains the code which is used to run our Chompy nightly.

More detailed instructions to follow, but when running `./eval`, make
sure to run with `--release`, because without it something crashes and I don't know why.
See #3.

## About the Evaluation

The nightly evaluation runs the following experiments:

- Generate Halide rewrites using Chompy
- Get flamegraph on above step
- Run Caviar using Chompy/Caviar rulesets
- Output summary statistics on the above step
- Run Caviar (with new version of Egg, and no fancy Caviar features) on their evaluation, using Explanations in Egg to
  get rewrite traces

- TODO: Do inter-ruleset derivability checks. At maximum, this would involve doing pairwise derivability checks between the following rulesets:
  - Chompy
  - Caviar
  - Enumo
  - Handwritten Halide
- TODO: Parse rewrite traces to try and get a sense of how many rewrites are actually useful, along with a sense of what rewrites are buggy

