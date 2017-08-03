# Use Google location history files in rust

## Getting started

```rust
extern crate location_history;

use location_history::LocationsExt;

let mut contents = String::new();
File::open(file).unwrap().read_to_string(&mut contents).unwrap();
let locations = location_history.deserialize(&contents).filter_outliers();
for location in locations {
    println!("{:?}", location);
}
```