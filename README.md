# chompy-evaluation

This contains the code which is used to run our Chompy nightly.

More detailed instructions to follow, but when running `./eval`, make
sure to run with `--release`, because without it something crashes and I don't know why.
See #3.

## Running the Evaluation

The evaluation runs the following experiments:

- Generate Halide rewrites using Chompy
- Run Caviar using Chompy/Caviar rulesets
- Output summary statistics on the above step

- TODO: Run Caviar using Chompy/Enumo rulesets
- TODO: Do inter-ruleset derivability checks. At maximum, this would involve doing pairwise derivability checks between the following rulesets:
  - Chompy
  - Caviar
  - Enumo
  - Handwritten Halide

