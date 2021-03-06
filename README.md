[![Build Status](https://travis-ci.org/danburkert/prost.svg?branch=master)](https://travis-ci.org/danburkert/prost)
[![Windows Build Status](https://ci.appveyor.com/api/projects/status/24rpba3x2vqe8lje/branch/master?svg=true)](https://ci.appveyor.com/project/danburkert/prost/branch/master)
[![Documentation](https://docs.rs/prost/badge.svg)](https://docs.rs/prost/)
[![Crate](https://img.shields.io/crates/v/prost.svg)](https://crates.io/crates/prost)

# *PROST!*

`prost` is a [Protocol Buffers](https://developers.google.com/protocol-buffers/)
implementation for the [Rust Language](https://www.rust-lang.org/). `prost`
generates simple, idiomatic Rust code from `proto2` and `proto3` files.

Compared to other Protocol Buffers implementations, `prost`

* Generates simple, idiomatic, and readable Rust types by taking advantage of
  Rust `derive` attributes.
* Retains comments from `.proto` files in generated Rust code.
* Allows existing Rust types (not generated from a `.proto`) to be serialized
  and deserialized by adding attributes.
* Uses the [`bytes::{Buf, BufMut}`](https://github.com/carllerche/bytes)
  abstractions for serialization instead of `std::io::{Read, Write}`.
* Respects the Protobuf `package` declaration when organizing generated code
  into Rust modules.
* Preserves unknown enum values during deserialization.
* Does not include support for runtime reflection or message descriptors.

## Using `prost` in a Cargo Project

First, add `prost` and its public dependencies to your `Cargo.toml` (see
[crates.io](https://crates.io/crates/prost) for the current versions):

```
[dependencies]
prost = <prost-version>
prost-derive = <prost-version>
bytes = <bytes-version>
```

The recommended way to add `.proto` compilation to a Cargo project is to use the
`prost-build` library to handle compilation at build-time. See the
[`prost-build` documentation](prost-build) for more details and examples.

Alternatively, the `prost-codegen` crate provides a `protoc` plugin which can be
used to manually compile `.proto` files into Rust source files. The resulting
Rust files can be added to a project source tree like any other. See the
[`prost-codegen` documentation](prost-codegen) for more details and examples.

## Generated Code

`prost` generates Rust code from source `.proto` files using the `proto2` or
`proto3` syntax. `prost`'s goal is to make the generated code as simple as
possible.

### Packages

Currently, all `.proto` files used with `prost` must contain a `package`
declaration. `prost` will translate the Protobuf package into a Rust module.
For example, given the `package` declaration:

```proto
package foo.bar;
```

All Rust types generated from the file will be in the `foo::bar` module.

### Messages

Given a simple message declaration:

```proto
// Sample message.
message Foo {
}
```

`prost` will generate the following Rust struct:

```rust
/// Sample message.
#[derive(Clone, Debug, PartialEq, Message)]
pub struct Foo {
}
```

### Fields

Fields in Protobuf messages are translated into Rust as public struct fields of the
corresponding type.

#### Scalar Values

Scalar value types are converted as follows:

| Protobuf Type | Rust Type |
| --- | --- |
| `double` | `f64` |
| `float` | `f32` |
| `int32` | `i32` |
| `int64` | `i64` |
| `uint32` | `u32` |
| `uint64` | `u64` |
| `sint32` | `i32` |
| `sint64` | `i64` |
| `fixed32` | `u32` |
| `fixed64` | `u64` |
| `sfixed32` | `i32` |
| `sfixed64` | `i64` |
| `bool` | `bool` |
| `string` | `String` |
| `bytes` | `Vec<u8>` |

#### Enumerations

All `.proto` enumeration types convert to the Rust `i32` type, so that unknown
values may be decoded. Additionally, each `.proto` enumeration type gets a
corresponding Rust `enum` type, with helper methods to convert `i32` field
values to the enum type (if possible).

#### Field Modifiers

Protobuf scalar value and enumeration message fields can have a modifier
depending on the Protobuf version. Modifiers change the corresponding type of
the Rust field:

| `.proto` Version | Modifier | Rust Type |
| --- | --- | --- |
| `proto2` | `optional` | `Option<T>` |
| `proto2` | `required` | `T` |
| `proto3` | default | `T` |
| `proto2`/`proto3` | repeated | `Vec<T>` |

#### Map Fields

Map fields are converted to a Rust `HashMap` with key and value type converted
from the Protobuf key and value types.

#### Message Fields

Message fields are converted to the corresponding struct type. The table of
field modifiers above applies to message fields, except that `proto3` message
fields without a modifier (the default) will be wrapped in an `Option`.
Typically message fields are unboxed. `prost` will automatically box a message
field if the field type and the parent type are recursively nested in order to
avoid an infinite sized struct.

#### Oneof Fields

Oneof fields convert to a Rust enum. Protobuf `oneof`s types are not named, so
`prost` uses the name of the `oneof` field for the resulting Rust enum, and
defines the enum in a module under the struct. For example, a `proto3` message
such as:

```proto
message Foo {
  oneof widget {
    int32 quux = 1;
    string bar = 2;
  }
}
```

generates the following Rust[1]:

```rust
pub struct Foo {
    pub widget: Option<foo::Widget>,
}
pub mod foo {
    pub enum Widget {
        Quux(i32),
        Bar(String),
    }
}
```

`oneof` fields are always wrapped in an `Option`.

[1] Annotations have been elided for clarity. See below for a full example.

### Services

`prost-build` allows a custom code-generator to be used for processing `service`
definitions. This can be used to output Rust traits according to an
application's specific needs.

### Generated Code Example

Example `.proto` file:

```proto
syntax = "proto3";
package tutorial;

message Person {
  string name = 1;
  int32 id = 2;  // Unique ID number for this person.
  string email = 3;

  enum PhoneType {
    MOBILE = 0;
    HOME = 1;
    WORK = 2;
  }

  message PhoneNumber {
    string number = 1;
    PhoneType type = 2;
  }

  repeated PhoneNumber phones = 4;
}

// Our address book file is just one of these.
message AddressBook {
  repeated Person people = 1;
}
```

and the generated Rust code (`tutorial.rs`):

```rust
#[derive(Clone, Debug, PartialEq, Message)]
pub struct Person {
    #[prost(string, tag="1")]
    pub name: String,
    /// Unique ID number for this person.
    #[prost(int32, tag="2")]
    pub id: i32,
    #[prost(string, tag="3")]
    pub email: String,
    #[prost(message, repeated, tag="4")]
    pub phones: Vec<person::PhoneNumber>,
}
pub mod person {
    #[derive(Clone, Debug, PartialEq, Message)]
    pub struct PhoneNumber {
        #[prost(string, tag="1")]
        pub number: String,
        #[prost(enumeration="PhoneType", tag="2")]
        pub type_: i32,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Enumeration)]
    pub enum PhoneType {
        Mobile = 0,
        Home = 1,
        Work = 2,
    }
}
/// Our address book file is just one of these.
#[derive(Clone, Debug, PartialEq, Message)]
pub struct AddressBook {
    #[prost(message, repeated, tag="1")]
    pub people: Vec<Person>,
}
```

## Serializing Existing Types

`prost` uses a custom derive macro to handle encoding and decoding types, which
means that if your existing Rust type is compatible with Protobuf types, you can
serialize and deserialize it by adding the appropriate derive and field
annotations.

Currently the best documentation on adding annotations is to look at the
generated code examples above.

## FAQ

1. **Could `prost` be implemented as a serializer for [Serde](https://serde.rs/)?**

  Probably not, however I would like to hear from a Serde expert on the matter.
  There are two complications with trying to serialize Protobuf messages with
  Serde:

  - Protobuf fields require a numbered tag, and curently there appears to be no
    mechanism suitable for this in `serde`.
  - The mapping of Protobuf type to Rust type is not 1-to-1. As a result,
    trait-based approaches to dispatching don't work very well. Example: six
    different Protobuf field types correspond to a Rust `Vec<i32>`: `repeated
    int32`, `repeated sint32`, `repeated sfixed32`, and their packed
    counterparts.

2. **Looks like a lot of field annotations. Can those be simplified?**

  Probably. Effort has not yet been spent on reducing the number of annotations.
  The recommended way of using `prost` is through [`prost-build`](prost-build),
  in which case the annotations are never seen.

## License

`prost` is distributed under the terms of the Apache License (Version 2.0).

See [LICENSE](LICENSE), for details.

Copyright 2017 Dan Burkert
