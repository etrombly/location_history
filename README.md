# Use Google location history files in rust

## Getting started

```rust
extern crate location_history;

use location_history::Locations;

let mut contents = String::new();
File::open(file).unwrap().read_to_string(&mut contents).unwrap();
let locations: Locations = Locations::new(&contents);
for location in &locations.locations {
    println!("{:?}", location);
}
```