# symlix-web

Static site generator for [symlix.fr](https://symlix.fr)

This repository is public but I did not make it with any other use case than mine in mind. It is mostly here for when you feel bored and want to help out with a few things or play with the data yourself.

## Usage

For website development purposes, the following will compile the static site generator and start a development server:

```
$ cargo run --release -- --serve
```

For deployment purposes, the following will compile a minimal static site generator and generate the website:

```
$ cargo run --release --no-default-features
```

## License

The source code in this repository is licensed under the [Apache 2.0 license](LICENSE). All rights to other assets are reserved (this notably covers the Symlix logo).

All code contributions must be licensed under the same license. When applicable, the intellectual property of any other submitted asset must be transfered to Théo Degioanni upon submission.

This is because I do not want to deal with IP law and all. Don't worry, I expect contributions to be trivial when it comes to assets anyway.
