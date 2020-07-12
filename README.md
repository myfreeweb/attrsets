[![crates.io](https://img.shields.io/crates/v/attrsets.svg)](https://crates.io/crates/attrsets)
[![API Docs](https://docs.rs/attrsets/badge.svg)](https://docs.rs/attrsets/)
[![unlicense](https://img.shields.io/badge/un-license-green.svg?style=flat)](http://unlicense.org)

# attrsets

Have you ever wanted to, say, define
[a few different](https://github.com/serde-rs/serde/issues/1741)
[`serde` serializations](https://github.com/serde-rs/serde/issues/1846)
for the same structures?

Well, you could always write two versions of a struct,
and `std::mem::transmute` between them to get different serializations:

```rust
#[derive(Deserialize, Serialize)]
pub struct Thing {
    pub flag: bool,
    pub time: DateTime<Utc>,
}

#[derive(Deserialize, Serialize)]
pub struct ThingCompact {
    #[serde(rename = "f")]
    pub flag: bool,

    #[serde(with = "ts_seconds")]
    #[serde(rename = "t")]
    pub time: DateTime<Utc>,
}
```

But that's unmaintainable! >_<

This procedural macro automates this process:

```rust
#[attrsets::attrsets(Compact)]
#[derive(Deserialize, Serialize)]
pub struct Thing {
    #[attrset(Compact, serde(rename = "f"))]
    pub flag: bool,

    #[attrset(Compact, serde(with = "ts_seconds"))]
    #[attrset(Compact, serde(rename = "t"))]
    pub time: DateTime<Utc>,
}
```

This example would basically expand into the above.

Every identifier in the `attrsets` attribute defines a suffix for a
new version of the struct.
The `attrset` field attribute wraps any other attribute,
only including it in the provided list of variants (comma separated).
Use `_` for the plain non-suffixed variant e.g.:

```rust
#[attrsets::attrsets(Readable)]
#[derive(Deserialize, Serialize)]
pub struct Thing {
    #[attrset(_, serde(rename = "f"))]
    pub flag: bool,

    #[attrset(_, serde(with = "ts_seconds"))]
    #[attrset(_, serde(rename = "t"))]
    pub time: DateTime<Utc>,
}
```

## Limitations

- errors are not nice
- no way to propagate the variant choices to nested structs yet
  - of course you can just parameterize the structs and define nice aliases like `type PostR = PostReadable<ImageReadable<GeoReadable>>` 

## License

This is free and unencumbered software released into the public domain.  
For more information, please refer to the `UNLICENSE` file or [unlicense.org](http://unlicense.org).
