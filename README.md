# Merger
A rust proc macro to allow merging/updating structs. This crate provides a trait 'MergeFrom', and a derive macro of the same name.
The behaviour currently mimics that of protobuf's MergeFrom:
- Scalar values get replaced.
- 'Repeated' values (`Vec`,`String`) append.
- Maps (`BTreeMap`,`HashMap`) recursively merge fields that exist in both, or insert.

## TODOs 
- Add support for specifying custom merge function (for example if you'd like `Vec` to also replace).
- Add support for ignoring fields (useful if you have fields that should only be set explicitly).

## Contributing
Yes please. 
