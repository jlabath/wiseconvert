# wise CSV to OFX converter

### running

First download the account statement as csv from wise.com then

    RUST_BACKTRACE=1 cargo run -- --statement statement_12345678_CAD_2022-02-01_2022-05-17.csv --out wise.qfx

### tidy can be used to pretty print OFX files

    tidy -xml -i somefile.ofx
