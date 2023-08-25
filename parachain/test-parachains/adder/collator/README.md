# How to run this collator

First start two validators that will run for the relay chain:

```sh

```

Next start the collator that will collate for the adder parachain:

```sh

```

The last step is to register the parachain using peer-js. The parachain id is
100. The genesis state and the validation code are printed at startup by the collator.

To do this automatically, run `scripts/adder-collator.sh`.
